<script lang="ts">
  import type { Provider } from '$lib/api/profiles';
  import type { ConnectionStatus } from '$lib/stores/profiles.svelte';
  import { X } from '@lucide/svelte';

  interface Props {
    name: string;
    provider: Provider;
    active: boolean;
    status: ConnectionStatus;
    profileId: string;
    droppable: boolean;
    dragHover: boolean;
    onactivate: () => void;
    onclose: () => void;
  }

  const { name, provider, active, status, profileId, droppable, dragHover, onactivate, onclose }: Props = $props();

  const providerColors: Record<Provider, string> = {
    aws: 'bg-amber-500',
    minio: 'bg-red-500',
    r2: 'bg-orange-500',
    custom: 'bg-blue-500',
  };

  const statusDot = $derived(
    status === 'connected'
      ? providerColors[provider]
      : status === 'connecting'
        ? 'bg-yellow-400 animate-pulse'
        : status === 'error'
          ? 'bg-red-500'
          : 'bg-zinc-500',
  );

  const dragClass = $derived(
    dragHover && droppable
      ? 'ring-2 ring-primary ring-inset bg-primary/10'
      : dragHover && !droppable
        ? 'ring-2 ring-destructive ring-inset opacity-50'
        : '',
  );

  let hoverTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (dragHover && droppable && !active) {
      hoverTimer = setTimeout(() => {
        onactivate();
      }, 500);
    } else {
      clearTimeout(hoverTimer);
    }
    return () => clearTimeout(hoverTimer);
  });
</script>

<!-- Use a div to avoid nested-button HTML violation -->
<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  role="tab"
  aria-selected={active}
  data-tab-profile-id={profileId}
  class="group flex h-8 min-w-0 max-w-48 cursor-pointer items-center gap-1.5 border-r border-border
    px-3 text-xs transition-colors select-none
    {active
    ? 'bg-background text-foreground'
    : 'bg-muted/50 text-muted-foreground hover:bg-muted hover:text-foreground'}
    {dragClass}"
  onclick={onactivate}
  onkeydown={(e) => e.key === 'Enter' && onactivate()}
  title={name}
>
  <span class="h-2 w-2 shrink-0 rounded-full {statusDot}"></span>
  <span class="truncate">{name}</span>
  <button
    type="button"
    class="ml-1 shrink-0 rounded p-0.5 opacity-0 transition-opacity group-hover:opacity-100
      {active ? 'opacity-60' : ''} hover:bg-accent hover:text-accent-foreground"
    onclick={(e) => {
      e.stopPropagation();
      onclose();
    }}
    title="Close tab"
    tabindex="-1"
  >
    <X class="h-3 w-3" />
  </button>
</div>
