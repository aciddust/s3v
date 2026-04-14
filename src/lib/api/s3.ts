import { invoke } from '@tauri-apps/api/core';

export interface BucketInfo {
  name: string;
  creation_date: string | null;
}

export interface S3Object {
  key: string;
  size: number;
  last_modified: string | null;
  is_folder: boolean;
  content_type: string | null;
}

export interface ListObjectsResult {
  objects: S3Object[];
  common_prefixes: string[];
  is_truncated: boolean;
  next_continuation_token: string | null;
}

export interface ObjectMetadata {
  key: string;
  size: number;
  last_modified: string | null;
  content_type: string | null;
  etag: string | null;
}

export function listBuckets(profileId: string): Promise<BucketInfo[]> {
  return invoke('list_buckets', { profileId });
}

export function listObjects(
  profileId: string,
  bucket: string,
  prefix: string,
  delimiter: string | null = '/',
  continuationToken: string | null = null,
): Promise<ListObjectsResult> {
  return invoke('list_objects', {
    profileId,
    bucket,
    prefix,
    delimiter,
    continuationToken,
  });
}

export function headObject(
  profileId: string,
  bucket: string,
  key: string,
): Promise<ObjectMetadata> {
  return invoke('head_object', { profileId, bucket, key });
}

export function deleteObjects(profileId: string, bucket: string, keys: string[]): Promise<void> {
  return invoke('delete_objects', { profileId, bucket, keys });
}

export function renameObject(
  profileId: string,
  bucket: string,
  oldKey: string,
  newKey: string,
): Promise<void> {
  return invoke('rename_object', { profileId, bucket, oldKey, newKey });
}

export function renameFolder(
  profileId: string,
  bucket: string,
  oldPrefix: string,
  newPrefix: string,
): Promise<void> {
  return invoke('rename_folder', { profileId, bucket, oldPrefix, newPrefix });
}

export function copyObject(
  profileId: string,
  sourceBucket: string,
  sourceKey: string,
  destBucket: string,
  destKey: string,
): Promise<void> {
  return invoke('copy_object', {
    profileId,
    sourceBucket,
    sourceKey,
    destBucket,
    destKey,
  });
}

export function moveObjects(
  profileId: string,
  bucket: string,
  keys: string[],
  destPrefix: string,
): Promise<void> {
  return invoke('move_objects', { profileId, bucket, keys, destPrefix });
}

export function createFolder(profileId: string, bucket: string, prefix: string): Promise<void> {
  return invoke('create_folder', { profileId, bucket, prefix });
}

export function getPresignedUrl(
  profileId: string,
  bucket: string,
  key: string,
  expirySecs: number,
): Promise<string> {
  return invoke('get_presigned_url', { profileId, bucket, key, expirySecs });
}

export interface MultipartUploadInfo {
  key: string;
  upload_id: string;
  initiated: string | null;
}

export function listMultipartUploads(
  profileId: string,
  bucket: string,
): Promise<MultipartUploadInfo[]> {
  return invoke('list_multipart_uploads', { profileId, bucket });
}

export function abortMultipartUpload(
  profileId: string,
  bucket: string,
  key: string,
  uploadId: string,
): Promise<void> {
  return invoke('abort_multipart_upload', { profileId, bucket, key, uploadId });
}

export interface ClassifiedPaths {
  files: string[];
  directories: string[];
}

export function classifyPaths(paths: string[]): Promise<ClassifiedPaths> {
  return invoke('classify_paths', { paths });
}

export function checkConflicts(profileId: string, bucket: string, keys: string[]): Promise<string[]> {
  return invoke('check_conflicts', { profileId, bucket, keys });
}

export function copyFolder(
  profileId: string, sourceBucket: string, sourcePrefix: string,
  destBucket: string, destPrefix: string,
  skipKeys: string[] = [], renameKeys: string[] = []
): Promise<string> {
  return invoke('copy_folder', { profileId, sourceBucket, sourcePrefix, destBucket, destPrefix, skipKeys, renameKeys });
}

export function moveFolder(
  profileId: string, sourceBucket: string, sourcePrefix: string,
  destBucket: string, destPrefix: string,
  skipKeys: string[] = [], renameKeys: string[] = []
): Promise<string> {
  return invoke('move_folder', { profileId, sourceBucket, sourcePrefix, destBucket, destPrefix, skipKeys, renameKeys });
}

export function cancelFolderOp(opId: string): Promise<void> {
  return invoke('cancel_folder_op', { opId });
}

export function crossProfileCopyObject(
  sourceProfileId: string, sourceBucket: string, sourceKey: string,
  destProfileId: string, destBucket: string, destKey: string,
): Promise<void> {
  return invoke('cross_profile_copy_object', {
    sourceProfileId, sourceBucket, sourceKey, destProfileId, destBucket, destKey,
  });
}

export function crossProfileCopyFolder(
  sourceProfileId: string, sourceBucket: string, sourcePrefix: string,
  destProfileId: string, destBucket: string, destPrefix: string,
): Promise<string> {
  return invoke('cross_profile_copy_folder', {
    sourceProfileId, sourceBucket, sourcePrefix, destProfileId, destBucket, destPrefix,
  });
}
