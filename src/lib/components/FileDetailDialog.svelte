<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { headObject, type ObjectMetadata } from '$lib/api/s3';
  import { Link, Download, X } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
    profileId: string;
    bucket: string;
    fileKey: string;
    ondownload: (key: string) => void;
    oncopyurl: (key: string) => void;
  }

  let { open = $bindable(), profileId, bucket, fileKey, ondownload, oncopyurl }: Props = $props();

  let metadata = $state<ObjectMetadata | null>(null);
  let loading = $state(false);

  $effect(() => {
    if (open && fileKey) {
      loading = true;
      metadata = null;
      headObject(profileId, bucket, fileKey)
        .then((m) => { metadata = m; })
        .catch(() => { metadata = null; })
        .finally(() => { loading = false; });
    }
  });

  function getFileName(key: string): string {
    return key.replace(/\/$/, '').split('/').at(-1) ?? key;
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '—';
    try {
      return new Date(dateStr).toLocaleString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return '—';
    }
  }

  const rows = $derived(
    metadata
      ? [
          { label: m.file_detail_name(), value: getFileName(metadata.key) },
          { label: m.file_detail_bucket(), value: bucket },
          { label: m.file_detail_key(), value: metadata.key, mono: true },
          { label: m.file_detail_size(), value: formatBytes(metadata.size) },
          { label: m.file_detail_content_type(), value: metadata.content_type ?? '—' },
          { label: m.file_detail_last_modified(), value: formatDate(metadata.last_modified) },
          { label: m.file_detail_etag(), value: metadata.etag ?? '—', mono: true },
        ]
      : [],
  );
</script>

<Dialog bind:open>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle>{m.file_detail_title()}</DialogTitle>
    </DialogHeader>

    {#if loading}
      <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
        {m.file_detail_loading()}
      </div>
    {:else if metadata}
      <div class="space-y-2">
        {#each rows as row}
          <div class="flex gap-3 text-xs">
            <span class="w-24 shrink-0 text-muted-foreground">{row.label}</span>
            <span
              class="flex-1 break-all text-foreground {row.mono ? 'font-mono text-[11px]' : ''}"
            >
              {row.value}
            </span>
          </div>
        {/each}
      </div>
    {/if}

    <DialogFooter class="flex justify-end gap-2 pt-2">
      <Button
        variant="outline"
        size="sm"
        onclick={() => { oncopyurl(fileKey); }}
        disabled={loading}
      >
        <Link class="h-3.5 w-3.5 mr-1.5" />
        {m.file_detail_copy_url()}
      </Button>
      <Button
        variant="outline"
        size="sm"
        onclick={() => { ondownload(fileKey); }}
        disabled={loading}
      >
        <Download class="h-3.5 w-3.5 mr-1.5" />
        {m.file_detail_download()}
      </Button>
      <Button variant="ghost" size="sm" onclick={() => { open = false; }}>
        <X class="h-3.5 w-3.5 mr-1.5" />
        {m.file_detail_close()}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
