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
    <button class:sel={settings.tab === k} onclick={() => updateWidgetSettings(instance, { tab: k })}><Icon size={14} /> {label}</button>
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
  .tabs { display: grid; grid-template-columns: repeat(4, 1fr); gap: 2px; padding: 3px; border-radius: 12px; background: var(--pill); margin-bottom: 12px; }
  .tabs button { display: flex; align-items: center; justify-content: center; gap: 4px; padding: 7px 2px; border-radius: 9px; font-size: 11.5px; font-weight: 500; }
  .tabs .sel { background: var(--accent); color: #fff; }
  /* Bottom dock: tab buttons last, the tab's options above them, its history on top. */
  .up { display: flex; flex-direction: column-reverse; }
  .up .tabs { margin: 12px 0 0; }
  .up .body { display: flex; flex-direction: column-reverse; gap: 12px; }
  .up :global(.hist > .head:first-child) { margin-top: 0; }
</style>
