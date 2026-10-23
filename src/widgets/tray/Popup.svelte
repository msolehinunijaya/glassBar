<script lang="ts">
  import type { PopupProps } from '../types';
  import type { TraySettings } from './index';
  import { liveTray, split } from './icons.svelte';
  import TrayIconButton from './TrayIconButton.svelte';
  import TrayButtons from './TrayButtons.svelte';

  let { settings }: PopupProps<TraySettings> = $props();
  const tray = liveTray();
  const parts = $derived(split(tray.v, settings));
</script>

{#if settings.native}
  <h3>Tray icons</h3>
  {#if settings.showOverflow}
    <div class="grid">
      {#each parts.overflow as i (i.id)}<TrayIconButton icon={i} size={20} />{/each}
    </div>
    {#if !parts.overflow.length}<p class="dim small">No more icons.</p>{/if}
  {/if}
  <p class="dim small">Click, double-click or right-click works as in the Windows tray. Icons appear as apps add or update them.</p>
{:else}
  <TrayButtons />
{/if}

<style>
  h3 { margin: 0 0 8px; font-size: 14px; font-weight: 600; }
  .grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: 2px; }
  .small { font-size: 11px; }
</style>
