<script lang="ts">
  import { bookmarkStore } from '$lib/stores/bookmarks.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { ChevronRight, Star } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    profileId: string;
    bucket: string;
    prefix: string;
    onnavigate: (bucket: string, prefix: string) => void;
  }

  const { profileId, bucket, prefix, onnavigate }: Props = $props();

  // Build path segments from prefix
  const segments = $derived(() => {
    const parts: { label: string; prefix: string }[] = [];
    if (!prefix) return parts;
    const chunks = prefix.split('/').filter(Boolean);
    let accumulated = '';
    for (const chunk of chunks) {
      accumulated += chunk + '/';
      parts.push({ label: chunk, prefix: accumulated });
    }
    return parts;
  });

  const isBookmarked = $derived(
    bookmarkStore.bookmarks.some(
      (b) => b.profileId === profileId && b.bucket === bucket && b.prefix === prefix,
    ),
  );

  function toggleBookmark() {
    if (isBookmarked) {
      const bm = bookmarkStore.bookmarks.find(
        (b) => b.profileId === profileId && b.bucket === bucket && b.prefix === prefix,
      );
      if (bm) bookmarkStore.remove(bm.id);
    } else {
      const label = prefix ? (prefix.split('/').filter(Boolean).at(-1) ?? bucket) : bucket;
      bookmarkStore.add({ profileId, bucket, prefix, label });
    }
  }
</script>

<div class="flex h-8 items-center gap-1 border-b border-border bg-background px-2 text-xs">
  {#if bucket}
    <!-- Bucket root -->
    <button
      class="rounded px-1.5 py-0.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
      onclick={() => onnavigate(bucket, '')}
    >
      {bucket}
    </button>

    <!-- Prefix segments -->
    {#each segments() as seg}
      <ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
      <button
        class="rounded px-1.5 py-0.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        onclick={() => onnavigate(bucket, seg.prefix)}
      >
        {seg.label}
      </button>
    {/each}

    <div class="flex-1"></div>

    <!-- Bookmark toggle -->
    <Button
      variant="ghost"
      size="icon"
      class="h-6 w-6"
      onclick={toggleBookmark}
      title={isBookmarked ? 'Remove bookmark' : 'Bookmark this location'}
    >
      <Star
        class="h-3.5 w-3.5 {isBookmarked
          ? 'fill-amber-400 text-amber-400'
          : 'text-muted-foreground'}"
      />
    </Button>
  {:else}
    <span class="text-muted-foreground">Select a bucket</span>
  {/if}
</div>
