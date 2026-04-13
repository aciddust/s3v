<script lang="ts">
  import { store } from './store.svelte';
  import { Package, Folder, ChevronRight, ChevronDown } from '@lucide/svelte';

  let { activeBucket, activePath, onNavigate }: {
    activeBucket: string;
    activePath: string;
    onNavigate: (bucket: string, path: string) => void;
  } = $props();

  let expandedBuckets = $state<Set<string>>(new Set());
  let expandedPaths = $state<Set<string>>(new Set());

  // Auto-expand active bucket on first render
  $effect(() => {
    if (activeBucket && !expandedBuckets.has(activeBucket)) {
      expandedBuckets = new Set([...expandedBuckets, activeBucket]);
    }
  });

  function getFolderName(path: string): string {
    return path.replace(/\/$/, '').split('/').at(-1) ?? path;
  }

  function getTopFolders(bucket: string): string[] {
    const paths = store.getFolderPathsForBucket(bucket);
    return [...paths].filter((p) => p.split('/').filter(Boolean).length === 1).sort();
  }

  function getSubFolders(bucket: string, parentPath: string): string[] {
    const paths = store.getFolderPathsForBucket(bucket);
    const depth = parentPath.split('/').filter(Boolean).length;
    return [...paths].filter((p) => {
      if (!p.startsWith(parentPath) || p === parentPath) return false;
      return p.split('/').filter(Boolean).length === depth + 1;
    }).sort();
  }

  function toggleBucketExpand(bucket: string) {
    const next = new Set(expandedBuckets);
    if (next.has(bucket)) next.delete(bucket);
    else next.add(bucket);
    expandedBuckets = next;
  }

  function toggleFolderExpand(path: string) {
    const next = new Set(expandedPaths);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    expandedPaths = next;
  }
</script>

<div class="flex h-full w-36 flex-col border-r border-border bg-background shrink-0 overflow-y-auto lg:w-44">
  <div class="space-y-0.5 py-1 px-1">
    {#each store.currentBuckets as bucket}
      {@const isExpanded = expandedBuckets.has(bucket)}
      {@const isActive = activeBucket === bucket && activePath === ''}

      <button
        class="flex w-full items-center gap-1 rounded px-2 py-1 text-xs transition-colors
          {isActive ? 'bg-accent text-accent-foreground' : 'text-foreground hover:bg-accent/50'}"
        onclick={() => { toggleBucketExpand(bucket); onNavigate(bucket, ''); }}
      >
        <span class="shrink-0" onclick={(e) => { e.stopPropagation(); toggleBucketExpand(bucket); }}>
          {#if isExpanded}<ChevronDown class="h-3.5 w-3.5 text-muted-foreground" />{:else}<ChevronRight class="h-3.5 w-3.5 text-muted-foreground" />{/if}
        </span>
        <Package class="h-3.5 w-3.5 shrink-0 text-amber-500" />
        <span class="truncate font-medium">{bucket}</span>
      </button>

      {#if isExpanded}
        {#each getTopFolders(bucket) as folderPath}
          {@const folderExpanded = expandedPaths.has(`${bucket}:${folderPath}`)}
          {@const folderActive = activeBucket === bucket && activePath === folderPath}
          {@const subs = getSubFolders(bucket, folderPath)}
          <div style="padding-left: 12px;">
            <button
              class="flex w-full items-center gap-1 rounded px-1.5 py-0.5 text-xs transition-colors
                {folderActive ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
              onclick={() => { if (subs.length) toggleFolderExpand(`${bucket}:${folderPath}`); onNavigate(bucket, folderPath); }}
            >
              {#if subs.length > 0}
                {#if folderExpanded}<ChevronDown class="h-3 w-3 shrink-0" />{:else}<ChevronRight class="h-3 w-3 shrink-0" />{/if}
              {:else}
                <div class="w-3"></div>
              {/if}
              <Folder class="h-3 w-3 shrink-0 text-blue-400" />
              <span class="truncate">{getFolderName(folderPath)}</span>
            </button>

            {#if folderExpanded}
              {#each subs as subPath}
                {@const subActive = activeBucket === bucket && activePath === subPath}
                <div style="padding-left: 12px;">
                  <button
                    class="flex w-full items-center gap-1 rounded px-1.5 py-0.5 text-xs transition-colors
                      {subActive ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
                    onclick={() => onNavigate(bucket, subPath)}
                  >
                    <div class="w-3"></div>
                    <Folder class="h-3 w-3 shrink-0 text-blue-400" />
                    <span class="truncate">{getFolderName(subPath)}</span>
                  </button>
                </div>
              {/each}
            {/if}
          </div>
        {/each}
      {/if}
    {/each}
  </div>
</div>
