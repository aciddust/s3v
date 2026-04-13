use super::types::TransferJobSummary;
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn enqueue_upload(
    state: State<'_, AppState>,
    profile_id: String,
    local_path: String,
    bucket: String,
    key: String,
) -> Result<String, AppError> {
    state
        .transfer_engine
        .enqueue_upload(profile_id, local_path, bucket, key)
        .await
}

#[tauri::command]
pub async fn enqueue_download(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    key: String,
    local_path: String,
) -> Result<String, AppError> {
    state
        .transfer_engine
        .enqueue_download(profile_id, bucket, key, local_path)
        .await
}

#[tauri::command]
pub async fn pause_transfer(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.transfer_engine.pause(id)
}

#[tauri::command]
pub async fn resume_transfer(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.transfer_engine.resume(id)
}

#[tauri::command]
pub async fn cancel_transfer(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.transfer_engine.cancel(id)
}

#[tauri::command]
pub async fn list_transfers(
    state: State<'_, AppState>,
) -> Result<Vec<TransferJobSummary>, AppError> {
    Ok(state.transfer_engine.list_transfers().await)
}

#[tauri::command]
pub async fn enqueue_folder_upload(
    state: State<'_, AppState>,
    profile_id: String,
    local_dir: String,
    bucket: String,
    remote_prefix: String,
    skip_keys: Vec<String>,
    rename_keys: Vec<String>,
) -> Result<Vec<String>, AppError> {
    use crate::s3::operations::resolve_rename_key;
    use std::path::Path;

    let base_path = Path::new(&local_dir);
    let mut job_ids = Vec::new();

    let mut stack = vec![base_path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let relative = path
                    .strip_prefix(base_path)
                    .map_err(|e| AppError::Transfer(e.to_string()))?;
                let mut remote_key = format!(
                    "{}{}",
                    remote_prefix,
                    relative.to_string_lossy().replace('\\', "/")
                );

                if skip_keys.contains(&remote_key) {
                    continue;
                }
                if rename_keys.contains(&remote_key) {
                    remote_key = resolve_rename_key(&remote_key);
                }

                let local_path = path.to_string_lossy().to_string();
                let id = state
                    .transfer_engine
                    .enqueue_upload(
                        profile_id.clone(),
                        local_path,
                        bucket.clone(),
                        remote_key,
                    )
                    .await?;
                job_ids.push(id);
            }
        }
    }

    Ok(job_ids)
}

#[tauri::command]
pub async fn enqueue_folder_download(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    remote_prefix: String,
    local_dir: String,
) -> Result<Vec<String>, AppError> {
    use crate::s3::client_pool;
    use crate::s3::operations;
    use std::path::Path;

    let profile = state.profile_manager.get(&profile_id).await?;
    let client = client_pool::get_or_create_client(&state.s3_clients, &profile).await?;

    let keys = operations::list_all_objects(&client, &bucket, &remote_prefix).await?;

    let base_path = Path::new(&local_dir);
    let mut job_ids = Vec::new();

    for key in &keys {
        let relative = key.strip_prefix(&remote_prefix).unwrap_or(key);
        let local_path = base_path.join(relative);

        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let id = state
            .transfer_engine
            .enqueue_download(
                profile_id.clone(),
                bucket.clone(),
                key.clone(),
                local_path.to_string_lossy().to_string(),
            )
            .await?;
        job_ids.push(id);
    }

    Ok(job_ids)
}
