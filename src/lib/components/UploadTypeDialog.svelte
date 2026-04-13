<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
    onfiles: () => void;
    onfolder: () => void;
    oncancel: () => void;
  }

  let { open = $bindable(), onfiles, onfolder, oncancel }: Props = $props();
  let selected = $state<'files' | 'folder'>('files');
</script>

<Dialog bind:open>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle>{m.upload_type_title()}</DialogTitle>
    </DialogHeader>
    <div class="space-y-2">
      <label class="flex items-center gap-2 cursor-pointer">
        <input type="radio" bind:group={selected} value="files" />
        {m.upload_type_files()}
      </label>
      <label class="flex items-center gap-2 cursor-pointer">
        <input type="radio" bind:group={selected} value="folder" />
        {m.upload_type_folder()}
      </label>
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => { oncancel(); open = false; }}>
        {m.confirm_cancel()}
      </Button>
      <Button onclick={() => {
        if (selected === 'files') onfiles();
        else onfolder();
        open = false;
      }}>
        {m.toolbar_upload()}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
