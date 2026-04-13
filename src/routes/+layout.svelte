<script>
  import { onMount } from 'svelte';
  import { initLocale } from '$lib/i18n.svelte';
  import '../app.css';

  let { children } = $props();

  onMount(() => {
    initLocale();
  });
</script>

{@render children()}
