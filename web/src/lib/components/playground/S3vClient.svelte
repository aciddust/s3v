<script lang="ts">
  import { store } from './store.svelte';
  import TreeSidebar from './TreeSidebar.svelte';
  import FilePanel from './FilePanel.svelte';
  import TransferPanel from './TransferPanel.svelte';
  import TabBar from './TabBar.svelte';
  import UploadDialog from './UploadDialog.svelte';
  import ProfileManager from './ProfileManager.svelte';
  import {
    Upload, Download, FolderPlus, Trash2, Columns2, Search, ArrowUpDown, CloudCog,
  } from '@lucide/svelte';

  let clientEl: HTMLDivElement | undefined = $state();

  function handleNavigateLeft(bucket: string, path: string) {
    if (bucket !== store.leftPanel.bucket) {
      store.saveCurrentBucket();
      store.selectBucket(bucket, 'left');
    }
    store.navigatePanel('left', path);
  }

  function handleUpload() {
    store.uploadDialogTarget = 'left';
    store.uploadDialogOpen = true;
  }

  function handleDownload() {
    store.downloadSelected();
  }

  let newFolderDialogOpen = $state(false);
  let newFolderName = $state('');

  function handleNewFolder() {
    newFolderName = '';
    newFolderDialogOpen = true;
  }

  function confirmNewFolder() {
    const name = newFolderName.trim();
    if (!name) return;
    store.bucketFiles.push({
      id: `vf-nf-${Date.now()}`,
      name,
      type: 'folder' as const,
      size: 0,
      path: store.leftPanel.currentPath,
      contentType: null,
      lastModified: new Date().toISOString(),
    });
    newFolderDialogOpen = false;
  }

  let deleteDialogOpen = $state(false);
  const deleteTargets = $derived(store.bucketFiles.filter((f) => store.selectedFiles.has(f.id)));

  function handleDelete() {
    if (store.selectedFiles.size === 0) return;
    deleteDialogOpen = true;
  }

  function confirmDelete() {
    store.deleteSelected();
    deleteDialogOpen = false;
  }

  function onResizePointerDown(e: PointerEvent) {
    if (e.button !== 0 || !clientEl) return;
    e.preventDefault();
    e.stopPropagation();

    const startX = e.clientX;
    const startY = e.clientY;
    const startW = clientEl.offsetWidth;
    const startH = clientEl.offsetHeight;
    const parent = clientEl.parentElement;
    if (!parent) return;

    function onMove(me: PointerEvent) {
      const dx = me.clientX - startX;
      const dy = me.clientY - startY;
      const newW = Math.max(500, Math.min(startW + dx, parent!.offsetWidth - 100));
      const newH = Math.max(300, startH + dy);
      store.s3vWidth = (newW / parent!.offsetWidth) * 100;
      store.s3vHeight = newH;
    }
    function onUp() {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    }
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }
</script>

<div
  bind:this={clientEl}
  data-s3v-window
  class="dark relative z-10 flex flex-col overflow-hidden rounded-lg border border-border bg-background text-foreground shadow-2xl"
  style="width: {store.s3vWidth}%; {typeof store.s3vHeight === 'number' && store.s3vHeight !== 100 ? `height: ${store.s3vHeight}px; align-self: start;` : ''}"
>
  <!-- TitleBar -->
  <div class="flex h-8 items-center gap-1 border-b border-border bg-background px-3 select-none shrink-0">
    <div class="flex items-center gap-1.5 mr-3">
      <div class="h-2.5 w-2.5 rounded-full bg-[#ff5f57]"></div>
      <div class="h-2.5 w-2.5 rounded-full bg-[#febc2e]"></div>
      <div class="h-2.5 w-2.5 rounded-full bg-[#28c840]"></div>
    </div>
    <span class="text-xs font-bold tracking-wider text-primary">S3V</span>
    <span class="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-default">File</span>
    <span class="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-default">Edit</span>
    <span class="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-default">View</span>
    <span class="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-default">Help</span>
    <div class="flex-1"></div>
    <button
      class="h-6 w-6 flex items-center justify-center rounded-md transition-colors {store.transferPanelVisible ? 'bg-secondary text-secondary-foreground' : 'hover:bg-accent/50 text-muted-foreground'}"
      onclick={() => (store.transferPanelVisible = !store.transferPanelVisible)}
      title="Transfer Panel"
    >
      <ArrowUpDown class="h-3.5 w-3.5" />
    </button>
    <button class="h-6 w-6 flex items-center justify-center rounded-md hover:bg-accent/50 transition-colors" onclick={() => (store.profileManagerOpen = true)} title="Profile Manager">
      <CloudCog class="h-3.5 w-3.5 text-muted-foreground" />
    </button>
  </div>

  <!-- TabBar -->
  <TabBar />

  <!-- Toolbar -->
  <div class="flex h-9 items-center gap-1 border-b border-border bg-background px-2 shrink-0">
    <button
      class="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-foreground hover:bg-accent transition-colors"
      onclick={handleUpload}
      title="Upload selected desktop files"
    >
      <Upload class="h-3.5 w-3.5" /> Upload
    </button>
    <button
      class="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-foreground hover:bg-accent transition-colors"
      onclick={handleDownload}
      title="Download selected files to desktop"
    >
      <Download class="h-3.5 w-3.5" /> Download
    </button>
    <button
      class="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-foreground hover:bg-accent transition-colors"
      onclick={handleNewFolder}
      title="Create new folder"
    >
      <FolderPlus class="h-3.5 w-3.5" /> Folder
    </button>
    <button
      class="flex h-7 items-center gap-1.5 rounded-md px-2 text-xs text-destructive hover:bg-accent transition-colors"
      onclick={handleDelete}
      title="Delete selected files"
    >
      <Trash2 class="h-3.5 w-3.5" /> Delete
    </button>
    <div class="flex-1"></div>
    <button
      class="flex h-7 w-7 items-center justify-center rounded-md transition-colors
        {store.dualPanel ? 'bg-secondary text-secondary-foreground' : 'hover:bg-accent text-muted-foreground'}"
      onclick={() => (store.dualPanel = !store.dualPanel)}
      title="Toggle Dual Panel"
    >
      <Columns2 class="h-3.5 w-3.5" />
    </button>
    <div class="relative">
      <Search class="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
      <input
        class="h-7 w-36 rounded-md border border-input bg-transparent pl-7 text-xs text-foreground placeholder:text-muted-foreground outline-none focus:border-ring lg:w-44"
        placeholder="Search..."
        bind:value={store.searchQuery}
      />
    </div>
  </div>

  <!-- Main area: sidebar + panels -->
  <div class="flex flex-1 min-h-0">
    <TreeSidebar activeBucket={store.leftPanel.bucket} activePath={store.leftPanel.currentPath} onNavigate={handleNavigateLeft} />
    <FilePanel bucket={store.leftPanel.bucket} currentPath={store.leftPanel.currentPath} files={store.leftPanelFiles} side="left" />
    {#if store.dualPanel}
      <FilePanel bucket={store.rightPanel.bucket} currentPath={store.rightPanel.currentPath} files={store.rightPanelFiles} side="right" />
    {/if}
  </div>

  <!-- Transfer panel (floating) -->
  <TransferPanel />

  <!-- New folder dialog -->
  {#if newFolderDialogOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="absolute inset-0 z-40 flex items-center justify-center bg-black/40" onclick={() => (newFolderDialogOpen = false)}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="w-72 rounded-lg border border-border bg-background p-4 shadow-2xl" onclick={(e) => e.stopPropagation()}>
        <h3 class="mb-3 text-sm font-medium">New Folder</h3>
        <input
          class="w-full rounded-md border border-input bg-transparent px-3 py-1.5 text-xs text-foreground placeholder:text-muted-foreground outline-none focus:border-ring"
          placeholder="Folder name"
          bind:value={newFolderName}
          onkeydown={(e) => e.key === 'Enter' && confirmNewFolder()}
          autofocus
        />
        <div class="mt-3 flex justify-end gap-2">
          <button class="rounded-md border border-border px-3 py-1 text-xs hover:bg-accent transition-colors" onclick={() => (newFolderDialogOpen = false)}>
            Cancel
          </button>
          <button
            class="rounded-md bg-primary px-3 py-1 text-xs text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
            disabled={!newFolderName.trim()}
            onclick={confirmNewFolder}
          >
            Create
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Delete confirm dialog -->
  {#if deleteDialogOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="absolute inset-0 z-40 flex items-center justify-center bg-black/40" onclick={() => (deleteDialogOpen = false)}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="w-80 rounded-lg border border-border bg-background p-4 shadow-2xl" onclick={(e) => e.stopPropagation()}>
        <h3 class="mb-2 text-sm font-medium">Delete — {deleteTargets.length}건</h3>
        <div class="mb-3 max-h-32 overflow-auto space-y-1">
          {#each deleteTargets as file (file.id)}
            <div class="rounded bg-muted/40 px-2 py-1 text-xs truncate">{file.name}</div>
          {/each}
        </div>
        <div class="flex items-center gap-2 mb-3 text-xs text-destructive">
          <svg class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
          <span>이 작업은 되돌릴 수 없습니다.</span>
        </div>
        <div class="flex justify-end gap-2">
          <button class="rounded-md border border-border px-3 py-1 text-xs hover:bg-accent transition-colors" onclick={() => (deleteDialogOpen = false)}>
            Cancel
          </button>
          <button
            class="rounded-md bg-destructive px-3 py-1 text-xs text-white hover:bg-destructive/90 transition-colors"
            onclick={confirmDelete}
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Upload file picker dialog -->
  <UploadDialog />

  <!-- Profile Manager dialog -->
  <ProfileManager />

  <!-- Resize handle (bottom-right corner) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="absolute bottom-0 right-0 h-4 w-4 cursor-nwse-resize"
    onpointerdown={onResizePointerDown}
  >
    <svg class="h-full w-full text-white/20 hover:text-white/40 transition-colors" viewBox="0 0 16 16" fill="currentColor">
      <circle cx="12" cy="12" r="1.5" />
      <circle cx="8" cy="12" r="1.5" />
      <circle cx="12" cy="8" r="1.5" />
    </svg>
  </div>
</div>
