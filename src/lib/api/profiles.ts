import { invoke } from '@tauri-apps/api/core';

export type Provider = 'aws' | 'minio' | 'r2' | 'custom';

export interface ProfileSummary {
  id: string;
  name: string;
  provider: Provider;
  default_bucket: string | null;
}

export interface ProfileInput {
  name: string;
  provider: Provider;
  endpoint: string | null;
  region: string;
  access_key_id: string;
  secret_access_key: string;
  path_style: boolean;
  default_bucket: string | null;
}

export interface ConnectionResult {
  success: boolean;
  message: string;
  bucket_count: number | null;
}

export interface ProfileDetail {
  id: string;
  name: string;
  provider: Provider;
  endpoint: string | null;
  region: string;
  access_key_id: string;
  secret_access_key: string;
  path_style: boolean;
  default_bucket: string | null;
}

export function listProfiles(): Promise<ProfileSummary[]> {
  return invoke('list_profiles');
}

export function getProfile(id: string): Promise<ProfileDetail> {
  return invoke('get_profile', { id });
}

export function createProfile(profile: ProfileInput): Promise<string> {
  return invoke('create_profile', { profile });
}

export function updateProfile(id: string, profile: ProfileInput): Promise<void> {
  return invoke('update_profile', { id, profile });
}

export function deleteProfile(id: string): Promise<void> {
  return invoke('delete_profile', { id });
}

export function testConnection(id: string): Promise<ConnectionResult> {
  return invoke('test_connection', { id });
}
