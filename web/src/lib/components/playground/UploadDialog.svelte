<script lang="ts">
  import { store } from './store.svelte';
  import type { VirtualFile } from './types';
  import {
    Folder, File, FileImage, FileText, FileCode, FileArchive,
    FileSpreadsheet, ChevronRight, ChevronDown, X, Upload,
  } from '@lucide/svelte';

  // Mock local file system
  interface LocalItem {
    name: string;
    type: 'file' | 'folder';
    size: number;
    contentType: string | null;
    children?: LocalItem[];
  }

  const LOCAL_FILES: LocalItem[] = [
    { name: 'Desktop', type: 'folder', size: 0, contentType: null, children: [
      { name: 'report.csv', type: 'file', size: 524_000, contentType: 'text/csv' },
      { name: 'banner.png', type: 'file', size: 1_800_000, contentType: 'image/png' },
      { name: 'presentation.pdf', type: 'file', size: 3_200_000, contentType: 'application/pdf' },
    ]},
    { name: 'Documents', type: 'folder', size: 0, contentType: null, children: [
      { name: 'notes.md', type: 'file', size: 8_400, contentType: 'text/markdown' },
      { name: 'budget.xlsx', type: 'file', size: 156_000, contentType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' },
      { name: 'contract.pdf', type: 'file', size: 890_000, contentType: 'application/pdf' },
    ]},
    { name: 'Downloads', type: 'folder', size: 0, contentType: null, children: [
      { name: 'archive.zip', type: 'file', size: 45_000_000, contentType: 'application/zip' },
      { name: 'installer.dmg', type: 'file', size: 120_000_000, contentType: 'application/x-apple-diskimage' },
      { name: 'photo-001.jpg', type: 'file', size: 4_500_000, contentType: 'image/jpeg' },
    ]},
    { name: 'Projects', type: 'folder', size: 0, contentType: null, children: [
      { name: 'index.ts', type: 'file', size: 2_400, contentType: 'text/typescript' },
      { name: 'styles.css', type: 'file', size: 5_600, contentType: 'text/css' },
      { name: 'config.json', type: 'file', size: 1_200, contentType: 'application/json' },
      { name: 'README.md', type: 'file', size: 3_800, contentType: 'text/markdown' },
    ]},
    { name: 'app-icon.svg', type: 'file', size: 14_000, contentType: 'image/svg+xml' },
    { name: 'database-dump.sql', type: 'file', size: 28_000_000, contentType: 'application/sql' },
  ];

  let expandedFolders = $state<Set<string>>(new Set(['Desktop']));
  let selectedItems = $state<Set<string>>(new Set());

  function getIcon(item: LocalItem) {
    if (item.type === 'folder') return Folder;
    const ext = item.name.split('.').at(-1)?.toLowerCase() ?? '';
    const ct = item.contentType ?? '';
    if (ct.startsWith('image/') || ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp'].includes(ext)) return FileImage;
    if (ct.startsWith('text/') || ['txt', 'md', 'csv', 'log', 'pdf'].includes(ext)) return FileText;
    if (['js', 'ts', 'json', 'html', 'css', 'py', 'rs'].includes(ext)) return FileCode;
    if (['zip', 'tar', 'gz', 'dmg', 'rar'].includes(ext)) return FileArchive;
    if (['xls', 'xlsx'].includes(ext)) return FileSpreadsheet;
    return File;
  }

  function getIconColor(item: LocalItem): string {
    if (item.type === 'folder') return 'text-blue-400';
    const ct = item.contentType ?? '';
    if (ct.startsWith('image/')) return 'text-purple-400';
    return 'text-muted-foreground';
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '—';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  function toggleFolder(name: string) {
    const next = new Set(expandedFolders);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    expandedFolders = next;
  }

  function toggleSelect(key: string, e: MouseEvent) {
    const next = (e.metaKey || e.ctrlKey) ? new Set(selectedItems) : new Set<string>();
    if (next.has(key)) next.delete(key);
    else next.add(key);
    selectedItems = next;
  }

  function getFlat(items: LocalItem[], parentKey = ''): { item: LocalItem; key: string }[] {
    const result: { item: LocalItem; key: string }[] = [];
    for (const item of items) {
      const key = parentKey ? `${parentKey}/${item.name}` : item.name;
      result.push({ item, key });
      if (item.type === 'folder' && item.children && expandedFolders.has(item.name)) {
        for (const child of item.children) {
          result.push({ item: child, key: `${key}/${child.name}` });
        }
      }
    }
    return result;
  }

  const flatItems = $derived(getFlat(LOCAL_FILES));

  function handleUpload() {
    const side = store.uploadDialogTarget;
    for (const { item, key } of flatItems) {
      if (selectedItems.has(key) && item.type === 'file') {
        const file: VirtualFile = {
          id: `vf-locupload-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
          name: item.name,
          type: 'file',
          size: item.size,
          path: side === 'left' ? store.leftPanel.currentPath : store.rightPanel.currentPath,
          contentType: item.contentType,
          lastModified: new Date().toISOString(),
        };
        store.bucketFiles.push(file);
        store.addTransfer(file, 'upload');
      }
    }
    selectedItems = new Set();
    store.uploadDialogOpen = false;
  }

  function close() {
    selectedItems = new Set();
    store.uploadDialogOpen = false;
  }

  const selectedCount = $derived([...selectedItems].filter((key) => {
    const found = flatItems.find((f) => f.key === key);
    return found && found.item.type === 'file';
  }).length);
</script>

{#if store.uploadDialogOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="absolute inset-0 z-40 flex items-center justify-center bg-black/40 backdrop-blur-[1px]" onclick={close}>
    <!-- Dialog -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dark flex w-[420px] flex-col overflow-hidden rounded-lg border border-border bg-background text-foreground shadow-2xl" onclick={(e) => e.stopPropagation()}>
      <!-- Header -->
      <div class="flex h-10 items-center justify-between border-b border-border px-4 shrink-0">
        <span class="text-sm font-medium">Upload Files</span>
        <button class="rounded-md p-1 hover:bg-accent transition-colors" onclick={close}>
          <X class="h-4 w-4 text-muted-foreground" />
        </button>
      </div>

      <!-- Sidebar label -->
      <div class="flex items-center gap-2 border-b border-border bg-muted/30 px-4 py-1.5 text-xs text-muted-foreground shrink-0">
        <span>Location:</span>
        <span class="font-medium text-foreground">My Computer</span>
      </div>

      <!-- File list -->
      <div class="h-64 overflow-auto">
        {#each flatItems as { item, key }}
          {@const Icon = getIcon(item)}
          {@const isSelected = selectedItems.has(key)}
          {@const isFolder = item.type === 'folder'}
          {@const depth = key.split('/').length - 1}

          <div
            class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer select-none transition-colors
              {isSelected ? 'bg-primary/15 text-foreground' : 'hover:bg-accent/30 text-foreground'}"
            style="padding-left: {12 + depth * 16}px;"
            onclick={(e) => isFolder ? toggleFolder(item.name) : toggleSelect(key, e)}
            ondblclick={() => isFolder && toggleFolder(item.name)}
            role="row"
            tabindex="0"
          >
            {#if isFolder}
              {#if expandedFolders.has(item.name)}
                <ChevronDown class="h-3 w-3 shrink-0 text-muted-foreground" />
              {:else}
                <ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
              {/if}
            {:else}
              <div class="w-3"></div>
            {/if}
            <Icon class="h-4 w-4 shrink-0 {getIconColor(item)}" />
            <span class="flex-1 truncate">{item.name}</span>
            {#if !isFolder}
              <span class="text-muted-foreground">{formatBytes(item.size)}</span>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between border-t border-border px-4 py-2.5 shrink-0">
        <span class="text-xs text-muted-foreground">
          {selectedCount > 0 ? `${selectedCount} file${selectedCount > 1 ? 's' : ''} selected` : 'Select files to upload'}
        </span>
        <div class="flex gap-2">
          <button class="rounded-md border border-border px-3 py-1 text-xs hover:bg-accent transition-colors" onclick={close}>
            Cancel
          </button>
          <button
            class="flex items-center gap-1.5 rounded-md bg-primary px-3 py-1 text-xs text-primary-foreground transition-colors hover:bg-primary/90 disabled:opacity-50"
            disabled={selectedCount === 0}
            onclick={handleUpload}
          >
            <Upload class="h-3 w-3" />
            Upload
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
