<script lang="ts">
  import { uiStore, type ContextMenuItem } from '$lib/stores/ui.svelte';
  import { logStore } from '$lib/stores/logs.svelte';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { Button } from '$lib/components/ui/button';
  import AboutDialog from './AboutDialog.svelte';
  import LanguageDialog from './LanguageDialog.svelte';
  import SettingsDialog from './SettingsDialog.svelte';
  import { ArrowUpDown, CloudCog, Settings, Terminal, Languages } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  let aboutOpen = $state(false);
  let languageOpen = $state(false);
  let settingsOpen = $state(false);

  interface Props {
    onupload?: () => void;
    ondownload?: () => void;
    onnewfolder?: () => void;
    onmultipartcleanup?: () => void;
  }

  const { onupload, ondownload, onnewfolder, onmultipartcleanup }: Props = $props();

  const profileId = $derived(profileStore.activeProfileId);

  let lastMenuCloseTime = 0;
  let lastMenuX = -1;

  function openMenu(e: MouseEvent, items: ContextMenuItem[]) {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    const x = rect.left;
    // If the menu was just closed from this same button (within 100ms), don't reopen
    if (Date.now() - lastMenuCloseTime < 100 && Math.abs(lastMenuX - x) < 2) {
      return;
    }
    uiStore.openContextMenu(x, rect.bottom + 2, items);
  }

  // Track when context menu closes so we can detect toggle clicks
  $effect(() => {
    if (!uiStore.contextMenu.open && lastMenuX >= 0) {
      lastMenuCloseTime = Date.now();
    }
    if (uiStore.contextMenu.open) {
      lastMenuX = uiStore.contextMenu.x;
    }
  });

  function fileMenu(e: MouseEvent) {
    openMenu(e, [
      { label: m.titlebar_upload_file(), action: () => onupload?.(), disabled: !profileId },
      { label: m.titlebar_download_selected(), action: () => ondownload?.(), disabled: !profileId },
      { label: m.titlebar_new_folder(), action: () => onnewfolder?.(), disabled: !profileId },
      { separator: true, label: '', action: () => {} },
      { label: m.titlebar_profile_manager(), action: () => (uiStore.profileManagerOpen = true) },
    ]);
  }

  function editMenu(e: MouseEvent) {
    openMenu(e, [
      {
        label: m.titlebar_select_all(),
        action: () => profileId && fileStore.selectAll(profileId),
        disabled: !profileId,
      },
      {
        label: m.titlebar_clear_selection(),
        action: () => profileId && fileStore.clearSelection(profileId),
        disabled: !profileId,
      },
    ]);
  }

  function viewMenu(e: MouseEvent) {
    openMenu(e, [
      {
        label: uiStore.dualPanel ? `✓ ${m.titlebar_dual_panel()}` : `  ${m.titlebar_dual_panel()}`,
        action: () => uiStore.toggleDualPanel(),
      },
      {
        label: logStore.open ? `✓ ${m.titlebar_log_panel()}` : `  ${m.titlebar_log_panel()}`,
        action: () => logStore.toggle(),
      },
      {
        label: uiStore.transferPanelVisible
          ? `✓ ${m.titlebar_transfer_panel()}`
          : `  ${m.titlebar_transfer_panel()}`,
        action: () => uiStore.toggleTransferPanel(),
      },
      { separator: true, label: '', action: () => {} },
      {
        label: m.titlebar_refresh(),
        action: () => profileId && fileStore.refresh(profileId),
        disabled: !profileId,
      },
    ]);
  }

  function helpMenu(e: MouseEvent) {
    openMenu(e, [
      {
        label: m.titlebar_cleanup_multipart(),
        action: () => onmultipartcleanup?.(),
        disabled: !profileId,
      },
      { label: m.titlebar_language(), action: () => (languageOpen = true) },
      { separator: true, label: '', action: () => {} },
      { label: m.titlebar_about(), action: () => (aboutOpen = true) },
    ]);
  }

  const menuEntries: Array<{ label: () => string; handler: (e: MouseEvent) => void }> = [
    { label: () => m.titlebar_menu_file(), handler: fileMenu },
    { label: () => m.titlebar_menu_edit(), handler: editMenu },
    { label: () => m.titlebar_menu_view(), handler: viewMenu },
    { label: () => m.titlebar_menu_help(), handler: helpMenu },
  ];
</script>

<div
  class="flex h-9 items-center gap-1 border-b border-border bg-background px-3 select-none"
  data-tauri-drag-region
>
  <!-- Branding -->
  <span class="mr-3 text-sm font-bold tracking-wider text-primary">S3V</span>

  <!-- Menu items -->
  {#each menuEntries as entry}
    <button
      class="rounded px-2 py-0.5 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
      onclick={(e) => entry.handler(e)}
    >
      {entry.label()}
    </button>
  {/each}

  <div class="flex-1" data-tauri-drag-region></div>

  <!-- Language -->
  <Button
    variant="ghost"
    size="icon"
    class="h-6 w-6"
    onclick={() => (languageOpen = true)}
    title={m.titlebar_language()}
  >
    <Languages class="h-3.5 w-3.5" />
  </Button>

  <!-- Transfer panel toggle -->
  <Button
    variant={uiStore.transferPanelVisible ? 'secondary' : 'ghost'}
    size="icon"
    class="h-6 w-6"
    onclick={() => uiStore.toggleTransferPanel()}
    title={m.transfer_panel_title()}
  >
    <ArrowUpDown class="h-3.5 w-3.5" />
  </Button>

  <!-- Log toggle -->
  <Button
    variant={logStore.open ? 'secondary' : 'ghost'}
    size="icon"
    class="h-6 w-6"
    onclick={() => logStore.toggle()}
    title={m.titlebar_toggle_log()}
  >
    <Terminal class="h-3.5 w-3.5" />
  </Button>

  <!-- Profile Manager -->
  <Button
    variant="ghost"
    size="icon"
    class="h-6 w-6"
    onclick={() => (uiStore.profileManagerOpen = true)}
    title={m.titlebar_profile_manager()}
  >
    <CloudCog class="h-3.5 w-3.5" />
  </Button>

  <!-- Settings -->
  <Button
    variant="ghost"
    size="icon"
    class="h-6 w-6"
    onclick={() => (settingsOpen = true)}
    title={m.titlebar_settings()}
  >
    <Settings class="h-3.5 w-3.5" />
  </Button>
</div>

<AboutDialog bind:open={aboutOpen} />
<LanguageDialog bind:open={languageOpen} />
<SettingsDialog bind:open={settingsOpen} />
