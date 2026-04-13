<script lang="ts">
  import { store } from './store.svelte';

  let renaming = $state(false);
  let renameValue = $state('');

  function handleDownload() {
    if (!store.contextMenu.target) return;
    store.downloadToDesktop(store.contextMenu.target);
    store.closeContextMenu();
  }

  function handleUpload() {
    if (!store.contextMenu.target) return;
    store.uploadToPanel(store.contextMenu.target, 'left');
    store.closeContextMenu();
  }

  function handleCopyUrl() {
    if (!store.contextMenu.target) return;
    const panel = store.contextMenu.source === 'right-panel' ? store.rightPanel : store.leftPanel;
    const fakeUrl = `https://${panel.bucket}.s3.amazonaws.com/${store.contextMenu.target.path}${store.contextMenu.target.name}?X-Amz-Expires=3600`;
    navigator.clipboard.writeText(fakeUrl).catch(() => {});
    store.closeContextMenu();
  }

  function handleRenameStart() {
    if (!store.contextMenu.target) return;
    renameValue = store.contextMenu.target.name;
    renaming = true;
  }

  function handleRenameConfirm() {
    if (!store.contextMenu.target || !renameValue.trim()) return;
    store.renameFile(store.contextMenu.target, renameValue.trim());
    renaming = false;
    store.closeContextMenu();
  }

  function handleDelete() {
    if (!store.contextMenu.target) return;
    store.deleteFile(store.contextMenu.target, store.contextMenu.source);
    store.closeContextMenu();
  }

  function handleBackdropClick() {
    renaming = false;
    store.closeContextMenu();
  }
</script>

{#if store.contextMenu.visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40" onclick={handleBackdropClick} oncontextmenu={(e) => { e.preventDefault(); handleBackdropClick(); }}></div>

  <div
    class="dark fixed z-50 min-w-40 rounded-md border border-border bg-popover py-1 shadow-md text-popover-foreground"
    style="left: {store.contextMenu.x}px; top: {store.contextMenu.y}px;"
  >
    {#if renaming}
      <div class="px-2 py-1.5">
        <input
          class="w-full rounded border border-border bg-background px-2 py-1 text-xs text-foreground outline-none focus:border-ring"
          bind:value={renameValue}
          onkeydown={(e) => e.key === 'Enter' && handleRenameConfirm()}
        />
      </div>
    {:else if store.contextMenu.source === 'desktop'}
      <button class="flex w-full items-center px-3 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors" onclick={handleUpload}>
        Upload
      </button>
      <div class="my-1 border-t border-border"></div>
      <button class="flex w-full items-center px-3 py-1.5 text-sm text-destructive hover:bg-accent cursor-pointer transition-colors" onclick={handleDelete}>
        Delete
      </button>
    {:else}
      <button class="flex w-full items-center px-3 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors" onclick={handleDownload}>
        Download
      </button>
      <button class="flex w-full items-center px-3 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors" onclick={handleCopyUrl}>
        Copy Key
      </button>
      <button class="flex w-full items-center px-3 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors" onclick={handleRenameStart}>
        Rename
      </button>
      <div class="my-1 border-t border-border"></div>
      <button class="flex w-full items-center px-3 py-1.5 text-sm text-destructive hover:bg-accent cursor-pointer transition-colors" onclick={handleDelete}>
        Delete
      </button>
    {/if}
  </div>
{/if}
