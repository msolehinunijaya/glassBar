<script lang="ts">
  // Capture: Screenshot | Screen Recording | OCR Snap | Clipboard. Opens on the tab used last.
  // On a bottom dock the panel is upside down, so what you click most is nearest the dock:
  // the tab buttons at the bottom, the tab's options above them, its history on top.
  import { Camera, ClipboardList, ScanText, Video } from '@lucide/svelte';
  import ClipTab from './ClipTab.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { OcrSettings } from './index';
  import OcrTab from './OcrTab.svelte';
  import RecordTab from './RecordTab.svelte';
  import ShotsTab from './ShotsTab.svelte';
  import { profile } from '../../lib/config.svelte';
  import CapIcon from './CapIcon.svelte';

  let { instance, settings }: PopupProps<OcrSettings> = $props();
  const up = $derived(profile().bar.edge === 'bottom');
  let root = $state<HTMLElement>();
  // Upside down and taller than the screen allows: start scrolled to the bottom (the buttons).
  $effect(() => {
    void settings.tab;
    if (!up || !root) return;
    const toEnd = () => { const s = root?.closest('.popup'); if (s) s.scrollTop = s.scrollHeight; };
    requestAnimationFrame(toEnd);
    const t = setTimeout(toEnd, 300); // history arrives a moment later
    return () => clearTimeout(t);
  });
  const tabs = [['shot', 'Screenshot', Camera], ['record', 'Recording', Video], ['ocr', 'OCR', ScanText], ['clip', 'Clipboard', ClipboardList]] as const;
</script>

<div class="cap" class:up bind:this={root}>
<div class="tabs">
  {#each tabs as [k, label, Icon] (k)}
    <button class:sel={settings.tab === k} onclick={() => updateWidgetSettings(instance, { tab: k })}><CapIcon {k} icon={Icon} look={settings.iconStyle} size={22} />{label}</button>
  {/each}
</div>
<div class="body">
{#if settings.tab === 'record'}<RecordTab />
{:else if settings.tab === 'ocr'}<OcrTab {instance} {settings} />
{:else if settings.tab === 'clip'}<ClipTab {instance} {settings} />
{:else}<ShotsTab {instance} {settings} />{/if}
</div>
</div>

<style>
  .tabs { display: grid; grid-template-columns: repeat(4, 1fr); gap: 2px; padding: 3px; border-radius: calc(12px * var(--rs)); background: var(--pill); margin-bottom: 12px; }
  /* Colour tile above the label: four tabs fit side by side without crowding. */
  .tabs button { min-width: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px; padding: 7px 2px 6px; border-radius: calc(9px * var(--rs)); font-size: 11px; font-weight: 500; }
  .tabs .sel { background: var(--accent); color: #fff; }
  /* Bottom dock: tab buttons last, the tab's options above them, its history on top. */
  .up { display: flex; flex-direction: column-reverse; }
  .up .tabs { margin: 12px 0 0; }
  .up .body { display: flex; flex-direction: column-reverse; gap: 12px; }
  .up :global(.hist > .head:first-child) { margin-top: 0; }
  /* ...and its history newest-last, so the newest item sits right above the options. */
  .up :global(.hist .list) { flex-direction: column-reverse; }
  .up :global(.hist .grid) { display: flex; flex-wrap: wrap-reverse; gap: 8px; }
  .up :global(.hist .grid > *) { flex: 0 0 calc(50% - 4px); min-width: 0; }
  .up :global(.hist .grid > .empty) { flex-basis: 100%; }
</style>
