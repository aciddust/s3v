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
    let client = get_client(&state, &profile_id).await?;

    // Expand folder keys: collect all children so the entire folder is deleted
    let mut all_keys = Vec::new();
    for key in &keys {
        if key.ends_with('/') {
            let children = operations::list_all_objects(&client, &bucket, key).await?;
            all_keys.extend(children);
        }
        all_keys.push(key.clone());
    }

    logger::info(
        "s3",
        format!("Deleting {} object(s) from {bucket}", all_keys.len()),
    );
    let result = operations::delete_objects(&client, &bucket, &all_keys).await;
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
pub async fn cross_profile_copy_object(
    state: State<'_, AppState>,
    source_profile_id: String,
    source_bucket: String,
    source_key: String,
    dest_profile_id: String,
    dest_bucket: String,
    dest_key: String,
) -> Result<(), AppError> {
    logger::info(
        "s3",
        format!(
            "Cross-profile copy [{source_profile_id}] {source_bucket}/{source_key} → [{dest_profile_id}] {dest_bucket}/{dest_key}"
        ),
    );
    let source_client = get_client(&state, &source_profile_id).await?;
    let dest_client = get_client(&state, &dest_profile_id).await?;
    operations::cross_profile_copy_object(
        &source_client,
        &dest_client,
        &source_bucket,
        &source_key,
        &dest_bucket,
        &dest_key,
    )
    .await
}

#[tauri::command]
pub async fn cross_profile_copy_folder(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    source_profile_id: String,
    source_bucket: String,
    source_prefix: String,
    dest_profile_id: String,
    dest_bucket: String,
    dest_prefix: String,
) -> Result<String, AppError> {
    let source_client = get_client(&state, &source_profile_id).await?;
    let dest_client = get_client(&state, &dest_profile_id).await?;
    let op_id = uuid::Uuid::new_v4().to_string();

    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    {
        let mut ops = state.folder_ops.lock().await;
        ops.insert(op_id.clone(), cancel_tx);
    }

    let folder_ops = state.folder_ops.clone();
    let op_id_clone = op_id.clone();

    tokio::spawn(async move {
        let all_keys = match operations::list_all_objects(
            &source_client,
            &source_bucket,
            &source_prefix,
        )
        .await
        {
            Ok(keys) => keys,
            Err(e) => {
                let _ = app_handle.emit(
                    "folder-op-progress",
                    serde_json::json!({
                        "id": op_id_clone,
                        "op": "copy",
                        "phase": "failed",
                        "done": 0, "total": 0,
                        "key": "",
                        "source_prefix": source_prefix,
                        "dest_prefix": dest_prefix,
                        "error": e.to_string(),
                    }),
                );
                let mut ops = folder_ops.lock().await;
                ops.remove(&op_id_clone);
                return;
            }
        };

        let total = all_keys.len();
        let _ = app_handle.emit(
            "folder-op-progress",
            serde_json::json!({
                "id": op_id_clone,
                "op": "copy",
                "phase": "started",
                "done": 0, "total": total,
                "key": "",
                "source_prefix": source_prefix,
                "dest_prefix": dest_prefix,
                "error": null,
            }),
        );

        let mut done = 0usize;
        for key in &all_keys {
            if *cancel_rx.borrow() {
                let _ = app_handle.emit(
                    "folder-op-progress",
                    serde_json::json!({
                        "id": op_id_clone,
                        "op": "copy",
                        "phase": "cancelled",
                        "done": done, "total": total,
                        "key": "",
                        "source_prefix": source_prefix,
                        "dest_prefix": dest_prefix,
                        "error": null,
                    }),
                );
                let mut ops = folder_ops.lock().await;
                ops.remove(&op_id_clone);
                return;
            }

            let relative = key.strip_prefix(&source_prefix).unwrap_or(key);
            let dest_key = format!("{}{}", dest_prefix, relative);

            match operations::cross_profile_copy_object(
                &source_client,
                &dest_client,
                &source_bucket,
                key,
                &dest_bucket,
                &dest_key,
            )
            .await
            {
                Ok(()) => {
                    done += 1;
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "copy",
                            "phase": "copying",
                            "done": done, "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": null,
                        }),
                    );
                }
                Err(e) => {
                    done += 1;
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "copy",
                            "phase": "failed",
                            "done": done, "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": e.to_string(),
                        }),
                    );
                }
            }
        }

        // Create destination folder marker
        let _ = operations::create_folder(&dest_client, &dest_bucket, &dest_prefix).await;

        let _ = app_handle.emit(
            "folder-op-progress",
            serde_json::json!({
                "id": op_id_clone,
                "op": "copy",
                "phase": "completed",
                "done": done, "total": total,
                "key": "",
                "source_prefix": source_prefix,
                "dest_prefix": dest_prefix,
                "error": null,
            }),
        );
        let mut ops = folder_ops.lock().await;
        ops.remove(&op_id_clone);
    });

    Ok(op_id)
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

#[derive(serde::Serialize)]
pub struct ClassifiedPaths {
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

#[tauri::command]
pub async fn classify_paths(paths: Vec<String>) -> Result<ClassifiedPaths, AppError> {
    let mut files = Vec::new();
    let mut directories = Vec::new();
    for path in paths {
        let metadata = std::fs::metadata(&path)?;
        if metadata.is_dir() {
            directories.push(path);
        } else {
            files.push(path);
        }
    }
    Ok(ClassifiedPaths { files, directories })
}

#[tauri::command]
pub async fn check_conflicts(
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    keys: Vec<String>,
) -> Result<Vec<String>, AppError> {
    let client = get_client(&state, &profile_id).await?;
    let mut conflicts = Vec::new();
    for key in &keys {
        match operations::head_object(&client, &bucket, key).await {
            Ok(_) => conflicts.push(key.clone()),
            Err(_) => {}
        }
    }
    Ok(conflicts)
}

#[tauri::command]
pub async fn copy_folder(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    source_bucket: String,
    source_prefix: String,
    dest_bucket: String,
    dest_prefix: String,
    skip_keys: Vec<String>,
    rename_keys: Vec<String>,
) -> Result<String, AppError> {
    let client = get_client(&state, &profile_id).await?;
    let op_id = uuid::Uuid::new_v4().to_string();

    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    {
        let mut ops = state.folder_ops.lock().await;
        ops.insert(op_id.clone(), cancel_tx);
    }

    let folder_ops = state.folder_ops.clone();
    let op_id_clone = op_id.clone();

    tokio::spawn(async move {
        let all_keys =
            match operations::list_all_objects(&client, &source_bucket, &source_prefix).await {
                Ok(keys) => keys,
                Err(e) => {
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "copy",
                            "phase": "failed",
                            "done": 0,
                            "total": 0,
                            "key": "",
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": e.to_string(),
                        }),
                    );
                    let mut ops = folder_ops.lock().await;
                    ops.remove(&op_id_clone);
                    return;
                }
            };

        let total = all_keys.len();
        let _ = app_handle.emit(
            "folder-op-progress",
            serde_json::json!({
                "id": op_id_clone,
                "op": "copy",
                "phase": "started",
                "done": 0,
                "total": total,
                "key": "",
                "source_prefix": source_prefix,
                "dest_prefix": dest_prefix,
                "error": null,
            }),
        );

        let mut done = 0usize;
        for key in &all_keys {
            // Check for cancellation
            if *cancel_rx.borrow() {
                let _ = app_handle.emit(
                    "folder-op-progress",
                    serde_json::json!({
                        "id": op_id_clone,
                        "op": "copy",
                        "phase": "cancelled",
                        "done": done,
                        "total": total,
                        "key": "",
                        "source_prefix": source_prefix,
                        "dest_prefix": dest_prefix,
                        "error": null,
                    }),
                );
                let mut ops = folder_ops.lock().await;
                ops.remove(&op_id_clone);
                return;
            }

            // Compute destination key: strip source prefix and prepend dest prefix
            let relative = key
                .strip_prefix(&source_prefix)
                .unwrap_or(key);
            let mut dest_key = format!("{}{}", dest_prefix, relative);

            // Apply skip/rename logic
            if skip_keys.contains(key) {
                done += 1;
                continue;
            }
            if rename_keys.contains(key) {
                let renamed_relative = operations::resolve_rename_key(relative);
                dest_key = format!("{}{}", dest_prefix, renamed_relative);
            }

            match operations::copy_object(
                &client,
                &source_bucket,
                key,
                &dest_bucket,
                &dest_key,
            )
            .await
            {
                Ok(()) => {
                    done += 1;
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "copy",
                            "phase": "copying",
                            "done": done,
                            "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": null,
                        }),
                    );
                }
                Err(e) => {
                    done += 1;
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "copy",
                            "phase": "failed",
                            "done": done,
                            "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": e.to_string(),
                        }),
                    );
                    // Continue with next key on individual failure
                }
            }
        }

        // Create destination folder marker
        let _ = operations::create_folder(&client, &dest_bucket, &dest_prefix).await;

        // Completed
        let _ = app_handle.emit(
            "folder-op-progress",
            serde_json::json!({
                "id": op_id_clone,
                "op": "copy",
                "phase": "completed",
                "done": done,
                "total": total,
                "key": "",
                "source_prefix": source_prefix,
                "dest_prefix": dest_prefix,
                "error": null,
            }),
        );
        let mut ops = folder_ops.lock().await;
        ops.remove(&op_id_clone);
    });

    Ok(op_id)
}

#[tauri::command]
pub async fn move_folder(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    source_bucket: String,
    source_prefix: String,
    dest_bucket: String,
    dest_prefix: String,
    skip_keys: Vec<String>,
    rename_keys: Vec<String>,
) -> Result<String, AppError> {
    let client = get_client(&state, &profile_id).await?;
    let op_id = uuid::Uuid::new_v4().to_string();

    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    {
        let mut ops = state.folder_ops.lock().await;
        ops.insert(op_id.clone(), cancel_tx);
    }

    let folder_ops = state.folder_ops.clone();
    let op_id_clone = op_id.clone();

    tokio::spawn(async move {
        let all_keys =
            match operations::list_all_objects(&client, &source_bucket, &source_prefix).await {
                Ok(keys) => keys,
                Err(e) => {
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "move",
                            "phase": "failed",
                            "done": 0,
                            "total": 0,
                            "key": "",
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": e.to_string(),
                        }),
                    );
                    let mut ops = folder_ops.lock().await;
                    ops.remove(&op_id_clone);
                    return;
                }
            };

        let total = all_keys.len();
        let _ = app_handle.emit(
            "folder-op-progress",
            serde_json::json!({
                "id": op_id_clone,
                "op": "move",
                "phase": "started",
                "done": 0,
                "total": total,
                "key": "",
                "source_prefix": source_prefix,
                "dest_prefix": dest_prefix,
                "error": null,
            }),
        );

        let mut done = 0usize;
        let mut copied_keys: Vec<String> = Vec::new();
        let mut copy_failed = false;

        // Copy phase
        for key in &all_keys {
            // Check for cancellation
            if *cancel_rx.borrow() {
                let _ = app_handle.emit(
                    "folder-op-progress",
                    serde_json::json!({
                        "id": op_id_clone,
                        "op": "move",
                        "phase": "cancelled",
                        "done": done,
                        "total": total,
                        "key": "",
                        "source_prefix": source_prefix,
                        "dest_prefix": dest_prefix,
                        "error": null,
                    }),
                );
                let mut ops = folder_ops.lock().await;
                ops.remove(&op_id_clone);
                return;
            }

            // Compute destination key: strip source prefix and prepend dest prefix
            let relative = key
                .strip_prefix(&source_prefix)
                .unwrap_or(key);
            let mut dest_key = format!("{}{}", dest_prefix, relative);

            // Apply skip/rename logic
            if skip_keys.contains(key) {
                done += 1;
                continue;
            }
            if rename_keys.contains(key) {
                let renamed_relative = operations::resolve_rename_key(relative);
                dest_key = format!("{}{}", dest_prefix, renamed_relative);
            }

            match operations::copy_object(
                &client,
                &source_bucket,
                key,
                &dest_bucket,
                &dest_key,
            )
            .await
            {
                Ok(()) => {
                    done += 1;
                    copied_keys.push(key.clone());
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "move",
                            "phase": "copying",
                            "done": done,
                            "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": null,
                        }),
                    );
                }
                Err(e) => {
                    done += 1;
                    copy_failed = true;
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "move",
                            "phase": "failed",
                            "done": done,
                            "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": e.to_string(),
                        }),
                    );
                    // Continue with next key on individual failure
                }
            }
        }

        // Create destination folder marker
        let _ = operations::create_folder(&client, &dest_bucket, &dest_prefix).await;

        // Delete phase: only if copy completed without cancellation and without failures
        if !copy_failed {
            for key in &copied_keys {
                // Check for cancellation during delete phase
                if *cancel_rx.borrow() {
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "move",
                            "phase": "cancelled",
                            "done": done,
                            "total": total,
                            "key": "",
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": null,
                        }),
                    );
                    let mut ops = folder_ops.lock().await;
                    ops.remove(&op_id_clone);
                    return;
                }

                let _ = app_handle.emit(
                    "folder-op-progress",
                    serde_json::json!({
                        "id": op_id_clone,
                        "op": "move",
                        "phase": "deleting",
                        "done": done,
                        "total": total,
                        "key": key,
                        "source_prefix": source_prefix,
                        "dest_prefix": dest_prefix,
                        "error": null,
                    }),
                );

                if let Err(e) = operations::delete_objects(&client, &source_bucket, &[key.clone()]).await {
                    let _ = app_handle.emit(
                        "folder-op-progress",
                        serde_json::json!({
                            "id": op_id_clone,
                            "op": "move",
                            "phase": "failed",
                            "done": done,
                            "total": total,
                            "key": key,
                            "source_prefix": source_prefix,
                            "dest_prefix": dest_prefix,
                            "error": e.to_string(),
                        }),
                    );
                }
            }

            // Delete folder marker
            let _ = operations::delete_objects(&client, &source_bucket, &[source_prefix.to_string()]).await;
        }

        // Completed
        let _ = app_handle.emit(
            "folder-op-progress",
            serde_json::json!({
                "id": op_id_clone,
                "op": "move",
                "phase": "completed",
                "done": done,
                "total": total,
                "key": "",
                "source_prefix": source_prefix,
                "dest_prefix": dest_prefix,
                "error": null,
            }),
        );
        let mut ops = folder_ops.lock().await;
        ops.remove(&op_id_clone);
    });

    Ok(op_id)
}

#[tauri::command]
pub async fn cancel_folder_op(
    state: State<'_, AppState>,
    op_id: String,
) -> Result<(), AppError> {
    let ops = state.folder_ops.lock().await;
    if let Some(cancel_tx) = ops.get(&op_id) {
        let _ = cancel_tx.send(true);
        Ok(())
    } else {
        Err(AppError::S3(format!("Folder operation '{}' not found", op_id)))
    }
}
