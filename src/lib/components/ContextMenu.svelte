<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';

  const ctx = $derived(uiStore.contextMenu);

  function handleAction(action: () => void) {
    action();
    uiStore.closeContextMenu();
  }

  $effect(() => {
    if (!ctx.open) return;

    function handleClick() {
      uiStore.closeContextMenu();
    }

    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') uiStore.closeContextMenu();
    }

    window.addEventListener('click', handleClick, { capture: true });
    window.addEventListener('keydown', handleKeydown);

    return () => {
      window.removeEventListener('click', handleClick, { capture: true });
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

{#if ctx.open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed z-50 min-w-40 rounded-md border border-border bg-popover py-1 shadow-md text-popover-foreground"
    style="left: {ctx.x}px; top: {ctx.y}px;"
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#each ctx.items as item}
      {#if item.separator}
        <div class="my-1 border-t border-border"></div>
      {:else}
        <button
          class="flex w-full items-center px-3 py-1.5 text-sm
            {item.disabled
            ? 'cursor-not-allowed text-muted-foreground'
            : 'hover:bg-accent hover:text-accent-foreground cursor-pointer'}
            transition-colors"
          disabled={item.disabled}
          onclick={() => handleAction(item.action)}
        >
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}
