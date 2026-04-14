use super::types::{BucketInfo, ListObjectsResult, MultipartUploadInfo, ObjectMetadata, S3Object};
use crate::error::AppError;
use aws_sdk_s3::Client as S3Client;

pub async fn list_buckets(client: &S3Client) -> Result<Vec<BucketInfo>, AppError> {
    let output = client
        .list_buckets()
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to list buckets: {e}")))?;
    Ok(output
        .buckets()
        .iter()
        .map(|b| BucketInfo {
            name: b.name().unwrap_or_default().to_string(),
            creation_date: b.creation_date().map(|d| d.to_string()),
        })
        .collect())
}

pub async fn list_objects(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
    delimiter: Option<&str>,
    continuation_token: Option<&str>,
) -> Result<ListObjectsResult, AppError> {
    let mut request = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .max_keys(1000);
    if let Some(d) = delimiter {
        request = request.delimiter(d);
    }
    if let Some(token) = continuation_token {
        request = request.continuation_token(token);
    }
    let output = request
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to list objects: {e}")))?;

    crate::logger::debug("s3", format!(
        "list_objects_v2 raw: bucket={}, prefix='{}', key_count={}, contents={}, common_prefixes={}",
        bucket,
        prefix,
        output.key_count().unwrap_or(0),
        output.contents().len(),
        output.common_prefixes().len(),
    ));
    for obj in output.contents() {
        crate::logger::debug(
            "s3",
            format!(
                "  object: key='{}', size={}",
                obj.key().unwrap_or_default(),
                obj.size().unwrap_or(0),
            ),
        );
    }
    for cp in output.common_prefixes() {
        crate::logger::debug(
            "s3",
            format!("  prefix: '{}'", cp.prefix().unwrap_or_default(),),
        );
    }

    let objects = output
        .contents()
        .iter()
        .filter(|obj| {
            let key = obj.key().unwrap_or_default();
            key != prefix
        })
        .map(|obj| {
            let key = obj.key().unwrap_or_default().to_string();
            S3Object {
                key: key.clone(),
                size: obj.size().unwrap_or(0),
                last_modified: obj.last_modified().map(|d| d.to_string()),
                is_folder: key.ends_with('/'),
                content_type: None,
            }
        })
        .collect();
    let common_prefixes = output
        .common_prefixes()
        .iter()
        .filter_map(|cp| cp.prefix().map(|s| s.to_string()))
        .collect();
    Ok(ListObjectsResult {
        objects,
        common_prefixes,
        is_truncated: output.is_truncated().unwrap_or(false),
        next_continuation_token: output.next_continuation_token().map(|s| s.to_string()),
    })
}

pub async fn head_object(
    client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<ObjectMetadata, AppError> {
    let output = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to head object: {e}")))?;
    Ok(ObjectMetadata {
        key: key.to_string(),
        size: output.content_length().unwrap_or(0),
        last_modified: output.last_modified().map(|d| d.to_string()),
        content_type: output.content_type().map(|s| s.to_string()),
        etag: output.e_tag().map(|s| s.to_string()),
    })
}

pub async fn delete_objects(
    client: &S3Client,
    bucket: &str,
    keys: &[String],
) -> Result<(), AppError> {
    use aws_sdk_s3::types::{Delete, ObjectIdentifier};
    let objects: Vec<ObjectIdentifier> = keys
        .iter()
        .map(|key| ObjectIdentifier::builder().key(key).build())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::S3(format!("Failed to build object identifier: {e}")))?;
    let delete = Delete::builder()
        .set_objects(Some(objects))
        .build()
        .map_err(|e| AppError::S3(format!("Failed to build delete request: {e}")))?;
    client
        .delete_objects()
        .bucket(bucket)
        .delete(delete)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to delete objects: {e}")))?;
    Ok(())
}

pub async fn copy_object(
    client: &S3Client,
    source_bucket: &str,
    source_key: &str,
    dest_bucket: &str,
    dest_key: &str,
) -> Result<(), AppError> {
    let encoded_key = urlencoding::encode(source_key);
    let copy_source = format!("{source_bucket}/{encoded_key}");
    client
        .copy_object()
        .copy_source(&copy_source)
        .bucket(dest_bucket)
        .key(dest_key)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to copy object: {e}")))?;
    Ok(())
}

/// Cross-profile copy: download from source client, upload to dest client.
/// Works across different S3 endpoints (e.g., AWS → R2, MinIO → S3).
pub async fn cross_profile_copy_object(
    source_client: &S3Client,
    dest_client: &S3Client,
    source_bucket: &str,
    source_key: &str,
    dest_bucket: &str,
    dest_key: &str,
) -> Result<(), AppError> {
    // Download from source
    let resp = source_client
        .get_object()
        .bucket(source_bucket)
        .key(source_key)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to get object from source: {e}")))?;

    let body = resp
        .body
        .collect()
        .await
        .map_err(|e| AppError::S3(format!("Failed to read object body: {e}")))?;
    let bytes = body.into_bytes();

    // Upload to destination
    dest_client
        .put_object()
        .bucket(dest_bucket)
        .key(dest_key)
        .body(aws_sdk_s3::primitives::ByteStream::from(bytes))
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to put object to destination: {e}")))?;

    Ok(())
}

pub async fn rename_object(
    client: &S3Client,
    bucket: &str,
    old_key: &str,
    new_key: &str,
) -> Result<(), AppError> {
    copy_object(client, bucket, old_key, bucket, new_key).await?;
    delete_objects(client, bucket, &[old_key.to_string()]).await?;
    Ok(())
}

/// Rename a folder by listing all objects under old_prefix and copying them to new_prefix.
pub async fn rename_folder(
    client: &S3Client,
    bucket: &str,
    old_prefix: &str,
    new_prefix: &str,
) -> Result<(), AppError> {
    // Collect all keys under old_prefix (no delimiter = recursive)
    let mut keys = Vec::new();
    let mut continuation_token: Option<String> = None;
    loop {
        let result = list_objects(
            client,
            bucket,
            old_prefix,
            None,
            continuation_token.as_deref(),
        )
        .await?;
        for obj in &result.objects {
            keys.push(obj.key.clone());
        }
        if result.is_truncated {
            continuation_token = result.next_continuation_token;
        } else {
            break;
        }
    }

    // Copy each object to new prefix
    for key in &keys {
        let suffix = key.strip_prefix(old_prefix).unwrap_or(key);
        let new_key = format!("{new_prefix}{suffix}");
        copy_object(client, bucket, key, bucket, &new_key).await?;
    }

    // Create new folder marker (handles empty folders)
    create_folder(client, bucket, new_prefix).await?;

    // Delete all old objects + old folder marker
    let mut all_old_keys = keys;
    all_old_keys.push(old_prefix.to_string());
    delete_objects(client, bucket, &all_old_keys).await?;

    Ok(())
}

/// Move = (copy + delete) per file.
/// `on_progress(done, total, key, phase)` where phase is "copying" | "deleting" | "moved".
pub async fn move_objects(
    client: &S3Client,
    bucket: &str,
    keys: &[String],
    dest_prefix: &str,
    on_progress: impl Fn(usize, usize, &str, &str),
) -> Result<(), AppError> {
    let total = keys.len();
    for (i, key) in keys.iter().enumerate() {
        let filename = key.rsplit('/').next().unwrap_or(key);
        let dest_key = format!("{dest_prefix}{filename}");

        on_progress(i, total, key, "copying");
        copy_object(client, bucket, key, bucket, &dest_key).await?;

        on_progress(i, total, key, "deleting");
        delete_objects(client, bucket, &[key.clone()]).await?;

        on_progress(i + 1, total, key, "moved");
    }
    Ok(())
}

pub async fn create_folder(client: &S3Client, bucket: &str, prefix: &str) -> Result<(), AppError> {
    let key = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };
    client
        .put_object()
        .bucket(bucket)
        .key(&key)
        .content_length(0)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to create folder: {e}")))?;
    Ok(())
}

pub async fn get_presigned_url(
    client: &S3Client,
    bucket: &str,
    key: &str,
    expiry_secs: u64,
) -> Result<String, AppError> {
    use aws_sdk_s3::presigning::PresigningConfig;
    use std::time::Duration;
    let presigning_config = PresigningConfig::builder()
        .expires_in(Duration::from_secs(expiry_secs))
        .build()
        .map_err(|e| AppError::S3(format!("Failed to build presigning config: {e}")))?;
    let presigned = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning_config)
        .await
        .map_err(|e| AppError::S3(format!("Failed to generate presigned URL: {e}")))?;
    Ok(presigned.uri().to_string())
}

pub async fn list_multipart_uploads(
    client: &S3Client,
    bucket: &str,
) -> Result<Vec<MultipartUploadInfo>, AppError> {
    let output = client
        .list_multipart_uploads()
        .bucket(bucket)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to list multipart uploads: {e}")))?;

    Ok(output
        .uploads()
        .iter()
        .map(|u| MultipartUploadInfo {
            key: u.key().unwrap_or_default().to_string(),
            upload_id: u.upload_id().unwrap_or_default().to_string(),
            initiated: u.initiated().map(|d| d.to_string()),
        })
        .collect())
}

pub async fn abort_multipart_upload(
    client: &S3Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
) -> Result<(), AppError> {
    client
        .abort_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .send()
        .await
        .map_err(|e| AppError::S3(format!("Failed to abort multipart upload: {e}")))?;
    Ok(())
}

/// List all object keys under a prefix recursively (no delimiter).
/// Handles pagination via continuation tokens.
pub async fn list_all_objects(
    client: &S3Client,
    bucket: &str,
    prefix: &str,
) -> Result<Vec<String>, AppError> {
    let mut keys = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut req = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix);

        if let Some(token) = &continuation_token {
            req = req.continuation_token(token);
        }

        let resp = req.send().await.map_err(|e| AppError::S3(e.to_string()))?;

        for obj in resp.contents() {
            if let Some(key) = obj.key() {
                if key != prefix {
                    keys.push(key.to_string());
                }
            }
        }

        match resp.next_continuation_token() {
            Some(token) => continuation_token = Some(token.to_string()),
            None => break,
        }
    }

    Ok(keys)
}

/// Given a key that conflicts, generate a renamed version with suffix.
/// "folder/file.txt" → "folder/file (1).txt"
/// "folder/" → "folder (1)/"
pub fn resolve_rename_key(key: &str) -> String {
    if key.ends_with('/') {
        let base = key.trim_end_matches('/');
        format!("{} (1)/", base)
    } else if let Some(dot_pos) = key.rfind('.') {
        let (name, ext) = key.split_at(dot_pos);
        format!("{} (1){}", name, ext)
    } else {
        format!("{} (1)", key)
    }
}
