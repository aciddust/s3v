use super::crypto::{self, CryptoBackend};
use super::types::{Profile, ProfileInput, ProfileStore, ProfileSummary};
use crate::error::AppError;
use std::path::PathBuf;
use uuid::Uuid;

pub struct ProfileManager {
    config_dir: PathBuf,
    crypto: CryptoBackend,
}

impl ProfileManager {
    pub fn new() -> Result<Self, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AppError::Profile("Cannot find config directory".into()))?
            .join("s3v");
        std::fs::create_dir_all(&config_dir)?;

        // Check settings.json for encryption mode
        let crypto = load_crypto_backend(&config_dir);

        Ok(Self { config_dir, crypto })
    }

    fn profiles_path(&self) -> PathBuf {
        self.config_dir.join(self.crypto.profiles_filename())
    }

    async fn load_profiles(&self) -> Result<ProfileStore, AppError> {
        let path = self.profiles_path();
        if !path.exists() {
            return Ok(ProfileStore::new());
        }

        let json_bytes = if self.crypto.is_sops() {
            crypto::sops_decrypt(&path).await?
        } else {
            let data = tokio::fs::read(&path).await?;
            self.crypto.decrypt(&data).await?
        };

        let store: ProfileStore = serde_json::from_slice(&json_bytes)?;
        Ok(store)
    }

    async fn save_profiles(&self, store: &ProfileStore) -> Result<(), AppError> {
        let json = serde_json::to_string_pretty(store)?;
        let path = self.profiles_path();

        if self.crypto.is_sops() {
            crypto::sops_encrypt(json.as_bytes(), &path, &self.config_dir).await?;
        } else {
            let encrypted = self.crypto.encrypt(json.as_bytes()).await?;
            tokio::fs::write(&path, &encrypted).await?;
        }

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<ProfileSummary>, AppError> {
        let store = self.load_profiles().await?;
        Ok(store
            .profiles
            .iter()
            .map(|p| ProfileSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                provider: p.provider.clone(),
                default_bucket: p.default_bucket.clone(),
            })
            .collect())
    }

    pub async fn get(&self, id: &str) -> Result<Profile, AppError> {
        let store = self.load_profiles().await?;
        store
            .profiles
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::Profile(format!("Profile not found: {id}")))
    }

    pub async fn create(&self, input: ProfileInput) -> Result<String, AppError> {
        let mut store = self.load_profiles().await?;
        let id = Uuid::new_v4().to_string();
        let profile = Profile {
            id: id.clone(),
            name: input.name,
            provider: input.provider,
            endpoint: input.endpoint,
            region: input.region,
            access_key_id: input.access_key_id,
            secret_access_key: input.secret_access_key,
            path_style: input.path_style,
            default_bucket: input.default_bucket,
        };
        store.profiles.push(profile);
        self.save_profiles(&store).await?;
        Ok(id)
    }

    pub async fn update(&self, id: &str, input: ProfileInput) -> Result<(), AppError> {
        let mut store = self.load_profiles().await?;
        let profile = store
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::Profile(format!("Profile not found: {id}")))?;

        profile.name = input.name;
        profile.provider = input.provider;
        profile.endpoint = input.endpoint;
        profile.region = input.region;
        profile.access_key_id = input.access_key_id;
        profile.secret_access_key = input.secret_access_key;
        profile.path_style = input.path_style;
        profile.default_bucket = input.default_bucket;

        self.save_profiles(&store).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut store = self.load_profiles().await?;
        let len_before = store.profiles.len();
        store.profiles.retain(|p| p.id != id);
        if store.profiles.len() == len_before {
            return Err(AppError::Profile(format!("Profile not found: {id}")));
        }
        self.save_profiles(&store).await?;
        Ok(())
    }
}

/// Read settings.json to determine encryption mode.
/// Default: builtin AES-256-GCM. If `"encryption": "sops"`, use sops+age.
fn load_crypto_backend(config_dir: &std::path::Path) -> CryptoBackend {
    let settings_path = config_dir.join("settings.json");
    if let Ok(data) = std::fs::read_to_string(&settings_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if val.get("encryption").and_then(|v| v.as_str()) == Some("sops") {
                return CryptoBackend::Sops;
            }
        }
    }
    CryptoBackend::builtin(config_dir)
}
