<script lang="ts">
  import { store } from './store.svelte';
  import MenuBar from './MenuBar.svelte';
  import DesktopFile from './DesktopFile.svelte';
  import S3vClient from './S3vClient.svelte';
  // import CoachOverlay from './CoachOverlay.svelte';

  let desktopEl: HTMLDivElement | undefined = $state();
  let dropHover = $state(false);

  // Rubber-band selection
  let selecting = $state(false);
  let selStartX = $state(0);
  let selStartY = $state(0);
  let selCurX = $state(0);
  let selCurY = $state(0);

  let selRect = $derived.by(() => {
    const x = Math.min(selStartX, selCurX);
    const y = Math.min(selStartY, selCurY);
    const w = Math.abs(selCurX - selStartX);
    const h = Math.abs(selCurY - selStartY);
    return { x, y, w, h };
  });

  function hitTest() {
    if (!desktopEl) return;
    const container = desktopEl.getBoundingClientRect();
    const r = selRect;
    const hits = new Set<string>();

    for (const item of store.desktopFiles) {
      const fileX = container.left + (item.x / 100) * container.width;
      const fileY = container.top + (item.y / 100) * container.height;
      const fileW = 64;
      const fileH = 72;

      if (
        fileX + fileW > r.x && fileX < r.x + r.w &&
        fileY + fileH > r.y && fileY < r.y + r.h
      ) {
        hits.add(item.file.id);
      }
    }
    store.setDesktopSelection(hits);
  }

  function onDesktopPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    // Only start rubber-band if clicking on empty desktop (not on a child element)
    const target = e.target as HTMLElement;
    if (target !== desktopEl) return;

    store.clearDesktopSelection();
    selecting = true;
    selStartX = e.clientX;
    selStartY = e.clientY;
    selCurX = e.clientX;
    selCurY = e.clientY;

    function onMove(me: PointerEvent) {
      selCurX = me.clientX;
      selCurY = me.clientY;
      hitTest();
    }
    function onUp() {
      selecting = false;
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    }
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  function onPointerEnter() {
    if (store.drag.active && store.drag.source !== 'desktop') {
      dropHover = true;
    }
  }

  function onPointerLeave() {
    dropHover = false;
  }

  function onPointerUp(e: PointerEvent) {
    if (!store.drag.active || !store.drag.file || store.drag.source === 'desktop') return;
    if (!desktopEl) return;

    // Only download to desktop if dropped outside the S3V window
    const s3vEl = desktopEl.querySelector('[data-s3v-window]');
    if (s3vEl) {
      const s3vRect = s3vEl.getBoundingClientRect();
      if (e.clientX >= s3vRect.left && e.clientX <= s3vRect.right &&
          e.clientY >= s3vRect.top && e.clientY <= s3vRect.bottom) {
        // Dropped inside S3V window but not on a panel drop zone — ignore
        dropHover = false;
        return;
      }
    }

    store.downloadToDesktop(store.drag.file);
    dropHover = false;
  }
</script>

<div
  class="relative overflow-hidden rounded-xl"
  style="background: linear-gradient(135deg, #1a3a5c 0%, #2d1b4e 50%, #1a3a5c 100%);"
>
  <MenuBar />

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="relative flex min-h-[640px] items-stretch gap-4 p-4 {dropHover ? 'outline-dashed outline-2 -outline-offset-2 outline-green-400/50' : ''}"
    bind:this={desktopEl}
    onpointerdown={onDesktopPointerDown}
    onpointerenter={onPointerEnter}
    onpointerleave={onPointerLeave}
    onpointerup={onPointerUp}
    role="region"
    aria-label="macOS Desktop"
  >
    <S3vClient />

    {#each store.desktopFiles as item (item.file.id)}
      <DesktopFile {item} />
    {/each}

    <!-- Rubber-band selection rectangle -->
    {#if selecting && selRect.w + selRect.h > 4}
      <div
        class="pointer-events-none fixed z-20 rounded-sm border border-blue-400/60 bg-blue-400/15"
        style="left: {selRect.x}px; top: {selRect.y}px; width: {selRect.w}px; height: {selRect.h}px;"
      ></div>
    {/if}
  </div>

  <!-- <CoachOverlay /> -->
</div>
