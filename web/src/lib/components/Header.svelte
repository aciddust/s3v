<script lang="ts">
  import { APP_NAME } from '$lib/constants';
  import { Button } from '$lib/components/ui/button';
  import { t, getLocale, setLocale, type Locale } from '$lib/i18n.svelte';
  import { Languages } from '@lucide/svelte';
  import logo from '$lib/assets/logo.png';

  const locales: { code: Locale; label: string }[] = [
    { code: 'ko', label: '한국어' },
    { code: 'en', label: 'English' },
    { code: 'ja', label: '日本語' },
  ];

  let open = $state(false);

  function selectLocale(locale: Locale) {
    setLocale(locale);
    open = false;
  }

  function handleClickOutside(e: MouseEvent) {
    if (!(e.target as HTMLElement).closest('.locale-menu')) {
      open = false;
    }
  }

  function smoothScrollTo(e: MouseEvent, id: string) {
    e.preventDefault();
    const el = document.getElementById(id);
    if (!el) return;

    const headerOffset = 64;
    const targetY = el.getBoundingClientRect().top + window.scrollY - headerOffset;
    const startY = window.scrollY;
    const diff = targetY - startY;
    const duration = 800;
    let start: number | null = null;

    function step(ts: number) {
      if (!start) start = ts;
      const elapsed = ts - start;
      const progress = Math.min(elapsed / duration, 1);
      // easeInOutCubic
      const ease = progress < 0.5
        ? 4 * progress * progress * progress
        : 1 - Math.pow(-2 * progress + 2, 3) / 2;
      window.scrollTo(0, startY + diff * ease);
      if (progress < 1) requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
  }
</script>

<svelte:window onclick={handleClickOutside} />

<header class="fixed top-0 z-50 w-full border-b bg-background/80 backdrop-blur-sm">
  <div class="mx-auto flex h-14 max-w-5xl items-center px-4">
    <a href="/" class="flex flex-1 items-center gap-2 text-lg font-bold">
      <img src={logo} alt="S3V" class="h-7 w-7 saturate-150 brightness-150" />
      {APP_NAME}
    </a>

    <nav class="hidden items-center gap-6 text-sm md:flex">
      <a href="#features" class="whitespace-nowrap text-muted-foreground transition-colors hover:text-foreground" onclick={(e) => smoothScrollTo(e, 'features')}>{t('nav.features')}</a>
      <a href="#playground" class="whitespace-nowrap text-muted-foreground transition-colors hover:text-foreground" onclick={(e) => smoothScrollTo(e, 'playground')}>{t('nav.playground')}</a>
      <a href="#download" class="whitespace-nowrap text-muted-foreground transition-colors hover:text-foreground" onclick={(e) => smoothScrollTo(e, 'download')}>{t('nav.download')}</a>
      <a href="#contact" class="whitespace-nowrap text-muted-foreground transition-colors hover:text-foreground" onclick={(e) => smoothScrollTo(e, 'contact')}>{t('nav.contact')}</a>
    </nav>

    <div class="flex flex-1 items-center justify-end gap-2">
      <a href="https://github.com/aciddust/s3v" target="_blank" rel="noopener noreferrer" class="flex h-8 items-center justify-center rounded-md px-1.5 text-muted-foreground transition-colors hover:text-foreground" title="GitHub">
        <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
          <path d="M9 18c-4.51 2-5-2-7-2" />
        </svg>
      </a>
      <div class="locale-menu relative">
        <button
          class="flex h-8 cursor-pointer items-center gap-1.5 rounded-md px-2 text-xs text-muted-foreground transition-colors hover:text-foreground"
          onclick={() => (open = !open)}
        >
          <Languages class="h-4 w-4" />
          <span class="uppercase">{getLocale()}</span>
        </button>
        {#if open}
          <div class="absolute right-0 top-full mt-1 min-w-28 overflow-hidden rounded-md border bg-background py-1 shadow-md">
            {#each locales as { code, label }}
              <button
                class="flex w-full cursor-pointer items-center gap-2 px-3 py-1.5 text-left text-sm transition-colors hover:bg-muted {code === getLocale() ? 'font-medium text-foreground' : 'text-muted-foreground'}"
                onclick={() => selectLocale(code)}
              >
                {label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
      <Button size="sm" href="#download" onclick={(e) => smoothScrollTo(e, 'download')}>{t('nav.download')}</Button>
    </div>
  </div>
</header>
