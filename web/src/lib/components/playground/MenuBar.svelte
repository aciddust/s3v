<!-- web/src/lib/components/playground/MenuBar.svelte -->
<script lang="ts">
  import { Apple } from '@lucide/svelte';
</script>

<div class="flex h-7 items-center gap-4 rounded-t-lg bg-black/50 px-3 text-xs text-white/80 backdrop-blur-md select-none">
  <Apple class="h-3.5 w-3.5 fill-current" />
  <span class="font-semibold">Finder</span>
  <span>File</span>
  <span>Edit</span>
  <span>View</span>
  <span>Go</span>
  <span>Window</span>
  <span>Help</span>
</div>
