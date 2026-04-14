<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { invoke } from '@tauri-apps/api/core';
  import { APP_NAME } from '$lib/constants';
  import { toast } from 'svelte-sonner';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
  }

  let { open = $bindable() }: Props = $props();

  let name = $state('');
  let email = $state('');
  let content = $state('');
  let sending = $state(false);

  async function handleSubmit() {
    if (!content.trim()) return;
    sending = true;
    try {
      await invoke('send_feedback', {
        name: `${APP_NAME}: ${name || 'Anonymous'}`,
        email: email || 'no-reply@s3v.app',
        content: content.trim(),
      });
      toast.success(m.feedback_success());
      name = '';
      email = '';
      content = '';
      open = false;
    } catch (e) {
      console.error('Feedback failed:', e);
      toast.error(m.feedback_error());
    } finally {
      sending = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSubmit();
    }
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-sm">
    <DialogHeader>
      <DialogTitle>{m.feedback_title()}</DialogTitle>
    </DialogHeader>

    <div class="space-y-3">
      <div class="space-y-1">
        <label for="fb-name" class="text-xs text-muted-foreground">{m.feedback_name()}</label>
        <Input id="fb-name" bind:value={name} placeholder={m.feedback_placeholder_optional()} />
      </div>

      <div class="space-y-1">
        <label for="fb-email" class="text-xs text-muted-foreground">{m.feedback_email()}</label>
        <Input
          id="fb-email"
          bind:value={email}
          placeholder={m.feedback_placeholder_optional()}
          type="email"
        />
      </div>

      <div class="space-y-1">
        <label for="fb-content" class="text-xs text-muted-foreground">{m.feedback_message()}</label>
        <!-- svelte-ignore a11y_autofocus -->
        <textarea
          id="fb-content"
          bind:value={content}
          placeholder={m.feedback_placeholder_content()}
          class="flex min-h-[100px] w-full rounded-md border border-input bg-background px-3 py-2 text-xs
            placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2
            focus-visible:ring-ring resize-none"
          autofocus
          onkeydown={handleKeydown}
        ></textarea>
      </div>

      <div class="flex justify-between items-center pt-1">
        <span class="text-[10px] text-muted-foreground">{m.feedback_shortcut()}</span>
        <div class="flex gap-2">
          <Button variant="ghost" size="sm" onclick={() => (open = false)}
            >{m.feedback_cancel()}</Button
          >
          <Button size="sm" onclick={handleSubmit} disabled={!content.trim() || sending}>
            {sending ? m.feedback_sending() : m.feedback_send()}
          </Button>
        </div>
      </div>
    </div>
  </DialogContent>
</Dialog>
