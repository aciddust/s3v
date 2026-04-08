use super::operations;
use super::types::{BucketInfo, ListObjectsResult, MultipartUploadInfo, ObjectMetadata};
use crate::error::AppError;
use crate::logger;
use crate::s3::client_pool;
use crate::state::AppState;
use tauri::{AppHandle, Emitter, State};

async fn get_client(
    state: &State<'_, AppState>,
    profile_id: &str,
) -> Result<aws_sdk_s3::Client, AppError> {
    let profile = state.profile_manager.get(profile_id).await?;
    client_pool::get_or_create_client(&state.s3_clients, &profile).await
}

#[tauri::command]
pub async fn list_buckets(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<Vec<BucketInfo>, AppError> {
    logger::debug("s3", "Listing buckets");
    let result = {
        let client = get_client(&state, &profile_id).await?;
        operations::list_buckets(&client).await
    };
    match &result {
        Ok(buckets) => logger::info("s3", format!("Found {} bucket(s)", buckets.len())),
        Err(e) => logger::error("s3", format!("list_buckets failed: {e}")),
    }
    result
}

#[tauri::command]
pub async fn list_objects(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    prefix: String,
    delimiter: Option<String>,
    continuation_token: Option<String>,
) -> Result<ListObjectsResult, AppError> {
    logger::debug("s3", format!("list_objects {bucket}/{prefix}"));
    let client = get_client(&state, &profile_id).await?;
    let result = operations::list_objects(
        &client,
        &bucket,
        &prefix,
        delimiter.as_deref(),
        continuation_token.as_deref(),
    )
    .await;
    match &result {
        Ok(r) => logger::debug(
            "s3",
            format!(
                "{} objects, {} prefixes in {bucket}/{prefix}",
                r.objects.len(),
                r.common_prefixes.len()
            ),
        ),
        Err(e) => logger::error("s3", format!("list_objects failed: {e}")),
    }
    result
}

#[tauri::command]
pub async fn head_object(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    key: String,
) -> Result<ObjectMetadata, AppError> {
    let client = get_client(&state, &profile_id).await?;
    operations::head_object(&client, &bucket, &key).await
}

#[tauri::command]
pub async fn delete_objects(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    keys: Vec<String>,
) -> Result<(), AppError> {
    logger::info(
        "s3",
        format!("Deleting {} object(s) from {bucket}", keys.len()),
    );
    let client = get_client(&state, &profile_id).await?;
    let result = operations::delete_objects(&client, &bucket, &keys).await;
    if let Err(e) = &result {
        logger::error("s3", format!("delete failed: {e}"));
    }
    result
}

#[tauri::command]
pub async fn rename_object(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    old_key: String,
    new_key: String,
) -> Result<(), AppError> {
    logger::info("s3", format!("Rename {old_key} → {new_key}"));
    let client = get_client(&state, &profile_id).await?;
    operations::rename_object(&client, &bucket, &old_key, &new_key).await
}

#[tauri::command]
pub async fn rename_folder(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    old_prefix: String,
    new_prefix: String,
) -> Result<(), AppError> {
    logger::info("s3", format!("Rename folder {old_prefix} → {new_prefix}"));
    let client = get_client(&state, &profile_id).await?;
    operations::rename_folder(&client, &bucket, &old_prefix, &new_prefix).await
}

#[tauri::command]
pub async fn copy_object(
    state: State<'_, AppState>,
    profile_id: String,
    source_bucket: String,
    source_key: String,
    dest_bucket: String,
    dest_key: String,
) -> Result<(), AppError> {
    logger::info(
        "s3",
        format!("Copy {source_bucket}/{source_key} → {dest_bucket}/{dest_key}"),
    );
    let client = get_client(&state, &profile_id).await?;
    operations::copy_object(
        &client,
        &source_bucket,
        &source_key,
        &dest_bucket,
        &dest_key,
    )
    .await
}

#[tauri::command]
pub async fn move_objects(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    keys: Vec<String>,
    dest_prefix: String,
) -> Result<(), AppError> {
    let move_id = uuid::Uuid::new_v4().to_string();
    let total = keys.len();
    logger::info(
        "s3",
        format!("[{move_id}] Moving {total} object(s) → {bucket}/{dest_prefix}"),
    );
    let _ = app_handle.emit(
        "move-progress",
        serde_json::json!({
            "id": move_id,
            "phase": "start",
            "done": 0,
            "total": total,
            "key": "",
            "dest_prefix": dest_prefix,
        }),
    );
    let client = get_client(&state, &profile_id).await?;
    let result = operations::move_objects(
        &client,
        &bucket,
        &keys,
        &dest_prefix,
        |done, total, key, phase| {
            let _ = app_handle.emit(
                "move-progress",
                serde_json::json!({
                    "id": move_id,
                    "phase": phase,
                    "done": done,
                    "total": total,
                    "key": key,
                    "dest_prefix": dest_prefix,
                }),
            );
        },
    )
    .await;
    match &result {
        Ok(()) => {
            logger::info(
                "s3",
                format!("[{move_id}] Moved {total} object(s) → {bucket}/{dest_prefix}"),
            );
            let _ = app_handle.emit(
                "move-progress",
                serde_json::json!({
                    "id": move_id,
                    "phase": "completed",
                    "done": total,
                    "total": total,
                    "key": "",
                    "dest_prefix": dest_prefix,
                }),
            );
        }
        Err(ref e) => {
            logger::error("s3", format!("[{move_id}] Move failed: {e}"));
            let _ = app_handle.emit(
                "move-progress",
                serde_json::json!({
                    "id": move_id,
                    "phase": "failed",
                    "done": 0,
                    "total": total,
                    "key": "",
                    "dest_prefix": dest_prefix,
                }),
            );
        }
    }
    result
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    prefix: String,
) -> Result<(), AppError> {
    logger::info("s3", format!("Create folder: {bucket}/{prefix}"));
    let client = get_client(&state, &profile_id).await?;
    operations::create_folder(&client, &bucket, &prefix).await
}

#[tauri::command]
pub async fn get_presigned_url(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    key: String,
    expiry_secs: u64,
) -> Result<String, AppError> {
    logger::info(
        "s3",
        format!("Presigned URL: {bucket}/{key} ({expiry_secs}s)"),
    );
    let client = get_client(&state, &profile_id).await?;
    operations::get_presigned_url(&client, &bucket, &key, expiry_secs).await
}

#[tauri::command]
pub async fn list_multipart_uploads(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
) -> Result<Vec<MultipartUploadInfo>, AppError> {
    logger::info("s3", format!("Listing multipart uploads in {bucket}"));
    let client = get_client(&state, &profile_id).await?;
    let result = operations::list_multipart_uploads(&client, &bucket).await?;
    logger::info(
        "s3",
        format!("Found {} incomplete multipart upload(s)", result.len()),
    );
    Ok(result)
}

#[tauri::command]
pub async fn abort_multipart_upload(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    key: String,
    upload_id: String,
) -> Result<(), AppError> {
    logger::info(
        "s3",
        format!("Aborting multipart upload: {bucket}/{key} (upload_id: {upload_id})"),
    );
    let client = get_client(&state, &profile_id).await?;
    operations::abort_multipart_upload(&client, &bucket, &key, &upload_id).await
}
