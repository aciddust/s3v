<script lang="ts">
  import type { VirtualFile, DragState } from './types';
  import { store } from './store.svelte';
  import {
    Folder, File, FileImage, FileText, Film, AudioLines,
    FileCode, FileArchive, FileSpreadsheet,
  } from '@lucide/svelte';

  let { file, source, onNavigate }: {
    file: VirtualFile;
    source: DragState['source'];
    onNavigate: (path: string) => void;
  } = $props();

  const selected = $derived(store.selectedFiles.has(file.id));

  function getExtension(name: string): string {
    return name.split('.').at(-1)?.toLowerCase() ?? '';
  }

  const FileIcon = $derived.by(() => {
    if (file.type === 'folder') return Folder;
    const ext = getExtension(file.name);
    const ct = file.contentType ?? '';
    if (ct.startsWith('image/') || ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp'].includes(ext)) return FileImage;
    if (ct.startsWith('video/') || ['mp4', 'mkv', 'avi', 'mov'].includes(ext)) return Film;
    if (ct.startsWith('audio/') || ['mp3', 'wav', 'ogg', 'flac'].includes(ext)) return AudioLines;
    if (ct.startsWith('text/') || ['txt', 'md', 'csv', 'log'].includes(ext)) return FileText;
    if (['js', 'ts', 'json', 'html', 'css', 'py', 'rs', 'go', 'sh', 'sql', 'yaml'].includes(ext)) return FileCode;
    if (['zip', 'tar', 'gz', 'bz2', 'rar', '7z'].includes(ext)) return FileArchive;
    if (['xls', 'xlsx'].includes(ext)) return FileSpreadsheet;
    return File;
  });

  const iconColor = $derived.by(() => {
    if (file.type === 'folder') return 'text-blue-400';
    const ct = file.contentType ?? '';
    if (ct.startsWith('image/')) return 'text-purple-400';
    if (ct.startsWith('video/')) return 'text-pink-400';
    if (ct.startsWith('audio/')) return 'text-green-400';
    return 'text-muted-foreground';
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '—';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '—';
    try {
      return new Date(dateStr).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    } catch { return '—'; }
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    if (file.type === 'folder') return; // folders navigate on dblclick, not drag

    const startX = e.clientX;
    const startY = e.clientY;
    let started = false;

    function onMove(me: PointerEvent) {
      if (!started) {
        const dx = me.clientX - startX;
        const dy = me.clientY - startY;
        if (Math.abs(dx) + Math.abs(dy) < 6) return;
        started = true;
        store.startDrag(file, source, me.clientX, me.clientY);
      }
    }
    function onUp() {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    }
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  function onClick(e: MouseEvent) {
    if (e.ctrlKey || e.metaKey) {
      store.toggleSelect(file.id);
    } else {
      store.clearSelection();
      store.toggleSelect(file.id);
    }
  }

  function onDblClick() {
    if (file.type === 'folder') {
      onNavigate(file.path + file.name + '/');
    }
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    store.openContextMenu(e.clientX, e.clientY, file, source);
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter') onClick(e as unknown as MouseEvent);
    if (e.key === ' ') { e.preventDefault(); store.toggleSelect(file.id); }
    if (e.key === 'Enter' && file.type === 'folder') onDblClick();
  }
</script>

<div
  role="row"
  tabindex="0"
  class="group flex items-center gap-2 border-b border-border/50 px-2 py-1 text-xs
    cursor-pointer select-none transition-colors
    {selected ? 'bg-primary/10 text-foreground' : 'hover:bg-accent/30 text-foreground'}"
  onpointerdown={onPointerDown}
  onclick={onClick}
  ondblclick={onDblClick}
  onkeydown={onKeyDown}
  oncontextmenu={onContextMenu}
>
  <FileIcon class="h-4 w-4 shrink-0 {iconColor}" />
  <span class="flex-1 truncate font-medium">{file.name}{file.type === 'folder' ? '/' : ''}</span>
  <span class="w-20 truncate text-muted-foreground">{file.type === 'folder' ? 'Folder' : (getExtension(file.name) || '—')}</span>
  <span class="w-16 text-right text-muted-foreground">{file.type === 'folder' ? '—' : formatBytes(file.size)}</span>
  <span class="w-24 text-right text-muted-foreground">{formatDate(file.lastModified)}</span>
</div>
