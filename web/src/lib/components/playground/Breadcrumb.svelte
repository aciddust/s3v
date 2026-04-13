<script lang="ts">
  import { ChevronRight } from '@lucide/svelte';

  let { bucket, currentPath, onNavigate }: {
    bucket: string;
    currentPath: string;
    onNavigate: (path: string) => void;
  } = $props();

  let segments = $derived.by(() => {
    const parts = currentPath.split('/').filter(Boolean);
    const result: { label: string; path: string }[] = [];
    let accumulated = '';
    for (const part of parts) {
      accumulated += part + '/';
      result.push({ label: part, path: accumulated });
    }
    return result;
  });
</script>

<div class="flex h-7 items-center gap-1 border-b border-border bg-background px-2 text-xs shrink-0">
  <button
    class="rounded px-1.5 py-0.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
    onclick={() => onNavigate('')}
  >
    {bucket}
  </button>

  {#each segments as seg}
    <ChevronRight class="h-3 w-3 shrink-0 text-muted-foreground" />
    <button
      class="rounded px-1.5 py-0.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
      onclick={() => onNavigate(seg.path)}
    >
      {seg.label}
    </button>
  {/each}
</div>
