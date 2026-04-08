<script lang="ts">
  import type { BucketInfo } from '$lib/api/s3';
  import { listObjects } from '$lib/api/s3';
  import { ChevronRight, ChevronDown, Package, Folder } from '@lucide/svelte';

  interface Props {
    profileId: string;
    buckets: BucketInfo[];
    activeBucket: string;
    activePrefix: string;
    onnavigate: (bucket: string, prefix: string) => void;
    onmovetoprefix?: (bucket: string, destPrefix: string, keys: string[]) => void;
  }

  const { profileId, buckets, activeBucket, activePrefix, onnavigate, onmovetoprefix }: Props =
    $props();

  let dragOverKey = $state<string | null>(null);

  function handleFolderDragOver(e: DragEvent, _bucket: string, _prefix: string) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dragOverKey = folderKey(_bucket, _prefix);
  }

  function handleFolderDragLeave() {
    dragOverKey = null;
  }

  function handleFolderDrop(e: DragEvent, bucket: string, prefix: string) {
    e.preventDefault();
    dragOverKey = null;
    const data = e.dataTransfer?.getData('application/x-s3v-keys');
    if (!data || !onmovetoprefix) return;
    try {
      const keys = JSON.parse(data) as string[];
      if (keys.length > 0) {
        onmovetoprefix(bucket, prefix, keys);
      }
    } catch {
      // ignore bad data
    }
  }

  // Track expanded buckets and folders
  let expandedBuckets = $state<Set<string>>(new Set());
  // Map of bucket -> Set of expanded folder prefixes
  let expandedFolders = $state<Map<string, Set<string>>>(new Map());
  // Map of bucket+prefix -> subfolders
  let subfolders = $state<Map<string, string[]>>(new Map());
  let loadingKeys = $state<Set<string>>(new Set());

  function bucketKey(bucket: string) {
    return `${bucket}:`;
  }

  function folderKey(bucket: string, prefix: string) {
    return `${bucket}:${prefix}`;
  }

  async function toggleBucket(bucket: string) {
    const key = bucketKey(bucket);
    const next = new Set(expandedBuckets);
    if (next.has(bucket)) {
      next.delete(bucket);
      expandedBuckets = next;
    } else {
      next.add(bucket);
      expandedBuckets = next;
      if (!subfolders.has(key)) {
        await loadSubfolders(bucket, '');
      }
    }
    onnavigate(bucket, '');
  }

  async function toggleFolder(bucket: string, prefix: string) {
    const bucketFolders = expandedFolders.get(bucket) ?? new Set<string>();
    const next = new Map(expandedFolders);
    const nextSet = new Set(bucketFolders);
    if (nextSet.has(prefix)) {
      nextSet.delete(prefix);
    } else {
      nextSet.add(prefix);
      const key = folderKey(bucket, prefix);
      if (!subfolders.has(key)) {
        await loadSubfolders(bucket, prefix);
      }
    }
    next.set(bucket, nextSet);
    expandedFolders = next;
    onnavigate(bucket, prefix);
  }

  async function loadSubfolders(bucket: string, prefix: string) {
    const key = folderKey(bucket, prefix);
    const loading = new Set(loadingKeys);
    loading.add(key);
    loadingKeys = loading;
    try {
      const result = await listObjects(profileId, bucket, prefix, '/');
      const nextSubs = new Map(subfolders);
      nextSubs.set(key, result.common_prefixes);
      subfolders = nextSubs;
    } catch {
      // silently fail
    } finally {
      const l = new Set(loadingKeys);
      l.delete(key);
      loadingKeys = l;
    }
  }

  function getBucketFolders(bucket: string): string[] {
    return subfolders.get(folderKey(bucket, '')) ?? [];
  }

  function getSubFolders(bucket: string, prefix: string): string[] {
    return subfolders.get(folderKey(bucket, prefix)) ?? [];
  }

  function isFolderExpanded(bucket: string, prefix: string): boolean {
    return expandedFolders.get(bucket)?.has(prefix) ?? false;
  }

  function getFolderName(prefix: string): string {
    const parts = prefix.replace(/\/$/, '').split('/');
    return parts.at(-1) ?? prefix;
  }
</script>

{#snippet folderTree(bucket: string, folders: string[], depth: number)}
  {#each folders as folder}
    {@const expanded = isFolderExpanded(bucket, folder)}
    {@const isActive = activeBucket === bucket && activePrefix === folder}
    {@const key = folderKey(bucket, folder)}
    <div style="padding-left: {depth * 12}px">
      <button
        class="flex w-full items-center gap-1 rounded px-1.5 py-0.5 text-xs
          {isActive
          ? 'bg-accent text-accent-foreground'
          : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}
          {dragOverKey === key ? 'ring-1 ring-primary bg-primary/10' : ''}
          transition-colors"
        onclick={() => toggleFolder(bucket, folder)}
        ondragover={(e) => handleFolderDragOver(e, bucket, folder)}
        ondragleave={handleFolderDragLeave}
        ondrop={(e) => handleFolderDrop(e, bucket, folder)}
      >
        {#if expanded}
          <ChevronDown class="h-3 w-3 shrink-0" />
        {:else}
          <ChevronRight class="h-3 w-3 shrink-0" />
        {/if}
        <Folder class="h-3 w-3 shrink-0 text-blue-400" />
        <span class="truncate">{getFolderName(folder)}</span>
      </button>

      {#if expanded}
        {@render folderTree(bucket, getSubFolders(bucket, folder), depth + 1)}
      {/if}
    </div>
  {/each}
{/snippet}

<div class="space-y-0.5 py-1">
  {#each buckets as bucket}
    {@const expanded = expandedBuckets.has(bucket.name)}
    {@const isActive = activeBucket === bucket.name && activePrefix === ''}
    <div>
      <button
        class="flex w-full items-center gap-1.5 rounded px-2 py-1 text-xs
          {isActive ? 'bg-accent text-accent-foreground' : 'text-foreground hover:bg-accent/50'}
          {dragOverKey === bucketKey(bucket.name) ? 'ring-1 ring-primary bg-primary/10' : ''}
          transition-colors"
        onclick={() => toggleBucket(bucket.name)}
        ondragover={(e) => handleFolderDragOver(e, bucket.name, '')}
        ondragleave={handleFolderDragLeave}
        ondrop={(e) => handleFolderDrop(e, bucket.name, '')}
      >
        {#if expanded}
          <ChevronDown class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        {:else}
          <ChevronRight class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        {/if}
        <Package class="h-3.5 w-3.5 shrink-0 text-amber-500" />
        <span class="truncate font-medium">{bucket.name}</span>
      </button>

      {#if expanded}
        {@render folderTree(bucket.name, getBucketFolders(bucket.name), 1)}
      {/if}
    </div>
  {/each}
</div>
