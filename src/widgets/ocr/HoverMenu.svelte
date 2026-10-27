<script lang="ts">
  // Quick menu shown while hovering a Capture button: the screenshot kinds, the recorders,
  // OCR Snap with its mode, or the last 5 copies. A click acts at once; the full panel is
  // still one click on the button away.
  import { AppWindow, Check, Monitor, Pin, ScanText, ScrollText, SquareDashed } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type ClipItem } from '../../lib/ipc';
  import { relTime } from '../../lib/time';
  import type { CaptureTab, OcrSettings } from './index';
  import { hotkeyLabel } from './hotkey';

  let { tab, instance, settings }: { tab: CaptureTab; instance: string; settings: OcrSettings } = $props();

  // The menu closes first so it isn't in the picture.
  const after = (f: () => unknown) => { ipc.popupHide(); setTimeout(f, 180); };
  const shots = [
    ['full', 'Full screen', Monitor], ['region', 'Area', SquareDashed],
    ['window', 'Window', AppWindow], ['scroll', 'Scrolling', ScrollText],
  ] as const;
  const take = (k: (typeof shots)[number][0]) => after(() => (k === 'full' ? ipc.shotFullscreen() : ipc.ocrSnapStart(k)).catch(() => {}));
  const recs = [['area', 'Area or screen', SquareDashed], ['app', 'Current app', AppWindow]] as const;
  const modes = [['text', 'Text'], ['code', 'Code'], ['terminal', 'Terminal']] as const;

  let clips = $state<ClipItem[] | null>(null);
  let copied = $state<number | null>(null);
  $effect(() => { if (tab === 'clip') ipc.clipList(null).then((l) => (clips = l.slice(0, 5))).catch(() => (clips = [])); });
  async function copy(c: ClipItem) {
    await ipc.clipCopy(c.id).catch(() => {});
    copied = c.id;
    setTimeout(() => ipc.popupHide(), 500);
  }
  const titles: Record<CaptureTab, string> = { shot: 'Screenshot', record: 'Screen recording', ocr: 'OCR Snap', clip: 'Pinned and recent copies' };
</script>

<div class="menu">
  <h4 class="dim">{titles[tab]}</h4>
  {#if tab === 'shot'}
    <div class="grid four">
      {#each shots as [k, label, Icon] (k)}
        <button class="opt" onclick={() => take(k)}><Icon size={18} /><span>{label}</span></button>
      {/each}
    </div>
  {:else if tab === 'record'}
    <div class="grid two">
      {#each recs as [k, label, Icon] (k)}
        <button class="opt rec" onclick={() => after(() => ipc.recordScreen(k).catch(() => {}))}><Icon size={18} /><span>{label}</span></button>
      {/each}
    </div>
  {:else if tab === 'ocr'}
    <button class="go" onclick={() => after(() => ipc.ocrSnapStart().catch(() => {}))}><ScanText size={16} /> Snap text <span class="k">{hotkeyLabel(settings.hotkey)}</span></button>
    <div class="seg">
      {#each modes as [m, label] (m)}
        <button class:sel={settings.mode === m} onclick={() => updateWidgetSettings(instance, { mode: m })}>{label}</button>
      {/each}
    </div>
  {:else}
    <div class="clips">
      {#each clips ?? [] as c (c.id)}
        <button class="clip" onclick={() => copy(c)} title="Copy again">
          {#if c.kind === 'image' && c.thumb}<img src={c.thumb} alt="Copied image" />{:else}<span class="txt">{c.text}</span>{/if}
          <span class="dim when">{#if copied === c.id}<Check size={12} /> Copied{:else}{#if c.pinned}<Pin size={11} class="pinned" />{/if}{relTime(c.at)}{/if}</span>
        </button>
      {:else}
        <p class="dim empty">{clips ? (settings.clipHistory ? 'Copy something and it shows up here.' : 'Clipboard history is off.') : ''}</p>
      {/each}
    </div>
  {/if}
</div>

<style>
  .menu { width: 100%; }
  h4 { margin: 0 2px 8px; font-size: 12px; font-weight: 500; }
  .grid { display: grid; gap: 6px; }
  .four { grid-template-columns: repeat(4, 1fr); }
  .two { grid-template-columns: repeat(2, 1fr); }
  .opt { display: flex; flex-direction: column; align-items: center; gap: 5px; padding: 10px 4px 8px; border-radius: calc(12px * var(--rs));
         background: var(--pill); font-size: 11px; font-weight: 500; transition: background 0.15s, transform 0.2s var(--spring); }
  .opt:hover { background: var(--pill-hover); }
  .opt:active { transform: scale(0.95); }
  .rec :global(svg) { color: #ff453a; }
  .go { display: flex; align-items: center; justify-content: center; gap: 8px; width: 100%; padding: 9px; border-radius: calc(12px * var(--rs));
        background: var(--accent); color: #fff; font-weight: 600; font-size: 13px; }
  .go .k { opacity: 0.75; font-size: 11px; font-weight: 500; }
  .seg { display: flex; gap: 2px; margin-top: 8px; padding: 3px; border-radius: calc(10px * var(--rs)); background: var(--pill); }
  .seg button { flex: 1; padding: 5px; border-radius: calc(8px * var(--rs)); font-size: 12px; }
  .seg .sel { background: var(--pill-hover); font-weight: 600; }
  .clips { display: flex; flex-direction: column; gap: 4px; }
  .clip { display: flex; align-items: center; gap: 8px; width: 100%; text-align: left; padding: 7px 10px; border-radius: calc(10px * var(--rs)); background: var(--pill); }
  .clip:hover { background: var(--pill-hover); }
  .txt { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; }
  .clip img { flex: 1; min-width: 0; max-height: 40px; object-fit: contain; object-position: left; border-radius: calc(4px * var(--rs)); }
  .when :global(.pinned) { color: var(--accent); }
  .when { flex: none; display: flex; align-items: center; gap: 3px; font-size: 10px; }
  .empty { text-align: center; padding: 14px 0; font-size: 12px; margin: 0; }
</style>
