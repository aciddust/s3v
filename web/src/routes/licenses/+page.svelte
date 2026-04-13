<script lang="ts">
  import { APP_NAME } from '$lib/constants';
</script>

<svelte:head>
  <title>Third-Party Licenses — {APP_NAME}</title>
</svelte:head>

<div class="mx-auto max-w-3xl px-4 py-20">
  <h1 class="mb-2 text-3xl font-bold">Third-Party Licenses</h1>
  <p class="mb-10 text-muted-foreground">
    {APP_NAME} is built with the following open-source software.
  </p>

  <!-- Tauri -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">Tauri</h2>
    <p class="mb-2 text-sm text-muted-foreground">Desktop application framework</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright (c) 2020-2023 Tauri Programme within The Commons Conservancy</p>
      <p class="mt-1">Licensed under either of Apache License, Version 2.0 or MIT License, at your option.</p>
      <div class="mt-2 flex gap-3">
        <a href="https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">MIT</a>
        <a href="https://github.com/tauri-apps/tauri/blob/dev/LICENSE-APACHE" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">Apache 2.0</a>
      </div>
    </div>
  </section>

  <!-- Wry (patched) -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">Wry <span class="text-sm font-normal text-muted-foreground">(modified)</span></h2>
    <p class="mb-2 text-sm text-muted-foreground">Cross-platform WebView rendering library</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright (c) 2020-2023 Ngo Iok Ui &amp; Tauri Programme within The Commons Conservancy</p>
      <p class="mt-1">Licensed under either of Apache License, Version 2.0 or MIT License, at your option.</p>
      <p class="mt-2 rounded bg-amber-500/10 px-2 py-1 text-amber-600 dark:text-amber-400">
        This software includes a modified version of Wry. Modifications are maintained in <code>src-tauri/patches/wry/</code>.
      </p>
      <div class="mt-2 flex gap-3">
        <a href="https://github.com/tauri-apps/wry/blob/dev/LICENSE-MIT" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">MIT</a>
        <a href="https://github.com/tauri-apps/wry/blob/dev/LICENSE-APACHE" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">Apache 2.0</a>
      </div>
    </div>
  </section>

  <!-- AWS SDK -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">AWS SDK for Rust</h2>
    <p class="mb-2 text-sm text-muted-foreground">S3 API client</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.</p>
      <p class="mt-1">Licensed under the Apache License, Version 2.0.</p>
      <div class="mt-2">
        <a href="https://github.com/awslabs/aws-sdk-rust/blob/main/LICENSE" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">Apache 2.0</a>
      </div>
    </div>
  </section>

  <!-- Tokio -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">Tokio</h2>
    <p class="mb-2 text-sm text-muted-foreground">Asynchronous runtime for Rust</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright (c) Tokio Contributors</p>
      <p class="mt-1">Licensed under the MIT License.</p>
      <div class="mt-2">
        <a href="https://github.com/tokio-rs/tokio/blob/master/LICENSE" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">MIT</a>
      </div>
    </div>
  </section>

  <!-- Svelte / SvelteKit -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">Svelte / SvelteKit</h2>
    <p class="mb-2 text-sm text-muted-foreground">Frontend framework</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright (c) Svelte Contributors</p>
      <p class="mt-1">Licensed under the MIT License.</p>
      <div class="mt-2">
        <a href="https://github.com/sveltejs/svelte/blob/main/LICENSE.md" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">MIT</a>
      </div>
    </div>
  </section>

  <!-- Tailwind CSS -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">Tailwind CSS</h2>
    <p class="mb-2 text-sm text-muted-foreground">Utility-first CSS framework</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright (c) Tailwind Labs, Inc.</p>
      <p class="mt-1">Licensed under the MIT License.</p>
      <div class="mt-2">
        <a href="https://github.com/tailwindlabs/tailwindcss/blob/master/LICENSE" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">MIT</a>
      </div>
    </div>
  </section>

  <!-- Lucide -->
  <section class="mb-8">
    <h2 class="mb-1 text-lg font-semibold">Lucide Icons</h2>
    <p class="mb-2 text-sm text-muted-foreground">Icon library</p>
    <div class="rounded-lg border border-border bg-muted/30 p-4 text-xs text-muted-foreground">
      <p>Copyright (c) Lucide Contributors</p>
      <p class="mt-1">Licensed under the ISC License.</p>
      <div class="mt-2">
        <a href="https://github.com/lucide-icons/lucide/blob/main/LICENSE" target="_blank" rel="noopener noreferrer" class="text-foreground underline hover:no-underline">ISC</a>
      </div>
    </div>
  </section>

  <p class="mt-10 text-xs text-muted-foreground">
    For the complete list of dependencies and their licenses, see the project's
    <a href="https://github.com/aciddust/s3v" target="_blank" rel="noopener noreferrer" class="underline hover:no-underline">source repository</a>.
  </p>
</div>
