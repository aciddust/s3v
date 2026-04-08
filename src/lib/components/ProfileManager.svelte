<script lang="ts">
  import { uiStore } from '$lib/stores/ui.svelte';
  import { profileStore } from '$lib/stores/profiles.svelte';
  import {
    listProfiles,
    getProfile,
    createProfile,
    updateProfile,
    deleteProfile,
    testConnection,
    type ProfileSummary,
    type ProfileInput,
    type Provider,
  } from '$lib/api/profiles';
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Separator } from '$lib/components/ui/separator';
  import { Plug, Trash2, Plus, Pencil, LoaderCircle, Check, X } from '@lucide/svelte';

  // State
  let profiles = $state<ProfileSummary[]>([]);
  let loading = $state(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);
  let saving = $state(false);
  let showForm = $state(false);
  let editingId = $state<string | null>(null);

  // Form fields
  let form = $state<ProfileInput>({
    name: '',
    provider: 'aws',
    endpoint: null,
    region: 'us-east-1',
    access_key_id: '',
    secret_access_key: '',
    path_style: false,
    default_bucket: null,
  });

  const providerColors: Record<Provider, string> = {
    aws: 'bg-amber-500',
    minio: 'bg-red-500',
    r2: 'bg-orange-500',
    custom: 'bg-blue-500',
  };

  const isEditing = $derived(editingId !== null);

  async function loadProfileList() {
    loading = true;
    try {
      profiles = await listProfiles();
    } finally {
      loading = false;
    }
  }

  async function handleConnect(profile: ProfileSummary) {
    await profileStore.openTab(profile);
    uiStore.profileManagerOpen = false;
  }

  async function handleDelete(id: string) {
    await deleteProfile(id);
    await profileStore.loadProfiles();
    await loadProfileList();
  }

  async function handleEdit(id: string) {
    const detail = await getProfile(id);
    editingId = id;
    form = {
      name: detail.name,
      provider: detail.provider,
      endpoint: detail.endpoint,
      region: detail.region,
      access_key_id: detail.access_key_id,
      secret_access_key: detail.secret_access_key,
      path_style: detail.path_style,
      default_bucket: detail.default_bucket,
    };
    testResult = null;
    showForm = true;
  }

  function handleProviderChange(provider: Provider) {
    form.provider = provider;
    if (provider === 'minio') {
      form.path_style = true;
      form.endpoint = 'http://localhost:9000';
      form.region = 'us-east-1';
    } else if (provider === 'aws') {
      form.path_style = false;
      form.endpoint = null;
      form.region = 'us-east-1';
    } else if (provider === 'r2') {
      form.path_style = false;
      form.endpoint = null;
      form.region = 'auto';
    } else {
      form.path_style = false;
      form.endpoint = '';
    }
  }

  async function handleSave() {
    saving = true;
    testResult = null;
    try {
      let id: string;

      if (editingId) {
        await updateProfile(editingId, form);
        id = editingId;
      } else {
        id = await createProfile(form);
      }

      const result = await testConnection(id);
      if (result.success || form.default_bucket) {
        testResult = {
          success: true,
          message: result.success
            ? result.message
            : `Saved with default bucket: ${form.default_bucket}`,
        };
        await profileStore.loadProfiles();
        const all = await listProfiles();
        const saved = all.find((p) => p.id === id);
        if (saved) {
          // Refresh if already open, otherwise open new tab
          const existingTab = profileStore.tabs.find((t) => t.profileId === id);
          if (existingTab) {
            await profileStore.refreshBuckets(id);
          } else {
            await profileStore.openTab(saved);
          }
          uiStore.profileManagerOpen = false;
        }
        await loadProfileList();
        resetForm();
      } else {
        testResult = {
          success: false,
          message:
            result.message +
            '\n\nTip: If your token is scoped to a specific bucket, fill in "Default Bucket" below.',
        };
      }
    } catch (e: any) {
      testResult = { success: false, message: e?.message ?? String(e) };
    } finally {
      saving = false;
    }
  }

  function resetForm() {
    form = {
      name: '',
      provider: 'aws',
      endpoint: null,
      region: 'us-east-1',
      access_key_id: '',
      secret_access_key: '',
      path_style: false,
      default_bucket: null,
    };
    testResult = null;
    showForm = false;
    editingId = null;
  }

  $effect(() => {
    if (uiStore.profileManagerOpen) {
      loadProfileList();
    }
  });

  const needsEndpoint = $derived(
    form.provider === 'minio' || form.provider === 'r2' || form.provider === 'custom',
  );
</script>

<Dialog bind:open={uiStore.profileManagerOpen}>
  <DialogContent class="max-w-lg">
    <DialogHeader>
      <DialogTitle>Profile Manager</DialogTitle>
    </DialogHeader>

    <!-- Profile list -->
    {#if !showForm}
      <div class="max-h-60 overflow-y-auto space-y-1">
        {#if loading}
          <div class="flex items-center justify-center py-4 text-muted-foreground text-sm">
            <LoaderCircle class="h-4 w-4 animate-spin mr-2" />
            Loading...
          </div>
        {:else if profiles.length === 0}
          <div class="py-4 text-center text-sm text-muted-foreground">
            No profiles yet. Create one below.
          </div>
        {:else}
          {#each profiles as profile}
            <div class="flex items-center gap-2 rounded-md border border-border px-3 py-2">
              <div class="h-2.5 w-2.5 rounded-full {providerColors[profile.provider]}"></div>
              <span class="flex-1 text-sm font-medium">{profile.name}</span>
              <span class="text-xs text-muted-foreground uppercase">{profile.provider}</span>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7"
                onclick={() => handleConnect(profile)}
                title="Connect"
              >
                <Plug class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7"
                onclick={() => handleEdit(profile.id)}
                title="Edit"
              >
                <Pencil class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 text-destructive hover:text-destructive"
                onclick={() => handleDelete(profile.id)}
                title="Delete"
              >
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
            </div>
          {/each}
        {/if}
      </div>

      <Separator />

      <Button variant="outline" class="w-full gap-2" onclick={() => (showForm = true)}>
        <Plus class="h-4 w-4" />
        New Profile
      </Button>
    {:else}
      <!-- Profile form (create or edit) -->
      <div class="space-y-3">
        <p class="text-sm font-medium">
          {isEditing ? 'Edit Profile' : 'New Profile'}
        </p>

        <!-- Name -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-name">Name</label>
          <Input id="profile-name" bind:value={form.name} placeholder="My S3 Profile" />
        </div>

        <!-- Provider -->
        <div class="space-y-1">
          <span class="text-xs text-muted-foreground">Provider</span>
          <div class="flex gap-2">
            {#each ['aws', 'minio', 'r2', 'custom'] as provider}
              <button
                class="flex items-center gap-1.5 rounded border px-2.5 py-1 text-xs transition-colors
                  {form.provider === provider
                  ? 'border-primary bg-primary/10 text-primary'
                  : 'border-border text-muted-foreground hover:border-primary/50'}"
                onclick={() => handleProviderChange(provider as Provider)}
              >
                <span class="h-2 w-2 rounded-full {providerColors[provider as Provider]}"></span>
                {provider}
              </button>
            {/each}
          </div>
        </div>

        <!-- Endpoint (conditional) -->
        {#if needsEndpoint}
          <div class="space-y-1">
            <label class="text-xs text-muted-foreground" for="profile-endpoint">Endpoint URL</label>
            <Input
              id="profile-endpoint"
              bind:value={form.endpoint as string}
              placeholder="http://localhost:9000"
            />
          </div>
        {/if}

        <!-- Region -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-region">Region</label>
          <Input id="profile-region" bind:value={form.region} placeholder="us-east-1" />
        </div>

        <!-- Access Key -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-access-key">Access Key ID</label
          >
          <Input
            id="profile-access-key"
            bind:value={form.access_key_id}
            placeholder="AKIAIOSFODNN7EXAMPLE"
          />
        </div>

        <!-- Secret Key -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-secret">Secret Access Key</label
          >
          <Input
            id="profile-secret"
            type="password"
            bind:value={form.secret_access_key}
            placeholder="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
          />
        </div>

        <!-- Path Style -->
        <div class="flex items-center gap-2">
          <Checkbox
            id="path-style"
            checked={form.path_style}
            onCheckedChange={(v) => (form.path_style = !!v)}
          />
          <label class="text-sm" for="path-style">Force path-style URLs</label>
        </div>

        <!-- Default Bucket (optional) -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-default-bucket">
            Default Bucket <span class="text-zinc-500">(optional — for bucket-scoped tokens)</span>
          </label>
          <Input
            id="profile-default-bucket"
            value={form.default_bucket ?? ''}
            oninput={(e) => {
              const v = (e.target as HTMLInputElement).value;
              form.default_bucket = v || null;
            }}
            placeholder="my-bucket"
          />
        </div>

        <!-- Test result -->
        {#if testResult}
          <div
            class="flex items-start gap-2 rounded-md px-3 py-2 text-sm whitespace-pre-wrap
              {testResult.success
              ? 'bg-green-500/10 text-green-700 dark:text-green-400'
              : 'bg-destructive/10 text-destructive'}"
          >
            {#if testResult.success}
              <Check class="h-4 w-4 shrink-0 mt-0.5" />
            {:else}
              <X class="h-4 w-4 shrink-0 mt-0.5" />
            {/if}
            {testResult.message}
          </div>
        {/if}

        <!-- Actions -->
        <div class="flex gap-2 justify-end">
          <Button variant="ghost" onclick={resetForm} disabled={saving}>Cancel</Button>
          <Button onclick={handleSave} disabled={saving || !form.name}>
            {#if saving}
              <LoaderCircle class="h-4 w-4 animate-spin mr-2" />
              Testing...
            {:else}
              {isEditing ? 'Update & Test' : 'Test & Save'}
            {/if}
          </Button>
        </div>
      </div>
    {/if}
  </DialogContent>
</Dialog>
