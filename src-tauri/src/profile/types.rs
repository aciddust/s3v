use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub provider: Provider,
    pub endpoint: Option<String>,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub path_style: bool,
    #[serde(default)]
    pub default_bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Aws,
    Rustfs,
    R2,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInput {
    pub name: String,
    pub provider: Provider,
    pub endpoint: Option<String>,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub path_style: bool,
    pub default_bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub id: String,
    pub name: String,
    pub provider: Provider,
    pub default_bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub message: String,
    pub bucket_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: Vec<Profile>,
}

impl ProfileStore {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }
}
