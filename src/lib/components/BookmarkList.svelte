<script lang="ts">
  import { bookmarkStore } from '$lib/stores/bookmarks.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Star, X } from '@lucide/svelte';

  interface Props {
    profileId: string;
    onnavigate: (bucket: string, prefix: string) => void;
  }

  const { profileId, onnavigate }: Props = $props();

  const bookmarks = $derived(bookmarkStore.getForProfile(profileId));
</script>

{#if bookmarks.length > 0}
  <div class="mt-2">
    <div class="flex items-center gap-1.5 px-2 py-1">
      <Star class="h-3 w-3 text-amber-400" />
      <span class="text-xs font-medium text-muted-foreground uppercase tracking-wider"
        >Bookmarks</span
      >
    </div>

    <div class="space-y-0.5">
      {#each bookmarks as bm}
        <div class="group flex items-center gap-1 rounded px-2 py-0.5 hover:bg-accent/50">
          <button
            class="flex-1 truncate text-left text-xs text-muted-foreground hover:text-foreground transition-colors"
            onclick={() => onnavigate(bm.bucket, bm.prefix)}
            title="{bm.bucket}/{bm.prefix}"
          >
            {bm.label}
          </button>
          <button
            class="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
            onclick={() => bookmarkStore.remove(bm.id)}
            title="Remove bookmark"
          >
            <X class="h-3 w-3 text-muted-foreground hover:text-destructive" />
          </button>
        </div>
      {/each}
    </div>
  </div>
{/if}
