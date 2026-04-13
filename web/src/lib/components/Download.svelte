<script lang="ts">
  import { DOWNLOADS } from '$lib/constants';
  import { Button } from '$lib/components/ui/button';
  import { Copy, Check, ShieldAlert, X, Terminal } from '@lucide/svelte';
  import { t } from '$lib/i18n.svelte';

  const platformSvgs: Record<string, { viewBox: string; paths: string }> = {
    macos: {
      viewBox: '0 0 384 512',
      paths: '<path fill="currentColor" d="M318.7 268.7c-.2-36.7 16.4-64.4 50-84.8-18.8-26.9-47.2-41.7-84.7-44.6-35.5-2.8-74.3 20.7-88.5 20.7-15 0-49.4-19.7-76.4-19.7C63.3 141.2 4 184.8 4 273.5q0 39.3 14.4 81.2c12.8 36.7 59 126.7 107.2 125.2 25.2-.6 43-17.9 75.8-17.9 31.8 0 48.3 17.9 76.4 17.9 48.6-.7 90.4-82.5 102.6-119.3-65.2-30.7-61.7-90-61.7-91.9zm-56.6-164.2c27.3-32.4 24.8-61.9 24-72.5-24.1 1.4-52 16.4-67.9 34.9-17.5 19.8-27.8 44.3-25.6 71.9 26.1 2 49.9-11.4 69.5-34.3z"/>',
    },
    'windows-x64': {
      viewBox: '0 0 448 512',
      paths: '<path fill="currentColor" d="M0 93.7l183.6-25.3v177.4H0V93.7zm0 324.6l183.6 25.3V268.4H0v149.9zm203.8 28L448 480V268.4H203.8v177.9zm0-380.6v180.1H448V32L203.8 65.7z"/>',
    },
    'windows-arm64': {
      viewBox: '0 0 448 512',
      paths: '<path fill="currentColor" d="M0 93.7l183.6-25.3v177.4H0V93.7zm0 324.6l183.6 25.3V268.4H0v149.9zm203.8 28L448 480V268.4H203.8v177.9zm0-380.6v180.1H448V32L203.8 65.7z"/>',
    },
  };

  type Tab = 'direct' | 'homebrew';
  let activeTab = $state<Tab>('direct');

  let macDialogOpen = $state(false);
  let macDownloadUrl = $state('');
  let copied = $state(false);
  let brewCopied = $state<string | null>(null);

  const quarantineCmd = "xattr -d 'com.apple.quarantine' /Applications/s3v.app";
  const brewInstallCmd = 'brew tap aciddust/tap && brew install --cask s3v';
  const brewUpdateCmd = 'brew update && brew upgrade --cask s3v';

  function handleMacClick(e: MouseEvent, url: string) {
    e.preventDefault();
    macDownloadUrl = url;
    macDialogOpen = true;
  }

  function proceedDownload() {
    window.open(macDownloadUrl, '_blank');
    macDialogOpen = false;
  }

  function copyCommand() {
    navigator.clipboard.writeText(quarantineCmd).catch(() => {});
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function copyBrewCmd(cmd: string) {
    navigator.clipboard.writeText(cmd).catch(() => {});
    brewCopied = cmd;
    setTimeout(() => (brewCopied = null), 2000);
  }
</script>

<section id="download" class="py-20">
  <div class="mx-auto max-w-5xl px-4 text-center">
    <h2 class="mb-4 text-3xl font-semibold text-foreground">{t('download.heading')}</h2>
    <p class="mb-8 text-muted-foreground">{t('download.desc')}</p>

    <!-- Tabs -->
    <div class="mx-auto mb-6 flex max-w-sm rounded-lg border border-border bg-muted/30 p-1">
      <button
        class="flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors cursor-pointer
          {activeTab === 'direct' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
        onclick={() => (activeTab = 'direct')}
      >
        {t('download.tab.direct')}
      </button>
      <button
        class="flex-1 rounded-md px-4 py-2 text-sm font-medium transition-colors cursor-pointer
          {activeTab === 'homebrew' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
        onclick={() => (activeTab = 'homebrew')}
      >
        <span class="flex items-center justify-center gap-1.5">
          <Terminal class="h-4 w-4" />
          {t('download.tab.homebrew')}
        </span>
      </button>
    </div>

    <!-- Direct Download -->
    {#if activeTab === 'direct'}
      <div class="mx-auto flex max-w-sm flex-col gap-3">
        {#each Object.entries(DOWNLOADS) as [platform, { url, label }]}
          {@const svg = platformSvgs[platform]}
          {#if url}
            {#if platform === 'macos'}
              <Button size="lg" class="w-full px-5" onclick={(e) => handleMacClick(e, url)}>
                <span class="flex w-full items-center">
                  {#if svg}
                    <svg class="h-5 w-5 shrink-0" viewBox={svg.viewBox} aria-hidden="true">{@html svg.paths}</svg>
                  {/if}
                  <span class="flex-1 text-center">{label} {t('download.button')}</span>
                </span>
              </Button>
            {:else}
              <Button size="lg" href={url} target="_blank" rel="noopener noreferrer" class="w-full px-5">
                <span class="flex w-full items-center">
                  {#if svg}
                    <svg class="h-5 w-5 shrink-0" viewBox={svg.viewBox} aria-hidden="true">{@html svg.paths}</svg>
                  {/if}
                  <span class="flex-1 text-center">{label} {t('download.button')}</span>
                </span>
              </Button>
            {/if}
          {:else}
            <Button size="lg" variant="outline" disabled class="w-full px-5">
              <span class="flex w-full items-center">
                {#if svg}
                  <svg class="h-5 w-5 shrink-0" viewBox={svg.viewBox} aria-hidden="true">{@html svg.paths}</svg>
                {/if}
                <span class="flex-1 text-center">{label} — {t('download.preparing')}</span>
              </span>
            </Button>
          {/if}
        {/each}
      </div>
    {/if}

    <!-- Homebrew -->
    {#if activeTab === 'homebrew'}
      <div class="mx-auto max-w-sm space-y-4">
        <p class="text-sm text-muted-foreground">{t('download.brew.desc')}</p>

        <!-- Install command -->
        <div class="rounded-lg border border-border bg-muted/30 p-3 text-left">
          <div class="mb-1.5 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/70">Install</div>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-sm text-foreground break-all select-all">{brewInstallCmd}</code>
            <button
              class="cursor-pointer shrink-0 rounded-md p-1.5 transition-colors hover:bg-accent"
              onclick={() => copyBrewCmd(brewInstallCmd)}
              title="Copy"
            >
              {#if brewCopied === brewInstallCmd}
                <Check class="h-3.5 w-3.5 text-green-500" />
              {:else}
                <Copy class="h-3.5 w-3.5 text-muted-foreground" />
              {/if}
            </button>
          </div>
        </div>

        <!-- Quarantine hint -->
        <div class="rounded-lg border border-border bg-amber-500/5 p-3 text-left">
          <div class="mb-1.5 flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-wider text-amber-500/70">
            <ShieldAlert class="h-3 w-3" />
            {t('download.mac.title')}
          </div>
          <p class="mb-2 text-xs text-muted-foreground">{t('download.mac.hint')}</p>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-sm text-foreground break-all select-all">{quarantineCmd}</code>
            <button
              class="cursor-pointer shrink-0 rounded-md p-1.5 transition-colors hover:bg-accent"
              onclick={() => copyBrewCmd(quarantineCmd)}
              title="Copy"
            >
              {#if brewCopied === quarantineCmd}
                <Check class="h-3.5 w-3.5 text-green-500" />
              {:else}
                <Copy class="h-3.5 w-3.5 text-muted-foreground" />
              {/if}
            </button>
          </div>
        </div>

        <!-- Update command -->
        <div class="rounded-lg border border-border bg-muted/30 p-3 text-left">
          <div class="mb-1.5 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/70">{t('download.brew.update')}</div>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-sm text-foreground break-all select-all">{brewUpdateCmd}</code>
            <button
              class="cursor-pointer shrink-0 rounded-md p-1.5 transition-colors hover:bg-accent"
              onclick={() => copyBrewCmd(brewUpdateCmd)}
              title="Copy"
            >
              {#if brewCopied === brewUpdateCmd}
                <Check class="h-3.5 w-3.5 text-green-500" />
              {:else}
                <Copy class="h-3.5 w-3.5 text-muted-foreground" />
              {/if}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</section>

<!-- macOS Security Dialog -->
{#if macDialogOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" onclick={() => (macDialogOpen = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="mx-4 w-full max-w-md rounded-xl border border-border bg-background p-6 shadow-2xl" onclick={(e) => e.stopPropagation()}>
      <!-- Header -->
      <div class="mb-4 flex items-start justify-between">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-full bg-amber-500/10">
            <ShieldAlert class="h-5 w-5 text-amber-500" />
          </div>
          <h3 class="text-lg font-semibold">{t('download.mac.title')}</h3>
        </div>
        <button class="cursor-pointer rounded-md p-1 hover:bg-accent transition-colors" onclick={() => (macDialogOpen = false)}>
          <X class="h-4 w-4 text-muted-foreground" />
        </button>
      </div>

      <!-- Content -->
      <div class="space-y-3 text-sm text-muted-foreground">
        <p>{t('download.mac.desc')}</p>

        <!-- Code block -->
        <div class="flex items-center gap-2 rounded-lg border border-border bg-muted/50 px-3 py-2.5">
          <code class="flex-1 text-xs text-foreground break-all select-all">{quarantineCmd}</code>
          <button
            class="cursor-pointer shrink-0 rounded-md p-1.5 transition-colors hover:bg-accent"
            onclick={copyCommand}
            title="Copy"
          >
            {#if copied}
              <Check class="h-3.5 w-3.5 text-green-500" />
            {:else}
              <Copy class="h-3.5 w-3.5 text-muted-foreground" />
            {/if}
          </button>
        </div>

        <p class="text-xs text-muted-foreground/70">{t('download.mac.hint')}</p>
      </div>

      <!-- Actions -->
      <div class="mt-5 flex gap-2 justify-end">
        <Button variant="outline" onclick={() => (macDialogOpen = false)}>{t('download.mac.cancel')}</Button>
        <Button onclick={proceedDownload}>{t('download.mac.proceed')}</Button>
      </div>
    </div>
  </div>
{/if}
