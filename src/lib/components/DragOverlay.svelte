<script lang="ts">
  import { dragStore } from '$lib/stores/drag.svelte';
  import { File, Download } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  const modifierLabel = $derived(
    dragStore.nearEdge
      ? m.drag_download()
      : dragStore.modifierKey === 'meta'
        ? m.drag_copy()
        : dragStore.modifierKey === 'shift'
          ? m.drag_move()
          : null,
  );
</script>

{#if dragStore.active && dragStore.payload}
  <div
    class="fixed pointer-events-none z-50 flex items-center gap-2 rounded-md border border-border bg-background/90 px-3 py-1.5 text-xs shadow-lg backdrop-blur-sm"
    style="left: {dragStore.x + 12}px; top: {dragStore.y + 12}px;"
  >
    {#if dragStore.nearEdge}
      <Download class="h-3.5 w-3.5 text-muted-foreground" />
    {:else}
      <File class="h-3.5 w-3.5 text-muted-foreground" />
    {/if}
    <span class="text-foreground">
      {dragStore.payload.keys.length === 1
        ? dragStore.payload.keys[0].replace(/\/$/, '').split('/').at(-1)
        : m.drag_items({ count: dragStore.payload.keys.length })}
    </span>
    {#if modifierLabel}
      <span class="text-muted-foreground">·</span>
      <span class="font-medium text-primary">{modifierLabel}</span>
    {/if}
  </div>
{/if}
