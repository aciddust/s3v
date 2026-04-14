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
  import { Plug, Trash2, Plus, Pencil, LoaderCircle, Check, X, Info } from '@lucide/svelte';
  import * as m from '$lib/paraglide/messages';

  // State
  let profiles = $state<ProfileSummary[]>([]);
  let loading = $state(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);
  let saving = $state(false);
  let showForm = $state(false);
  let editingId = $state<string | null>(null);
  let pathStyleInfoOpen = $state(false);

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
    rustfs: 'bg-red-500',
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
    if (provider === 'rustfs') {
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
            : m.profile_saved_with_bucket({ bucket: form.default_bucket ?? '' }),
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
          message: result.message + '\n\n' + m.profile_tip_bucket(),
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
    form.provider === 'rustfs' || form.provider === 'r2' || form.provider === 'custom',
  );
</script>

<Dialog bind:open={uiStore.profileManagerOpen}>
  <DialogContent class="max-w-lg [&_*]:select-none [&_input]:select-text [&_textarea]:select-text">
    <DialogHeader>
      <DialogTitle>{m.profile_title()}</DialogTitle>
    </DialogHeader>

    <!-- Profile list -->
    {#if !showForm}
      <div class="max-h-60 overflow-y-auto space-y-1">
        {#if loading}
          <div class="flex items-center justify-center py-4 text-muted-foreground text-sm">
            <LoaderCircle class="h-4 w-4 animate-spin mr-2" />
            {m.profile_loading()}
          </div>
        {:else if profiles.length === 0}
          <div class="py-4 text-center text-sm text-muted-foreground">
            {m.profile_empty()}
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
                class="h-7 w-7 cursor-pointer"
                onclick={() => handleConnect(profile)}
                title={m.profile_connect()}
              >
                <Plug class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 cursor-pointer"
                onclick={() => handleEdit(profile.id)}
                title={m.profile_edit()}
              >
                <Pencil class="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 cursor-pointer text-destructive hover:text-destructive"
                onclick={() => handleDelete(profile.id)}
                title={m.profile_delete()}
              >
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
            </div>
          {/each}
        {/if}
      </div>

      <Separator />

      <Button
        variant="outline"
        class="w-full gap-2 cursor-pointer"
        onclick={() => (showForm = true)}
      >
        <Plus class="h-4 w-4" />
        {m.profile_new()}
      </Button>
    {:else}
      <!-- Profile form (create or edit) -->
      <div class="space-y-3">
        <p class="text-sm font-medium">
          {isEditing ? m.profile_edit_title() : m.profile_new_title()}
        </p>

        <!-- Name -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-name">{m.profile_name()}</label>
          <Input
            id="profile-name"
            bind:value={form.name}
            placeholder={m.profile_placeholder_name()}
          />
        </div>

        <!-- Provider -->
        <div class="space-y-1">
          <span class="text-xs text-muted-foreground">{m.profile_provider()}</span>
          <div class="flex gap-2">
            {#each ['aws', 'rustfs', 'r2', 'custom'] as provider}
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
            <label class="text-xs text-muted-foreground" for="profile-endpoint"
              >{m.profile_endpoint()}</label
            >
            <Input
              id="profile-endpoint"
              bind:value={form.endpoint as string}
              placeholder="http://localhost:9000"
            />
          </div>
        {/if}

        <!-- Region -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-region"
            >{m.profile_region()}</label
          >
          <Input id="profile-region" bind:value={form.region} placeholder="us-east-1" />
        </div>

        <!-- Access Key -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-access-key"
            >{m.profile_access_key()}</label
          >
          <Input
            id="profile-access-key"
            bind:value={form.access_key_id}
            placeholder="AKIAIOSFODNN7EXAMPLE"
          />
        </div>

        <!-- Secret Key -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-secret"
            >{m.profile_secret_key()}</label
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
          <label class="text-sm" for="path-style">{m.profile_path_style()}</label>
          <button
            type="button"
            class="cursor-pointer text-muted-foreground hover:text-foreground transition-colors"
            onclick={() => (pathStyleInfoOpen = true)}
            title={m.profile_path_style_info_title()}
          >
            <Info class="h-3.5 w-3.5" />
          </button>
        </div>

        <!-- Default Bucket (optional) -->
        <div class="space-y-1">
          <label class="text-xs text-muted-foreground" for="profile-default-bucket">
            {m.profile_default_bucket()}
            <span class="text-zinc-500">({m.profile_default_bucket_hint()})</span>
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
          <Button variant="ghost" class="cursor-pointer" onclick={resetForm} disabled={saving}
            >{m.profile_cancel()}</Button
          >
          <Button class="cursor-pointer" onclick={handleSave} disabled={saving || !form.name}>
            {#if saving}
              <LoaderCircle class="h-4 w-4 animate-spin mr-2" />
              {m.profile_testing()}
            {:else}
              {isEditing ? m.profile_update_test() : m.profile_test_save()}
            {/if}
          </Button>
        </div>
      </div>
    {/if}
  </DialogContent>
</Dialog>

<Dialog bind:open={pathStyleInfoOpen}>
  <DialogContent class="max-w-sm select-none">
    <DialogHeader>
      <DialogTitle>{m.profile_path_style_info_title()}</DialogTitle>
    </DialogHeader>
    <p class="text-sm text-muted-foreground whitespace-pre-line">
      {m.profile_path_style_info_body()}
    </p>
  </DialogContent>
</Dialog>
