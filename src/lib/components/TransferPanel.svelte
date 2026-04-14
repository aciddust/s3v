<script lang="ts">
  import { transferStore } from '$lib/stores/transfers.svelte';
  import { moveStore } from '$lib/stores/moves.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Badge } from '$lib/components/ui/badge';
  import TransferItem from './TransferItem.svelte';
  import MoveItem from './MoveItem.svelte';
  import { ChevronUp, ChevronDown, ArrowUpDown, ListX } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  const jobs = $derived(transferStore.jobs);
  const moveJobs = $derived(moveStore.jobs);
  const activeCount = $derived(transferStore.activeCount);
  const queuedCount = $derived(transferStore.queuedCount);
  const visible = $derived(uiStore.transferPanelVisible);
  const expanded = $derived(uiStore.transferPanelExpanded);

  const totalBadge = $derived(activeCount + queuedCount + moveStore.activeCount);
  const hasDone = $derived(
    jobs.some(
      (j) => j.status === 'completed' || j.status === 'failed' || j.status === 'cancelled',
    ) || moveJobs.some((j) => j.phase === 'completed' || j.phase === 'failed'),
  );

  function clearAll() {
    transferStore.clearCompleted();
    moveStore.clearCompleted();
  }
</script>

{#if visible}
  <div
    class="fixed bottom-6 right-4 z-40 w-96 rounded-lg border border-border bg-background shadow-lg"
  >
    <!-- Header -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      role="button"
      tabindex="0"
      class="flex h-7 w-full items-center gap-2 px-3 hover:bg-accent/30 transition-colors select-none cursor-pointer rounded-t-lg"
      onclick={() => uiStore.toggleTransferPanelContent()}
    >
      <ArrowUpDown class="h-3.5 w-3.5 text-muted-foreground" />
      <span class="text-xs font-medium">{m.transfer_panel_title()}</span>

      {#if totalBadge > 0}
        <Badge variant="secondary" class="h-4 px-1.5 text-[10px]">{totalBadge}</Badge>
      {/if}

      <div class="flex-1"></div>

      {#if expanded && hasDone}
        <button
          class="p-0.5 rounded hover:bg-accent/50"
          onclick={(e) => {
            e.stopPropagation();
            clearAll();
          }}
          title="Clear completed"
        >
          <ListX class="h-3.5 w-3.5 text-muted-foreground" />
        </button>
      {/if}

      {#if expanded}
        <ChevronDown class="h-3.5 w-3.5 text-muted-foreground" />
      {:else}
        <ChevronUp class="h-3.5 w-3.5 text-muted-foreground" />
      {/if}
    </div>

    <!-- Panel content -->
    <div
      class="grid transition-[grid-template-rows] duration-300 ease-in-out"
      style="grid-template-rows: {expanded ? '1fr' : '0fr'};"
    >
      <div class="overflow-hidden">
        <ScrollArea class="h-48">
          {#if jobs.length === 0 && moveJobs.length === 0}
            <div class="flex items-center justify-center py-6 text-xs text-muted-foreground">
              No transfers
            </div>
          {:else}
            {#each moveJobs as job}
              <MoveItem {job} />
            {/each}
            {#each jobs as job}
              <TransferItem {job} />
            {/each}
          {/if}
        </ScrollArea>
      </div>
    </div>
  </div>
{/if}
