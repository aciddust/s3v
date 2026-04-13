<script lang="ts">
  import { browser } from '$app/environment';

  const STORAGE_KEY = 'playground-onboarded';

  let dismissed = $state(false);
  // SSR-safe: default true to hide overlay during prerender, then check localStorage on mount
  let alreadySeen = $state(true);

  $effect(() => {
    if (browser) {
      alreadySeen = localStorage.getItem(STORAGE_KEY) === 'true';
    }
  });

  function dismiss() {
    dismissed = true;
    if (browser) {
      localStorage.setItem(STORAGE_KEY, 'true');
    }
  }

  let visible = $derived(!alreadySeen && !dismissed);
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="absolute inset-0 z-30 flex items-center justify-center rounded-xl bg-black/60 backdrop-blur-[2px]"
    onclick={dismiss}
  >
    <!-- Pulse rings on key areas -->
    <div class="absolute right-[82%] top-[20%] h-14 w-14 animate-ping rounded-full border-2 border-blue-400/40"></div>
    <div class="absolute left-[15%] top-[40%] h-14 w-14 animate-ping rounded-full border-2 border-blue-400/40"></div>

    <!-- Central hint -->
    <div class="rounded-xl border border-blue-400/30 bg-blue-400/10 px-6 py-5 text-center shadow-2xl backdrop-blur-sm">
      <div class="mb-2 text-3xl">🖱️</div>
      <p class="text-sm leading-relaxed text-blue-100">
        파일을 드래그해서 S3V에 업로드해보세요<br />
        <span class="text-blue-300/60">우클릭으로 더 많은 옵션을 확인할 수 있습니다</span>
      </p>
      <p class="mt-3 text-xs text-blue-300/40">클릭하여 시작</p>
    </div>
  </div>
{/if}
