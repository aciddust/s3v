use serde::{Deserialize, Serialize};

/// 8 MB default chunk size
pub const DEFAULT_CHUNK_SIZE: u64 = 8 * 1024 * 1024;

/// 100 MB multipart threshold
pub const DEFAULT_MULTIPART_THRESHOLD: u64 = 100 * 1024 * 1024;

/// Default max retries per part/chunk
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default max concurrent transfers
pub const DEFAULT_MAX_CONCURRENT: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    Upload,
    Download,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    Queued,
    Active,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPath {
    pub local: String,
    pub remote_bucket: String,
    pub remote_key: String,
}

#[derive(Debug, Clone)]
pub struct TransferConfig {
    pub chunk_size: u64,
    pub multipart_threshold: u64,
    pub max_retries: u32,
    pub max_concurrent: usize,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            multipart_threshold: DEFAULT_MULTIPART_THRESHOLD,
            max_retries: DEFAULT_MAX_RETRIES,
            max_concurrent: DEFAULT_MAX_CONCURRENT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferJob {
    pub id: String,
    pub profile_id: String,
    pub transfer_type: TransferType,
    pub path: TransferPath,
    pub status: TransferStatus,
    pub progress: TransferProgress,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferJobSummary {
    pub id: String,
    pub profile_id: String,
    pub transfer_type: TransferType,
    pub path: TransferPath,
    pub status: TransferStatus,
    pub progress: TransferProgress,
    pub error: Option<String>,
}

impl From<&TransferJob> for TransferJobSummary {
    fn from(job: &TransferJob) -> Self {
        Self {
            id: job.id.clone(),
            profile_id: job.profile_id.clone(),
            transfer_type: job.transfer_type.clone(),
            path: job.path.clone(),
            status: job.status.clone(),
            progress: job.progress.clone(),
            error: job.error.clone(),
        }
    }
}

// Tauri event payloads

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgressEvent {
    pub id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStatusEvent {
    pub id: String,
    pub status: TransferStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCompletedEvent {
    pub id: String,
    pub transfer_type: TransferType,
    pub path: TransferPath,
}
