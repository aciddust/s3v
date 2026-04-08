use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("S3 error: {0}")]
    S3(String),

    #[error("Profile error: {0}")]
    Profile(String),

    #[error("Transfer error: {0}")]
    Transfer(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
