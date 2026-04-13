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
    Copy,
    Link,
    Columns2,
    Search,
    Wrench,
  } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    disabled?: boolean;
    onupload?: () => void;
    ondownload?: () => void;
    onnewfolder?: () => void;
    ondelete?: () => void;
    onmove?: () => void;
    oncopy?: () => void;
    onshareurl?: () => void;
    shareDisabled?: boolean;
    onmultipartcleanup?: () => void;
  }

  const {
    disabled = false,
    onupload,
    ondownload,
    onnewfolder,
    ondelete,
    onmove,
    oncopy,
    onshareurl,
    shareDisabled = false,
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
    title={m.toolbar_upload()}
  >
    <Upload class="h-3.5 w-3.5" />
    {m.toolbar_upload()}
  </Button>

  <!-- Download -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={ondownload}
    title={m.toolbar_download()}
  >
    <Download class="h-3.5 w-3.5" />
    {m.toolbar_download()}
  </Button>

  <!-- New Folder -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={onnewfolder}
    title={m.toolbar_new_folder()}
  >
    <FolderPlus class="h-3.5 w-3.5" />
    {m.toolbar_folder()}
  </Button>

  <!-- Delete -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs text-destructive hover:text-destructive"
    {disabled}
    onclick={ondelete}
    title={m.toolbar_delete()}
  >
    <Trash2 class="h-3.5 w-3.5" />
    {m.toolbar_delete()}
  </Button>

  <!-- Move -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={onmove}
    title={m.toolbar_move()}
  >
    <Move class="h-3.5 w-3.5" />
    {m.toolbar_move()}
  </Button>

  <!-- Copy -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    {disabled}
    onclick={oncopy}
    title={m.toolbar_copy()}
  >
    <Copy class="h-3.5 w-3.5" />
    {m.toolbar_copy()}
  </Button>

  <!-- Share URL -->
  <Button
    variant="ghost"
    size="sm"
    class="h-7 gap-1.5 px-2 text-xs"
    disabled={disabled || shareDisabled}
    onclick={onshareurl}
    title={m.toolbar_share_url()}
  >
    <Link class="h-3.5 w-3.5" />
    {m.toolbar_share()}
  </Button>

  <div class="flex-1"></div>

  <!-- Multipart Cleanup -->
  <Button
    variant="ghost"
    size="icon"
    class="h-7 w-7"
    {disabled}
    onclick={onmultipartcleanup}
    title={m.toolbar_cleanup_uploads()}
  >
    <Wrench class="h-3.5 w-3.5" />
  </Button>

  <!-- Dual Panel toggle -->
  <Button
    variant={uiStore.dualPanel ? 'secondary' : 'ghost'}
    size="icon"
    class="h-7 w-7"
    onclick={() => uiStore.toggleDualPanel()}
    title={m.toolbar_toggle_dual()}
  >
    <Columns2 class="h-3.5 w-3.5" />
  </Button>

  <!-- Search -->
  <div class="relative">
    <Search class="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
    <Input class="h-7 w-48 pl-7 text-xs" placeholder={m.toolbar_search()} bind:value={uiStore.searchQuery} />
  </div>
</div>
