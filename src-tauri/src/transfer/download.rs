use aws_sdk_s3::Client as S3Client;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

use super::types::{TransferConfig, TransferPath, TransferProgress};
use crate::error::AppError;

/// Execute a download, dispatching to simple or ranged based on file size.
pub async fn execute_download(
    client: &S3Client,
    path: &TransferPath,
    config: &TransferConfig,
    cancel_rx: watch::Receiver<bool>,
    progress_tx: Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<(), AppError> {
    // HEAD request to get size
    let head = client
        .head_object()
        .bucket(&path.remote_bucket)
        .key(&path.remote_key)
        .send()
        .await
        .map_err(|e| AppError::Transfer(format!("HEAD request failed: {e}")))?;

    let file_size = head.content_length().unwrap_or(0) as u64;

    if file_size <= config.multipart_threshold {
        simple_download(client, path, file_size, &progress_tx).await
    } else {
        ranged_download(client, path, file_size, config, cancel_rx, &progress_tx).await
    }
}

async fn simple_download(
    client: &S3Client,
    path: &TransferPath,
    file_size: u64,
    progress_tx: &Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<(), AppError> {
    let resp = client
        .get_object()
        .bucket(&path.remote_bucket)
        .key(&path.remote_key)
        .send()
        .await
        .map_err(|e| AppError::Transfer(format!("Download failed: {e}")))?;

    let body = resp
        .body
        .collect()
        .await
        .map_err(|e| AppError::Transfer(format!("Failed to read download body: {e}")))?;

    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&path.local).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Transfer(format!("Failed to create directory: {e}")))?;
    }

    tokio::fs::write(&path.local, body.into_bytes())
        .await
        .map_err(|e| AppError::Transfer(format!("Failed to write file: {e}")))?;

    progress_tx(TransferProgress {
        bytes_transferred: file_size,
        total_bytes: file_size,
    });

    Ok(())
}

async fn ranged_download(
    client: &S3Client,
    path: &TransferPath,
    file_size: u64,
    config: &TransferConfig,
    cancel_rx: watch::Receiver<bool>,
    progress_tx: &Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<(), AppError> {
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&path.local).parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Transfer(format!("Failed to create directory: {e}")))?;
    }

    let mut file = tokio::fs::File::create(&path.local)
        .await
        .map_err(|e| AppError::Transfer(format!("Failed to create file: {e}")))?;

    let chunk_size = config.chunk_size;
    let mut offset: u64 = 0;

    while offset < file_size {
        // Check for cancellation
        if *cancel_rx.borrow() {
            // Clean up partial file
            drop(file);
            let _ = tokio::fs::remove_file(&path.local).await;
            return Err(AppError::Transfer("Download cancelled".into()));
        }

        let end = std::cmp::min(offset + chunk_size - 1, file_size - 1);
        let range = format!("bytes={offset}-{end}");

        let chunk_data =
            download_range_with_retry(client, path, &range, config.max_retries).await?;

        file.write_all(&chunk_data)
            .await
            .map_err(|e| AppError::Transfer(format!("Failed to write chunk: {e}")))?;

        offset = end + 1;

        progress_tx(TransferProgress {
            bytes_transferred: offset,
            total_bytes: file_size,
        });
    }

    file.flush()
        .await
        .map_err(|e| AppError::Transfer(format!("Failed to flush file: {e}")))?;

    Ok(())
}

async fn download_range_with_retry(
    client: &S3Client,
    path: &TransferPath,
    range: &str,
    max_retries: u32,
) -> Result<Vec<u8>, AppError> {
    let mut last_err = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }

        match client
            .get_object()
            .bucket(&path.remote_bucket)
            .key(&path.remote_key)
            .range(range)
            .send()
            .await
        {
            Ok(resp) => {
                let body =
                    resp.body.collect().await.map_err(|e| {
                        AppError::Transfer(format!("Failed to read range body: {e}"))
                    })?;
                return Ok(body.into_bytes().to_vec());
            }
            Err(e) => {
                last_err = Some(format!("Range {range} attempt {attempt}: {e}"));
            }
        }
    }

    Err(AppError::Transfer(
        last_err.unwrap_or_else(|| "Download range failed".into()),
    ))
}
