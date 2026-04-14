<script lang="ts">
  import { dragStore } from '$lib/stores/drag.svelte';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { Monitor } from '@lucide/svelte';

  const profile = $derived(
    dragStore.tabDragPayload
      ? profileStore.tabs.find((t) => t.profileId === dragStore.tabDragPayload!.profileId)?.profile
      : null,
  );
</script>

{#if dragStore.tabDragActive && dragStore.tabDragPayload && profile}
  <div
    class="fixed pointer-events-none z-50 flex items-center gap-2 rounded-md border border-border bg-background/90 px-3 py-1.5 text-xs shadow-lg backdrop-blur-sm"
    style="left: {dragStore.tabDragX + 12}px; top: {dragStore.tabDragY + 12}px;"
  >
    <Monitor class="h-3.5 w-3.5 text-muted-foreground" />
    <span class="text-foreground">{profile.name}</span>
  </div>
{/if}
