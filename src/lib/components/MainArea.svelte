<script lang="ts">
  import type { BucketInfo } from '$lib/api/s3';
  import { uiStore } from '$lib/stores/ui.svelte';
  import TreeSidebar from './TreeSidebar.svelte';
  import FilePanel from './FilePanel.svelte';

  interface Props {
    profileId: string;
    buckets: BucketInfo[];
    activeBucket: string;
    activePrefix: string;
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
    buckets,
    activeBucket,
    activePrefix,
    onnavigate,
    oncontextmenu,
    onbgcontextmenu,
    onmovetoprefix,
    oncopytoprefix,
    onfileopen,
  }: Props = $props();

  const sidebarWidth = $derived(uiStore.sidebarWidth);
</script>

<div class="flex flex-1 min-h-0 overflow-hidden">
  <!-- Tree sidebar -->
  <TreeSidebar
    {profileId}
    {buckets}
    {activeBucket}
    {activePrefix}
    width={sidebarWidth}
    {onnavigate}
    {onmovetoprefix}
    {onbgcontextmenu}
  />

  <!-- Resize handle -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="w-1 cursor-col-resize bg-border hover:bg-primary/50 transition-colors"
    onmousedown={(e) => {
      e.preventDefault();
      const startX = e.clientX;
      const startWidth = sidebarWidth;
      function onMove(me: MouseEvent) {
        const delta = me.clientX - startX;
        uiStore.sidebarWidth = Math.max(160, Math.min(400, startWidth + delta));
      }
      function onUp() {
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      }
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    }}
  ></div>

  <!-- File panel -->
  <div class="flex flex-1 flex-col overflow-hidden">
    <FilePanel
      {profileId}
      {onnavigate}
      {oncontextmenu}
      {onbgcontextmenu}
      {onmovetoprefix}
      {oncopytoprefix}
      {onfileopen}
    />
  </div>
</div>
