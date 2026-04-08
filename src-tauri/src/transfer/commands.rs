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
