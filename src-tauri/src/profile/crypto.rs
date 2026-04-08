use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Encryption backend for profile storage.
pub enum CryptoBackend {
    /// Built-in AES-256-GCM with auto-generated key.
    Builtin { key_path: PathBuf },
    /// External sops + age.
    Sops,
}

impl CryptoBackend {
    pub fn builtin(config_dir: &Path) -> Self {
        Self::Builtin {
            key_path: config_dir.join("encryption.key"),
        }
    }

    pub fn profiles_filename(&self) -> &str {
        match self {
            Self::Builtin { .. } => "profiles.enc",
            Self::Sops => "profiles.sops.json",
        }
    }

    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
        match self {
            Self::Builtin { key_path } => builtin_encrypt(key_path, plaintext),
            Self::Sops => unreachable!("sops encrypt is handled separately"),
        }
    }

    pub async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, AppError> {
        match self {
            Self::Builtin { key_path } => builtin_decrypt(key_path, ciphertext),
            Self::Sops => unreachable!("sops decrypt is handled separately"),
        }
    }

    pub fn is_sops(&self) -> bool {
        matches!(self, Self::Sops)
    }
}

/// Ensure the encryption key exists, generating one if needed.
fn load_or_create_key(key_path: &Path) -> Result<Key<Aes256Gcm>, AppError> {
    if key_path.exists() {
        let encoded = std::fs::read_to_string(key_path)
            .map_err(|e| AppError::Encryption(format!("Failed to read key file: {e}")))?;
        let bytes = B64
            .decode(encoded.trim())
            .map_err(|e| AppError::Encryption(format!("Invalid key encoding: {e}")))?;
        if bytes.len() != 32 {
            return Err(AppError::Encryption("Invalid key length".into()));
        }
        Ok(*Key::<Aes256Gcm>::from_slice(&bytes))
    } else {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let encoded = B64.encode(key_bytes);
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(key_path, &encoded)
            .map_err(|e| AppError::Encryption(format!("Failed to write key file: {e}")))?;

        // Restrict permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(key_path, perms)?;
        }

        Ok(*Key::<Aes256Gcm>::from_slice(&key_bytes))
    }
}

/// Encrypt plaintext with AES-256-GCM. Output: base64(nonce || ciphertext).
fn builtin_encrypt(key_path: &Path, plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    let key = load_or_create_key(key_path)?;
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AppError::Encryption(format!("Encryption failed: {e}")))?;

    // nonce (12 bytes) || ciphertext
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    let encoded = B64.encode(&output);
    Ok(encoded.into_bytes())
}

/// Decrypt base64(nonce || ciphertext) with AES-256-GCM.
fn builtin_decrypt(key_path: &Path, data: &[u8]) -> Result<Vec<u8>, AppError> {
    let key = load_or_create_key(key_path)?;
    let cipher = Aes256Gcm::new(&key);

    let encoded = std::str::from_utf8(data)
        .map_err(|e| AppError::Encryption(format!("Invalid UTF-8: {e}")))?;
    let raw = B64
        .decode(encoded.trim())
        .map_err(|e| AppError::Encryption(format!("Invalid base64: {e}")))?;

    if raw.len() < 12 {
        return Err(AppError::Encryption("Ciphertext too short".into()));
    }

    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Encryption(format!("Decryption failed: {e}")))
}

/// Encrypt/decrypt via sops CLI.
pub async fn sops_encrypt(
    plaintext: &[u8],
    output_path: &Path,
    config_dir: &Path,
) -> Result<(), AppError> {
    let tmp_path = config_dir.join("profiles.tmp.json");
    tokio::fs::write(&tmp_path, plaintext).await?;

    let output = tokio::process::Command::new("sops")
        .arg("encrypt")
        .arg("--input-type")
        .arg("json")
        .arg("--output-type")
        .arg("json")
        .arg("--output")
        .arg(output_path)
        .arg(&tmp_path)
        .output()
        .await
        .map_err(|e| AppError::Encryption(format!("Failed to run sops: {e}")))?;

    let _ = tokio::fs::remove_file(&tmp_path).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Encryption(format!(
            "sops encrypt failed: {stderr}"
        )));
    }
    Ok(())
}

pub async fn sops_decrypt(input_path: &Path) -> Result<Vec<u8>, AppError> {
    let output = tokio::process::Command::new("sops")
        .arg("decrypt")
        .arg(input_path)
        .output()
        .await
        .map_err(|e| AppError::Encryption(format!("Failed to run sops: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Encryption(format!(
            "sops decrypt failed: {stderr}"
        )));
    }
    Ok(output.stdout)
}
