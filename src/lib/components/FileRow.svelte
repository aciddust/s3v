<script lang="ts">
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { dragStore } from '$lib/stores/drag.svelte';
  import {
    Folder,
    File,
    FileImage,
    FileText,
    Film,
    AudioLines,
    FileCode,
    FileArchive,
    FileSpreadsheet,
  } from '@lucide/svelte';

  interface Props {
    itemKey: string;
    size: number;
    lastModified: string | null;
    isFolder: boolean;
    contentType: string | null;
    selected: boolean;
    ontoggle: (e: MouseEvent) => void;
    ondblclick: () => void;
    oncontextmenu: (e: MouseEvent) => void;
    ondragstart: (e: MouseEvent) => void;
    ondrop?: () => void;
  }

  const {
    itemKey,
    size,
    lastModified,
    isFolder,
    contentType,
    selected,
    ontoggle,
    ondblclick,
    oncontextmenu,
    ondragstart,
    ondrop,
  }: Props = $props();

  function getFileName(key: string): string {
    return key.replace(/\/$/, '').split('/').at(-1) ?? key;
  }

  function getExtension(key: string): string {
    return key.split('.').at(-1)?.toLowerCase() ?? '';
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '—';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '—';
    try {
      return new Date(dateStr).toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return '—';
    }
  }

  const FileIcon = $derived(
    (() => {
      if (isFolder) return Folder;
      const ext = getExtension(itemKey);
      const ct = contentType ?? '';
      if (
        ct.startsWith('image/') ||
        ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'ico'].includes(ext)
      )
        return FileImage;
      if (ct.startsWith('video/') || ['mp4', 'mkv', 'avi', 'mov', 'webm'].includes(ext))
        return Film;
      if (ct.startsWith('audio/') || ['mp3', 'wav', 'ogg', 'flac', 'aac'].includes(ext))
        return AudioLines;
      if (ct.startsWith('text/') || ['txt', 'md', 'csv', 'log'].includes(ext)) return FileText;
      if (
        [
          'js',
          'ts',
          'jsx',
          'tsx',
          'html',
          'css',
          'json',
          'yaml',
          'yml',
          'toml',
          'sh',
          'py',
          'rs',
          'go',
          'java',
          'c',
          'cpp',
        ].includes(ext)
      )
        return FileCode;
      if (['zip', 'tar', 'gz', 'bz2', 'rar', '7z'].includes(ext)) return FileArchive;
      if (['xls', 'xlsx', 'ods', 'csv'].includes(ext)) return FileSpreadsheet;
      return File;
    })(),
  );

  const iconColor = $derived(
    (() => {
      if (isFolder) return 'text-blue-400';
      const ext = getExtension(itemKey);
      const ct = contentType ?? '';
      if (ct.startsWith('image/') || ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp'].includes(ext))
        return 'text-purple-400';
      if (ct.startsWith('video/')) return 'text-pink-400';
      if (ct.startsWith('audio/')) return 'text-green-400';
      return 'text-muted-foreground';
    })(),
  );

  /** Start drag on mousedown (with a small threshold to avoid accidental drags). */
  function handleMouseDown(e: MouseEvent) {
    console.log('[FileRow] mousedown', { isFolder, itemKey, button: e.button });
    if (e.button !== 0) return;
    // Don't drag from checkbox area
    if ((e.target as HTMLElement).closest('button')) return;

    const startX = e.clientX;
    const startY = e.clientY;
    let started = false;

    function onMove(me: MouseEvent) {
      if (!started) {
        const dx = me.clientX - startX;
        const dy = me.clientY - startY;
        if (Math.abs(dx) + Math.abs(dy) < 6) return; // threshold
        started = true;
        console.log('[FileRow] drag threshold reached, calling ondragstart', { isFolder, itemKey });
        ondragstart(e);
      }
    }
    function onUp() {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }
</script>

<div
  role="row"
  tabindex="0"
  class="group flex items-center gap-2 border-b border-border/50 px-2 py-1 text-xs
    cursor-pointer select-none
    {selected ? 'bg-primary/10 text-foreground' : 'hover:bg-accent/30 text-foreground'}
    {isFolder && ondrop && dragStore.active && dragStore.hoverDropKey === itemKey
    ? 'ring-2 ring-primary ring-inset bg-primary/5'
    : ''}"
  data-drop-key={isFolder && ondrop ? itemKey : undefined}
  onclick={ontoggle}
  onkeydown={(e) => e.key === 'Enter' && ontoggle(e as unknown as MouseEvent)}
  {ondblclick}
  {oncontextmenu}
  onmousedown={handleMouseDown}
>
  <!-- Checkbox (toggle selection without clearing others, like Cmd+Click) -->
  <button
    type="button"
    class="flex shrink-0 items-center"
    onclick={(e) => {
      e.stopPropagation();
      ontoggle(new MouseEvent('click', { metaKey: true, ctrlKey: true }));
    }}
    tabindex={-1}
    aria-label="Select row"
  >
    <Checkbox checked={selected} onCheckedChange={() => {}} class="h-3.5 w-3.5" tabindex={-1} />
  </button>

  <!-- Icon -->
  <FileIcon class="h-4 w-4 shrink-0 {iconColor}" />

  <!-- Name -->
  <span class="flex-1 truncate font-medium">{getFileName(itemKey)}</span>

  <!-- Type -->
  <span class="w-24 truncate text-muted-foreground">
    {isFolder ? 'Folder' : (contentType ?? (getExtension(itemKey) || '—'))}
  </span>

  <!-- Size -->
  <span class="w-20 text-right text-muted-foreground">
    {isFolder ? '—' : formatBytes(size)}
  </span>

  <!-- Date -->
  <span class="w-28 text-right text-muted-foreground">
    {formatDate(lastModified)}
  </span>
</div>
