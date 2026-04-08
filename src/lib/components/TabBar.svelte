<script lang="ts">
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import ProfileTab from './ProfileTab.svelte';
  import { Plus, ChevronLeft, ChevronRight } from '@lucide/svelte';

  const tabs = $derived(profileStore.tabs);
  const activeTabIndex = $derived(profileStore.activeTabIndex);

  let scrollEl: HTMLDivElement | undefined = $state();
  let canScrollLeft = $state(false);
  let canScrollRight = $state(false);

  function updateScrollState() {
    if (!scrollEl) return;
    canScrollLeft = scrollEl.scrollLeft > 0;
    canScrollRight = scrollEl.scrollLeft + scrollEl.clientWidth < scrollEl.scrollWidth - 1;
  }

  function scrollBy(delta: number) {
    scrollEl?.scrollBy({ left: delta, behavior: 'smooth' });
  }

  $effect(() => {
    // Re-check whenever tabs change
    tabs.length;
    // Use a microtask so DOM has updated
    queueMicrotask(updateScrollState);
  });
</script>

<div class="flex h-8 items-end border-b border-border bg-muted/30">
  {#if canScrollLeft}
    <button
      class="flex h-8 w-6 shrink-0 items-center justify-center text-muted-foreground
        hover:bg-muted hover:text-foreground transition-colors"
      onclick={() => scrollBy(-120)}
      title="Scroll tabs left"
    >
      <ChevronLeft class="h-3.5 w-3.5" />
    </button>
  {/if}

  <div
    bind:this={scrollEl}
    class="flex flex-1 items-end overflow-x-hidden"
    onscroll={updateScrollState}
  >
    {#each tabs as tab, i}
      <ProfileTab
        name={tab.profile.name}
        provider={tab.profile.provider}
        active={i === activeTabIndex}
        status={tab.status}
        onactivate={() => profileStore.setActiveTab(tab.profileId)}
        onclose={() => profileStore.closeTab(tab.profileId)}
      />
    {/each}

    <!-- Add new profile button -->
    <button
      class="flex h-8 w-8 shrink-0 items-center justify-center text-muted-foreground
        hover:bg-muted hover:text-foreground transition-colors"
      onclick={() => (uiStore.profileManagerOpen = true)}
      title="Open Profile Manager"
    >
      <Plus class="h-3.5 w-3.5" />
    </button>
  </div>

  {#if canScrollRight}
    <button
      class="flex h-8 w-6 shrink-0 items-center justify-center text-muted-foreground
        hover:bg-muted hover:text-foreground transition-colors"
      onclick={() => scrollBy(120)}
      title="Scroll tabs right"
    >
      <ChevronRight class="h-3.5 w-3.5" />
    </button>
  {/if}
</div>
