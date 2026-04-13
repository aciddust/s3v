use crate::error::AppError;
use crate::profile::manager::ProfileManager;
use crate::s3::client_pool::{self, ClientPool};
use crate::transfer::engine::TransferEngine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

pub type ProfileId = String;

pub struct AppState {
    pub s3_clients: ClientPool,
    pub profile_manager: Arc<ProfileManager>,
    pub transfer_engine: TransferEngine,
    pub folder_ops: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
}

impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Result<Self, AppError> {
        let s3_clients = client_pool::new_pool();
        let profile_manager = Arc::new(ProfileManager::new()?);
        let transfer_engine =
            TransferEngine::new(app_handle, s3_clients.clone(), profile_manager.clone());
        Ok(Self {
            s3_clients,
            profile_manager,
            transfer_engine,
            folder_ops: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
