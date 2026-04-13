<script lang="ts">
  import { store } from './store.svelte';
  import { Upload, Download, Copy, Check, CircleAlert, X, ChevronUp, ChevronDown, ArrowUpDown, ListX } from '@lucide/svelte';

  const directionIcons = { upload: Upload, download: Download, copy: Copy } as const;

  const statusColors: Record<string, string> = {
    queued: 'text-muted-foreground',
    active: 'text-blue-500',
    completed: 'text-green-500',
    failed: 'text-destructive',
  };

  const hasDone = $derived(store.transfers.some((t) => t.status === 'completed' || t.status === 'failed'));

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }
</script>

{#if store.transferPanelVisible}
<div class="absolute bottom-4 right-4 z-30 w-80 rounded-lg border border-border bg-background shadow-lg">
  <!-- Header -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    role="button"
    tabindex="0"
    class="flex h-7 w-full items-center gap-2 px-3 hover:bg-accent/30 transition-colors select-none cursor-pointer rounded-t-lg"
    onclick={() => (store.transferPanelExpanded = !store.transferPanelExpanded)}
  >
    <ArrowUpDown class="h-3.5 w-3.5 text-muted-foreground" />
    <span class="text-xs font-medium">Transfers</span>

    {#if store.activeTransferCount > 0}
      <span class="rounded-full bg-secondary px-1.5 text-[10px] text-secondary-foreground">{store.activeTransferCount}</span>
    {/if}

    <div class="flex-1"></div>

    {#if store.transferPanelExpanded && hasDone}
      <button
        class="p-0.5 rounded hover:bg-accent/50"
        onclick={(e) => { e.stopPropagation(); store.clearCompletedTransfers(); }}
        title="Clear completed"
      >
        <ListX class="h-3.5 w-3.5 text-muted-foreground" />
      </button>
    {/if}

    {#if store.transferPanelExpanded}
      <ChevronDown class="h-3.5 w-3.5 text-muted-foreground" />
    {:else}
      <ChevronUp class="h-3.5 w-3.5 text-muted-foreground" />
    {/if}
  </div>

  <!-- Panel content -->
  <div
    class="grid transition-[grid-template-rows] duration-300 ease-in-out"
    style="grid-template-rows: {store.transferPanelExpanded ? '1fr' : '0fr'};"
  >
    <div class="overflow-hidden">
      <div class="max-h-36 overflow-auto">
        {#if store.transfers.length === 0}
          <div class="flex items-center justify-center py-4 text-xs text-muted-foreground">No transfers</div>
        {:else}
          {#each store.transfers as transfer (transfer.id)}
            {@const Icon = directionIcons[transfer.direction]}
            <div class="flex items-center gap-2 border-b border-border/50 px-3 py-1.5">
              <div class="shrink-0 {statusColors[transfer.status] ?? 'text-muted-foreground'}">
                <Icon class="h-3.5 w-3.5" />
              </div>
              <div class="flex min-w-0 flex-1 flex-col gap-0.5">
                <div class="flex items-center justify-between gap-2">
                  <span class="truncate text-xs font-medium">{transfer.file.name}</span>
                  <span class="shrink-0 text-xs text-muted-foreground">{Math.round(transfer.progress)}%</span>
                </div>
                <div class="h-1 w-full overflow-hidden rounded-full bg-secondary">
                  <div
                    class="h-full rounded-full transition-all duration-200
                      {transfer.status === 'completed' ? 'bg-green-500' : transfer.status === 'failed' ? 'bg-destructive' : 'bg-blue-500'}"
                    style="width: {transfer.progress}%;"
                  ></div>
                </div>
                <div class="flex items-center justify-between text-[10px] text-muted-foreground">
                  <span>{formatBytes(transfer.file.size)}</span>
                  <span class={statusColors[transfer.status]}>{transfer.status}</span>
                </div>
              </div>
              {#if transfer.status === 'completed'}
                <button class="shrink-0 rounded-md p-0.5 hover:bg-accent/50" onclick={() => store.removeTransfer(transfer.id)}>
                  <Check class="h-3.5 w-3.5 text-green-500" />
                </button>
              {:else if transfer.status === 'failed'}
                <button class="shrink-0 rounded-md p-0.5 hover:bg-accent/50" onclick={() => store.removeTransfer(transfer.id)}>
                  <CircleAlert class="h-3.5 w-3.5 text-destructive" />
                </button>
              {:else}
                <button class="shrink-0 rounded-md p-0.5 hover:bg-accent/50" onclick={() => store.removeTransfer(transfer.id)} title="Cancel">
                  <X class="h-3 w-3 text-muted-foreground" />
                </button>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
</div>
{/if}
