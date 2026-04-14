<script lang="ts">
  import { listObjects } from '$lib/api/s3';
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Folder, FolderOpen, ChevronRight, ChevronDown, CornerDownRight } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
    profileId: string;
    bucket: string;
    title?: string;
    confirmText?: string;
    onconfirm: (prefix: string) => void;
  }

  let {
    open = $bindable(),
    profileId,
    bucket,
    title = 'Move to...',
    confirmText,
    onconfirm,
  }: Props = $props();

  interface FolderNode {
    prefix: string;
    name: string;
    children: FolderNode[];
    expanded: boolean;
    loaded: boolean;
  }

  let root = $state<FolderNode>({
    prefix: '',
    name: '/',
    children: [],
    expanded: true,
    loaded: false,
  });
  let selectedPrefix = $state('');

  $effect(() => {
    if (open && profileId && bucket) {
      const newRoot: FolderNode = {
        prefix: '',
        name: bucket,
        children: [],
        expanded: true,
        loaded: false,
      };
      root = newRoot;
      selectedPrefix = '';
      loadFolder(newRoot.prefix);
    }
  });

  function updateNode(
    tree: FolderNode,
    targetPrefix: string,
    patch: Partial<FolderNode>,
  ): FolderNode {
    if (tree.prefix === targetPrefix) {
      return { ...tree, ...patch };
    }
    return {
      ...tree,
      children: tree.children.map((c) => updateNode(c, targetPrefix, patch)),
    };
  }

  async function loadFolder(prefix: string) {
    try {
      const result = await listObjects(profileId, bucket, prefix, '/');
      const children: FolderNode[] = result.common_prefixes.map((p) => ({
        prefix: p,
        name: p.replace(prefix, '').replace(/\/$/, ''),
        children: [],
        expanded: false,
        loaded: false,
      }));
      root = updateNode(root, prefix, { children, loaded: true });
    } catch (e) {
      console.error('Failed to load folders:', e);
    }
  }

  async function toggleNode(node: FolderNode) {
    const shouldExpand = !node.expanded;
    root = updateNode(root, node.prefix, { expanded: shouldExpand });
    if (shouldExpand && !node.loaded) {
      await loadFolder(node.prefix);
    }
  }

  function selectNode(prefix: string) {
    selectedPrefix = prefix;
  }

  function handleConfirm() {
    onconfirm(selectedPrefix);
    open = false;
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle>{title}</DialogTitle>
    </DialogHeader>

    <div class="rounded-md border border-border bg-muted/20 max-h-64 overflow-auto p-1 text-xs">
      {#snippet folderTree(node: FolderNode, depth: number)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div
          role="treeitem"
          aria-selected={selectedPrefix === node.prefix}
          class="flex items-center gap-1 rounded px-1 py-1 cursor-pointer transition-colors
            {selectedPrefix === node.prefix ? 'bg-primary/15 text-primary' : 'hover:bg-accent/30'}"
          style="padding-left: {depth * 16 + 4}px;"
          onclick={() => selectNode(node.prefix)}
          ondblclick={() => toggleNode(node)}
          onkeydown={(e) => e.key === 'Enter' && selectNode(node.prefix)}
        >
          <button
            class="shrink-0 p-0.5 rounded hover:bg-accent/50"
            onclick={(e) => {
              e.stopPropagation();
              toggleNode(node);
            }}
          >
            {#if node.expanded}
              <ChevronDown class="h-3 w-3 text-muted-foreground" />
            {:else}
              <ChevronRight class="h-3 w-3 text-muted-foreground" />
            {/if}
          </button>
          {#if node.expanded}
            <FolderOpen class="h-3.5 w-3.5 shrink-0 text-blue-400" />
          {:else}
            <Folder class="h-3.5 w-3.5 shrink-0 text-blue-400" />
          {/if}
          <span class="truncate">{node.name}</span>
        </div>

        {#if node.expanded}
          {#each node.children as child}
            {@render folderTree(child, depth + 1)}
          {/each}
          {#if node.loaded && node.children.length === 0}
            <div
              class="text-muted-foreground py-0.5"
              style="padding-left: {(depth + 1) * 16 + 4}px;"
            >
              {m.folder_picker_empty()}
            </div>
          {/if}
        {/if}
      {/snippet}

      {@render folderTree(root, 0)}
    </div>

    <div class="flex items-center gap-2 text-xs text-muted-foreground">
      <CornerDownRight class="h-3 w-3" />
      <span class="font-mono truncate">{bucket}/{selectedPrefix || m.folder_picker_root()}</span>
    </div>

    <div class="flex justify-end gap-2">
      <Button variant="ghost" size="sm" onclick={() => (open = false)}
        >{m.folder_picker_cancel()}</Button
      >
      <Button size="sm" onclick={handleConfirm}>{confirmText || m.folder_picker_confirm()}</Button>
    </div>
  </DialogContent>
</Dialog>
