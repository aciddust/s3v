<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
    title: string;
    label?: string;
    placeholder?: string;
    defaultValue?: string;
    confirmText?: string;
    onconfirm: (value: string) => void;
    oncancel: () => void;
  }

  let {
    open = $bindable(),
    title,
    label = '',
    placeholder = '',
    defaultValue = '',
    confirmText = 'OK',
    onconfirm,
    oncancel,
  }: Props = $props();

  let value = $state('');

  $effect(() => {
    if (open) {
      value = defaultValue;
    }
  });

  function handleConfirm() {
    if (!value.trim()) return;
    onconfirm(value.trim());
    open = false;
  }

  function handleCancel() {
    oncancel();
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleConfirm();
    }
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle>{title}</DialogTitle>
    </DialogHeader>
    {#if label}
      <p class="text-xs text-muted-foreground">{label}</p>
    {/if}
    <!-- svelte-ignore a11y_autofocus -->
    <Input bind:value {placeholder} autofocus onkeydown={handleKeydown} />
    <div class="flex justify-end gap-2">
      <Button variant="ghost" size="sm" onclick={handleCancel}>{m.input_cancel()}</Button>
      <Button size="sm" onclick={handleConfirm} disabled={!value.trim()}>{confirmText}</Button>
    </div>
  </DialogContent>
</Dialog>
