<script lang="ts">
  import type { TransferJobSummary } from '$lib/api/transfers';
  import { pauseTransfer, resumeTransfer, cancelTransfer } from '$lib/api/transfers';
  import { transferStore } from '$lib/stores/transfers.svelte';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { Progress } from '$lib/components/ui/progress';
  import { Button } from '$lib/components/ui/button';
  import { Upload, Download, Pause, Play, X, Check, CircleAlert, FolderOpen } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    job: TransferJobSummary;
  }

  const { job }: Props = $props();

  const fileName = $derived(job.path.remote_key.split('/').at(-1) ?? job.path.remote_key);

  const percent = $derived(() => {
    const total = job.progress.total_bytes;
    if (total === 0) return 0;
    return Math.round((job.progress.bytes_transferred / total) * 100);
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  const statusColors: Record<string, string> = {
    queued: 'text-muted-foreground',
    active: 'text-blue-500',
    paused: 'text-amber-500',
    completed: 'text-green-500',
    failed: 'text-destructive',
    cancelled: 'text-muted-foreground',
  };

  const isDone = $derived(
    job.status === 'completed' || job.status === 'failed' || job.status === 'cancelled',
  );

  function handleDblClick() {
    if (job.status === 'completed' && job.path.local) {
      revealItemInDir(job.path.local).catch(() => {});
    }
  }
</script>

<div
  role="listitem"
  class="flex items-center gap-2 border-b border-border/50 px-3 py-1.5 {job.status === 'completed' && job.path.local ? 'cursor-pointer' : ''}"
  ondblclick={handleDblClick}
>
  <!-- Type icon -->
  <div class="shrink-0 {statusColors[job.status] ?? 'text-muted-foreground'}">
    {#if job.transfer_type === 'upload'}
      <Upload class="h-3.5 w-3.5" />
    {:else}
      <Download class="h-3.5 w-3.5" />
    {/if}
  </div>

  <!-- File name + progress -->
  <div class="flex min-w-0 flex-1 flex-col gap-0.5">
    <div class="flex items-center justify-between gap-2">
      <span class="truncate text-xs font-medium" title={job.path.remote_key}>{fileName}</span>
      <span class="shrink-0 text-xs text-muted-foreground">{percent()}%</span>
    </div>
    <Progress value={percent()} class="h-1" />
    <div class="flex items-center justify-between text-[10px] text-muted-foreground">
      <span
        >{formatBytes(job.progress.bytes_transferred)} / {formatBytes(
          job.progress.total_bytes,
        )}</span
      >
      {#if job.error}
        <span class="text-destructive truncate max-w-32" title={job.error}>{job.error}</span>
      {:else}
        <span class={statusColors[job.status]}>{job.status === 'queued' ? m.transfer_status_queued() :
 job.status === 'active' ? m.transfer_status_active() :
 job.status === 'paused' ? m.transfer_status_paused() :
 job.status === 'completed' ? m.transfer_status_completed() :
 job.status === 'failed' ? m.transfer_status_failed() :
 m.transfer_status_cancelled()}</span>
      {/if}
    </div>
  </div>

  <!-- Controls -->
  <div class="flex shrink-0 items-center gap-0.5">
    {#if job.status === 'active'}
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6"
        onclick={() => pauseTransfer(job.id)}
        title={m.transfer_pause()}
      >
        <Pause class="h-3 w-3" />
      </Button>
    {:else if job.status === 'paused'}
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6"
        onclick={() => resumeTransfer(job.id)}
        title={m.transfer_resume()}
      >
        <Play class="h-3 w-3" />
      </Button>
    {:else if isDone}
      {#if job.status === 'completed' && job.path.local}
        <Button
          variant="ghost"
          size="icon"
          class="h-6 w-6"
          onclick={() => revealItemInDir(job.path.local).catch(() => {})}
          title={m.transfer_open_folder()}
        >
          <FolderOpen class="h-3 w-3" />
        </Button>
      {/if}
      <button
        class="group relative h-6 w-6 flex items-center justify-center rounded-md hover:bg-accent/50"
        onclick={() => transferStore.removeJob(job.id)}
        title={m.transfer_dismiss()}
      >
        {#if job.status === 'completed'}
          <Check class="h-3.5 w-3.5 text-green-500 group-hover:hidden" />
        {:else if job.status === 'failed'}
          <CircleAlert class="h-3.5 w-3.5 text-destructive group-hover:hidden" />
        {:else}
          <X class="h-3.5 w-3.5 text-muted-foreground group-hover:hidden" />
        {/if}
        <X class="h-3.5 w-3.5 text-muted-foreground hidden group-hover:block" />
      </button>
    {/if}

    {#if job.status === 'active' || job.status === 'queued' || job.status === 'paused'}
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6"
        onclick={() => cancelTransfer(job.id)}
        title={m.transfer_cancel()}
      >
        <X class="h-3 w-3" />
      </Button>
    {/if}
  </div>
</div>
