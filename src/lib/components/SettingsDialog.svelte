<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { FlaskConical } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
  }

  let { open = $bindable() }: Props = $props();
</script>

<Dialog bind:open>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle>{m.settings_title()}</DialogTitle>
    </DialogHeader>

    <div class="space-y-4">
      <!-- General -->
      <label class="flex items-center justify-between gap-3">
        <div class="space-y-0.5">
          <div class="text-sm font-medium">{m.settings_auto_show_transfers()}</div>
          <div class="text-xs text-muted-foreground">{m.settings_auto_show_transfers_desc()}</div>
        </div>
        <button
          role="switch"
          aria-checked={settingsStore.autoShowTransfers}
          class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors
            {settingsStore.autoShowTransfers ? 'bg-primary' : 'bg-muted-foreground/30'}"
          onclick={() => settingsStore.setAutoShowTransfers(!settingsStore.autoShowTransfers)}
        >
          <span
            class="pointer-events-none inline-block h-4 w-4 rounded-full bg-background shadow transition-transform
              {settingsStore.autoShowTransfers ? 'translate-x-[18px]' : 'translate-x-[2px]'}"
          ></span>
        </button>
      </label>

      <!-- Separator + Experimental section -->
      <div class="border-t border-border pt-3">
        <div class="flex items-center gap-1.5 mb-3">
          <FlaskConical class="h-3.5 w-3.5 text-muted-foreground" />
          <span class="text-xs font-medium text-muted-foreground uppercase tracking-wider">{m.settings_experimental()}</span>
        </div>

        <label class="flex items-center justify-between gap-3">
          <div class="space-y-0.5">
            <div class="text-sm font-medium">{m.settings_folder_drag_download()}</div>
            <div class="text-xs text-muted-foreground">{m.settings_folder_drag_download_desc()}</div>
          </div>
          <button
            role="switch"
            aria-checked={settingsStore.folderDragDownload}
            class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors
              {settingsStore.folderDragDownload ? 'bg-primary' : 'bg-muted-foreground/30'}"
            onclick={() => settingsStore.setFolderDragDownload(!settingsStore.folderDragDownload)}
          >
            <span
              class="pointer-events-none inline-block h-4 w-4 rounded-full bg-background shadow transition-transform
                {settingsStore.folderDragDownload ? 'translate-x-[18px]' : 'translate-x-[2px]'}"
            ></span>
          </button>
        </label>
      </div>
    </div>
  </DialogContent>
</Dialog>
