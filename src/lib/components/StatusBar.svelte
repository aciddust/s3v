<script lang="ts">
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { Circle } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  const activeTab = $derived(profileStore.activeTab);
  const profileId = $derived(profileStore.activeProfileId);

  const fileState = $derived(profileId ? fileStore.getState(profileId) : null);

  const objectCount = $derived(fileState ? fileState.objects.length + fileState.folders.length : 0);

  const selectedCount = $derived(fileState ? fileState.selected.size : 0);

  const selectedSize = $derived(() => {
    if (!fileState || fileState.selected.size === 0) return 0;
    return fileState.objects
      .filter((o) => fileState!.selected.has(o.key))
      .reduce((sum, o) => sum + o.size, 0);
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }
</script>

<div
  class="flex h-6 items-center gap-3 border-t border-border bg-muted/30 px-3 text-xs text-muted-foreground"
>
  <!-- Object count -->
  {#if fileState}
    <span>{m.status_items({ count: objectCount })}</span>

    {#if selectedCount > 0}
      <span class="text-foreground">
        {m.status_selected({ count: selectedCount, size: formatBytes(selectedSize()) })}
      </span>
    {/if}
  {:else}
    <span>{m.status_no_profile()}</span>
  {/if}

  <div class="flex-1"></div>

  <!-- Connection status -->
  {#if activeTab}
    <div class="flex items-center gap-1">
      <Circle
        class="h-2 w-2 fill-current {activeTab.status === 'connected'
          ? 'text-green-500'
          : activeTab.status === 'connecting'
            ? 'text-yellow-400 animate-pulse'
            : activeTab.status === 'error'
              ? 'text-red-500'
              : 'text-zinc-500'}"
      />
      <span>
        {activeTab.profile.name}
        ({activeTab.status === 'connected'
          ? m.status_connected()
          : activeTab.status === 'connecting'
            ? m.status_connecting()
            : activeTab.status === 'error'
              ? m.status_error()
              : m.status_disconnected()})
      </span>
    </div>
  {/if}
</div>
