<script lang="ts">
  import type { VirtualFile, DragState } from './types';
  import { store } from './store.svelte';
  import Breadcrumb from './Breadcrumb.svelte';
  import FileList from './FileList.svelte';

  let { bucket, currentPath, files, side }: {
    bucket: string;
    currentPath: string;
    files: VirtualFile[];
    side: 'left' | 'right';
  } = $props();

  let dropHover = $state(false);

  const source: DragState['source'] = $derived(side === 'left' ? 'left-panel' : 'right-panel');

  const parentPath = $derived.by(() => {
    if (!currentPath) return null;
    const trimmed = currentPath.replace(/\/$/, '');
    const lastSlash = trimmed.lastIndexOf('/');
    return lastSlash >= 0 ? trimmed.substring(0, lastSlash + 1) : '';
  });

  function onNavigate(path: string) {
    store.navigatePanel(side, path);
  }

  function onPointerEnter() {
    if (store.drag.active && store.drag.source !== source) {
      dropHover = true;
    }
  }

  function onPointerLeave() {
    dropHover = false;
  }

  function onPointerUp(e: PointerEvent) {
    if (!store.drag.active || !store.drag.file) return;
    if (store.drag.source === source) return;

    e.stopPropagation();

    if (store.drag.source === 'desktop') {
      store.uploadToPanel(store.drag.file, side);
    } else {
      const sourceSide = store.drag.source === 'left-panel' ? 'left' as const : 'right' as const;
      if (store.drag.modifier) {
        // Cmd/Ctrl held = copy
        store.copyBetweenPanels(store.drag.file, side);
      } else {
        // No modifier = move
        store.moveBetweenPanels(store.drag.file, side, sourceSide);
      }
    }
    store.endDrag();
    dropHover = false;
  }
</script>

<div
  class="flex min-h-0 flex-1 flex-col overflow-hidden transition-colors
    {side === 'left' ? 'border-r border-border' : ''}
    {dropHover ? 'outline-dashed outline-2 -outline-offset-2 ' + (store.drag.source === 'desktop' ? 'bg-primary/5 outline-blue-400/50' : 'bg-primary/5 outline-amber-400/50') : ''}"
  onpointerenter={onPointerEnter}
  onpointerleave={onPointerLeave}
  onpointerup={onPointerUp}
  role="region"
>
  <Breadcrumb {bucket} {currentPath} {onNavigate} />
  <FileList {files} {source} {onNavigate} {parentPath} />
</div>
