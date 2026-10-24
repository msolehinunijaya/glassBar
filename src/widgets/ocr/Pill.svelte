<script lang="ts">
  // Four buttons on the dock: Screenshot, Recording, OCR Snap, Clipboard. Each opens the
  // panel on its tab. Drag one sideways (or right-click › Move) to change their order.
  import { Camera, ClipboardList, ScanText, Video } from '@lucide/svelte';
  import { flip } from 'svelte/animate';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import { insertIndex } from '../../lib/geometry';
  import { showMenu } from '../../lib/menu';
  import type { PillProps } from '../types';
  import { moveKey } from '../apps/groups';
  import type { OcrSettings, CaptureTab } from './index';
  import { hotkeyLabel } from './hotkey';
  import CapIcon from './CapIcon.svelte';

  let { instance, settings, vertical }: PillProps<OcrSettings> = $props();
  // The hotkey actually registered (the default may have fallen back to Ctrl+Alt+T).
  let activeKey = $state('');
  $effect(() => { settings.hotkey; ipc.ocrHotkeyStatus().then((s) => (activeKey = s.hotkey)).catch(() => {}); });
  const ALL: CaptureTab[] = ['shot', 'record', 'ocr', 'clip'];
  const STYLES = [['tile', 'Colour'], ['plain', 'Simple'], ['outline', 'Outline']] as const;
  const meta = $derived({
    shot: { icon: Camera, label: 'Screenshot' },
    record: { icon: Video, label: 'Screen recording' },
    ocr: { icon: ScanText, label: `OCR Snap (${hotkeyLabel(activeKey || settings.hotkey)})` },
    clip: { icon: ClipboardList, label: 'Clipboard history' },
  } as const);
  // Saved order, plus any tab the saved order doesn't know yet.
  const saved = $derived([...(settings.icons ?? []).filter((k) => ALL.includes(k)), ...ALL.filter((k) => !(settings.icons ?? []).includes(k))]);
  let live = $state<CaptureTab[] | null>(null);
  const order = $derived(live ?? saved);

  let row = $state<HTMLElement>();
  let dragging = $state<CaptureTab | null>(null);
  let swallow = false;
  function down(e: PointerEvent, k: CaptureTab) {
    if (e.button !== 0) return;
    e.stopPropagation(); // no press-and-hold edit mode from these buttons
    const axis = (ev: PointerEvent) => (vertical ? ev.clientY : ev.clientX);
    const start = axis(e);
    let picked = false;
    const move = (ev: PointerEvent) => {
      if (!picked && Math.abs(axis(ev) - start) > 6) { picked = true; dragging = k; live = [...order]; }
      if (!picked || !live || !row) return;
      const others = [...row.querySelectorAll<HTMLElement>('.ic:not(.drag)')].map((b) => { const r = b.getBoundingClientRect(); return vertical ? r.top + r.height / 2 : r.left + r.width / 2; });
      const next = moveKey(live, k, insertIndex(others, axis(ev))) as CaptureTab[];
      if (next.join() !== live.join()) live = next;
    };
    const up = () => {
      removeEventListener('pointermove', move); removeEventListener('pointerup', up);
      if (!picked) return;
      swallow = true;
      if (live) updateWidgetSettings(instance, { icons: live });
      dragging = null; live = null;
    };
    addEventListener('pointermove', move); addEventListener('pointerup', up);
  }
  function click(e: MouseEvent, k: CaptureTab) {
    if (swallow) { swallow = false; e.stopPropagation(); return; } // the click that ended a drag
    updateWidgetSettings(instance, { tab: k }); // the click goes on to open the panel on this tab
  }
  async function menu(e: MouseEvent, k: CaptureTab) {
    e.preventDefault(); e.stopPropagation();
    const i = order.indexOf(k);
    await showMenu([
      ...(i > 0 ? [{ text: vertical ? 'Move up' : 'Move left', action: () => updateWidgetSettings(instance, { icons: moveKey(order, k, i - 1) }) }] : []),
      ...(i < order.length - 1 ? [{ text: vertical ? 'Move down' : 'Move right', action: () => updateWidgetSettings(instance, { icons: moveKey(order, k, i + 1) }) }] : []),
      { text: 'Icon style', items: STYLES.map(([v, text]) => ({ text, checked: (settings.iconStyle ?? 'tile') === v, action: () => updateWidgetSettings(instance, { iconStyle: v }) })) },
    ]);
  }
</script>

<div class="pill capture-pill" class:vertical bind:this={row}>
  {#each order as k (k)}
    {@const M = meta[k].icon}
    <button class="ic" class:drag={dragging === k} class:sel={settings.tab === k} title={meta[k].label} aria-label={meta[k].label}
            animate:flip={{ duration: 200 }} onpointerdown={(e) => down(e, k)} onclick={(e) => click(e, k)} oncontextmenu={(e) => menu(e, k)}>
      <CapIcon {k} icon={M} look={settings.iconStyle} size={22} />
    </button>
  {/each}
</div>

<style>
  .capture-pill { gap: 1px; padding: 0 4px; }
  .capture-pill.vertical { flex-direction: column; padding: 4px 0; }
  .ic { display: grid; place-items: center; width: 30px; height: 30px; border-radius: calc(9px * var(--rs)); transition: background 0.15s, transform 0.25s var(--spring); }
  .ic:hover { background: var(--pill-hover); }
  .ic:active { transform: scale(0.88); }
  .ic.drag { transform: scale(1.18); background: var(--pill-hover); box-shadow: 0 4px 12px rgba(0,0,0,0.25); z-index: 1; }
</style>
