<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';

  export type ConfirmAction = 'delete' | 'move' | 'copy' | 'upload';

  interface Props {
    open: boolean;
    action: ConfirmAction;
    files: string[];
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { open = $bindable(), action, files, onconfirm, oncancel }: Props = $props();

  const actionLabels: Record<
    ConfirmAction,
    { title: string; confirm: string; variant: 'default' | 'destructive' }
  > = {
    delete: { title: 'Delete', confirm: 'Delete', variant: 'destructive' },
    move: { title: 'Move', confirm: 'Move', variant: 'default' },
    copy: { title: 'Copy', confirm: 'Copy', variant: 'default' },
    upload: { title: 'Upload', confirm: 'Upload', variant: 'default' },
  };

  const label = $derived(actionLabels[action]);
  const maxShow = 10;
  const visibleFiles = $derived(files.slice(0, maxShow));
  const remainCount = $derived(files.length - maxShow);

  function displayName(key: string): string {
    return key.replace(/\/$/, '').split('/').at(-1) ?? key;
  }

  function handleConfirm() {
    open = false;
    onconfirm();
  }

  function handleCancel() {
    open = false;
    oncancel();
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle>{label.title} — {files.length}건</DialogTitle>
    </DialogHeader>

    <ul class="max-h-60 overflow-auto space-y-0.5">
      {#each visibleFiles as file}
        <li class="flex items-center gap-2 rounded px-2 py-1 text-xs bg-muted/40">
          <span class="truncate text-foreground">{displayName(file)}</span>
        </li>
      {/each}
    </ul>

    {#if remainCount > 0}
      <p class="text-xs text-muted-foreground px-2">외 {remainCount}건</p>
    {/if}

    <div class="flex justify-end gap-2 pt-1">
      <Button variant="ghost" size="sm" onclick={handleCancel}>Cancel</Button>
      <Button variant={label.variant} size="sm" onclick={handleConfirm}>{label.confirm}</Button>
    </div>
  </DialogContent>
</Dialog>
