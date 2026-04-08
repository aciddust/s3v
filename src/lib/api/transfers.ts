import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type TransferType = 'upload' | 'download';

export type TransferStatus = 'queued' | 'active' | 'paused' | 'completed' | 'failed' | 'cancelled';

export interface TransferPath {
  local: string;
  remote_bucket: string;
  remote_key: string;
}

export interface TransferProgress {
  bytes_transferred: number;
  total_bytes: number;
}

export interface TransferJobSummary {
  id: string;
  profile_id: string;
  transfer_type: TransferType;
  path: TransferPath;
  status: TransferStatus;
  progress: TransferProgress;
  error: string | null;
}

export interface TransferProgressEvent {
  id: string;
  bytes_transferred: number;
  total_bytes: number;
}

export interface TransferStatusEvent {
  id: string;
  status: TransferStatus;
  error: string | null;
}

export interface TransferCompletedEvent {
  id: string;
  transfer_type: TransferType;
  path: TransferPath;
}

export function enqueueUpload(
  profileId: string,
  localPath: string,
  bucket: string,
  key: string,
): Promise<string> {
  return invoke('enqueue_upload', { profileId, localPath, bucket, key });
}

export function enqueueDownload(
  profileId: string,
  bucket: string,
  key: string,
  localPath: string,
): Promise<string> {
  return invoke('enqueue_download', { profileId, bucket, key, localPath });
}

export function pauseTransfer(id: string): Promise<void> {
  return invoke('pause_transfer', { id });
}

export function resumeTransfer(id: string): Promise<void> {
  return invoke('resume_transfer', { id });
}

export function cancelTransfer(id: string): Promise<void> {
  return invoke('cancel_transfer', { id });
}

export function listTransfers(): Promise<TransferJobSummary[]> {
  return invoke('list_transfers');
}

export function onTransferProgress(
  handler: (event: TransferProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<TransferProgressEvent>('transfer:progress', (e) => handler(e.payload));
}

export function onTransferStatusChanged(
  handler: (event: TransferStatusEvent) => void,
): Promise<UnlistenFn> {
  return listen<TransferStatusEvent>('transfer:status', (e) => handler(e.payload));
}

export function onTransferCompleted(
  handler: (event: TransferCompletedEvent) => void,
): Promise<UnlistenFn> {
  return listen<TransferCompletedEvent>('transfer:completed', (e) => handler(e.payload));
}
