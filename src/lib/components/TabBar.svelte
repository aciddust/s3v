<script lang="ts">
  import { onMount } from 'svelte';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { dragStore } from '$lib/stores/drag.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import ProfileTab from './ProfileTab.svelte';
  import { Plus, ChevronLeft, ChevronRight } from '@lucide/svelte';

  interface Props {
    ontabdrop?: (targetProfileId: string, modifier: 'meta' | 'shift' | null) => void;
  }

  const { ontabdrop }: Props = $props();

  const allTabs = $derived(profileStore.tabs);
  // Hide tabs that are "docked" in the right panel
  const hiddenProfileId = $derived(uiStore.dualPanel ? uiStore.rightPanelProfileId : null);
  const tabs = $derived(
    hiddenProfileId ? allTabs.filter((t) => t.profileId !== hiddenProfileId) : allTabs,
  );

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
    // Re-check whenever tabs change — intentional reactive dependency tracking
    // oxlint-disable-next-line no-unused-expressions
    tabs.length;
    // Use a microtask so DOM has updated
    queueMicrotask(updateScrollState);
  });

  onMount(() => {
    function onInternalDrop(e: Event) {
      if (!dragStore.payload) return;
      const target = e.target as HTMLElement;
      const tabEl = target.closest('[data-tab-profile-id]') as HTMLElement | null;
      if (!tabEl) return;

      const targetProfileId = tabEl.dataset.tabProfileId!;
      // Skip dropping on the same tab the drag originated from
      const sourceProfileId = dragStore.payload.profileId.replace(/::right$/, '');
      if (sourceProfileId === targetProfileId) {
        dragStore.consume();
        return;
      }

      const modifier =
        (e as CustomEvent<{ modifier: 'meta' | 'shift' | null }>).detail?.modifier ?? null;
      ontabdrop?.(targetProfileId, modifier);
    }

    scrollEl?.addEventListener('internaldrop', onInternalDrop);
    return () => {
      scrollEl?.removeEventListener('internaldrop', onInternalDrop);
    };
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
    {#each tabs as tab}
      <ProfileTab
        name={tab.profile.name}
        provider={tab.profile.provider}
        active={tab.profileId === profileStore.activeProfileId}
        status={tab.status}
        profileId={tab.profileId}
        droppable={dragStore.active &&
          dragStore.payload?.profileId?.replace(/::right$/, '') !== tab.profileId}
        dragHover={dragStore.active && dragStore.hoverTabProfileId === tab.profileId}
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
