<script lang="ts">
  import { dragStore } from '$lib/stores/drag.svelte';
  import { File } from '@lucide/svelte';
</script>

{#if dragStore.active && dragStore.payload}
  <div
    class="fixed pointer-events-none z-50 flex items-center gap-2 rounded-md border border-border bg-background/90 px-3 py-1.5 text-xs shadow-lg backdrop-blur-sm"
    style="left: {dragStore.x + 12}px; top: {dragStore.y + 12}px;"
  >
    <File class="h-3.5 w-3.5 text-muted-foreground" />
    <span class="text-foreground">
      {dragStore.payload.keys.length === 1
        ? dragStore.payload.keys[0].replace(/\/$/, '').split('/').at(-1)
        : `${dragStore.payload.keys.length} items`}
    </span>
  </div>
{/if}
