<script lang="ts">
  import { PanelsTopLeft, MousePointerClick, PlugZap } from '@lucide/svelte';
  import * as Card from '$lib/components/ui/card';
  import { t } from '$lib/i18n.svelte';

  const features = [
    { titleKey: 'features.dualPanel.title', descKey: 'features.dualPanel.desc', Icon: PanelsTopLeft },
    { titleKey: 'features.dnd.title', descKey: 'features.dnd.desc', Icon: MousePointerClick },
    { titleKey: 'features.multiConnect.title', descKey: 'features.multiConnect.desc', Icon: PlugZap },
  ] as const;
</script>

<section id="features" class="bg-muted/30 py-20">
  <div class="mx-auto max-w-5xl px-4">
    <h2 class="mb-12 text-center text-3xl font-semibold text-foreground">{t('features.heading')}</h2>
    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each features as feature}
        <Card.Root class="transition-shadow hover:shadow-md">
          <Card.Header>
            <div class="mb-2 flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
              <feature.Icon class="h-5 w-5 text-primary" />
            </div>
            <Card.Title>{t(feature.titleKey)}</Card.Title>
          </Card.Header>
          <Card.Content>
            <p class="text-sm text-muted-foreground">{t(feature.descKey)}</p>
          </Card.Content>
        </Card.Root>
      {/each}
    </div>
  </div>
</section>
