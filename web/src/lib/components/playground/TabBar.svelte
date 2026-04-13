<script lang="ts">
  import { store } from './store.svelte';
  import { X, Plus } from '@lucide/svelte';

  const providerColors: Record<string, string> = {
    aws: 'bg-amber-500',
    minio: 'bg-red-500',
    r2: 'bg-orange-500',
    custom: 'bg-blue-500',
  };
</script>

<div class="flex h-8 items-end border-b border-border bg-muted/30 shrink-0">
  <div class="flex flex-1 items-end overflow-x-auto">
    {#each store.profiles as profile (profile.id)}
      {@const active = store.activeProfileId === profile.id}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="group flex h-8 min-w-0 max-w-48 cursor-pointer items-center gap-1.5 border-r border-border px-3 text-xs transition-colors select-none
          {active ? 'bg-background text-foreground' : 'bg-muted/50 text-muted-foreground hover:bg-muted hover:text-foreground'}"
        onclick={() => store.switchProfile(profile.id)}
      >
        <div class="h-2 w-2 shrink-0 rounded-full {providerColors[profile.provider] ?? 'bg-zinc-500'}"></div>
        <span class="truncate" title={profile.name}>{profile.name}</span>
        {#if store.profiles.length > 1}
          <button
            class="ml-auto shrink-0 rounded p-0.5 transition-colors hover:bg-accent hover:text-accent-foreground
              {active ? 'opacity-60' : 'opacity-0 group-hover:opacity-100'}"
            onclick={(e) => { e.stopPropagation(); store.disconnectProfile(profile.id); }}
            title="Close"
          >
            <X class="h-3 w-3" />
          </button>
        {/if}
      </div>
    {/each}
  </div>

  <button
    class="flex h-8 w-8 shrink-0 items-center justify-center text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
    onclick={() => (store.profileManagerOpen = true)}
    title="Add Profile"
  >
    <Plus class="h-3.5 w-3.5" />
  </button>
</div>
