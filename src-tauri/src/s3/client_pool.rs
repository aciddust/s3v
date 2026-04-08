use crate::error::AppError;
use crate::profile::types::Profile;
use aws_sdk_s3::config::{
    timeout::TimeoutConfig, BehaviorVersion, Builder as S3ConfigBuilder, Credentials, Region,
};
use aws_sdk_s3::Client as S3Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub type ClientPool = Arc<RwLock<HashMap<String, S3Client>>>;

pub fn new_pool() -> ClientPool {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn get_or_create_client(
    pool: &ClientPool,
    profile: &Profile,
) -> Result<S3Client, AppError> {
    {
        let clients = pool.read().await;
        if let Some(client) = clients.get(&profile.id) {
            return Ok(client.clone());
        }
    }
    let client = create_client(profile)?;
    let mut clients = pool.write().await;
    clients.insert(profile.id.clone(), client.clone());
    Ok(client)
}

pub async fn invalidate(pool: &ClientPool, profile_id: &str) {
    let mut clients = pool.write().await;
    clients.remove(profile_id);
}

fn create_client(profile: &Profile) -> Result<S3Client, AppError> {
    let credentials = Credentials::new(
        &profile.access_key_id,
        &profile.secret_access_key,
        None,
        None,
        "s3v",
    );
    let timeout_config = TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(10))
        .operation_timeout(Duration::from_secs(300))
        .build();

    let mut config_builder = S3ConfigBuilder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(profile.region.clone()))
        .credentials_provider(credentials)
        .force_path_style(profile.path_style)
        .timeout_config(timeout_config);
    if let Some(endpoint) = &profile.endpoint {
        config_builder = config_builder.endpoint_url(endpoint);
    }
    Ok(S3Client::from_conf(config_builder.build()))
}
