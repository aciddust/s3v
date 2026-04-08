use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct S3Object {
    pub key: String,
    pub size: i64,
    pub last_modified: Option<String>,
    pub is_folder: bool,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListObjectsResult {
    pub objects: Vec<S3Object>,
    pub common_prefixes: Vec<String>,
    pub is_truncated: bool,
    pub next_continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: i64,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MultipartUploadInfo {
    pub key: String,
    pub upload_id: String,
    pub initiated: Option<String>,
}
