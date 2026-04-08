use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::CompletedMultipartUpload;
use aws_sdk_s3::types::CompletedPart;
use aws_sdk_s3::Client as S3Client;
use std::sync::Arc;
use tokio::sync::watch;

use super::types::{TransferConfig, TransferPath, TransferProgress};
use crate::error::AppError;

/// Execute an upload, dispatching to simple or multipart based on file size.
pub async fn execute_upload(
    client: &S3Client,
    path: &TransferPath,
    config: &TransferConfig,
    cancel_rx: watch::Receiver<bool>,
    progress_tx: Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<(), AppError> {
    crate::logger::info(
        "transfer",
        format!(
            "Starting upload: {} → {}/{}",
            path.local, path.remote_bucket, path.remote_key
        ),
    );
    let metadata = tokio::fs::metadata(&path.local).await.map_err(|e| {
        crate::logger::error(
            "transfer",
            format!("Cannot read local file '{}': {e}", path.local),
        );
        AppError::Transfer(format!("Cannot read local file '{}': {e}", path.local))
    })?;
    let file_size = metadata.len();
    crate::logger::info(
        "transfer",
        format!(
            "File size: {} bytes, multipart threshold: {}",
            file_size, config.multipart_threshold
        ),
    );

    if file_size <= config.multipart_threshold {
        simple_upload(client, path, file_size, &progress_tx).await
    } else {
        multipart_upload(client, path, file_size, config, cancel_rx, &progress_tx).await
    }
}

async fn simple_upload(
    client: &S3Client,
    path: &TransferPath,
    file_size: u64,
    progress_tx: &Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<(), AppError> {
    let body = ByteStream::from_path(&path.local)
        .await
        .map_err(|e| AppError::Transfer(format!("Failed to read file for upload: {e}")))?;

    client
        .put_object()
        .bucket(&path.remote_bucket)
        .key(&path.remote_key)
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::Transfer(format!("Upload failed: {e}")))?;

    progress_tx(TransferProgress {
        bytes_transferred: file_size,
        total_bytes: file_size,
    });

    Ok(())
}

async fn multipart_upload(
    client: &S3Client,
    path: &TransferPath,
    file_size: u64,
    config: &TransferConfig,
    cancel_rx: watch::Receiver<bool>,
    progress_tx: &Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<(), AppError> {
    let create_resp = client
        .create_multipart_upload()
        .bucket(&path.remote_bucket)
        .key(&path.remote_key)
        .send()
        .await
        .map_err(|e| AppError::Transfer(format!("Create multipart upload failed: {e}")))?;

    let upload_id = create_resp
        .upload_id()
        .ok_or_else(|| AppError::Transfer("No upload ID returned".into()))?
        .to_string();

    let result = upload_parts(
        client,
        path,
        file_size,
        config,
        &upload_id,
        cancel_rx,
        progress_tx,
    )
    .await;

    match result {
        Ok(completed_parts) => {
            let completed_upload = CompletedMultipartUpload::builder()
                .set_parts(Some(completed_parts))
                .build();

            client
                .complete_multipart_upload()
                .bucket(&path.remote_bucket)
                .key(&path.remote_key)
                .upload_id(&upload_id)
                .multipart_upload(completed_upload)
                .send()
                .await
                .map_err(|e| {
                    AppError::Transfer(format!("Complete multipart upload failed: {e}"))
                })?;

            Ok(())
        }
        Err(e) => {
            let _ = client
                .abort_multipart_upload()
                .bucket(&path.remote_bucket)
                .key(&path.remote_key)
                .upload_id(&upload_id)
                .send()
                .await;
            Err(e)
        }
    }
}

async fn upload_parts(
    client: &S3Client,
    path: &TransferPath,
    file_size: u64,
    config: &TransferConfig,
    upload_id: &str,
    cancel_rx: watch::Receiver<bool>,
    progress_tx: &Arc<dyn Fn(TransferProgress) + Send + Sync>,
) -> Result<Vec<CompletedPart>, AppError> {
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(&path.local).await.map_err(|e| {
        AppError::Transfer(format!("Failed to open file for multipart upload: {e}"))
    })?;

    let chunk_size = config.chunk_size as usize;
    let mut completed_parts = Vec::new();
    let mut part_number: i32 = 1;
    let mut bytes_uploaded: u64 = 0;

    loop {
        // Check for cancellation before each part
        if *cancel_rx.borrow() {
            return Err(AppError::Transfer("Upload cancelled".into()));
        }

        let mut buf = vec![0u8; chunk_size];
        let mut filled = 0;
        while filled < chunk_size {
            let n = file
                .read(&mut buf[filled..])
                .await
                .map_err(|e| AppError::Transfer(format!("Failed to read file chunk: {e}")))?;
            if n == 0 {
                break;
            }
            filled += n;
        }

        if filled == 0 {
            break;
        }

        buf.truncate(filled);
        let part_body = ByteStream::from(buf);

        let etag = upload_part_with_retry(
            client,
            path,
            upload_id,
            part_number,
            part_body,
            filled,
            config.max_retries,
        )
        .await?;

        let completed_part = CompletedPart::builder()
            .e_tag(etag)
            .part_number(part_number)
            .build();
        completed_parts.push(completed_part);

        bytes_uploaded += filled as u64;
        progress_tx(TransferProgress {
            bytes_transferred: bytes_uploaded,
            total_bytes: file_size,
        });

        part_number += 1;

        if filled < chunk_size {
            break;
        }
    }

    Ok(completed_parts)
}

async fn upload_part_with_retry(
    client: &S3Client,
    path: &TransferPath,
    upload_id: &str,
    part_number: i32,
    body: ByteStream,
    content_length: usize,
    max_retries: u32,
) -> Result<String, AppError> {
    // We need to keep the bytes for retries since ByteStream is consumed on send.
    // For the first attempt, use the provided body. For retries, re-create from bytes.
    let bytes_data: Vec<u8>;
    let mut last_err = None;

    // Collect the body bytes up front so we can retry
    let body_bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Transfer(format!("Failed to collect body bytes: {e}")))?
        .into_bytes();
    bytes_data = body_bytes.to_vec();

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }

        let retry_body = ByteStream::from(bytes_data.clone());

        match client
            .upload_part()
            .bucket(&path.remote_bucket)
            .key(&path.remote_key)
            .upload_id(upload_id)
            .part_number(part_number)
            .content_length(content_length as i64)
            .body(retry_body)
            .send()
            .await
        {
            Ok(resp) => {
                let etag = resp.e_tag().unwrap_or_default().to_string();
                return Ok(etag);
            }
            Err(e) => {
                last_err = Some(format!("Part {part_number} attempt {attempt}: {e}"));
            }
        }
    }

    Err(AppError::Transfer(
        last_err.unwrap_or_else(|| "Upload part failed".into()),
    ))
}
