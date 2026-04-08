<script lang="ts">
  import { uiStore, type ContextMenuItem } from '$lib/stores/ui.svelte';
  import { logStore } from '$lib/stores/logs.svelte';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { Button } from '$lib/components/ui/button';
  import AboutDialog from './AboutDialog.svelte';
  import { Settings, Terminal } from '@lucide/svelte';

  let aboutOpen = $state(false);

  interface Props {
    onupload?: () => void;
    ondownload?: () => void;
    onnewfolder?: () => void;
    onmultipartcleanup?: () => void;
  }

  const { onupload, ondownload, onnewfolder, onmultipartcleanup }: Props = $props();

  const profileId = $derived(profileStore.activeProfileId);

  function openMenu(e: MouseEvent, items: ContextMenuItem[]) {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    uiStore.openContextMenu(rect.left, rect.bottom + 2, items);
  }

  function fileMenu(e: MouseEvent) {
    openMenu(e, [
      { label: 'Upload File...', action: () => onupload?.(), disabled: !profileId },
      { label: 'Download Selected', action: () => ondownload?.(), disabled: !profileId },
      { label: 'New Folder', action: () => onnewfolder?.(), disabled: !profileId },
      { separator: true, label: '', action: () => {} },
      { label: 'Profile Manager', action: () => (uiStore.profileManagerOpen = true) },
    ]);
  }

  function editMenu(e: MouseEvent) {
    openMenu(e, [
      {
        label: 'Select All',
        action: () => profileId && fileStore.selectAll(profileId),
        disabled: !profileId,
      },
      {
        label: 'Clear Selection',
        action: () => profileId && fileStore.clearSelection(profileId),
        disabled: !profileId,
      },
    ]);
  }

  function viewMenu(e: MouseEvent) {
    openMenu(e, [
      {
        label: uiStore.dualPanel ? '✓ Dual Panel' : '  Dual Panel',
        action: () => uiStore.toggleDualPanel(),
      },
      { label: logStore.open ? '✓ Log Panel' : '  Log Panel', action: () => logStore.toggle() },
      {
        label: uiStore.transferPanelExpanded ? '✓ Transfer Panel' : '  Transfer Panel',
        action: () => uiStore.toggleTransferPanel(),
      },
      { separator: true, label: '', action: () => {} },
      {
        label: 'Refresh',
        action: () => profileId && fileStore.refresh(profileId),
        disabled: !profileId,
      },
    ]);
  }

  function helpMenu(e: MouseEvent) {
    openMenu(e, [
      {
        label: 'Cleanup Multipart Uploads',
        action: () => onmultipartcleanup?.(),
        disabled: !profileId,
      },
      { separator: true, label: '', action: () => {} },
      { label: 'About S3V', action: () => (aboutOpen = true) },
    ]);
  }

  const menus: Record<string, (e: MouseEvent) => void> = {
    File: fileMenu,
    Edit: editMenu,
    View: viewMenu,
    Help: helpMenu,
  };
</script>

<div
  class="flex h-9 items-center gap-1 border-b border-border bg-background px-3 select-none"
  data-tauri-drag-region
>
  <!-- Branding -->
  <span class="mr-3 text-sm font-bold tracking-wider text-primary">S3V</span>

  <!-- Menu items -->
  {#each Object.keys(menus) as item}
    <button
      class="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
      onclick={(e) => menus[item](e)}
    >
      {item}
    </button>
  {/each}

  <div class="flex-1" data-tauri-drag-region></div>

  <!-- Log toggle -->
  <Button
    variant={logStore.open ? 'secondary' : 'ghost'}
    size="icon"
    class="h-6 w-6"
    onclick={() => logStore.toggle()}
    title="Toggle Log Panel"
  >
    <Terminal class="h-3.5 w-3.5" />
  </Button>

  <!-- Settings -->
  <Button
    variant="ghost"
    size="icon"
    class="h-6 w-6"
    onclick={() => (uiStore.profileManagerOpen = true)}
    title="Profile Manager"
  >
    <Settings class="h-3.5 w-3.5" />
  </Button>
</div>

<AboutDialog bind:open={aboutOpen} />
