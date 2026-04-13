<script lang="ts">
  import { logStore, type LogEntry } from '$lib/stores/logs.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { Button } from '$lib/components/ui/button';
  import { X, Trash2, ClipboardCopy, Download, Copy } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';

  let scrollRef = $state<HTMLDivElement | null>(null);
  let copiedLineId = $state<number | null>(null);
  let panelHeight = $state(200);
  let resizing = $state(false);

  function startResize(e: MouseEvent) {
    e.preventDefault();
    resizing = true;
    const startY = e.clientY;
    const startHeight = panelHeight;

    function onMove(me: MouseEvent) {
      const delta = startY - me.clientY;
      panelHeight = Math.max(100, Math.min(600, startHeight + delta));
    }
    function onUp() {
      resizing = false;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  const activeBucket = $derived(() => {
    const pid = profileStore.activeProfileId;
    if (!pid) return undefined;
    return fileStore.getState(pid).bucket || undefined;
  });

  const levelColors: Record<string, string> = {
    debug: 'text-zinc-500',
    info: 'text-sky-400',
    warn: 'text-amber-400',
    error: 'text-red-400',
  };

  $effect(() => {
    logStore.entries.length;
    if (scrollRef) {
      requestAnimationFrame(() => {
        scrollRef?.scrollTo({ top: scrollRef.scrollHeight });
      });
    }
  });

  function copyLine(idx: number, entry: LogEntry) {
    logStore.copyLine(entry);
    copiedLineId = idx;
    toast('Copied to clipboard');
    setTimeout(() => {
      if (copiedLineId === idx) copiedLineId = null;
    }, 800);
  }

  function copyAll() {
    logStore.copyAll();
    toast(`Copied ${logStore.entries.length} lines`);
  }

  function downloadLog() {
    logStore.downloadLog(activeBucket());
    toast('Log file downloaded');
  }
</script>

{#if logStore.open}
  <div class="flex flex-col border-t border-border bg-[#0a0a12]" style="height: {panelHeight}px;">
    <!-- Resize handle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="h-2 cursor-row-resize hover:bg-primary/20 transition-colors duration-200 shrink-0 {resizing ? 'bg-primary/30' : ''}"
      onmousedown={startResize}
    ></div>
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-1 border-b border-border/50">
      <span class="text-xs font-medium text-muted-foreground">Log</span>
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          class="h-5 w-5 text-zinc-500 hover:text-amber-400/80 hover:bg-transparent"
          onclick={copyAll}
          title="Copy all"
        >
          <ClipboardCopy class="h-3 w-3" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-5 w-5 text-zinc-500 hover:text-amber-400/80 hover:bg-transparent"
          onclick={downloadLog}
          title="Download log"
        >
          <Download class="h-3 w-3" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-5 w-5 text-zinc-500 hover:text-amber-400/80 hover:bg-transparent"
          onclick={() => logStore.clear()}
          title="Clear logs"
        >
          <Trash2 class="h-3 w-3" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-5 w-5 text-zinc-500 hover:text-amber-400/80 hover:bg-transparent"
          onclick={() => (logStore.open = false)}
          title="Close"
        >
          <X class="h-3 w-3" />
        </Button>
      </div>
    </div>

    <!-- Log entries -->
    <div class="flex-1 overflow-auto font-mono text-[11px] leading-[18px]" bind:this={scrollRef}>
      {#each logStore.entries as entry, idx}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="group relative flex items-start gap-2 px-3 pr-8 cursor-default transition-colors
            {copiedLineId === idx ? 'bg-emerald-500/10' : 'hover:bg-white/[0.04]'}"
          oncontextmenu={(e) => {
            e.preventDefault();
            copyLine(idx, entry);
          }}
        >
          <span class="text-zinc-600 shrink-0 select-none">{entry.timestamp}</span>
          <span
            class="w-11 shrink-0 uppercase {levelColors[entry.level] ??
              'text-zinc-500'} select-none"
          >
            {entry.level}
          </span>
          <span class="text-violet-400/70 shrink-0 select-none">[{entry.target}]</span>
          <span class="text-zinc-300 break-all flex-1">{entry.message}</span>

          <button
            class="absolute right-1 top-0.5 p-0.5 rounded opacity-0 group-hover:opacity-100
              transition-opacity text-zinc-500 hover:text-zinc-200 hover:bg-zinc-700/50"
            onclick={() => copyLine(idx, entry)}
            title="Copy line"
          >
            <Copy class="h-3 w-3" />
          </button>
        </div>
      {/each}
      {#if logStore.entries.length === 0}
        <div class="flex items-center justify-center h-full text-zinc-600 text-xs">
          No log entries yet
        </div>
      {/if}
    </div>
  </div>
{/if}
