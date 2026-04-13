<script lang="ts">
  import { store } from './store.svelte';
  import { Copy, ArrowRight, Upload } from '@lucide/svelte';

  const isPanelSource = $derived(store.drag.source === 'left-panel' || store.drag.source === 'right-panel');
</script>

{#if store.drag.active && store.drag.file}
  <div
    class="pointer-events-none fixed z-50 flex flex-col gap-1"
    style="left: {store.drag.x + 14}px; top: {store.drag.y + 14}px;"
  >
    <!-- File name -->
    <div class="flex items-center gap-1.5 rounded-md border border-blue-400/40 bg-blue-400/15 px-2 py-1 text-xs text-blue-300 shadow-lg backdrop-blur-sm">
      <span class="max-w-32 truncate">{store.drag.file.name}</span>
    </div>

    <!-- Action tooltip -->
    {#if store.drag.source === 'desktop'}
      <div class="flex items-center gap-1 rounded-md bg-black/70 px-2 py-0.5 text-[10px] text-white/80 backdrop-blur-sm">
        <Upload class="h-3 w-3" /> Upload
      </div>
    {:else if isPanelSource}
      <div class="flex items-center gap-1 rounded-md bg-black/70 px-2 py-0.5 text-[10px] text-white/80 backdrop-blur-sm">
        {#if store.drag.modifier}
          <Copy class="h-3 w-3" /> Copy
        {:else}
          <ArrowRight class="h-3 w-3" /> Move
        {/if}
      </div>
    {/if}
  </div>
{/if}
