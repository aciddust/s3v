<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import * as m from '$lib/paraglide/messages';

  export interface ConflictResult {
    overwrite: string[];
    skip: string[];
    rename: string[];
  }

  interface Props {
    open: boolean;
    conflicts: string[];
    onresult: (result: ConflictResult) => void;
    oncancel: () => void;
  }

  let { open = $bindable(), conflicts, onresult, oncancel }: Props = $props();

  let actions = $state<Record<string, 'overwrite' | 'skip' | 'rename'>>({});

  $effect(() => {
    if (open) {
      actions = {};
      for (const key of conflicts) {
        actions[key] = 'overwrite';
      }
    }
  });

  function displayName(key: string): string {
    const parts = key.split('/');
    return parts[parts.length - 1] || parts[parts.length - 2] + '/';
  }

  function applyAll(action: 'overwrite' | 'skip' | 'rename') {
    for (const key of conflicts) {
      actions[key] = action;
    }
    actions = { ...actions };
  }

  function handleConfirm() {
    const result: ConflictResult = { overwrite: [], skip: [], rename: [] };
    for (const [key, action] of Object.entries(actions)) {
      result[action].push(key);
    }
    onresult(result);
    open = false;
  }

  function handleCancel() {
    oncancel();
    open = false;
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-lg">
    <DialogHeader>
      <DialogTitle>{m.conflict_title()}</DialogTitle>
      <DialogDescription>{m.conflict_description({ count: conflicts.length })}</DialogDescription>
    </DialogHeader>

    <ul class="max-h-64 overflow-y-auto space-y-1 pr-1">
      {#each conflicts as key (key)}
        <li class="flex items-center gap-2 rounded px-2 py-1 bg-muted/40">
          <span class="truncate text-xs text-foreground flex-1" title={key}>{displayName(key)}</span
          >
          <select
            class="shrink-0 rounded border border-input bg-background px-1.5 py-0.5 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            value={actions[key]}
            onchange={(e) => {
              actions = {
                ...actions,
                [key]: (e.currentTarget as HTMLSelectElement).value as
                  | 'overwrite'
                  | 'skip'
                  | 'rename',
              };
            }}
          >
            <option value="overwrite">{m.conflict_overwrite()}</option>
            <option value="skip">{m.conflict_skip()}</option>
            <option value="rename">{m.conflict_rename()}</option>
          </select>
        </li>
      {/each}
    </ul>

    <div class="flex items-center gap-2 pt-1">
      <span class="text-xs text-muted-foreground">{m.conflict_apply_all()}</span>
      <Button variant="outline" size="sm" onclick={() => applyAll('overwrite')}
        >{m.conflict_overwrite()}</Button
      >
      <Button variant="outline" size="sm" onclick={() => applyAll('skip')}
        >{m.conflict_skip()}</Button
      >
      <Button variant="outline" size="sm" onclick={() => applyAll('rename')}
        >{m.conflict_rename()}</Button
      >
    </div>

    <div class="flex justify-end gap-2 pt-1">
      <Button variant="ghost" size="sm" onclick={handleCancel}>{m.conflict_cancel()}</Button>
      <Button variant="default" size="sm" onclick={handleConfirm}>{m.conflict_confirm()}</Button>
    </div>
  </DialogContent>
</Dialog>
