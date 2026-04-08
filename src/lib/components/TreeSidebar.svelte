<script lang="ts">
  import type { BucketInfo } from '$lib/api/s3';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import BucketTree from './BucketTree.svelte';
  import BookmarkList from './BookmarkList.svelte';
  import { Separator } from '$lib/components/ui/separator';

  interface Props {
    profileId: string;
    buckets: BucketInfo[];
    activeBucket: string;
    activePrefix: string;
    width?: number;
    onnavigate: (bucket: string, prefix: string) => void;
    onmovetoprefix?: (bucket: string, destPrefix: string, keys: string[]) => void;
  }

  const {
    profileId,
    buckets,
    activeBucket,
    activePrefix,
    width = 220,
    onnavigate,
    onmovetoprefix,
  }: Props = $props();
</script>

<div
  class="flex h-full flex-col border-r border-border bg-background"
  style="width: {width}px; min-width: {width}px;"
>
  <ScrollArea class="flex-1">
    <BucketTree {profileId} {buckets} {activeBucket} {activePrefix} {onnavigate} {onmovetoprefix} />

    {#if buckets.length > 0}
      <Separator class="my-1" />
    {/if}

    <BookmarkList {profileId} {onnavigate} />
  </ScrollArea>
</div>
