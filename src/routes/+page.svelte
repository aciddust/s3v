<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event';
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
  import TabDragOverlay from '$lib/components/TabDragOverlay.svelte';
  import FileDetailDialog from '$lib/components/FileDetailDialog.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import type { ConfirmAction } from '$lib/components/ConfirmDialog.svelte';
  import UploadTypeDialog from '$lib/components/UploadTypeDialog.svelte';
  import ConflictDialog from '$lib/components/ConflictDialog.svelte';
  import type { ConflictResult } from '$lib/components/ConflictDialog.svelte';
  import { Button } from '$lib/components/ui/button';

  // Stores
  import { profileStore } from '$lib/stores/profiles.svelte';
  import { fileStore } from '$lib/stores/files.svelte';
  import { transferStore } from '$lib/stores/transfers.svelte';
  import { bookmarkStore } from '$lib/stores/bookmarks.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { logStore } from '$lib/stores/logs.svelte';
  import { moveStore } from '$lib/stores/moves.svelte';
  import { uiStore, type ContextMenuItem } from '$lib/stores/ui.svelte';
  import { dragStore } from '$lib/stores/drag.svelte';

  // API
  import {
    deleteObjects,
    createFolder,
    renameObject,
    renameFolder,
    getPresignedUrl,
    moveObjects,
    copyObject,
    copyFolder,
    moveFolder,
    checkConflicts,
    classifyPaths,
    crossProfileCopyObject,
    crossProfileCopyFolder,
  } from '$lib/api/s3';
  import { enqueueUpload, enqueueDownload, enqueueFolderUpload, enqueueFolderDownload, onTransferCompleted, onTransferAdded } from '$lib/api/transfers';
  import { listProfiles, type ProfileSummary } from '$lib/api/profiles';

  // Utils
  import { matchBinding } from '$lib/utils/keys';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import * as m from '$lib/paraglide/messages';

  // Derived state
  const activeTab = $derived(profileStore.activeTab);
  const activeProfileId = $derived(profileStore.activeProfileId);
  const activePanel = $derived(uiStore.activePanel);
  const rightProfileId = $derived(uiStore.rightPanelProfileId);
  const activeStoreId = $derived(
    activeProfileId
      ? uiStore.dualPanel && activePanel === 'right'
        ? `${rightProfileId ?? activeProfileId}::right`
        : activeProfileId
      : null,
  );
  /** The profileId to use for S3 API calls on the currently active panel */
  const activeOpProfileId = $derived(
    activeStoreId ? activeStoreId.replace(/::right$/, '') : null,
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

  // File detail dialog state
  let fileDetailOpen = $state(false);
  let fileDetailBucket = $state('');
  let fileDetailKey = $state('');

  function handleFileOpen(bucket: string, key: string) {
    fileDetailBucket = bucket;
    fileDetailKey = key;
    fileDetailOpen = true;
  }

  async function handleFileDetailDownload(key: string) {
    if (!activeOpProfileId) return;
    fileDetailOpen = false;
    try {
      const destDir = await open({ multiple: false, directory: true });
      if (!destDir) return;
      const dir = Array.isArray(destDir) ? destDir[0] : destDir;
      const filename = key.split('/').pop() || key;
      await enqueueDownload(activeOpProfileId, fileDetailBucket, key, `${dir}/${filename}`);
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
    } catch (e) {
      console.error('Download failed:', e);
    }
  }

  async function handleFileDetailCopyUrl(key: string) {
    if (!activeOpProfileId) return;
    try {
      const url = await getPresignedUrl(activeOpProfileId, fileDetailBucket, key, 3600);
      await writeText(url);
      toast.success(m.file_detail_url_copied());
    } catch (e) {
      console.error('Copy URL failed:', e);
      toast.error(m.page_share_failed());
    }
  }

  // Drag state
  let isDragOver = $state(false);
  let multipartCleanupOpen = $state(false);
  let folderPickerOpen = $state(false);
  let folderPickerTitle = $state('');
  let folderPickerConfirmText = $state('');
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

  // --- Upload type dialog state ---
  let uploadTypeOpen = $state(false);

  // --- Conflict dialog state ---
  let conflictDialogOpen = $state(false);
  let conflictKeys = $state<string[]>([]);
  let conflictCallback = $state<((result: ConflictResult) => void) | null>(null);

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
    settingsStore.load();
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

    // Refresh panels when folder copy/move/move-objects completes
    const unlistenFolderOp = listen<{ phase: string }>('folder-op-progress', (e) => {
      if (e.payload.phase === 'completed' && activeProfileId) {
        fileStore.refresh(activeProfileId);
        if (uiStore.dualPanel) {
          fileStore.refresh(`${activeProfileId}::right`);
        }
      }
    });

    const unlistenMoveProgress = listen<{ phase: string }>('move-progress', (e) => {
      if (e.payload.phase === 'completed' && activeProfileId) {
        fileStore.refresh(activeProfileId);
        if (uiStore.dualPanel) {
          fileStore.refresh(`${activeProfileId}::right`);
        }
      }
    });

    // Auto-show transfer panel when any transfer is added (including native drag downloads)
    const unlistenTransferAdded = onTransferAdded(() => {
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
    });

    const unlistenFolderOpAdded = listen<{ phase: string }>('folder-op-progress', (e) => {
      if (e.payload.phase === 'started' && settingsStore.autoShowTransfers) {
        uiStore.showTransferPanel();
      }
    });

    const unlistenDrop = getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === 'enter') {
        isDragOver = true;
      } else if (event.payload.type === 'leave') {
        isDragOver = false;
      } else if (event.payload.type === 'drop') {
        isDragOver = false;
        handleFileDrop(event.payload.paths, event.payload.position);
      }
    });

    function handleInternalTabDrop(e: Event) {
      const detail = (e as CustomEvent<{ profileId: string; side: 'left' | 'right' }>).detail;
      if (!detail) return;
      handlePanelTabDrop(detail.side, detail.profileId);
    }
    document.addEventListener('internaltabdrop', handleInternalTabDrop);

    return () => {
      document.removeEventListener('internaltabdrop', handleInternalTabDrop);
      unlistenUploadDone.then((fn) => fn());
      unlistenTransferAdded.then((fn) => fn());
      unlistenFolderOpAdded.then((fn) => fn());
      unlistenFolderOp.then((fn) => fn());
      unlistenMoveProgress.then((fn) => fn());
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
    if (!activeOpProfileId || !activeStoreId || !fileState) return;
    if (fileState.selected.size === 0) {
      toast.warning(m.page_select_to_delete());
      return;
    }
    const keys = [...fileState.selected];
    const pid = activeOpProfileId;
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
    if (!activeOpProfileId || !activeStoreId || !fileState || !fileState.bucket) return;
    const storeId = activeStoreId;
    const pid = activeOpProfileId;
    showInputDialog({
      title: m.page_new_folder(),
      placeholder: m.page_folder_placeholder(),
      confirmText: m.page_create(),
      onconfirm: async (name) => {
        if (!pid || !fileState) return;
        const folderPrefix = fileState.prefix + name.replace(/\/$/, '') + '/';
        try {
          await createFolder(pid, fileState.bucket, folderPrefix);
          await fileStore.refresh(storeId);
        } catch (e) {
          console.error('Create folder failed:', e);
        }
      },
    });
  }

  function handleRename() {
    if (!activeOpProfileId || !activeStoreId || !fileState || fileState.selected.size !== 1) return;
    const storeId = activeStoreId;
    const pid = activeOpProfileId;
    const oldKey = [...fileState.selected][0];
    const isDir = oldKey.endsWith('/');
    const parts = oldKey.replace(/\/$/, '').split('/');
    const oldName = parts.at(-1) ?? oldKey;
    showInputDialog({
      title: isDir ? m.page_rename_folder() : m.page_rename(),
      defaultValue: oldName,
      confirmText: m.page_rename(),
      onconfirm: async (newName) => {
        if (!pid || !fileState || newName === oldName) return;
        try {
          if (isDir) {
            const parentPrefix = parts.slice(0, -1).join('/');
            const oldPrefix = oldKey;
            const newPrefix = (parentPrefix ? parentPrefix + '/' : '') + newName + '/';
            await renameFolder(pid, fileState.bucket, oldPrefix, newPrefix);
          } else {
            const prefix = oldKey.substring(0, oldKey.lastIndexOf('/') + 1);
            const newKey = prefix + newName;
            await renameObject(pid, fileState.bucket, oldKey, newKey);
          }
          await fileStore.refresh(storeId);
        } catch (e) {
          console.error('Rename failed:', e);
        }
      },
    });
  }

  function handleUpload() {
    if (!activeOpProfileId || !fileState || !fileState.bucket) {
      console.warn('Upload: no active profile or bucket');
      return;
    }
    uploadTypeOpen = true;
  }

  async function handleUploadFiles() {
    if (!activeOpProfileId || !fileState || !fileState.bucket) return;
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const fileNames = paths.map((p) => String(p).split(/[\\/]/).at(-1) ?? String(p));
      const pid = activeOpProfileId;
      const bucket = fileState.bucket;
      const prefix = fileState.prefix;
      showConfirm('upload', fileNames, async () => {
        try {
          for (let i = 0; i < paths.length; i++) {
            const key = prefix + fileNames[i];
            await enqueueUpload(pid, String(paths[i]), bucket, key);
          }
          if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
          await transferStore.reload();
        } catch (e) {
          console.error('Upload failed:', e);
        }
      });
    } catch (e) {
      console.error('Upload failed:', e);
    }
  }

  async function handleUploadFolder() {
    if (!activeOpProfileId || !fileState || !fileState.bucket) return;
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });
      if (!selected) return;
      const localDir = Array.isArray(selected) ? selected[0] : selected;
      const dirName = String(localDir).split(/[\\/]/).at(-1) ?? String(localDir);
      const pid = activeOpProfileId;
      const bucket = fileState.bucket;
      const remotePrefix = fileState.prefix + dirName + '/';

      // Check for conflicts
      const conflicts = await checkConflicts(pid, bucket, [remotePrefix]);
      if (conflicts.length > 0) {
        conflictKeys = conflicts;
        conflictCallback = async (result: ConflictResult) => {
          try {
            await enqueueFolderUpload(pid, String(localDir), bucket, remotePrefix, result.skip, result.rename);
            if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
            await transferStore.reload();
          } catch (e) {
            console.error('Folder upload failed:', e);
          }
        };
        conflictDialogOpen = true;
      } else {
        try {
          await enqueueFolderUpload(pid, String(localDir), bucket, remotePrefix);
          if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
          await transferStore.reload();
        } catch (e) {
          console.error('Folder upload failed:', e);
        }
      }
    } catch (e) {
      console.error('Folder upload failed:', e);
    }
  }

  async function handleDownload() {
    if (!activeOpProfileId || !fileState) return;
    const selected = Array.from(fileState.selected);
    if (selected.length === 0) {
      toast.warning(m.page_select_to_download());
      return;
    }
    try {
      const destDir = await open({
        multiple: false,
        directory: true,
      });
      if (!destDir) return;
      const dir = Array.isArray(destDir) ? destDir[0] : destDir;
      for (const key of selected) {
        if (key.endsWith('/')) {
          const folderName = key.slice(0, -1).split('/').pop() || '';
          const folderDest = `${dir}/${folderName}`;
          await enqueueFolderDownload(activeOpProfileId, fileState.bucket, key, folderDest);
        } else {
          const filename = key.split('/').pop() || key;
          await enqueueDownload(activeOpProfileId, fileState.bucket, key, `${dir}/${filename}`);
        }
      }
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
      fileStore.clearSelection(activeStoreId!);
    } catch (e) {
      console.error('Download failed:', e);
    }
  }

  async function handleShare() {
    if (!activeOpProfileId || !fileState) return;
    if (fileState.selected.size !== 1) {
      toast.warning(m.page_select_one_to_share());
      return;
    }
    const shareKey = [...fileState.selected][0];
    if (shareKey.endsWith('/')) {
      toast.warning(m.page_share_folder_not_supported());
      return;
    }
    try {
      const url = await getPresignedUrl(activeOpProfileId, fileState.bucket, shareKey, 3600);
      await writeText(url);
      toast.success(m.page_url_copied());
    } catch (e) {
      console.error('Share URL failed:', e);
      toast.error(m.page_share_failed());
    }
  }

  function handleMove() {
    if (!activeOpProfileId || !activeStoreId || !fileState) return;
    if (fileState.selected.size === 0) {
      toast.warning(m.page_select_to_move());
      return;
    }
    const keys = [...fileState.selected];
    const pid = activeOpProfileId;
    const storeId = activeStoreId;
    const bkt = fileState.bucket;
    folderPickerCallback = async (destPrefix: string) => {
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
      try {
        for (const key of keys) {
          if (key.endsWith('/')) {
            const folderName = key.slice(0, -1).split('/').pop() || '';
            await moveFolder(pid, bkt, key, bkt, destPrefix + folderName + '/');
          } else {
            await moveObjects(pid, bkt, [key], destPrefix);
          }
        }
        await fileStore.refresh(storeId);
        if (uiStore.dualPanel) {
          await fileStore.refresh(`${pid}::right`);
        }
      } catch (e) {
        console.error('Move failed:', e);
      }
    };
    folderPickerTitle = m.page_move_to();
    folderPickerConfirmText = '';
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
      { label: m.page_context_download(), action: handleDownload, disabled: keys.length === 0, separator: false },
      {
        label: m.page_context_copy_key(),
        action: () => handleCopyKey(keys),
        disabled: keys.length === 0,
        separator: false,
      },
      { label: m.page_context_share_url(), action: handleShare, disabled: keys.length !== 1, separator: false },
      { label: '', action: () => {}, separator: true },
      { label: m.page_context_rename(), action: handleRename, disabled: keys.length !== 1, separator: false },
      { label: m.page_context_delete(), action: handleDelete, disabled: keys.length === 0, separator: false },
      { label: '', action: () => {}, separator: true },
      { label: m.page_context_copy_to(), action: () => handleCopy(), disabled: keys.length === 0, separator: false },
    ];
    uiStore.openContextMenu(e.clientX, e.clientY, items);
  }

  function handleBgContextMenu(e: MouseEvent) {
    e.preventDefault();
    const items: ContextMenuItem[] = [
      { label: m.page_context_upload(), action: handleUpload, disabled: !isConnected, separator: false },
      { label: m.page_context_new_folder(), action: handleNewFolder, disabled: !isConnected, separator: false },
      { label: '', action: () => {}, separator: true },
      {
        label: m.page_context_refresh(),
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
      toast.success(m.page_copied_keys({ count: keys.length }));
    } catch (e) {
      console.error('Copy key failed:', e);
      toast.error(m.page_copy_key_failed());
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

  async function handleCopy() {
    if (!activeOpProfileId || !activeStoreId || !fileState) return;
    const selected = Array.from(fileState.selected);
    if (selected.length === 0) return;

    const pid = activeOpProfileId;
    const bkt = fileState.bucket;
    const storeId = activeStoreId;
    folderPickerCallback = async (destPrefix: string) => {
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
      try {
        for (const key of selected) {
          if (key.endsWith('/')) {
            const folderName = key.slice(0, -1).split('/').pop() || '';
            await copyFolder(pid!, bkt, key, bkt, destPrefix + folderName + '/');
          } else {
            const filename = key.split('/').pop() || key;
            const jobId = moveStore.addJob('copy', key, destPrefix + filename);
            try {
              await copyObject(pid!, bkt, key, bkt, destPrefix + filename);
              moveStore.completeJob(jobId, 'completed');
            } catch (e) {
              moveStore.completeJob(jobId, 'failed');
              throw e;
            }
          }
        }
        await fileStore.refresh(storeId!);
        if (uiStore.dualPanel) {
          await fileStore.refresh(`${pid}::right`);
        }
      } catch (e) {
        console.error('Copy failed:', e);
      }
    };
    folderPickerTitle = m.page_copy_to();
    folderPickerConfirmText = m.folder_picker_confirm_copy();
    folderPickerOpen = true;
  }

  function handleCopyToPrefix(
    sourceProfileId: string,
    destProfileId: string,
    sourceBucket: string,
    destBucket: string,
    destPrefix: string,
    keys: string[],
  ) {
    const isCrossProfile = sourceProfileId !== destProfileId;
    showConfirm('copy', keys, async () => {
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
      try {
        for (const key of keys) {
          if (key.endsWith('/')) {
            const folderName = key.slice(0, -1).split('/').pop() || '';
            if (isCrossProfile) {
              await crossProfileCopyFolder(
                sourceProfileId, sourceBucket, key,
                destProfileId, destBucket, destPrefix + folderName + '/',
              );
            } else {
              await copyFolder(destProfileId, sourceBucket, key, destBucket, destPrefix + folderName + '/');
            }
          } else {
            const filename = key.split('/').pop() || key;
            const jobId = moveStore.addJob('copy', key, destPrefix + filename);
            try {
              if (isCrossProfile) {
                await crossProfileCopyObject(
                  sourceProfileId, sourceBucket, key,
                  destProfileId, destBucket, destPrefix + filename,
                );
              } else {
                await copyObject(destProfileId, sourceBucket, key, destBucket, destPrefix + filename);
              }
              moveStore.completeJob(jobId, 'completed');
            } catch (e) {
              moveStore.completeJob(jobId, 'failed');
              throw e;
            }
          }
        }
        await fileStore.refresh(destProfileId);
        await fileStore.refresh(sourceProfileId);
        if (uiStore.dualPanel) {
          await fileStore.refresh(`${destProfileId}::right`);
        }
      } catch (e) {
        console.error('Copy failed:', e);
        toast.error(String(e));
      }
    });
  }

  function handleMoveToPrefix(bucket: string, destPrefix: string, keys: string[]) {
    if (!activeOpProfileId) return;
    const pid = activeOpProfileId;
    showConfirm('move', keys, async () => {
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
      try {
        for (const key of keys) {
          if (key.endsWith('/')) {
            const folderName = key.slice(0, -1).split('/').pop() || '';
            await moveFolder(pid, bucket, key, bucket, destPrefix + folderName + '/');
          } else {
            await moveObjects(pid, bucket, [key], destPrefix);
          }
        }
        await fileStore.refresh(pid);
        if (uiStore.dualPanel) {
          await fileStore.refresh(`${pid}::right`);
        }
      } catch (e) {
        console.error('Move failed:', e);
      }
    });
  }

  function handleTabDrop(targetProfileId: string, _modifier: 'meta' | 'shift' | null) {
    const data = dragStore.consume();
    if (!data || !data.keys.length) return;

    const sourceProfileId = data.profileId.replace(/::right$/, '');
    const targetState = fileStore.getState(targetProfileId);
    if (!targetState.bucket) return;

    // Same bucket + same prefix → nothing to do
    if (sourceProfileId === targetProfileId && data.bucket === targetState.bucket && data.sourcePrefix === targetState.prefix) return;

    const keys = data.keys;
    const sourceBucket = data.bucket;
    const destBucket = targetState.bucket;
    const destPrefix = targetState.prefix;

    const isCrossProfile = sourceProfileId !== targetProfileId;

    // Cross-tab always copies
    showConfirm('copy', keys, async () => {
      if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
      try {
        for (const key of keys) {
          if (key.endsWith('/')) {
            const folderName = key.slice(0, -1).split('/').pop() || '';
            if (isCrossProfile) {
              await crossProfileCopyFolder(
                sourceProfileId, sourceBucket, key,
                targetProfileId, destBucket, destPrefix + folderName + '/',
              );
            } else {
              await copyFolder(sourceProfileId, sourceBucket, key, destBucket, destPrefix + folderName + '/');
            }
          } else {
            const filename = key.split('/').pop() || key;
            const jobId = moveStore.addJob('copy', key, destPrefix + filename);
            try {
              if (isCrossProfile) {
                await crossProfileCopyObject(
                  sourceProfileId, sourceBucket, key,
                  targetProfileId, destBucket, destPrefix + filename,
                );
              } else {
                await copyObject(sourceProfileId, sourceBucket, key, destBucket, destPrefix + filename);
              }
              moveStore.completeJob(jobId, 'completed');
            } catch (e) {
              moveStore.completeJob(jobId, 'failed');
              throw e;
            }
          }
        }
        // Refresh both source and target
        await fileStore.refresh(sourceProfileId);
        await fileStore.refresh(targetProfileId);
      } catch (e) {
        console.error('Cross-tab copy failed:', e);
        toast.error(String(e));
      }
    });
  }

  async function handlePanelTabDrop(side: 'left' | 'right', droppedProfileId: string) {
    const isDual = uiStore.dualPanel;
    const droppedTab = profileStore.tabs.find((t) => t.profileId === droppedProfileId);

    if (side === 'right') {
      uiStore.setRightPanelProfile(droppedProfileId);
      if (!isDual) {
        uiStore.dualPanel = true;
      }
      if (droppedTab?.buckets.length) {
        const ok = await fileStore.navigate(`${droppedProfileId}::right`, droppedTab.buckets[0].name, '');
        if (!ok) {
          toast.error(m.page_connection_failed?.() ?? `Failed to connect to "${droppedTab.profile.name}". Please check the connection settings.`);
          uiStore.setRightPanelProfile(null);
          if (!isDual) uiStore.dualPanel = false;
        }
      }
    } else {
      // Preserve the right panel's current profile before switching left
      const prevProfileId = activeProfileId;
      if (!isDual) {
        // Single → dual: push current to right, set dropped to left
        if (prevProfileId) {
          const currentState = fileStore.getState(prevProfileId);
          uiStore.setRightPanelProfile(prevProfileId);
          uiStore.dualPanel = true;
          if (currentState.bucket) {
            fileStore.navigate(`${prevProfileId}::right`, currentState.bucket, currentState.prefix);
          }
        }
      } else if (!rightProfileId && prevProfileId && prevProfileId !== droppedProfileId) {
        // Dual with shared profile → pin the current profile to right before switching left
        const currentRightState = fileStore.getState(`${prevProfileId}::right`);
        uiStore.setRightPanelProfile(prevProfileId);
        if (currentRightState.bucket) {
          fileStore.navigate(`${prevProfileId}::right`, currentRightState.bucket, currentRightState.prefix);
        }
      }
      profileStore.setActiveTab(droppedProfileId);
    }
  }

  async function handleFileDrop(paths: string[], position?: { x: number; y: number }) {
    if (!paths.length || !activeProfileId) return;

    // Determine target panel and profileId based on drop position
    let targetState = fileState;
    let dropProfileId: string = activeOpProfileId ?? activeProfileId;
    if (uiStore.dualPanel && position && activeProfileId) {
      const sidebarAndHandle = uiStore.sidebarWidth + 4; // sidebar + resize handle
      const panelAreaX = position.x - sidebarAndHandle;
      const panelAreaWidth = window.innerWidth - sidebarAndHandle;
      const droppedOnRight = panelAreaX > panelAreaWidth / 2;
      const rightPid = rightProfileId ?? activeProfileId;
      targetState = droppedOnRight
        ? fileStore.getState(`${rightPid}::right`)
        : fileStore.getState(activeProfileId);
      dropProfileId = droppedOnRight ? rightPid : activeProfileId;
    }

    if (!targetState || !targetState.bucket) return;

    const pid = dropProfileId;
    const bucket = targetState.bucket;
    const prefix = targetState.prefix;

    // Classify dropped paths into files and directories
    const classified = await classifyPaths(paths);

    // Handle files with existing confirm flow
    if (classified.files.length > 0) {
      const fileNames = classified.files.map((p) => p.split(/[\\/]/).at(-1) ?? p);
      showConfirm('upload', fileNames, async () => {
        if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
        for (let i = 0; i < classified.files.length; i++) {
          const key = prefix + fileNames[i];
          try {
            await enqueueUpload(pid, classified.files[i], bucket, key);
          } catch (err) {
            console.error('Drop upload failed:', err);
          }
        }
        await transferStore.reload();
      });
    }

    // Handle directories with folder upload
    for (const dir of classified.directories) {
      const dirName = dir.split(/[\\/]/).at(-1) ?? dir;
      const remotePrefix = prefix + dirName + '/';
      try {
        await enqueueFolderUpload(pid, dir, bucket, remotePrefix);
        if (settingsStore.autoShowTransfers) uiStore.showTransferPanel();
        await transferStore.reload();
      } catch (err) {
        console.error('Drop folder upload failed:', err);
      }
    }
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
    oncopy={handleCopy}
    onshareurl={handleShare}
    shareDisabled={fileState ? [...fileState.selected].some((k) => k.endsWith('/')) : false}
    onmultipartcleanup={() => (multipartCleanupOpen = true)}
  />
  <TabBar ontabdrop={handleTabDrop} />

  {#if activeTab && isConnected && activeProfileId && leftFileState}
    <div class="flex flex-1 min-h-0 overflow-hidden">
      <MainArea
        profileId={activeProfileId}
        buckets={activeTab.buckets}
        activeBucket={leftFileState.bucket}
        activePrefix={leftFileState.prefix}
        rightProfileId={uiStore.rightPanelProfileId ?? undefined}
        rightProfileName={uiStore.rightPanelProfileId
          ? profileStore.tabs.find(t => t.profileId === uiStore.rightPanelProfileId)?.profile.name
          : undefined}
        onnavigate={handleNavigate}
        oncontextmenu={handleContextMenu}
        onbgcontextmenu={handleBgContextMenu}
        onmovetoprefix={handleMoveToPrefix}
        oncopytoprefix={handleCopyToPrefix}
        onfileopen={handleFileOpen}
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
          <h2 class="text-lg font-semibold text-foreground">{m.page_connection_failed()}</h2>
          <p class="text-sm text-red-400 bg-red-500/5 rounded-md px-3 py-2 font-mono break-all">
            {activeTab.error}
          </p>
          <div class="flex gap-2 justify-center pt-1">
            <button
              class="text-xs text-primary hover:underline"
              onclick={() => activeProfileId && profileStore.refreshBuckets(activeProfileId)}
            >
              {m.page_retry()}
            </button>
            <span class="text-zinc-600">·</span>
            <button
              class="text-xs text-muted-foreground hover:underline"
              onclick={() => (uiStore.profileManagerOpen = true)}
            >
              {m.page_edit_profile()}
            </button>
          </div>
        {:else if activeTab?.status === 'connecting'}
          <div
            class="inline-flex h-10 w-10 items-center justify-center rounded-full bg-yellow-500/10 mb-2"
          >
            <span class="text-yellow-400 text-lg animate-pulse">⟳</span>
          </div>
          <h2 class="text-lg font-semibold text-foreground">{m.page_connecting()}</h2>
          <p class="text-sm text-muted-foreground">{activeTab.profile.name}</p>
        {:else}
          <h2 class="text-2xl font-bold text-foreground">S3V</h2>
          <p class="text-sm text-muted-foreground mb-4">
            {profileStore.tabs.length === 0
              ? m.page_connect_prompt()
              : m.page_select_tab()}
          </p>

          {#if lastProfile}
            <div
              class="w-full min-w-[20rem] max-w-[30rem] mx-auto rounded-lg border border-border bg-muted/20 overflow-hidden"
            >
              <div class="px-3 py-1.5 border-b border-border/50">
                <span class="text-[10px] font-medium uppercase tracking-wider text-muted-foreground"
                  >{m.page_recent()}</span
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
                <span class="text-[10px] text-muted-foreground shrink-0">{m.page_connect()}</span>
              </button>
            </div>
          {/if}

          <Button variant="outline" size="sm" onclick={() => (uiStore.profileManagerOpen = true)}>
            {m.page_manage_profiles()}
          </Button>
        {/if}
      </div>
    </div>
  {/if}

  <DragOverlay />
  <TabDragOverlay />
  <FileDetailDialog
    bind:open={fileDetailOpen}
    profileId={activeProfileId ?? ''}
    bucket={fileDetailBucket}
    fileKey={fileDetailKey}
    ondownload={handleFileDetailDownload}
    oncopyurl={handleFileDetailCopyUrl}
  />
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
    title={folderPickerTitle || m.page_move_to()}
    confirmText={folderPickerConfirmText}
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
  <UploadTypeDialog
    bind:open={uploadTypeOpen}
    onfiles={handleUploadFiles}
    onfolder={handleUploadFolder}
    oncancel={() => {}}
  />
  {#if conflictDialogOpen}
    <ConflictDialog
      bind:open={conflictDialogOpen}
      conflicts={conflictKeys}
      onresult={(result) => {
        conflictCallback?.(result);
        conflictCallback = null;
      }}
      oncancel={() => {
        conflictCallback = null;
      }}
    />
  {/if}
</div>
