<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import * as m from '$lib/paraglide/messages';
  import { getLocale, locales, switchLocale, type Locale } from '$lib/i18n.svelte';

  interface Props {
    open: boolean;
  }

  let { open = $bindable() }: Props = $props();

  let selected = $state<Locale>(getLocale() as Locale);

  const labels: Record<Locale, string> = {
    en: 'English',
    ko: '한국어',
    ja: '日本語',
  };

  $effect(() => {
    if (open) {
      selected = getLocale() as Locale;
    }
  });

  function handleOk() {
    switchLocale(selected);
    open = false;
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-xs">
    <DialogHeader>
      <DialogTitle>{m.language_title()}</DialogTitle>
    </DialogHeader>

    <div class="space-y-1">
      {#each locales as tag}
        <button
          class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors
            {selected === tag ? 'bg-primary/10 text-primary' : 'hover:bg-accent/30 text-foreground'}"
          onclick={() => (selected = tag)}
        >
          <span
            class="flex h-4 w-4 items-center justify-center rounded-full border
              {selected === tag ? 'border-primary' : 'border-muted-foreground/40'}"
          >
            {#if selected === tag}
              <span class="h-2 w-2 rounded-full bg-primary"></span>
            {/if}
          </span>
          {labels[tag]}
        </button>
      {/each}
    </div>

    <div class="flex justify-end gap-2 pt-1">
      <Button variant="ghost" size="sm" onclick={() => (open = false)}>{m.language_cancel()}</Button>
      <Button size="sm" onclick={handleOk}>{m.language_ok()}</Button>
    </div>
  </DialogContent>
</Dialog>
