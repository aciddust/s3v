<script lang="ts">
  import type { VirtualFile, DragState, SortField } from './types';
  import { store } from './store.svelte';
  import FileRow from './FileRow.svelte';
  import { ChevronUp, ChevronDown, CornerLeftUp } from '@lucide/svelte';

  let { files, source, onNavigate, parentPath }: {
    files: VirtualFile[];
    source: DragState['source'];
    onNavigate: (path: string) => void;
    parentPath: string | null;
  } = $props();

  function setSort(field: SortField) {
    store.setSort(field);
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden min-h-0">
  <!-- Column headers -->
  <div class="flex items-center gap-2 border-b border-border bg-muted/30 px-2 py-1 text-xs text-muted-foreground select-none shrink-0">
    <div class="w-4"></div>
    <button class="flex flex-1 items-center gap-1 hover:text-foreground transition-colors" onclick={() => setSort('name')}>
      Name
      {#if store.sortField === 'name'}
        {#if store.sortOrder === 'asc'}<ChevronUp class="h-3 w-3" />{:else}<ChevronDown class="h-3 w-3" />{/if}
      {/if}
    </button>
    <span class="w-20">Type</span>
    <button class="flex w-16 items-center justify-end gap-1 hover:text-foreground transition-colors" onclick={() => setSort('size')}>
      {#if store.sortField === 'size'}
        {#if store.sortOrder === 'asc'}<ChevronUp class="h-3 w-3" />{:else}<ChevronDown class="h-3 w-3" />{/if}
      {/if}
      Size
    </button>
    <button class="flex w-24 items-center justify-end gap-1 hover:text-foreground transition-colors" onclick={() => setSort('lastModified')}>
      {#if store.sortField === 'lastModified'}
        {#if store.sortOrder === 'asc'}<ChevronUp class="h-3 w-3" />{:else}<ChevronDown class="h-3 w-3" />{/if}
      {/if}
      Modified
    </button>
  </div>

  <!-- File rows -->
  <div class="flex-1 overflow-auto min-h-0">
    {#if parentPath !== null}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="flex items-center gap-2 px-2 py-1 text-xs cursor-pointer hover:bg-accent/30 transition-colors border-b border-border/30"
        ondblclick={() => onNavigate(parentPath)}
      >
        <CornerLeftUp class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        <span class="text-muted-foreground">..</span>
      </div>
    {/if}
    {#each files as file (file.id)}
      <FileRow {file} {source} {onNavigate} />
    {:else}
      <div class="flex items-center justify-center py-8 text-xs text-muted-foreground">No items</div>
    {/each}
  </div>
</div>
