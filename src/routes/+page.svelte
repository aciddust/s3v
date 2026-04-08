<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Toaster, toast } from 'svelte-sonner';

  // Components
  import TitleBar from '$lib/components/TitleBar.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import TabBar from '$lib/components/TabBar.svelte';
  import MainArea from '$lib/components/MainArea.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import TransferPanel from '$lib/components/TransferPanel.svelte';
  import ContextMenu from '$lib/components/ContextMenu.svelte';
  import ProfileManager from '$lib/components/ProfileManager.svelte';
  import LogPanel from '$lib/components/LogPanel.svelte';
  import MultipartCleanup from '$lib/components/MultipartCleanup.svelte';
  import InputDialog from '$lib/components/InputDialog.svelte';
  import FolderPicker from '$lib/components/FolderPicker.svelte';
  import DragOverlay from '$lib/components/DragOverlay.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import type { ConfirmAction } from '$lib/components/ConfirmDialog.svelte';
  import { Button } from '$lib/components/ui/button';

  // Stores
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { transferStore } from '$lib/stores/transfers.svelte';
  import { bookmarkStore } from '$lib/stores/bookmarks.svelte';
  import { logStore } from '$lib/stores/logs.svelte';
  import { moveStore } from '$lib/stores/moves.svelte';
  import { uiStore, type ContextMenuItem } from '$lib/stores/ui.svelte';

  // API
  import {
    deleteObjects,
    createFolder,
    renameObject,
    renameFolder,
    getPresignedUrl,
    moveObjects,
    copyObject,
  } from '$lib/api/s3';
  import { enqueueUpload, enqueueDownload, onTransferCompleted } from '$lib/api/transfers';
  import { listProfiles, type ProfileSummary } from '$lib/api/profiles';

  // Utils
  import { matchBinding } from '$lib/utils/keys';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';

  // Derived state
  const activeTab = $derived(profileStore.activeTab);
  const activeProfileId = $derived(profileStore.activeProfileId);
  const activePanel = $derived(uiStore.activePanel);
  const activeStoreId = $derived(
    activeProfileId
      ? uiStore.dualPanel && activePanel === 'right'
        ? `${activeProfileId}::right`
        : activeProfileId
      : null,
  );
  const leftFileState = $derived(activeProfileId ? fileStore.getState(activeProfileId) : null);
  const fileState = $derived(activeStoreId ? fileStore.getState(activeStoreId) : null);
  const isConnected = $derived(activeTab?.connected ?? false);

  // Available profiles for empty state
  let availableProfiles = $state<ProfileSummary[]>([]);
  const lastProfileId = $derived(
    typeof localStorage !== 'undefined' ? localStorage.getItem('s3v:lastProfile') : null,
  );
  const lastProfile = $derived(
    lastProfileId ? availableProfiles.find((p) => p.id === lastProfileId) : null,
  );

  // Track last connected profile
  $effect(() => {
    if (activeProfileId && isConnected) {
      localStorage.setItem('s3v:lastProfile', activeProfileId);
    }
  });

  // Drag state
  let isDragOver = $state(false);
  let multipartCleanupOpen = $state(false);
  let folderPickerOpen = $state(false);
  let folderPickerCallback = $state<(prefix: string) => void>(() => {});
  let inputDialogOpen = $state(false);
  let inputDialogConfig = $state<{
    title: string;
    label?: string;
    placeholder?: string;
    defaultValue?: string;
    confirmText?: string;
    onconfirm: (value: string) => void;
  }>({
    title: '',
    onconfirm: () => {},
  });

  function showInputDialog(config: typeof inputDialogConfig) {
    inputDialogConfig = config;
    inputDialogOpen = true;
  }

  // --- Confirm dialog state ---
  let confirmDialogOpen = $state(false);
  let confirmDialogAction = $state<ConfirmAction>('delete');
  let confirmDialogFiles = $state<string[]>([]);
  let confirmDialogCallback = $state<() => void>(() => {});

  function showConfirm(action: ConfirmAction, files: string[], onconfirm: () => void) {
    confirmDialogAction = action;
    confirmDialogFiles = files;
    confirmDialogCallback = onconfirm;
    confirmDialogOpen = true;
  }

  onMount(() => {
    bookmarkStore.load();
    transferStore.init();
    moveStore.init();
    logStore.init();
    profileStore.loadProfiles();
    listProfiles()
      .then((p) => {
        availableProfiles = p;
      })
      .catch(() => {});

    const unlistenUploadDone = onTransferCompleted((e) => {
      if (e.transfer_type === 'upload' && activeProfileId) {
        // Refresh both panels on upload complete
        fileStore.refresh(activeProfileId);
        if (uiStore.dualPanel) {
          fileStore.refresh(`${activeProfileId}::right`);
        }
      }
    });

    const unlistenDrop = getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === 'enter') {
        isDragOver = true;
      } else if (event.payload.type === 'leave') {
        isDragOver = false;
      } else if (event.payload.type === 'drop') {
        isDragOver = false;
        handleFileDrop(event.payload.paths);
      }
    });

    return () => {
      unlistenUploadDone.then((fn) => fn());
      unlistenDrop.then((fn) => fn());
      transferStore.destroy();
      moveStore.destroy();
      logStore.destroy();
    };
  });

  // --- Navigation ---
  function handleNavigate(bucket: string, prefix: string) {
    if (!activeStoreId) return;
    fileStore.navigate(activeStoreId, bucket, prefix);
  }

  // --- Handlers ---
  function handleDelete() {
    if (!activeProfileId || !activeStoreId || !fileState) return;
    if (fileState.selected.size === 0) {
      toast.warning('Select files to delete');
      return;
    }
    const keys = [...fileState.selected];
    const pid = activeProfileId;
    const bkt = fileState.bucket;
    const storeId = activeStoreId;
    showConfirm('delete', keys, async () => {
      try {
        await deleteObjects(pid, bkt, keys);
        await fileStore.refresh(storeId);
      } catch (e) {
        console.error('Delete failed:', e);
      }
    });
  }

  function handleNewFolder() {
    if (!activeProfileId || !activeStoreId || !fileState || !fileState.bucket) return;
    const storeId = activeStoreId;
    showInputDialog({
      title: 'New Folder',
      placeholder: 'folder-name',
      confirmText: 'Create',
      onconfirm: async (name) => {
        if (!activeProfileId || !fileState) return;
        const folderPrefix = fileState.prefix + name.replace(/\/$/, '') + '/';
        try {
          await createFolder(activeProfileId, fileState.bucket, folderPrefix);
          await fileStore.refresh(storeId);
        } catch (e) {
          console.error('Create folder failed:', e);
        }
      },
    });
  }

  function handleRename() {
    if (!activeProfileId || !activeStoreId || !fileState || fileState.selected.size !== 1) return;
    const storeId = activeStoreId;
    const oldKey = [...fileState.selected][0];
    const isDir = oldKey.endsWith('/');
    // "a/b/test/" → "test", "a/b/file.txt" → "file.txt"
    const parts = oldKey.replace(/\/$/, '').split('/');
    const oldName = parts.at(-1) ?? oldKey;
    showInputDialog({
      title: isDir ? 'Rename Folder' : 'Rename',
      defaultValue: oldName,
      confirmText: 'Rename',
      onconfirm: async (newName) => {
        if (!activeProfileId || !fileState || newName === oldName) return;
        try {
          if (isDir) {
            const parentPrefix = parts.slice(0, -1).join('/');
            const oldPrefix = oldKey;
            const newPrefix = (parentPrefix ? parentPrefix + '/' : '') + newName + '/';
            await renameFolder(activeProfileId, fileState.bucket, oldPrefix, newPrefix);
          } else {
            const prefix = oldKey.substring(0, oldKey.lastIndexOf('/') + 1);
            const newKey = prefix + newName;
            await renameObject(activeProfileId, fileState.bucket, oldKey, newKey);
          }
          await fileStore.refresh(storeId);
        } catch (e) {
          console.error('Rename failed:', e);
        }
      },
    });
  }

  async function handleUpload() {
    if (!activeProfileId || !fileState || !fileState.bucket) {
      console.warn('Upload: no active profile or bucket');
      return;
    }
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const fileNames = paths.map((p) => String(p).split(/[\\/]/).at(-1) ?? String(p));
      const pid = activeProfileId;
      const bucket = fileState.bucket;
      const prefix = fileState.prefix;
      showConfirm('upload', fileNames, async () => {
        try {
          for (let i = 0; i < paths.length; i++) {
            const key = prefix + fileNames[i];
            await enqueueUpload(pid, String(paths[i]), bucket, key);
          }
          uiStore.transferPanelExpanded = true;
          await transferStore.reload();
        } catch (e) {
          console.error('Upload failed:', e);
        }
      });
    } catch (e) {
      console.error('Upload failed:', e);
    }
  }

  async function handleDownload() {
    if (!activeProfileId || !fileState) return;
    if (fileState.selected.size === 0) {
      toast.warning('Select files to download');
      return;
    }
    try {
      const destDir = await open({
        multiple: false,
        directory: true,
      });
      if (!destDir) return;
      const dir = Array.isArray(destDir) ? destDir[0] : destDir;
      for (const key of fileState.selected) {
        const fileName = key.split('/').filter(Boolean).at(-1) ?? key;
        const localPath = dir + '/' + fileName;
        await enqueueDownload(activeProfileId, fileState.bucket, key, localPath);
      }
    } catch (e) {
      console.error('Download failed:', e);
    }
  }

  async function handleShare() {
    if (!activeProfileId || !fileState) return;
    if (fileState.selected.size !== 1) {
      toast.warning('Select exactly one file to share');
      return;
    }
    const shareKey = [...fileState.selected][0];
    try {
      const url = await getPresignedUrl(activeProfileId, fileState.bucket, shareKey, 3600);
      await writeText(url);
      toast.success('URL copied to clipboard (1h expiry)');
    } catch (e) {
      console.error('Share URL failed:', e);
      toast.error('Failed to generate share URL');
    }
  }

  function handleMove() {
    if (!activeProfileId || !activeStoreId || !fileState) return;
    if (fileState.selected.size === 0) {
      toast.warning('Select files to move');
      return;
    }
    const keys = [...fileState.selected];
    const pid = activeProfileId;
    const storeId = activeStoreId;
    const bkt = fileState.bucket;
    folderPickerCallback = (destPrefix: string) => {
      uiStore.transferPanelExpanded = true;
      moveObjects(pid, bkt, keys, destPrefix)
        .then(() => fileStore.refresh(storeId))
        .catch((e) => console.error('Move failed:', e));
    };
    folderPickerOpen = true;
  }

  function handleContextMenu(e: MouseEvent, keys: string[]) {
    e.preventDefault();
    // Ensure right-clicked item(s) are selected so actions work
    if (activeStoreId && keys.length > 0) {
      const selected = fileState?.selected;
      if (!selected || !keys.every((k) => selected.has(k))) {
        fileStore.clearSelection(activeStoreId);
        for (const key of keys) {
          fileStore.toggleSelect(activeStoreId, key);
        }
      }
    }
    const items: ContextMenuItem[] = [
      { label: 'Download', action: handleDownload, disabled: keys.length === 0, separator: false },
      {
        label: 'Copy Key',
        action: () => handleCopyKey(keys),
        disabled: keys.length === 0,
        separator: false,
      },
      { label: 'Share URL', action: handleShare, disabled: keys.length !== 1, separator: false },
      { label: '', action: () => {}, separator: true },
      { label: 'Rename', action: handleRename, disabled: keys.length !== 1, separator: false },
      { label: 'Delete', action: handleDelete, disabled: keys.length === 0, separator: false },
    ];
    uiStore.openContextMenu(e.clientX, e.clientY, items);
  }

  function handleBgContextMenu(e: MouseEvent) {
    e.preventDefault();
    const items: ContextMenuItem[] = [
      { label: 'Upload', action: handleUpload, disabled: !isConnected, separator: false },
      { label: 'New Folder', action: handleNewFolder, disabled: !isConnected, separator: false },
      { label: '', action: () => {}, separator: true },
      {
        label: 'Refresh',
        action: () => {
          if (activeStoreId) fileStore.refresh(activeStoreId);
        },
        disabled: !isConnected,
        separator: false,
      },
    ];
    uiStore.openContextMenu(e.clientX, e.clientY, items);
  }

  async function handleCopyKey(keys: string[]) {
    if (keys.length === 0) return;
    try {
      await writeText(keys.join('\n'));
      toast.success(`Copied ${keys.length} key(s)`);
    } catch (e) {
      console.error('Copy key failed:', e);
      toast.error('Failed to copy key');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Skip when input/textarea is focused
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

    const action = matchBinding(e);
    if (!action) return;

    e.preventDefault();

    switch (action) {
      case 'delete':
        handleDelete();
        break;
      case 'selectAll':
        if (activeStoreId) fileStore.selectAll(activeStoreId);
        break;
      case 'copyKey':
        if (fileState) handleCopyKey([...fileState.selected]);
        break;
      case 'rename':
        handleRename();
        break;
      case 'refresh':
        if (activeStoreId) fileStore.refresh(activeStoreId);
        break;
    }
  }

  function handleCopyToPrefix(
    sourceBucket: string,
    destBucket: string,
    destPrefix: string,
    keys: string[],
  ) {
    if (!activeProfileId) return;
    const pid = activeProfileId;
    showConfirm('copy', keys, async () => {
      try {
        for (const key of keys) {
          const fileName = key.replace(/\/$/, '').split('/').at(-1) ?? key;
          const destKey = destPrefix + fileName;
          await copyObject(pid, sourceBucket, key, destBucket, destKey);
        }
        await fileStore.refresh(pid);
        if (uiStore.dualPanel) {
          await fileStore.refresh(`${pid}::right`);
        }
      } catch (e) {
        console.error('Copy failed:', e);
      }
    });
  }

  function handleMoveToPrefix(bucket: string, destPrefix: string, keys: string[]) {
    if (!activeProfileId) return;
    const pid = activeProfileId;
    showConfirm('move', keys, async () => {
      try {
        await moveObjects(pid, bucket, keys, destPrefix);
        await fileStore.refresh(pid);
        if (uiStore.dualPanel) {
          await fileStore.refresh(`${pid}::right`);
        }
      } catch (e) {
        console.error('Move failed:', e);
      }
    });
  }

  function handleFileDrop(paths: string[]) {
    if (!paths.length || !activeProfileId || !fileState || !fileState.bucket) return;

    const pid = activeProfileId;
    const bucket = fileState.bucket;
    const prefix = fileState.prefix;
    const fileNames = paths.map((p) => p.split(/[\\/]/).at(-1) ?? p);

    showConfirm('upload', fileNames, async () => {
      uiStore.transferPanelExpanded = true;
      for (let i = 0; i < paths.length; i++) {
        const key = prefix + fileNames[i];
        try {
          await enqueueUpload(pid, paths[i], bucket, key);
        } catch (err) {
          console.error('Drop upload failed:', err);
        }
      }
      await transferStore.reload();
    });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex h-screen flex-col bg-background text-foreground overflow-hidden select-none"
  class:ring-2={isDragOver}
  class:ring-primary={isDragOver}
  class:ring-inset={isDragOver}
>
  <TitleBar
    onupload={handleUpload}
    ondownload={handleDownload}
    onnewfolder={handleNewFolder}
    onmultipartcleanup={() => (multipartCleanupOpen = true)}
  />
  <Toolbar
    disabled={!isConnected}
    onupload={handleUpload}
    ondownload={handleDownload}
    onnewfolder={handleNewFolder}
    ondelete={handleDelete}
    onmove={handleMove}
    onshareurl={handleShare}
    onmultipartcleanup={() => (multipartCleanupOpen = true)}
  />
  <TabBar />

  {#if activeTab && isConnected && activeProfileId && leftFileState}
    <div class="flex flex-1 min-h-0 overflow-hidden">
      <MainArea
        profileId={activeProfileId}
        buckets={activeTab.buckets}
        activeBucket={leftFileState.bucket}
        activePrefix={leftFileState.prefix}
        onnavigate={handleNavigate}
        oncontextmenu={handleContextMenu}
        onbgcontextmenu={handleBgContextMenu}
        onmovetoprefix={handleMoveToPrefix}
        oncopytoprefix={handleCopyToPrefix}
      />
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center">
      <div class="text-center space-y-3 max-w-md px-6">
        {#if activeTab?.status === 'error'}
          <div
            class="inline-flex h-10 w-10 items-center justify-center rounded-full bg-red-500/10 mb-2"
          >
            <span class="text-red-500 text-lg">!</span>
          </div>
          <h2 class="text-lg font-semibold text-foreground">Connection Failed</h2>
          <p class="text-sm text-red-400 bg-red-500/5 rounded-md px-3 py-2 font-mono break-all">
            {activeTab.error}
          </p>
          <div class="flex gap-2 justify-center pt-1">
            <button
              class="text-xs text-primary hover:underline"
              onclick={() => activeProfileId && profileStore.refreshBuckets(activeProfileId)}
            >
              Retry
            </button>
            <span class="text-zinc-600">·</span>
            <button
              class="text-xs text-muted-foreground hover:underline"
              onclick={() => (uiStore.profileManagerOpen = true)}
            >
              Edit Profile
            </button>
          </div>
        {:else if activeTab?.status === 'connecting'}
          <div
            class="inline-flex h-10 w-10 items-center justify-center rounded-full bg-yellow-500/10 mb-2"
          >
            <span class="text-yellow-400 text-lg animate-pulse">⟳</span>
          </div>
          <h2 class="text-lg font-semibold text-foreground">Connecting...</h2>
          <p class="text-sm text-muted-foreground">{activeTab.profile.name}</p>
        {:else}
          <h2 class="text-2xl font-bold text-foreground">S3V</h2>
          <p class="text-sm text-muted-foreground mb-4">
            {profileStore.tabs.length === 0
              ? 'Connect to an S3-compatible storage provider.'
              : 'Select a tab to browse.'}
          </p>

          {#if lastProfile}
            <div
              class="w-full max-w-xs mx-auto rounded-lg border border-border bg-muted/20 overflow-hidden"
            >
              <div class="px-3 py-1.5 border-b border-border/50">
                <span class="text-[10px] font-medium uppercase tracking-wider text-muted-foreground"
                  >Recent connected</span
                >
              </div>
              <button
                class="w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-accent/30"
                onclick={() => lastProfile && profileStore.openTab(lastProfile)}
              >
                <div
                  class="flex h-7 w-7 items-center justify-center rounded-md bg-primary/15 text-primary text-[10px] font-bold shrink-0"
                >
                  {lastProfile.provider.toUpperCase().slice(0, 2)}
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-xs font-medium text-foreground truncate">{lastProfile.name}</div>
                  <div class="text-[10px] text-muted-foreground">{lastProfile.provider}</div>
                </div>
                <span class="text-[10px] text-muted-foreground shrink-0">Connect</span>
              </button>
            </div>
          {/if}

          <Button variant="outline" size="sm" onclick={() => (uiStore.profileManagerOpen = true)}>
            Manage Profiles
          </Button>
        {/if}
      </div>
    </div>
  {/if}

  <DragOverlay />
  {#if confirmDialogOpen}
    <ConfirmDialog
      bind:open={confirmDialogOpen}
      action={confirmDialogAction}
      files={confirmDialogFiles}
      onconfirm={() => {
        confirmDialogCallback();
      }}
      oncancel={() => {}}
    />
  {/if}
  <LogPanel />
  <TransferPanel />
  <StatusBar />
  <ContextMenu />
  <Toaster position="bottom-right" theme="dark" />
  <ProfileManager />
  <MultipartCleanup bind:open={multipartCleanupOpen} />
  <FolderPicker
    bind:open={folderPickerOpen}
    profileId={activeProfileId ?? ''}
    bucket={fileState?.bucket ?? ''}
    title="Move to..."
    onconfirm={folderPickerCallback}
  />
  <InputDialog
    bind:open={inputDialogOpen}
    title={inputDialogConfig.title}
    label={inputDialogConfig.label ?? ''}
    placeholder={inputDialogConfig.placeholder ?? ''}
    defaultValue={inputDialogConfig.defaultValue ?? ''}
    confirmText={inputDialogConfig.confirmText ?? 'OK'}
    onconfirm={inputDialogConfig.onconfirm}
    oncancel={() => {}}
  />
</div>
