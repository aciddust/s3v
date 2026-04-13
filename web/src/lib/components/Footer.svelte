<script lang="ts">
  import { APP_NAME, APP_VERSION, APP_DESCRIPTION } from '$lib/constants';
</script>

<footer class="border-t bg-zinc-950 py-10 text-zinc-400">
  <div class="mx-auto flex max-w-5xl flex-col gap-6 px-4">
    <!-- Top row: branding left, icons right -->
    <div class="flex items-start justify-between">
      <div>
        <h3 class="text-lg font-bold text-white">{APP_NAME}</h3>
        <p class="mt-1 text-sm">{APP_DESCRIPTION}</p>
      </div>
      <div class="flex items-center gap-3">
        <a href="https://github.com/aciddust/s3v" target="_blank" rel="noopener noreferrer" class="transition-colors hover:text-white" title="GitHub">
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
            <path d="M9 18c-4.51 2-5-2-7-2" />
          </svg>
        </a>
      </div>
    </div>

    <!-- Bottom row: license + copyright -->
    <div class="space-y-1 text-xs text-zinc-500">
      <p>
        {APP_NAME} is released under MIT License with Commons Clause. Current Version: {APP_VERSION}
      </p>
      <p>
        Third-party components are licensed under their respective terms.
        <a href="/licenses" class="underline transition-colors hover:text-zinc-300">View Licenses</a>
      </p>
      <p>&copy; {new Date().getFullYear()} d3fau1t All rights reserved.</p>
    </div>
  </div>
</footer>
