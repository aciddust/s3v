<script lang="ts">
  import {
    listMultipartUploads,
    abortMultipartUpload,
    type MultipartUploadInfo,
  } from '$lib/api/s3';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Badge } from '$lib/components/ui/badge';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { LoaderCircle, Trash2, RefreshCw } from '@lucide/svelte';
  import { formatDate } from '$lib/utils/format';

  interface Props {
    open: boolean;
  }

  let { open = $bindable() }: Props = $props();

  let uploads = $state<MultipartUploadInfo[]>([]);
  let selected = $state<Set<string>>(new Set());
  let loading = $state(false);
  let aborting = $state(false);

  const profileId = $derived(profileStore.activeProfileId);
  const bucket = $derived(profileStore.activeTab?.buckets[0]?.name ?? '');

  async function loadUploads() {
    if (!profileId || !bucket) return;
    loading = true;
    try {
      uploads = await listMultipartUploads(profileId, bucket);
      selected = new Set();
    } catch (e) {
      console.error('Failed to list multipart uploads:', e);
      uploads = [];
    } finally {
      loading = false;
    }
  }

  function toggleSelect(uploadId: string) {
    const next = new Set(selected);
    if (next.has(uploadId)) {
      next.delete(uploadId);
    } else {
      next.add(uploadId);
    }
    selected = next;
  }

  function toggleAll() {
    if (selected.size === uploads.length) {
      selected = new Set();
    } else {
      selected = new Set(uploads.map((u) => u.upload_id));
    }
  }

  async function handleAbort() {
    if (!profileId || !bucket || selected.size === 0) return;
    aborting = true;
    try {
      for (const upload of uploads) {
        if (selected.has(upload.upload_id)) {
          await abortMultipartUpload(profileId, bucket, upload.key, upload.upload_id);
        }
      }
      await loadUploads();
    } catch (e) {
      console.error('Failed to abort multipart uploads:', e);
    } finally {
      aborting = false;
    }
  }

  async function handleAbortAll() {
    if (!profileId || !bucket || uploads.length === 0) return;
    aborting = true;
    try {
      for (const upload of uploads) {
        await abortMultipartUpload(profileId, bucket, upload.key, upload.upload_id);
      }
      await loadUploads();
    } catch (e) {
      console.error('Failed to abort all:', e);
    } finally {
      aborting = false;
    }
  }

  $effect(() => {
    if (open && profileId && bucket) {
      loadUploads();
    }
  });
</script>

<Dialog bind:open>
  <DialogContent class="max-w-lg">
    <DialogHeader>
      <DialogTitle>Incomplete Multipart Uploads</DialogTitle>
    </DialogHeader>

    <p class="text-xs text-muted-foreground">
      Incomplete uploads consume storage. Select and abort uploads you no longer need.
    </p>

    <!-- Actions bar -->
    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm" class="gap-1.5" onclick={loadUploads} disabled={loading}>
        <RefreshCw class="h-3.5 w-3.5 {loading ? 'animate-spin' : ''}" />
        Refresh
      </Button>
      {#if uploads.length > 0}
        <Button
          variant="destructive"
          size="sm"
          class="gap-1.5"
          onclick={handleAbort}
          disabled={aborting || selected.size === 0}
        >
          <Trash2 class="h-3.5 w-3.5" />
          Abort Selected ({selected.size})
        </Button>
        <Button
          variant="outline"
          size="sm"
          class="gap-1.5 text-destructive hover:text-destructive"
          onclick={handleAbortAll}
          disabled={aborting}
        >
          Abort All
        </Button>
      {/if}
    </div>

    <!-- Upload list -->
    <ScrollArea class="max-h-64">
      {#if loading}
        <div class="flex items-center justify-center py-6 text-sm text-muted-foreground">
          <LoaderCircle class="h-4 w-4 animate-spin mr-2" />
          Loading...
        </div>
      {:else if uploads.length === 0}
        <div class="py-6 text-center text-sm text-muted-foreground">
          No incomplete multipart uploads found.
        </div>
      {:else}
        <!-- Header -->
        <div
          class="flex items-center gap-2 px-2 py-1 text-xs text-muted-foreground border-b border-border"
        >
          <button class="shrink-0" onclick={toggleAll}>
            <Checkbox checked={selected.size === uploads.length && uploads.length > 0} />
          </button>
          <span class="flex-1">Key</span>
          <span class="w-36 text-right">Initiated</span>
        </div>

        {#each uploads as upload}
          <div
            class="flex items-center gap-2 px-2 py-1.5 border-b border-border/50 hover:bg-muted/30
              {selected.has(upload.upload_id) ? 'bg-primary/5' : ''}"
          >
            <button class="shrink-0" onclick={() => toggleSelect(upload.upload_id)}>
              <Checkbox checked={selected.has(upload.upload_id)} />
            </button>
            <span class="flex-1 text-xs truncate font-mono" title={upload.key}>
              {upload.key}
            </span>
            <span class="w-36 text-right text-xs text-muted-foreground">
              {formatDate(upload.initiated ?? '')}
            </span>
          </div>
        {/each}

        <div class="px-2 py-1.5 text-xs text-muted-foreground">
          {uploads.length} incomplete upload{uploads.length !== 1 ? 's' : ''}
        </div>
      {/if}
    </ScrollArea>
  </DialogContent>
</Dialog>
