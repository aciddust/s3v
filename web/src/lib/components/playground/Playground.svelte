<script lang="ts">
  import { store } from './store.svelte';
  import MacosDesktop from './MacosDesktop.svelte';
  import DragGhost from './DragGhost.svelte';
  import ContextMenu from './ContextMenu.svelte';
  import { t } from '$lib/i18n.svelte';

  function onPointerMove(e: PointerEvent) {
    if (store.drag.active) {
      store.updateDrag(e.clientX, e.clientY, e.metaKey || e.ctrlKey);
    }
  }

  function onPointerUp() {
    if (store.drag.active) {
      store.endDrag();
    }
  }

  function onKeyChange(e: KeyboardEvent) {
    if (store.drag.active) {
      store.drag.modifier = e.metaKey || e.ctrlKey;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<svelte:window onkeydown={onKeyChange} onkeyup={onKeyChange} />

<section
  id="playground"
  class="py-20"
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
>
  <div class="mx-auto max-w-7xl px-4">
    <h2 class="mb-4 text-center text-3xl font-semibold text-foreground">{t('playground.heading')}</h2>
    <p class="mb-12 text-center text-muted-foreground">
      {t('playground.desc')}
    </p>

    <!-- Desktop only: full interactive experience (1024px+) -->
    <div class="hidden select-none lg:block">
      <MacosDesktop />
    </div>

    <!-- Below 1024px: fallback message -->
    <div class="flex flex-col items-center gap-4 rounded-xl border border-border bg-muted/30 p-8 text-center lg:hidden">
      <div class="text-4xl">🖥️</div>
      <p class="text-sm text-muted-foreground">
        {t('playground.mobile')}
      </p>
    </div>
  </div>
</section>

<DragGhost />
<ContextMenu />
