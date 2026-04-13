<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { fileStore } from '$lib/stores/files.svelte';

  import Breadcrumb from './Breadcrumb.svelte';
  import FileList from './FileList.svelte';

  interface Props {
    profileId: string;
    onnavigate: (bucket: string, prefix: string) => void;
    oncontextmenu: (e: MouseEvent, keys: string[]) => void;
    onbgcontextmenu?: (e: MouseEvent) => void;
    onmovetoprefix?: (bucket: string, destPrefix: string, keys: string[]) => void;
    oncopytoprefix?: (
      sourceBucket: string,
      destBucket: string,
      destPrefix: string,
      keys: string[],
    ) => void;
    onfileopen?: (bucket: string, key: string) => void;
  }

  const {
    profileId,
    onnavigate,
    oncontextmenu,
    onbgcontextmenu,
    onmovetoprefix,
    oncopytoprefix,
    onfileopen,
  }: Props = $props();

  const leftState = $derived(fileStore.getState(profileId));
  const rightPanelId = $derived(`${profileId}::right`);
  const rightState = $derived(fileStore.getState(rightPanelId));
  const dualPanel = $derived(uiStore.dualPanel);
  const activePanel = $derived(uiStore.activePanel);

  // Initialize right panel when dual panel mode is enabled
  let prevDualPanel = $state(false);
  $effect(() => {
    if (dualPanel && !prevDualPanel) {
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
    fileStore.navigate(`${profileId}::right`, bucket, prefix);
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
</script>

<div class="flex flex-1 min-h-0 flex-col overflow-hidden">
  {#if dualPanel}
    <!-- Dual panel mode: two independent side-by-side panels -->
    <div class="flex flex-1 overflow-hidden">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="flex flex-1 flex-col overflow-hidden border-r border-border {activePanel === 'left'
          ? 'ring-1 ring-primary/40 ring-inset'
          : ''}"
        onmousedown={activateLeft}
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
      </div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="flex flex-1 flex-col overflow-hidden {activePanel === 'right'
          ? 'ring-1 ring-primary/40 ring-inset'
          : ''}"
        onmousedown={activateRight}
        onfocusin={activateRight}
      >
        <Breadcrumb
          {profileId}
          bucket={rightState.bucket}
          prefix={rightState.prefix}
          onnavigate={navigateRight}
        />
        <FileList
          profileId={`${profileId}::right`}
          onnavigate={navigateRight}
          {oncontextmenu}
          {onbgcontextmenu}
          {onmovetoprefix}
          {oncopytoprefix}
          {onfileopen}
        />
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
    <div class="flex flex-1 flex-col overflow-hidden min-h-0">
      <FileList
        {profileId}
        onnavigate={navigateLeft}
        {oncontextmenu}
        {onbgcontextmenu}
        {onmovetoprefix}
        {oncopytoprefix}
        {onfileopen}
      />
    </div>
  {/if}
</div>
