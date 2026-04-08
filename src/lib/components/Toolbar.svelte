<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import {
    Upload,
    Download,
    FolderPlus,
    Trash2,
    Move,
    Link,
    Columns2,
    Search,
    Wrench,
  } from '@lucide/svelte';

  interface Props {
    disabled?: boolean;
    onupload?: () => void;
    ondownload?: () => void;
    onnewfolder?: () => void;
    ondelete?: () => void;
    onmove?: () => void;
    onshareurl?: () => void;
    onmultipartcleanup?: () => void;
  }

  const {
    disabled = false,
    onupload,
    ondownload,
    onnewfolder,
    ondelete,
    onmove,
    onshareurl,
    onmultipartcleanup,
  }: Props = $props();
</script>

<div class="flex h-10 items-center gap-1 border-b border-border bg-background px-2">
  <!-- Upload -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={onupload}
    title="Upload"
  >
    <Upload class="h-3.5 w-3.5" />
    Upload
  </Button>

  <!-- Download -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={ondownload}
    title="Download"
  >
    <Download class="h-3.5 w-3.5" />
    Download
  </Button>

  <!-- New Folder -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={onnewfolder}
    title="New Folder"
  >
    <FolderPlus class="h-3.5 w-3.5" />
    Folder
  </Button>

  <!-- Delete -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs text-destructive hover:text-destructive"
    {disabled}
    onclick={ondelete}
    title="Delete"
  >
    <Trash2 class="h-3.5 w-3.5" />
    Delete
  </Button>

  <!-- Move -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={onmove}
    title="Move"
  >
    <Move class="h-3.5 w-3.5" />
    Move
  </Button>

  <!-- Share URL -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={onshareurl}
    title="Share URL"
  >
    <Link class="h-3.5 w-3.5" />
    Share
  </Button>

  <div class="flex-1"></div>

  <!-- Multipart Cleanup -->
  <Button
    variant="ghost"
    size="icon"
    class="h-7 w-7"
    {disabled}
    onclick={onmultipartcleanup}
    title="Cleanup incomplete uploads"
  >
    <Wrench class="h-3.5 w-3.5" />
  </Button>

  <!-- Dual Panel toggle -->
  <Button
    variant={uiStore.dualPanel ? 'secondary' : 'ghost'}
    size="icon"
    class="h-7 w-7"
    onclick={() => uiStore.toggleDualPanel()}
    title="Toggle Dual Panel"
  >
    <Columns2 class="h-3.5 w-3.5" />
  </Button>

  <!-- Search -->
  <div class="relative">
    <Search class="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
    <Input class="h-7 w-48 pl-7 text-xs" placeholder="Search..." bind:value={uiStore.searchQuery} />
  </div>
</div>
