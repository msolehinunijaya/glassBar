<script lang="ts">
  // Colour picker tab: Pick (freezes the screen, a loupe follows the pointer), what gets
  // copied (HEX / RGB / HSL), and the last 16 colours (click = copy, right-click = remove).
  import { Check, Pipette } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import { showMenu } from '../../lib/menu';
  import { formatColor, textOn, type ColorFormat } from './color';
  import { hotkeyLabel } from './hotkey';
  import type { OcrSettings } from './index';

  let { instance, settings }: { instance: string; settings: OcrSettings } = $props();
  let key = $state('');
  $effect(() => { settings.colorHotkey; ipc.colorHotkeyStatus().then((s) => (key = s.hotkey)).catch(() => {}); });
  const fmt = $derived(settings.colorFormat ?? 'hex');
  const colors = $derived(settings.colors ?? []);
  let copied = $state<string | null>(null);
  const FORMATS: [ColorFormat, string][] = [['hex', 'HEX'], ['rgb', 'RGB'], ['hsl', 'HSL']];

  // The panel closes first so it isn't in the picture.
  const pick = () => { ipc.popupHide(); setTimeout(() => ipc.ocrSnapStart('color').catch(() => {}), 180); };
  async function copy(hex: string) {
    await ipc.ocrCopy(formatColor(hex, fmt)).catch(() => {});
    copied = hex;
    setTimeout(() => { if (copied === hex) copied = null; }, 1200);
  }
  const remove = (hex: string) => updateWidgetSettings(instance, { colors: colors.filter((c) => c !== hex) });
  async function menu(e: MouseEvent, hex: string) {
    e.preventDefault();
    await showMenu([
      ...FORMATS.map(([f, label]) => ({ text: `Copy ${label}  ${formatColor(hex, f)}`, action: () => ipc.ocrCopy(formatColor(hex, f)) })),
      '-',
      { text: 'Remove', action: () => remove(hex) },
    ]);
  }
</script>

<button class="go" onclick={pick}><Pipette size={16} /> Pick a colour <span class="k">{hotkeyLabel(key || settings.colorHotkey || 'Super+Shift+C')}</span></button>

<div class="row">
  <span class="dim lbl">Copy as</span>
  <div class="seg" role="radiogroup" aria-label="Colour format">
    {#each FORMATS as [f, label] (f)}
      <button role="radio" aria-checked={fmt === f} class:sel={fmt === f} onclick={() => updateWidgetSettings(instance, { colorFormat: f })}>{label}</button>
    {/each}
  </div>
</div>

<div class="hist">
  <div class="head"><h4 class="dim">Recent colours</h4>{#if colors.length}<button class="link" onclick={() => updateWidgetSettings(instance, { colors: [] })}>Clear</button>{/if}</div>
  {#if colors.length}
    <div class="sw">
      {#each colors as c (c)}
        <button class="chip" style:background={c} style:color={textOn(c)} title="{formatColor(c, fmt)} · click to copy, right-click for more"
                onclick={() => copy(c)} oncontextmenu={(e) => menu(e, c)} aria-label="Copy {formatColor(c, fmt)}">
          {#if copied === c}<Check size={14} strokeWidth={3} />{/if}
        </button>
      {/each}
    </div>
    {#if copied}<p class="dim small">Copied {formatColor(copied, fmt)}</p>{/if}
  {:else}
    <p class="dim small empty">Colours you pick show up here.</p>
  {/if}
</div>

<style>
  .go { display: flex; align-items: center; justify-content: center; gap: 8px; width: 100%; height: 36px; border-radius: var(--r-inner);
        background: var(--accent); color: var(--on-accent); font-weight: 600; font-size: 13px; transition: transform 0.15s var(--ease); }
  .go:active { transform: scale(0.98); }
  .go .k { opacity: 0.75; font-size: 11px; font-weight: 500; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin: 12px 2px 0; }
  .lbl { font-size: 12px; }
  .seg { display: flex; gap: 2px; padding: 3px; border-radius: var(--r-inner); background: var(--pill); }
  .seg button { padding: 5px 12px; border-radius: calc(8px * var(--rs)); font-size: 12px; }
  .seg .sel { background: var(--accent); color: var(--on-accent); font-weight: 600; }
  .hist { margin-top: var(--s-4); padding-top: var(--s-3); border-top: 1px solid var(--border); }
  .head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
  h4 { margin: 0 2px; font-size: 12px; font-weight: 500; }
  .link { color: var(--accent); font-size: 12px; }
  .sw { display: grid; grid-template-columns: repeat(8, 1fr); gap: 6px; }
  .chip { display: grid; place-items: center; aspect-ratio: 1; border-radius: var(--r-inner); box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.12);
          transition: transform 0.2s var(--spring); }
  .chip:hover { transform: scale(1.08); }
  .chip:active { transform: scale(0.94); }
  .chip:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .small { font-size: 12px; margin: 8px 2px 0; font-variant-numeric: tabular-nums; }
  .empty { text-align: center; padding: 8px 0; }
</style>
