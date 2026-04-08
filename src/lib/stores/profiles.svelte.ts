import { listProfiles, type ProfileSummary } from '$lib/api/profiles';
import { listBuckets, type BucketInfo } from '$lib/api/s3';
import { fileStore } from '$lib/stores/files.svelte';

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface ProfileTab {
  profileId: string;
  profile: ProfileSummary;
  buckets: BucketInfo[];
  connected: boolean;
  status: ConnectionStatus;
  error: string | null;
}

class ProfileStore {
  tabs = $state<ProfileTab[]>([]);
  activeTabIndex = $state<number>(0);

  activeTab = $derived<ProfileTab | null>(this.tabs[this.activeTabIndex] ?? null);

  activeProfileId = $derived<string | null>(this.tabs[this.activeTabIndex]?.profileId ?? null);

  async loadProfiles(): Promise<void> {
    const profiles = await listProfiles();
    const profileMap = new Map(profiles.map((p) => [p.id, p]));

    // Remove tabs for deleted profiles, update profile info for existing ones
    this.tabs = this.tabs
      .filter((t) => profileMap.has(t.profileId))
      .map((t) => {
        const updated = profileMap.get(t.profileId)!;
        return t.profile.name !== updated.name || t.profile.provider !== updated.provider
          ? { ...t, profile: updated }
          : t;
      });

    // Keep activeTabIndex in bounds
    if (this.activeTabIndex >= this.tabs.length) {
      this.activeTabIndex = Math.max(0, this.tabs.length - 1);
    }
  }

  async openTab(profile: ProfileSummary): Promise<void> {
    const existing = this.tabs.findIndex((t) => t.profileId === profile.id);
    if (existing >= 0) {
      this.activeTabIndex = existing;
      return;
    }

    const tab: ProfileTab = {
      profileId: profile.id,
      profile,
      buckets: [],
      connected: false,
      status: 'connecting',
      error: null,
    };
    this.tabs = [...this.tabs, tab];
    this.activeTabIndex = this.tabs.length - 1;

    await this.refreshBuckets(profile.id);

    // Auto-navigate to the first bucket
    const updated = this.tabs.find((t) => t.profileId === profile.id);
    if (updated?.connected && updated.buckets.length > 0) {
      await fileStore.navigate(profile.id, updated.buckets[0].name, '');
    }
  }

  closeTab(profileId: string): void {
    const idx = this.tabs.findIndex((t) => t.profileId === profileId);
    if (idx < 0) return;
    this.tabs = this.tabs.filter((t) => t.profileId !== profileId);
    if (this.activeTabIndex >= this.tabs.length) {
      this.activeTabIndex = Math.max(0, this.tabs.length - 1);
    }
  }

  setActiveTab(profileId: string): void {
    const idx = this.tabs.findIndex((t) => t.profileId === profileId);
    if (idx >= 0) this.activeTabIndex = idx;
  }

  async refreshBuckets(profileId: string): Promise<void> {
    const idx = this.tabs.findIndex((t) => t.profileId === profileId);
    if (idx < 0) return;

    // Set connecting status
    this.tabs = this.tabs.map((t, i) =>
      i === idx ? { ...t, status: 'connecting' as const, error: null } : t,
    );

    const tab = this.tabs[idx];
    const defaultBucket = tab.profile.default_bucket;

    // If default_bucket is set, skip listBuckets entirely (bucket-scoped tokens can't call it)
    if (defaultBucket) {
      const buckets: BucketInfo[] = [{ name: defaultBucket, creation_date: null }];
      this.tabs = this.tabs.map((t, i) =>
        i === idx
          ? {
              ...t,
              buckets,
              connected: true,
              status: 'connected' as const,
              error: null,
            }
          : t,
      );
      return;
    }

    try {
      const buckets = await listBuckets(profileId);
      this.tabs = this.tabs.map((t, i) =>
        i === idx
          ? {
              ...t,
              buckets,
              connected: true,
              status: 'connected' as const,
              error: null,
            }
          : t,
      );
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      this.tabs = this.tabs.map((t, i) =>
        i === idx
          ? {
              ...t,
              buckets: [],
              connected: false,
              status: 'error' as const,
              error:
                message +
                '\n\nTip: If your token is scoped to a specific bucket, set "Default Bucket" in the profile.',
            }
          : t,
      );
    }
  }
}

export const profileStore = new ProfileStore();
