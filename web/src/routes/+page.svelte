<script lang="ts">
  import Header from '$lib/components/Header.svelte';
  import Hero from '$lib/components/Hero.svelte';
  import Features from '$lib/components/Features.svelte';
  import Playground from '$lib/components/playground/Playground.svelte';
  import Download from '$lib/components/Download.svelte';
  import Contact from '$lib/components/Contact.svelte';
  import Footer from '$lib/components/Footer.svelte';
  import { getLocale } from '$lib/i18n.svelte';
</script>

<Header />

{#key getLocale()}
  <main class="animate-fade-in">
    <Hero />
    <Features />
    <Playground />
    <Download />
    <Contact />
  </main>
{/key}

<Footer />

<style>
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  :global(.animate-fade-in) {
    animation: fadeIn 0.3s ease-out;
  }
</style>
