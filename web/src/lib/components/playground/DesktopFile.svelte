<script lang="ts">
  import type { DesktopItem } from './types';
  import { store } from './store.svelte';
  import { Folder, FileImage, File } from '@lucide/svelte';

  let { item }: { item: DesktopItem } = $props();

  const isFolder = $derived(item.file.type === 'folder');
  const isImage = $derived(!isFolder && (item.file.contentType?.startsWith('image/') ?? false));
  const selected = $derived(store.selectedDesktopFiles.has(item.file.id));

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    e.stopPropagation();

    // Select on click
    store.toggleDesktopSelect(item.file.id, e.metaKey || e.ctrlKey);

    const startX = e.clientX;
    const startY = e.clientY;
    let started = false;

    function onMove(me: PointerEvent) {
      if (!started) {
        const dx = me.clientX - startX;
        const dy = me.clientY - startY;
        if (Math.abs(dx) + Math.abs(dy) < 6) return;
        started = true;
        store.startDrag(item.file, 'desktop', me.clientX, me.clientY);
      }
    }
    function onUp() {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    }
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  function onContextMenu(e: MouseEvent) {
    e.preventDefault();
    if (!selected) store.toggleDesktopSelect(item.file.id);
    store.openContextMenu(e.clientX, e.clientY, item.file, 'desktop');
  }
</script>

<div
  class="absolute flex w-14 cursor-grab flex-col items-center gap-0.5 select-none active:cursor-grabbing"
  style="left: {item.x}%; top: {item.y}%;"
  onpointerdown={onPointerDown}
  oncontextmenu={onContextMenu}
  role="button"
  tabindex="0"
>
  <div class="flex h-10 w-10 items-center justify-center rounded-lg backdrop-blur-sm transition-colors
    {selected ? 'bg-white/25 ring-2 ring-blue-400/60' : 'bg-white/10 hover:bg-white/20'}">
    {#if isFolder}
      <Folder class="h-6 w-6 text-blue-400" />
    {:else if isImage}
      <FileImage class="h-6 w-6 text-purple-400" />
    {:else}
      <File class="h-6 w-6 text-white/70" />
    {/if}
  </div>
  <span class="max-w-[64px] truncate text-center text-[10px] leading-tight text-white drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]
    {selected ? 'rounded bg-blue-500/50 px-1' : ''}">
    {item.file.name}
  </span>
</div>
