<script lang="ts">
  // Capture: Screenshot | Screen Recording | OCR Snap. Opens on the tab used last.
  import { Camera, ScanText, Video } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { OcrSettings } from './index';
  import OcrTab from './OcrTab.svelte';
  import RecordTab from './RecordTab.svelte';
  import ShotsTab from './ShotsTab.svelte';

  let { instance, settings }: PopupProps<OcrSettings> = $props();
  const tabs = [['shot', 'Screenshot', Camera], ['record', 'Recording', Video], ['ocr', 'OCR Snap', ScanText]] as const;
</script>

<div class="tabs">
  {#each tabs as [k, label, Icon] (k)}
    <button class:sel={settings.tab === k} onclick={() => updateWidgetSettings(instance, { tab: k })}><Icon size={14} /> {label}</button>
  {/each}
</div>
{#if settings.tab === 'record'}<RecordTab />
{:else if settings.tab === 'ocr'}<OcrTab {instance} {settings} />
{:else}<ShotsTab {instance} {settings} />{/if}

<style>
  .tabs { display: grid; grid-template-columns: repeat(3, 1fr); gap: 2px; padding: 3px; border-radius: 12px; background: var(--pill); margin-bottom: 12px; }
  .tabs button { display: flex; align-items: center; justify-content: center; gap: 5px; padding: 7px 4px; border-radius: 9px; font-size: 12px; font-weight: 500; }
  .tabs .sel { background: var(--accent); color: #fff; }
</style>
