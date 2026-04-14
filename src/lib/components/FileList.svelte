<script lang="ts">
  import { onMount } from 'svelte';
  import { fileStore, type SortField } from '$lib/stores/files.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { dragStore } from '$lib/stores/drag.svelte';
  import type { S3Object } from '$lib/api/s3';
  import FileRow from './FileRow.svelte';
  import { ChevronUp, ChevronDown, CornerLeftUp } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    profileId: string;
    onnavigate: (bucket: string, prefix: string) => void;
    oncontextmenu: (e: MouseEvent, keys: string[]) => void;
    onbgcontextmenu?: (e: MouseEvent) => void;
    onmovetoprefix?: (bucket: string, destPrefix: string, keys: string[]) => void;
    oncopytoprefix?: (
      sourceProfileId: string,
      destProfileId: string,
      sourceBucket: string,
      destBucket: string,
      destPrefix: string,
      keys: string[],
    ) => void;
    onfileopen?: (bucket: string, key: string) => void;
  }

  const {
    profileId,
    onnavigate,
    oncontextmenu,
    onbgcontextmenu,
    onmovetoprefix,
    oncopytoprefix,
    onfileopen,
  }: Props = $props();

  const fs = $derived(fileStore.getState(profileId));
  const sortedItems = $derived(fileStore.getSortedItems(profileId));
  const searchQuery = $derived(uiStore.searchQuery.toLowerCase());
  const hasParent = $derived(fs.prefix.length > 0);

  const filteredItems = $derived(
    searchQuery
      ? sortedItems.filter((item) => {
          const key = typeof item === 'string' ? item : item.key;
          return key.toLowerCase().includes(searchQuery);
        })
      : sortedItems,
  );

  let lastClickedKey = $state<string | null>(null);
  let containerEl: HTMLDivElement | undefined = $state();

  function getItemKey(item: string | S3Object): string {
    return typeof item === 'string' ? item : item.key;
  }

  function isFolder(item: string | S3Object): boolean {
    return typeof item === 'string';
  }

  function handleClick(e: MouseEvent, key: string) {
    if (e.shiftKey && lastClickedKey) {
      fileStore.selectRange(profileId, lastClickedKey, key);
    } else if (e.ctrlKey || e.metaKey) {
      fileStore.toggleSelect(profileId, key);
    } else {
      fileStore.clearSelection(profileId);
      fileStore.toggleSelect(profileId, key);
    }
    lastClickedKey = key;
  }

  function handleDblClick(item: string | S3Object) {
    if (isFolder(item)) {
      const prefix = item as string;
      onnavigate(fs.bucket, prefix);
    } else {
      const obj = item as S3Object;
      onfileopen?.(fs.bucket, obj.key);
    }
  }

  function handleContextMenu(e: MouseEvent, key: string) {
    e.preventDefault();
    const selected = fs.selected;
    const keys = selected.has(key) ? [...selected] : [key];
    oncontextmenu(e, keys);
  }

  function handleFileDragStart(e: MouseEvent, key: string) {
    const selected = fs.selected;
    const keys = selected.has(key) ? [...selected] : [key];
    dragStore.start({ profileId, bucket: fs.bucket, keys, sourcePrefix: fs.prefix }, e);
  }

  function setSort(field: SortField) {
    const current = fs.sortField;
    const order = current === field && fs.sortOrder === 'asc' ? 'desc' : 'asc';
    fileStore.setSort(profileId, field, order);
  }

  function handleDrop(destPrefix: string, modifier: 'meta' | 'shift' | null = null) {
    const data = dragStore.consume();
    if (!data || !data.keys.length) return;

    const isCross = data.profileId !== profileId || data.bucket !== fs.bucket;
    const sameBucket = data.bucket === fs.bucket;
    const samePrefix = data.sourcePrefix === destPrefix;

    if (sameBucket && samePrefix) return;

    // Prevent dropping a folder into itself
    if (sameBucket && data.keys.some((k) => k === destPrefix)) return;

    const sourceProfileId = data.profileId.replace(/::right$/, '');
    const destProfileId = profileId.replace(/::right$/, '');

    if (isCross) {
      if (modifier === 'shift') {
        onmovetoprefix?.(data.bucket, destPrefix, data.keys);
      } else {
        oncopytoprefix?.(sourceProfileId, destProfileId, data.bucket, fs.bucket, destPrefix, data.keys);
      }
    } else {
      if (modifier === 'meta') {
        oncopytoprefix?.(sourceProfileId, destProfileId, data.bucket, fs.bucket, destPrefix, data.keys);
      } else {
        onmovetoprefix?.(data.bucket, destPrefix, data.keys);
      }
    }
  }

  function getParentPrefix(prefix: string): string {
    const trimmed = prefix.replace(/\/$/, '');
    const lastSlash = trimmed.lastIndexOf('/');
    return lastSlash >= 0 ? trimmed.substring(0, lastSlash + 1) : '';
  }

  function handleSelectAll() {
    if (fs.selected.size === filteredItems.length) {
      fileStore.clearSelection(profileId);
    } else {
      fileStore.selectAll(profileId);
    }
  }

  // Listen for custom internaldrop event from dragStore (mouseup on drop target)
  onMount(() => {
    function onInternalDrop(e: Event) {
      if (!dragStore.payload) return;

      const modifier = (e as CustomEvent<{ modifier: 'meta' | 'shift' | null }>).detail?.modifier ?? null;

      const target = e.target as HTMLElement;
      const dropRow = target.closest('[data-drop-key]') as HTMLElement | null;
      if (dropRow) {
        const dropKey = dropRow.dataset.dropKey!;
        const destPrefix = dropKey === '__..__' ? getParentPrefix(fs.prefix) : dropKey;
        handleDrop(destPrefix, modifier);
      } else {
        handleDrop(fs.prefix, modifier);
      }
    }

    containerEl?.addEventListener('internaldrop', onInternalDrop);
    return () => {
      containerEl?.removeEventListener('internaldrop', onInternalDrop);
    };
  });

  // Infinite scroll — load more when scrolled near bottom
  let sentinelEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!sentinelEl || !containerEl) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) {
          const s = fileStore.getState(profileId);
          if (s.hasMore && !s.loadingMore) {
            fileStore.loadMore(profileId);
          }
        }
      },
      { root: containerEl, rootMargin: '200px' },
    );
    observer.observe(sentinelEl);
    return () => observer.disconnect();
  });
</script>

<div class="flex flex-1 flex-col overflow-hidden min-h-0">
  <!-- Column headers -->
  <div
    class="flex items-center gap-2 border-b border-border bg-muted/30 px-2 py-1 text-xs
      text-muted-foreground select-none shrink-0"
  >
    <button
      type="button"
      class="h-3.5 w-3.5 rounded-sm border border-border
        {fs.selected.size > 0 && fs.selected.size === filteredItems.length ? 'bg-primary' : ''}"
      onclick={handleSelectAll}
      aria-label="Select all"
    ></button>

    <div class="w-4"></div>

    <button
      type="button"
      class="flex flex-1 items-center gap-1 hover:text-foreground transition-colors"
      onclick={() => setSort('name')}
    >
      Name
      {#if fs.sortField === 'name'}
        {#if fs.sortOrder === 'asc'}
          <ChevronUp class="h-3 w-3" />
        {:else}
          <ChevronDown class="h-3 w-3" />
        {/if}
      {/if}
    </button>

    <span class="w-24">Type</span>

    <button
      type="button"
      class="flex w-20 items-center justify-end gap-1 hover:text-foreground transition-colors"
      onclick={() => setSort('size')}
    >
      {#if fs.sortField === 'size'}
        {#if fs.sortOrder === 'asc'}
          <ChevronUp class="h-3 w-3" />
        {:else}
          <ChevronDown class="h-3 w-3" />
        {/if}
      {/if}
      Size
    </button>

    <button
      type="button"
      class="flex w-28 items-center justify-end gap-1 hover:text-foreground transition-colors"
      onclick={() => setSort('lastModified')}
    >
      {#if fs.sortField === 'lastModified'}
        {#if fs.sortOrder === 'asc'}
          <ChevronUp class="h-3 w-3" />
        {:else}
          <ChevronDown class="h-3 w-3" />
        {/if}
      {/if}
      Modified
    </button>
  </div>

  <!-- File rows -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={containerEl}
    class="flex-1 overflow-auto min-h-0"
    oncontextmenu={(e) => {
      const target = e.target as HTMLElement;
      if (target.closest('[role="row"]')) return;
      e.preventDefault();
      onbgcontextmenu?.(e);
    }}
  >
    {#if fs.loading}
      <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
        {m.filelist_loading()}
      </div>
    {:else if filteredItems.length === 0}
      <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
        {fs.bucket ? m.filelist_no_items() : m.filelist_select_bucket()}
      </div>
    {:else}
      {#if hasParent}
        {@const parentPrefix = getParentPrefix(fs.prefix)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="flex items-center gap-2 px-2 py-1 text-xs cursor-pointer hover:bg-accent/30 transition-colors border-b border-border/30
            {dragStore.active && dragStore.hoverDropKey === '__..__'
            ? 'ring-2 ring-primary ring-inset bg-primary/5'
            : ''}"
          data-drop-key="__..__"
          ondblclick={() => onnavigate(fs.bucket, parentPrefix)}
        >
          <div class="w-5 shrink-0"></div>
          <CornerLeftUp class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span class="text-muted-foreground">..</span>
        </div>
      {/if}
      {#each filteredItems as item}
        {@const key = getItemKey(item)}
        {@const folder = isFolder(item)}
        {@const obj = folder ? null : (item as S3Object)}
        <FileRow
          itemKey={key}
          size={obj?.size ?? 0}
          lastModified={obj?.last_modified ?? null}
          isFolder={folder}
          contentType={obj?.content_type ?? null}
          selected={fs.selected.has(key)}
          ontoggle={(e: MouseEvent) => handleClick(e, key)}
          ondblclick={() => handleDblClick(item)}
          oncontextmenu={(e: MouseEvent) => handleContextMenu(e, key)}
          ondragstart={(e: MouseEvent) => handleFileDragStart(e, key)}
          ondrop={folder ? () => handleDrop(key) : undefined}
        />
      {/each}

      <!-- Infinite scroll sentinel -->
      <div bind:this={sentinelEl} class="h-1 shrink-0"></div>
      {#if fs.loadingMore}
        <div class="flex items-center justify-center py-3 text-xs text-muted-foreground">
          Loading more...
        </div>
      {/if}
    {/if}
  </div>
</div>
