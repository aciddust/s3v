<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { TriangleAlert } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  export type ConfirmAction = 'delete' | 'move' | 'copy' | 'upload';

  interface Props {
    open: boolean;
    action: ConfirmAction;
    files: string[];
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { open = $bindable(), action, files, onconfirm, oncancel }: Props = $props();

  const actionLabels = $derived<
    Record<ConfirmAction, { title: string; confirm: string; variant: 'default' | 'destructive'; warning: string }>
  >({
    delete: { title: m.confirm_delete(), confirm: m.confirm_delete(), variant: 'destructive', warning: m.confirm_delete_warning() },
    move: { title: m.confirm_move(), confirm: m.confirm_move(), variant: 'default', warning: m.confirm_move_warning() },
    copy: { title: m.confirm_copy(), confirm: m.confirm_copy(), variant: 'default', warning: m.confirm_copy_warning() },
    upload: { title: m.confirm_upload(), confirm: m.confirm_upload(), variant: 'default', warning: m.confirm_upload_warning() },
  });

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
      <DialogTitle>{label.title} — {m.confirm_count({ count: files.length })}</DialogTitle>
    </DialogHeader>

    <ul class="max-h-60 overflow-auto space-y-0.5">
      {#each visibleFiles as file}
        <li class="flex items-center gap-2 rounded px-2 py-1 text-xs bg-muted/40">
          <span class="truncate text-foreground">{displayName(file)}</span>
        </li>
      {/each}
    </ul>

    {#if remainCount > 0}
      <p class="text-xs text-muted-foreground px-2">{m.confirm_remaining({ count: remainCount })}</p>
    {/if}

    <div class="flex items-center gap-1.5 px-2 pt-2 text-xs {action === 'delete' || action === 'move' ? 'text-destructive' : 'text-muted-foreground'}">
      <TriangleAlert class="h-3.5 w-3.5 shrink-0" />
      <span>{label.warning}</span>
    </div>

    <div class="flex justify-end gap-2 pt-1">
      <Button variant="ghost" size="sm" onclick={handleCancel}>{m.confirm_cancel()}</Button>
      <Button variant={label.variant} size="sm" onclick={handleConfirm}>{label.confirm}</Button>
    </div>
  </DialogContent>
</Dialog>
