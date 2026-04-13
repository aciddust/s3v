<script lang="ts">
  import { APP_DESCRIPTION } from '$lib/constants';
  import { Button } from '$lib/components/ui/button';
  import { t } from '$lib/i18n.svelte';
  import logo from '$lib/assets/logo.png';

  function smoothScrollTo(e: MouseEvent, id: string) {
    e.preventDefault();
    const el = document.getElementById(id);
    if (!el) return;
    const targetY = el.getBoundingClientRect().top + window.scrollY - 64;
    const startY = window.scrollY;
    const diff = targetY - startY;
    const duration = 800;
    let start: number | null = null;
    function step(ts: number) {
      if (!start) start = ts;
      const p = Math.min((ts - start) / duration, 1);
      const ease = p < 0.5 ? 4 * p * p * p : 1 - Math.pow(-2 * p + 2, 3) / 2;
      window.scrollTo(0, startY + diff * ease);
      if (p < 1) requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
  }
</script>

<section class="flex flex-col items-center justify-center px-4 pt-32 pb-20 text-center">
  <img src={logo} alt="S3V" class="mb-6 h-20 w-20 saturate-150 brightness-150 drop-shadow-[0_0_20px_rgba(139,92,246,0.3)] sm:h-24 sm:w-24" />
  <h1 class="max-w-3xl text-4xl font-bold leading-snug tracking-tight text-foreground sm:text-5xl sm:leading-snug">
    {#each t('hero.title').split('\n') as line, i}
      {#if i > 0}<br />{/if}{line}
    {/each}
  </h1>
  <p class="mt-4 text-lg text-muted-foreground">{APP_DESCRIPTION}</p>
  <div class="mt-8">
    <Button size="lg" href="#download" onclick={(e) => smoothScrollTo(e, 'download')}>{t('hero.cta')}</Button>
  </div>
</section>
