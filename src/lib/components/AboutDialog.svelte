<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import FeedbackDialog from './FeedbackDialog.svelte';
  import { APP_NAME, APP_VERSION, SUPPORT_EMAIL } from '$lib/constants';
  import * as m from '$lib/paraglide/messages';

  interface Props {
    open: boolean;
  }

  let { open = $bindable() }: Props = $props();
  let feedbackOpen = $state(false);
</script>

<Dialog bind:open>
  <DialogContent class="max-w-xs">
    <DialogHeader>
      <DialogTitle class="text-center">{APP_NAME}</DialogTitle>
    </DialogHeader>

    <div class="flex flex-col items-center gap-3 py-2">
      <img src="/s3v-logo.png" alt={APP_NAME} class="h-16 w-16 rounded-xl" />

      <div class="text-center space-y-1">
        <p class="text-sm font-medium text-foreground">v{APP_VERSION}</p>
        <p class="text-xs text-muted-foreground">{m.app_description()}</p>
      </div>

      <div class="w-full border-t border-border"></div>

      <div class="text-center space-y-0.5">
        <p class="text-[10px] uppercase tracking-wider text-muted-foreground">{m.about_support()}</p>
        <button
          class="text-xs text-primary hover:underline"
          onclick={() => { feedbackOpen = true; }}
        >
          {SUPPORT_EMAIL}
        </button>
      </div>

      <Button variant="ghost" size="sm" onclick={() => (open = false)}>{m.about_close()}</Button>
    </div>
  </DialogContent>
</Dialog>

<FeedbackDialog bind:open={feedbackOpen} />
