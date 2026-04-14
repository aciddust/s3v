<script lang="ts">
  import { store } from './store.svelte';
  import { Plug, Trash2, Plus, Pencil, Check, X } from '@lucide/svelte';

  type Provider = 'aws' | 'rustfs' | 'r2' | 'custom';

  interface MockProfile {
    id: string;
    name: string;
    provider: Provider;
    endpoint: string | null;
    region: string;
    access_key_id: string;
    secret_access_key: string;
    path_style: boolean;
    default_bucket: string | null;
  }

  const providerColors: Record<Provider, string> = {
    aws: 'bg-amber-500',
    rustfs: 'bg-red-500',
    r2: 'bg-orange-500',
    custom: 'bg-blue-500',
  };

  // Pre-populated mock profiles
  let savedProfiles = $state<MockProfile[]>([
    { id: 'demo', name: 'Production (AWS)', provider: 'aws', endpoint: null, region: 'ap-northeast-2', access_key_id: 'AKIAIOSFODNN7EXAMPLE', secret_access_key: '••••••••••••••••', path_style: false, default_bucket: null },
    { id: 'r2-assets', name: 'CDN Assets (R2)', provider: 'r2', endpoint: 'https://abc123.r2.cloudflarestorage.com', region: 'auto', access_key_id: 'R2EXAMPLEKEY1234', secret_access_key: '••••••••••••••••', path_style: false, default_bucket: 'assets' },
    { id: 'rustfs-local', name: 'Local Dev (RustFS)', provider: 'rustfs', endpoint: 'http://localhost:9000', region: 'us-east-1', access_key_id: 'rustfsadmin', secret_access_key: '••••••••••••••••', path_style: true, default_bucket: null },
  ]);

  let showForm = $state(false);
  let editingId = $state<string | null>(null);
  let testResult = $state<{ success: boolean; message: string } | null>(null);
  let showDisabledAlert = $state(false);

  function onDisabledFieldClick() {
    showDisabledAlert = true;
    setTimeout(() => (showDisabledAlert = false), 2000);
  }

  let form = $state({
    name: '',
    provider: 'aws' as Provider,
    endpoint: null as string | null,
    region: 'us-east-1',
    access_key_id: '',
    secret_access_key: '',
    path_style: false,
    default_bucket: null as string | null,
  });

  const needsEndpoint = $derived(
    form.provider === 'rustfs' || form.provider === 'r2' || form.provider === 'custom',
  );

  function handleProviderChange(provider: Provider) {
    form.provider = provider;
    if (provider === 'rustfs') {
      form.path_style = true;
      form.endpoint = 'http://localhost:9000';
      form.region = 'us-east-1';
      form.access_key_id = 'rustfsadmin';
      form.secret_access_key = '••••••••••••••••';
    } else if (provider === 'aws') {
      form.path_style = false;
      form.endpoint = null;
      form.region = 'ap-northeast-2';
      form.access_key_id = 'AKIAIOSFODNN7EXAMPLE';
      form.secret_access_key = '••••••••••••••••';
    } else if (provider === 'r2') {
      form.path_style = false;
      form.endpoint = 'https://abc123.r2.cloudflarestorage.com';
      form.region = 'auto';
      form.access_key_id = 'R2EXAMPLEKEY1234';
      form.secret_access_key = '••••••••••••••••';
    } else {
      form.path_style = false;
      form.endpoint = 'https://s3.example.com';
      form.region = 'us-east-1';
      form.access_key_id = 'CUSTOM_ACCESS_KEY';
      form.secret_access_key = '••••••••••••••••';
    }
  }

  const bucketCounts: Record<string, number> = { aws: 3, r2: 1, rustfs: 5, custom: 2 };

  function handleConnect(profile: MockProfile) {
    store.connectProfile({ id: profile.id, name: profile.name, provider: profile.provider });
    const count = bucketCounts[profile.provider] ?? 1;
    testResult = { success: true, message: `Connected — ${count} bucket(s) found` };
    setTimeout(() => {
      store.profileManagerOpen = false;
      testResult = null;
    }, 800);
  }

  function handleEdit(profile: MockProfile) {
    editingId = profile.id;
    form = { ...profile };
    testResult = null;
    showForm = true;
  }

  function handleDelete(id: string) {
    savedProfiles = savedProfiles.filter((p) => p.id !== id);
    store.disconnectProfile(id);
  }

  function handleSave() {
    if (!form.name) return;

    if (editingId) {
      savedProfiles = savedProfiles.map((p) => p.id === editingId ? { ...form, id: editingId } : p);
    } else {
      const id = `profile-${Date.now()}`;
      savedProfiles.push({ ...form, id });
    }
    // Simulate test success
    testResult = { success: true, message: 'Connection successful — 2 bucket(s) found' };
    setTimeout(() => {
      const profile = editingId
        ? savedProfiles.find((p) => p.id === editingId)!
        : savedProfiles[savedProfiles.length - 1];
      store.connectProfile({ id: profile.id, name: profile.name, provider: profile.provider });
      resetForm();
      store.profileManagerOpen = false;
      testResult = null;
    }, 1000);
  }

  function resetForm() {
    form = { name: '', provider: 'aws', endpoint: null, region: 'us-east-1', access_key_id: '', secret_access_key: '', path_style: false, default_bucket: null };
    testResult = null;
    showForm = false;
    editingId = null;
  }

  function close() {
    resetForm();
    store.profileManagerOpen = false;
  }
</script>

{#if store.profileManagerOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="absolute inset-0 z-40 flex items-center justify-center bg-black/50 backdrop-blur-[1px]" onclick={close}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="w-[400px] max-h-[90%] overflow-y-auto rounded-lg border border-border bg-background shadow-2xl" onclick={(e) => e.stopPropagation()}>
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-border px-4 py-2">
        <h2 class="text-sm font-semibold">Profile Manager</h2>
        <button class="rounded-md p-1 hover:bg-accent transition-colors" onclick={close}>
          <X class="h-4 w-4 text-muted-foreground" />
        </button>
      </div>

      <div class="p-4">
        {#if !showForm}
          <!-- Profile list -->
          <div class="max-h-44 overflow-y-auto space-y-1">
            {#each savedProfiles as profile (profile.id)}
              {@const isConnected = store.profiles.some((p) => p.id === profile.id)}
              <div class="flex items-center gap-2 rounded-md border border-border px-3 py-2">
                <div class="h-2.5 w-2.5 rounded-full {providerColors[profile.provider]}"></div>
                <span class="flex-1 text-sm font-medium truncate">{profile.name}</span>
                <span class="text-xs text-muted-foreground uppercase">{profile.provider}</span>
                <button
                  class="h-7 w-7 flex items-center justify-center rounded-md hover:bg-accent transition-colors {isConnected ? 'text-green-500' : 'text-muted-foreground'}"
                  onclick={() => handleConnect(profile)}
                  title={isConnected ? 'Connected' : 'Connect'}
                >
                  {#if isConnected}<Check class="h-3.5 w-3.5" />{:else}<Plug class="h-3.5 w-3.5" />{/if}
                </button>
                <button
                  class="h-7 w-7 flex items-center justify-center rounded-md text-muted-foreground hover:bg-accent transition-colors"
                  onclick={() => handleEdit(profile)}
                  title="Edit"
                >
                  <Pencil class="h-3.5 w-3.5" />
                </button>
                <button
                  class="h-7 w-7 flex items-center justify-center rounded-md text-destructive hover:bg-accent transition-colors"
                  onclick={() => handleDelete(profile.id)}
                  title="Delete"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </div>
            {:else}
              <div class="py-4 text-center text-sm text-muted-foreground">No profiles yet.</div>
            {/each}
          </div>

          {#if testResult}
            <div class="mt-3 flex items-start gap-2 rounded-md px-3 py-2 text-sm
              {testResult.success ? 'bg-green-500/10 text-green-400' : 'bg-destructive/10 text-destructive'}">
              {#if testResult.success}<Check class="h-4 w-4 shrink-0 mt-0.5" />{:else}<X class="h-4 w-4 shrink-0 mt-0.5" />{/if}
              {testResult.message}
            </div>
          {/if}

          <div class="mt-3 border-t border-border pt-3">
            <button
              class="flex w-full items-center justify-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-accent transition-colors"
              onclick={() => (showForm = true)}
            >
              <Plus class="h-4 w-4" /> New Profile
            </button>
          </div>
        {:else}
          <!-- Profile form -->
          <div class="space-y-2">
            <p class="text-sm font-medium">{editingId ? 'Edit Profile' : 'New Profile'}</p>

            <!-- Name -->
            <div class="space-y-1">
              <label class="text-xs text-muted-foreground" for="pg-name">Name</label>
              <input id="pg-name" class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs outline-none focus:border-ring" bind:value={form.name} placeholder="My S3 Profile" />
            </div>

            <!-- Provider -->
            <div class="space-y-1">
              <span class="text-xs text-muted-foreground">Provider</span>
              <div class="flex gap-2">
                {#each ['aws', 'rustfs', 'r2', 'custom'] as provider}
                  <button
                    class="flex items-center gap-1.5 rounded border px-2.5 py-1 text-xs transition-colors
                      {form.provider === provider ? 'border-primary bg-primary/10 text-primary' : 'border-border text-muted-foreground hover:border-primary/50'}"
                    onclick={() => handleProviderChange(provider as Provider)}
                  >
                    <span class="h-2 w-2 rounded-full {providerColors[provider as Provider]}"></span>
                    {provider}
                  </button>
                {/each}
              </div>
            </div>

            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="relative" onclick={onDisabledFieldClick}>
              <!-- Disabled field notice -->
              {#if showDisabledAlert}
                <div class="absolute -top-8 left-1/2 -translate-x-1/2 z-10 rounded-md bg-popover border border-border px-3 py-1 text-xs text-muted-foreground shadow-lg whitespace-nowrap">
                  체험 환경에서는 연결 정보를 수정할 수 없습니다
                </div>
              {/if}

              <div class="space-y-2 opacity-60 pointer-events-none">
                <!-- Endpoint + Region -->
                <div class="grid gap-2 {needsEndpoint ? 'grid-cols-2' : 'grid-cols-1'}">
                  {#if needsEndpoint}
                    <div class="space-y-1">
                      <label class="text-xs text-muted-foreground">Endpoint URL</label>
                      <input class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs" value={form.endpoint ?? ''} disabled />
                    </div>
                  {/if}
                  <div class="space-y-1">
                    <label class="text-xs text-muted-foreground">Region</label>
                    <input class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs" value={form.region} disabled />
                  </div>
                </div>

                <!-- Access Key + Secret Key -->
                <div class="grid grid-cols-2 gap-2">
                  <div class="space-y-1">
                    <label class="text-xs text-muted-foreground">Access Key ID</label>
                    <input class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs" value={form.access_key_id} disabled />
                  </div>
                  <div class="space-y-1">
                    <label class="text-xs text-muted-foreground">Secret Access Key</label>
                    <input class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs" type="password" value={form.secret_access_key} disabled />
                  </div>
                </div>

                <!-- Path Style + Default Bucket -->
                <div class="grid grid-cols-2 gap-2 items-end">
                  <label class="flex items-center gap-2 py-1">
                    <input type="checkbox" class="h-3.5 w-3.5 rounded border-border" checked={form.path_style} disabled />
                    <span class="text-xs">Path-style URLs</span>
                  </label>
                  <div class="space-y-1">
                    <label class="text-xs text-muted-foreground">Default Bucket</label>
                    <input class="h-7 w-full rounded-md border border-input bg-transparent px-2 text-xs" value={form.default_bucket ?? ''} disabled />
                  </div>
                </div>
              </div>
            </div>

            <!-- Test result -->
            {#if testResult}
              <div class="flex items-start gap-2 rounded-md px-3 py-2 text-sm
                {testResult.success ? 'bg-green-500/10 text-green-400' : 'bg-destructive/10 text-destructive'}">
                {#if testResult.success}<Check class="h-4 w-4 shrink-0 mt-0.5" />{:else}<X class="h-4 w-4 shrink-0 mt-0.5" />{/if}
                {testResult.message}
              </div>
            {/if}

            <!-- Actions -->
            <div class="flex gap-2 justify-end">
              <button class="rounded-md px-3 py-1.5 text-sm hover:bg-accent transition-colors" onclick={resetForm}>Cancel</button>
              <button
                class="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
                disabled={!form.name}
                onclick={handleSave}
              >
                {editingId ? 'Update & Test' : 'Test & Save'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
