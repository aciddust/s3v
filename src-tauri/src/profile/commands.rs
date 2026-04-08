use super::types::{ConnectionResult, Profile, ProfileInput, ProfileSummary};
use crate::error::AppError;
use crate::logger;
use crate::s3::client_pool;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> Result<Vec<ProfileSummary>, AppError> {
    state.profile_manager.list().await
}

#[tauri::command]
pub async fn get_profile(state: State<'_, AppState>, id: String) -> Result<Profile, AppError> {
    state.profile_manager.get(&id).await
}

#[tauri::command]
pub async fn create_profile(
    state: State<'_, AppState>,
    profile: ProfileInput,
) -> Result<String, AppError> {
    logger::info(
        "profile",
        format!(
            "Creating profile: {} ({:?})",
            profile.name, profile.provider
        ),
    );
    let id = state.profile_manager.create(profile).await?;
    logger::info("profile", format!("Profile created: {id}"));
    Ok(id)
}

#[tauri::command]
pub async fn update_profile(
    state: State<'_, AppState>,
    id: String,
    profile: ProfileInput,
) -> Result<(), AppError> {
    logger::info("profile", format!("Updating profile: {id}"));
    client_pool::invalidate(&state.s3_clients, &id).await;
    state.profile_manager.update(&id, profile).await
}

#[tauri::command]
pub async fn delete_profile(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    logger::info("profile", format!("Deleting profile: {id}"));
    client_pool::invalidate(&state.s3_clients, &id).await;
    state.profile_manager.delete(&id).await
}

#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<ConnectionResult, AppError> {
    let profile = state.profile_manager.get(&id).await?;
    logger::info(
        "connection",
        format!(
            "Testing connection: {} ({})",
            profile.name,
            profile.endpoint.as_deref().unwrap_or("default")
        ),
    );

    client_pool::invalidate(&state.s3_clients, &id).await;
    let client = client_pool::get_or_create_client(&state.s3_clients, &profile).await?;

    // If default_bucket is set, test with list_objects on that bucket instead of list_buckets
    // (bucket-scoped tokens can't call list_buckets)
    let test_future = if let Some(ref bucket) = profile.default_bucket {
        let b = bucket.clone();
        let c = client.clone();
        Box::pin(async move {
            c.list_objects_v2()
                .bucket(&b)
                .max_keys(1)
                .send()
                .await
                .map(|_| format!("Connected to bucket: {b}"))
                .map_err(|e| format!("Connection failed: {e}"))
        })
            as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
    } else {
        let c = client.clone();
        Box::pin(async move {
            c.list_buckets()
                .send()
                .await
                .map(|output| format!("Connected. Found {} bucket(s).", output.buckets().len()))
                .map_err(|e| format!("Connection failed: {e}"))
        })
            as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
    };

    match tokio::time::timeout(std::time::Duration::from_secs(10), test_future).await {
        Ok(Ok(msg)) => {
            logger::info("connection", &msg);
            Ok(ConnectionResult {
                success: true,
                message: msg,
                bucket_count: None,
            })
        }
        Ok(Err(msg)) => {
            logger::error("connection", &msg);
            client_pool::invalidate(&state.s3_clients, &id).await;
            Ok(ConnectionResult {
                success: false,
                message: msg,
                bucket_count: None,
            })
        }
        Err(_) => {
            logger::error(
                "connection",
                format!("Connection timed out for {}", profile.name),
            );
            client_pool::invalidate(&state.s3_clients, &id).await;
            Ok(ConnectionResult {
                success: false,
                message: "Connection timed out (10s)".into(),
                bucket_count: None,
            })
        }
    }
}
