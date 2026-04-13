#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
mod promise;

use crate::error::AppError;
use crate::s3::{client_pool, operations};
use crate::state::AppState;
use tauri::State;

/// Expand any folder keys (ending with `/`) into individual file keys.
/// Non-folder keys pass through unchanged.
async fn expand_folder_keys(
    state: &AppState,
    profile_id: &str,
    bucket: &str,
    keys: Vec<String>,
) -> Result<Vec<String>, AppError> {
    let mut expanded = Vec::new();
    let mut has_folders = false;

    for key in &keys {
        if key.ends_with('/') {
            has_folders = true;
            break;
        }
    }

    if !has_folders {
        return Ok(keys);
    }

    let profile = state.profile_manager.get(profile_id).await?;
    let client = client_pool::get_or_create_client(&state.s3_clients, &profile).await?;

    for key in keys {
        if key.ends_with('/') {
            let children = operations::list_all_objects(&client, bucket, &key).await?;
            expanded.extend(children);
        } else {
            expanded.push(key);
        }
    }

    Ok(expanded)
}

/// Compute common prefix from the ORIGINAL dragged keys (before folder expansion).
/// For folders, uses the parent directory so the folder name is preserved in
/// the relative path. For files, uses everything up to the last `/`.
fn compute_common_prefix_for_drag(keys: &[String]) -> String {
    if keys.is_empty() {
        return String::new();
    }

    fn parent_prefix(key: &str) -> &str {
        let trimmed = key.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(pos) => &key[..=pos],
            None => "",
        }
    }

    let parents: Vec<&str> = keys.iter().map(|k| parent_prefix(k)).collect();

    if parents.len() == 1 {
        return parents[0].to_string();
    }

    let first = parents[0];
    let mut prefix_len = first.len();

    for p in &parents[1..] {
        let common = first
            .bytes()
            .zip(p.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        prefix_len = prefix_len.min(common);
    }

    let prefix = &first[..prefix_len];
    match prefix.rfind('/') {
        Some(pos) => first[..=pos].to_string(),
        None => String::new(),
    }
}

#[tauri::command]
pub async fn start_native_drag(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    bucket: String,
    keys: Vec<String>,
) -> Result<(), AppError> {
    // Compute common prefix from ORIGINAL keys (before expansion)
    let common_prefix = compute_common_prefix_for_drag(&keys);

    // Expand folder keys into individual file keys
    let expanded_keys = expand_folder_keys(&state, &profile_id, &bucket, keys).await?;

    if expanded_keys.is_empty() {
        return Err(AppError::S3("No files to drag".into()));
    }

    #[cfg(target_os = "macos")]
    {
        macos::initiate_drag(app, &state, profile_id, bucket, expanded_keys, common_prefix).await
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, state, profile_id, bucket, expanded_keys, common_prefix);
        Err(AppError::S3("Native drag not supported on this platform".into()))
    }
}
