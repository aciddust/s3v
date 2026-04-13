<script lang="ts">
  import { FEEDBACK_API, APP_NAME } from '$lib/constants';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Send } from '@lucide/svelte';
  import { t } from '$lib/i18n.svelte';

  let name = $state('');
  let email = $state('');
  let message = $state('');
  let status = $state<'idle' | 'sending' | 'success' | 'error'>('idle');

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim() || !email.trim() || !message.trim()) return;

    status = 'sending';
    try {
      const res = await fetch(FEEDBACK_API, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          app: APP_NAME,
          name: name.trim(),
          email: email.trim(),
          message: message.trim(),
        }),
      });
      if (!res.ok) throw new Error('Failed');
      status = 'success';
      name = '';
      email = '';
      message = '';
    } catch {
      status = 'error';
    }
  }
</script>

<section id="contact" class="bg-muted/30 py-20">
  <div class="mx-auto max-w-md px-4">
    <h2 class="mb-8 text-center text-3xl font-semibold text-foreground">{t('contact.heading')}</h2>
    <form class="space-y-4" onsubmit={handleSubmit}>
      <div>
        <Input placeholder={t('contact.name')} bind:value={name} required />
      </div>
      <div>
        <Input type="email" placeholder={t('contact.email')} bind:value={email} required />
      </div>
      <div>
        <Textarea placeholder={t('contact.message')} bind:value={message} rows={5} required />
      </div>
      <Button type="submit" class="w-full" disabled={status === 'sending'}>
        {#if status === 'sending'}
          {t('contact.sending')}
        {:else}
          <Send class="mr-2 h-4 w-4" />
          {t('contact.submit')}
        {/if}
      </Button>
      {#if status === 'success'}
        <p class="text-center text-sm text-green-600">{t('contact.success')}</p>
      {/if}
      {#if status === 'error'}
        <p class="text-center text-sm text-destructive">{t('contact.error')}</p>
      {/if}
    </form>
  </div>
</section>
