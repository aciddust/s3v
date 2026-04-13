<script lang="ts">
  import type { MoveJob } from '$lib/stores/moves.svelte';
  import { moveStore } from '$lib/stores/moves.svelte';
  import { Progress } from '$lib/components/ui/progress';
  import { FolderInput, FolderOutput, Check, CircleAlert, LoaderCircle, X } from '@lucide/svelte';

  interface Props {
    job: MoveJob;
  }

  const { job }: Props = $props();

  const percent = $derived(job.total === 0 ? 0 : Math.round((job.done / job.total) * 100));

  const fileName = $derived(job.key ? (job.key.split('/').at(-1) ?? job.key) : '');

  const isCopy = $derived(job.op === 'copy');

  const actionLabel = $derived(isCopy ? 'Copying' : 'Moving');

  const phaseLabel = $derived.by(() => {
    switch (job.phase) {
      case 'start':
      case 'started':
        return `${actionLabel}...`;
      case 'copying':
        return fileName ? `${actionLabel} ${fileName}` : `${actionLabel}...`;
      case 'deleting':
        return `Deleting ${fileName}`;
      case 'moved':
        return `${job.done}/${job.total} done`;
      case 'completed':
        return 'Completed';
      case 'failed':
        return 'Failed';
      case 'cancelled':
        return 'Cancelled';
      default:
        return job.phase;
    }
  });

  function formatEta(seconds: number | null): string {
    if (seconds === null) return '';
    if (seconds < 1) return '< 1s';
    if (seconds < 60) return `~${seconds}s`;
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `~${m}m ${s}s`;
  }

  const isActive = $derived(job.phase !== 'completed' && job.phase !== 'failed');

  const statusColor = $derived(
    job.phase === 'completed'
      ? 'text-green-500'
      : job.phase === 'failed'
        ? 'text-destructive'
        : 'text-blue-500',
  );
</script>

<div class="flex items-center gap-2 border-b border-border/50 px-3 py-1.5">
  <div class="shrink-0 {statusColor}">
    {#if isCopy}
      <FolderOutput class="h-3.5 w-3.5" />
    {:else}
      <FolderInput class="h-3.5 w-3.5" />
    {/if}
  </div>

  <div class="flex min-w-0 flex-1 flex-col gap-0.5">
    <div class="flex items-center justify-between gap-2">
      <span
        class="truncate text-xs font-medium"
        title="{isCopy ? 'Copy' : 'Move'} {job.total} file(s) → {job.destPrefix || '/'}"
      >
        {job.total === 1 ? fileName || 'Move' : `${job.total} files`} → {job.destPrefix || '/'}
      </span>
      {#if !(isCopy && job.total <= 1 && isActive)}
        <span class="shrink-0 text-xs text-muted-foreground">{percent}%</span>
      {/if}
    </div>
    <Progress value={percent} class="h-1" />
    <div class="flex items-center justify-between text-[10px] text-muted-foreground">
      <span class="truncate {statusColor}">{phaseLabel}</span>
      {#if isActive && job.eta !== null}
        <span class="shrink-0">{formatEta(job.eta)}</span>
      {/if}
    </div>
  </div>

  <div class="flex shrink-0 items-center gap-0.5">
    {#if isActive}
      <LoaderCircle class="h-3.5 w-3.5 animate-spin text-blue-500" />
    {:else}
      <button
        class="group relative h-6 w-6 flex items-center justify-center rounded-md hover:bg-accent/50"
        onclick={() => moveStore.removeJob(job.id)}
        title="Dismiss"
      >
        {#if job.phase === 'completed'}
          <Check class="h-3.5 w-3.5 text-green-500 group-hover:hidden" />
        {:else}
          <CircleAlert class="h-3.5 w-3.5 text-destructive group-hover:hidden" />
        {/if}
        <X class="h-3.5 w-3.5 text-muted-foreground hidden group-hover:block" />
      </button>
    {/if}
  </div>
</div>
