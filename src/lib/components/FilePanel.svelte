<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { dragStore } from '$lib/stores/drag.svelte';

  import Breadcrumb from './Breadcrumb.svelte';
  import FileList from './FileList.svelte';

  interface Props {
    profileId: string;
    rightProfileId?: string;
    rightProfileName?: string;
    onnavigate: (bucket: string, prefix: string) => void;
    oncontextmenu: (e: MouseEvent, keys: string[]) => void;
    onbgcontextmenu?: (e: MouseEvent) => void;
    onmovetoprefix?: (bucket: string, destPrefix: string, keys: string[]) => void;
    oncopytoprefix?: (
      sourceProfileId: string,
      sourceBucket: string,
      destBucket: string,
      destPrefix: string,
      keys: string[],
    ) => void;
    onfileopen?: (bucket: string, key: string) => void;
  }

  const {
    profileId,
    rightProfileId,
    rightProfileName,
    onnavigate,
    oncontextmenu,
    onbgcontextmenu,
    onmovetoprefix,
    oncopytoprefix,
    onfileopen,
  }: Props = $props();

  const leftState = $derived(fileStore.getState(profileId));
  const effectiveRightProfileId = $derived(rightProfileId ?? profileId);
  const rightPanelId = $derived(`${effectiveRightProfileId}::right`);
  const rightState = $derived(fileStore.getState(rightPanelId));
  const dualPanel = $derived(uiStore.dualPanel);
  const activePanel = $derived(uiStore.activePanel);
  const tabDragActive = $derived(dragStore.tabDragActive);
  const hoverPanelSide = $derived(dragStore.hoverPanelSide);

  // Initialize right panel when dual panel mode is enabled (only if no independent profile)
  let prevDualPanel = $state(false);
  $effect(() => {
    if (dualPanel && !prevDualPanel && !rightProfileId) {
      fileStore.initRightPanel(profileId);
    }
    prevDualPanel = dualPanel;
  });

  function navigateLeft(bucket: string, prefix: string) {
    fileStore.navigate(profileId, bucket, prefix);
    if (activePanel === 'left') {
      onnavigate(bucket, prefix);
    }
  }

  function navigateRight(bucket: string, prefix: string) {
    fileStore.navigate(rightPanelId, bucket, prefix);
    if (activePanel === 'right') {
      onnavigate(bucket, prefix);
    }
  }

  function activateLeft() {
    uiStore.setActivePanel('left');
  }

  function activateRight() {
    uiStore.setActivePanel('right');
  }

  function handleMouseNav(e: MouseEvent, storeKey: string) {
    if (e.button === 3) {
      e.preventDefault();
      fileStore.goBack(storeKey);
    } else if (e.button === 4) {
      e.preventDefault();
      fileStore.goForward(storeKey);
    }
  }
</script>

<div class="flex flex-1 min-h-0 flex-col overflow-hidden">
  {#if dualPanel}
    <!-- Dual panel mode: two independent side-by-side panels -->
    <div class="flex flex-1 overflow-hidden">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        data-panel-side="left"
        class="relative flex flex-1 flex-col overflow-hidden border-r border-border {activePanel === 'left'
          ? 'ring-1 ring-primary/40 ring-inset'
          : ''}"
        onmousedown={activateLeft}
        onmouseup={(e) => handleMouseNav(e, profileId)}
        onfocusin={activateLeft}
      >
        <Breadcrumb
          {profileId}
          bucket={leftState.bucket}
          prefix={leftState.prefix}
          onnavigate={navigateLeft}
        />
        <FileList
          {profileId}
          onnavigate={navigateLeft}
          {oncontextmenu}
          {onbgcontextmenu}
          {onmovetoprefix}
          {oncopytoprefix}
          {onfileopen}
        />
        {#if tabDragActive && hoverPanelSide === 'left'}
          <div class="absolute inset-0 bg-primary/10 ring-2 ring-primary ring-inset pointer-events-none z-10"></div>
        {/if}
      </div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        data-panel-side="right"
        class="relative flex flex-1 flex-col overflow-hidden {activePanel === 'right'
          ? 'ring-1 ring-primary/40 ring-inset'
          : ''}"
        onmousedown={activateRight}
        onmouseup={(e) => handleMouseNav(e, rightPanelId)}
        onfocusin={activateRight}
      >
        <Breadcrumb
          profileId={effectiveRightProfileId}
          bucket={rightState.bucket}
          prefix={rightState.prefix}
          onnavigate={navigateRight}
          profileName={rightProfileId ? rightProfileName : undefined}
        />
        <FileList
          profileId={rightPanelId}
          onnavigate={navigateRight}
          {oncontextmenu}
          {onbgcontextmenu}
          {onmovetoprefix}
          {oncopytoprefix}
          {onfileopen}
        />
        {#if tabDragActive && hoverPanelSide === 'right'}
          <div class="absolute inset-0 bg-primary/10 ring-2 ring-primary ring-inset pointer-events-none z-10"></div>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Single panel mode -->
    <Breadcrumb
      {profileId}
      bucket={leftState.bucket}
      prefix={leftState.prefix}
      onnavigate={navigateLeft}
    />
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="relative flex flex-1 flex-col overflow-hidden min-h-0"
      onmouseup={(e) => handleMouseNav(e, profileId)}
    >
      <FileList
        {profileId}
        onnavigate={navigateLeft}
        {oncontextmenu}
        {onbgcontextmenu}
        {onmovetoprefix}
        {oncopytoprefix}
        {onfileopen}
      />
      {#if tabDragActive && hoverPanelSide}
        <div class="absolute inset-0 flex pointer-events-none z-10">
          <div class="flex-1 {hoverPanelSide === 'left' ? 'bg-primary/10 ring-2 ring-primary ring-inset' : ''}"></div>
          <div class="w-px bg-border"></div>
          <div class="flex-1 {hoverPanelSide === 'right' ? 'bg-primary/10 ring-2 ring-primary ring-inset' : ''}"></div>
        </div>
      {/if}
    </div>
  {/if}
</div>
