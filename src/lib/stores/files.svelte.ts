import { listObjects, type S3Object } from '$lib/api/s3';

export type SortField = 'name' | 'size' | 'lastModified';
export type SortOrder = 'asc' | 'desc';

interface HistoryEntry {
  bucket: string;
  prefix: string;
}

export interface FileState {
  bucket: string;
  prefix: string;
  objects: S3Object[];
  folders: string[];
  selected: Set<string>;
  sortField: SortField;
  sortOrder: SortOrder;
  loading: boolean;
  loadingMore: boolean;
  continuationToken: string | null;
  hasMore: boolean;
  historyBack: HistoryEntry[];
  historyForward: HistoryEntry[];
}

function makeDefaultFileState(): FileState {
  return {
    bucket: '',
    prefix: '',
    objects: [],
    folders: [],
    selected: new Set(),
    sortField: 'name',
    sortOrder: 'asc',
    loading: false,
    loadingMore: false,
    continuationToken: null,
    hasMore: false,
    historyBack: [],
    historyForward: [],
  };
}

class FileStore {
  private stateMap = $state<Map<string, FileState>>(new Map());

  /** Extract the real profile ID from a store key (strips '::right' suffix). */
  private resolveProfileId(storeKey: string): string {
    return storeKey.replace(/::right$/, '');
  }

  getState(storeKey: string): FileState {
    return this.stateMap.get(storeKey) ?? makeDefaultFileState();
  }

  private updateState(storeKey: string, patch: Partial<FileState>): void {
    const current = this.getState(storeKey);
    const next = new Map(this.stateMap);
    next.set(storeKey, { ...current, ...patch });
    this.stateMap = next;
  }

  async navigate(storeKey: string, bucket: string, prefix: string, skipHistory = false): Promise<void> {
    const profileId = this.resolveProfileId(storeKey);
    const current = this.getState(storeKey);

    // Push current location to back history (unless navigating via back/forward, first load, or same location)
    const historyPatch: Partial<FileState> = {};
    const sameLocation = current.bucket === bucket && current.prefix === prefix;
    if (!skipHistory && current.bucket && !sameLocation) {
      historyPatch.historyBack = [...current.historyBack, { bucket: current.bucket, prefix: current.prefix }];
      historyPatch.historyForward = [];
    }

    this.updateState(storeKey, {
      bucket,
      prefix,
      loading: true,
      selected: new Set(),
      objects: [],
      folders: [],
      continuationToken: null,
      hasMore: false,
      ...historyPatch,
    });
    try {
      const result = await listObjects(profileId, bucket, prefix);
      this.updateState(storeKey, {
        objects: result.objects.filter((o) => !o.is_folder),
        folders: result.common_prefixes,
        loading: false,
        continuationToken: result.next_continuation_token,
        hasMore: result.is_truncated,
      });
    } catch (e) {
      console.error('[files] navigate failed:', e);
      this.updateState(storeKey, {
        objects: [],
        folders: [],
        loading: false,
        hasMore: false,
      });
    }
  }

  async loadMore(storeKey: string): Promise<void> {
    const state = this.getState(storeKey);
    if (!state.bucket || !state.hasMore || state.loadingMore || !state.continuationToken) return;

    const profileId = this.resolveProfileId(storeKey);
    this.updateState(storeKey, { loadingMore: true });
    try {
      const result = await listObjects(
        profileId,
        state.bucket,
        state.prefix,
        '/',
        state.continuationToken,
      );
      this.updateState(storeKey, {
        objects: [...state.objects, ...result.objects.filter((o) => !o.is_folder)],
        folders: [...state.folders, ...result.common_prefixes],
        loadingMore: false,
        continuationToken: result.next_continuation_token,
        hasMore: result.is_truncated,
      });
    } catch (e) {
      console.error('[files] loadMore failed:', e);
      this.updateState(storeKey, { loadingMore: false });
    }
  }

  /** Copy current left-panel state to right panel so it starts at the same location. */
  async initRightPanel(profileId: string): Promise<void> {
    const rightKey = `${profileId}::right`;
    const left = this.getState(profileId);
    if (left.bucket) {
      await this.navigate(rightKey, left.bucket, left.prefix);
    }
  }

  async refresh(storeKey: string): Promise<void> {
    const state = this.getState(storeKey);
    if (!state.bucket) return;
    await this.navigate(storeKey, state.bucket, state.prefix, true);
  }

  canGoBack(storeKey: string): boolean {
    return this.getState(storeKey).historyBack.length > 0;
  }

  canGoForward(storeKey: string): boolean {
    return this.getState(storeKey).historyForward.length > 0;
  }

  async goBack(storeKey: string): Promise<void> {
    const state = this.getState(storeKey);
    if (state.historyBack.length === 0) return;
    const prev = state.historyBack[state.historyBack.length - 1];
    const newBack = state.historyBack.slice(0, -1);
    const newForward = [...state.historyForward, { bucket: state.bucket, prefix: state.prefix }];
    const profileId = this.resolveProfileId(storeKey);
    this.updateState(storeKey, {
      bucket: prev.bucket,
      prefix: prev.prefix,
      loading: true,
      selected: new Set(),
      objects: [],
      folders: [],
      continuationToken: null,
      hasMore: false,
      historyBack: newBack,
      historyForward: newForward,
    });
    try {
      const result = await listObjects(profileId, prev.bucket, prev.prefix);
      this.updateState(storeKey, {
        objects: result.objects.filter((o) => !o.is_folder),
        folders: result.common_prefixes,
        loading: false,
        continuationToken: result.next_continuation_token,
        hasMore: result.is_truncated,
      });
    } catch (e) {
      console.error('[files] goBack failed:', e);
      this.updateState(storeKey, { loading: false });
    }
  }

  async goForward(storeKey: string): Promise<void> {
    const state = this.getState(storeKey);
    if (state.historyForward.length === 0) return;
    const next = state.historyForward[state.historyForward.length - 1];
    const newForward = state.historyForward.slice(0, -1);
    const newBack = [...state.historyBack, { bucket: state.bucket, prefix: state.prefix }];
    const profileId = this.resolveProfileId(storeKey);
    this.updateState(storeKey, {
      bucket: next.bucket,
      prefix: next.prefix,
      loading: true,
      selected: new Set(),
      objects: [],
      folders: [],
      continuationToken: null,
      hasMore: false,
      historyBack: newBack,
      historyForward: newForward,
    });
    try {
      const result = await listObjects(profileId, next.bucket, next.prefix);
      this.updateState(storeKey, {
        objects: result.objects.filter((o) => !o.is_folder),
        folders: result.common_prefixes,
        loading: false,
        continuationToken: result.next_continuation_token,
        hasMore: result.is_truncated,
      });
    } catch (e) {
      console.error('[files] goForward failed:', e);
      this.updateState(storeKey, { loading: false });
    }
  }

  toggleSelect(profileId: string, key: string): void {
    const state = this.getState(profileId);
    const next = new Set(state.selected);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    this.updateState(profileId, { selected: next });
  }

  selectRange(profileId: string, fromKey: string, toKey: string): void {
    const state = this.getState(profileId);
    const sorted = this.getSortedItems(profileId);
    const allKeys = sorted.map((item) => (typeof item === 'string' ? item : item.key));
    const fromIdx = allKeys.indexOf(fromKey);
    const toIdx = allKeys.indexOf(toKey);
    if (fromIdx < 0 || toIdx < 0) return;
    const [start, end] = fromIdx <= toIdx ? [fromIdx, toIdx] : [toIdx, fromIdx];
    const next = new Set(state.selected);
    for (let i = start; i <= end; i++) {
      next.add(allKeys[i]);
    }
    this.updateState(profileId, { selected: next });
  }

  selectAll(profileId: string): void {
    const sorted = this.getSortedItems(profileId);
    const keys = sorted.map((item) => (typeof item === 'string' ? item : item.key));
    this.updateState(profileId, { selected: new Set(keys) });
  }

  clearSelection(profileId: string): void {
    this.updateState(profileId, { selected: new Set() });
  }

  setSort(profileId: string, field: SortField, order: SortOrder): void {
    this.updateState(profileId, { sortField: field, sortOrder: order });
  }

  /** Returns folders first (sorted by name), then files sorted by sortField/sortOrder. */
  getSortedItems(profileId: string): (string | S3Object)[] {
    const state = this.getState(profileId);
    const sortedFolders = [...state.folders].sort((a, b) => a.localeCompare(b));

    const sortedFiles = [...state.objects].sort((a, b) => {
      let cmp = 0;
      if (state.sortField === 'name') {
        cmp = a.key.localeCompare(b.key);
      } else if (state.sortField === 'size') {
        cmp = a.size - b.size;
      } else if (state.sortField === 'lastModified') {
        const aTime = a.last_modified ? new Date(a.last_modified).getTime() : 0;
        const bTime = b.last_modified ? new Date(b.last_modified).getTime() : 0;
        cmp = aTime - bTime;
      }
      return state.sortOrder === 'asc' ? cmp : -cmp;
    });

    return [...sortedFolders, ...sortedFiles];
  }
}

export const fileStore = new FileStore();
