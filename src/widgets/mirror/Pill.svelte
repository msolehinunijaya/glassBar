<script lang="ts">
  // A round tile: the camera icon with "Mirror" under it. Click for the live mirror.
  import { Video } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { MirrorSettings } from './mirror';

  let { size, vertical }: PillProps<MirrorSettings> = $props();
</script>

{#if vertical}
  <div class="side" title="Mirror"><Video size={18} class="dim" /><span class="side-c">Mirror</span></div>
{:else}
  <div class="pill mirror" class:small={size === 'small'} title="Mirror">
    <Video size={size === 'small' ? 16 : 15} />
    {#if size !== 'small'}<span>Mirror</span>{/if}
  </div>
{/if}

<style>
  .mirror {
    flex-direction: column; justify-content: center; gap: 1px;
    aspect-ratio: 1; padding: 0; border-radius: 50%; color: var(--fg-dim);
  }
  .mirror:hover { color: var(--fg); }
  .mirror span { font-size: 9px; font-weight: 600; line-height: 1; }
</style>
