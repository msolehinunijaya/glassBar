<script lang="ts">
  import { Copy, ScanText, Trash2 } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type OcrItem } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { relTime } from '../../lib/time';
  import type { OcrSettings } from './index';
  import { hotkeyLabel } from './hotkey';

  let { instance, settings }: { instance: string; settings: OcrSettings } = $props();
  const hist = live<OcrItem[]>('ocr', ipc.ocrHistory, []);
  let hotkeyError = $state<string | null>(null);
  let activeKey = $state('');
  let copied = $state<number | null>(null);
  $effect(() => { ipc.ocrHotkeyStatus().then((s) => { hotkeyError = s.error; activeKey = s.hotkey; }); });
  const modes = [['text', 'Text'], ['code', 'Code'], ['terminal', 'Terminal']] as const;

  async function snap() {
    await ipc.popupHide();
    setTimeout(() => ipc.ocrSnapStart(), 150); // let the panel fade out of the screenshot
  }
  async function copy(i: OcrItem) {
    await ipc.ocrCopy(i.text);
    copied = i.id;
    setTimeout(() => (copied = null), 1200);
  }
</script>

<button class="go" onclick={snap}><ScanText size={16} /> Snap text <span class="k">{hotkeyLabel(activeKey || settings.hotkey)}</span></button>
{#if hotkeyError}<p class="err small">{hotkeyError}. Pick another in the widget settings.</p>{/if}

<div class="seg">
  {#each modes as [m, label] (m)}
    <button class:sel={settings.mode === m} onclick={() => updateWidgetSettings(instance, { mode: m })}>{label}</button>
  {/each}
</div>
<div class="seg two">
  <button class:sel={settings.engine === 'windows'} onclick={() => updateWidgetSettings(instance, { engine: 'windows' })}>Windows OCR</button>
  <button class:sel={settings.engine === 'ai'} onclick={() => updateWidgetSettings(instance, { engine: 'ai' })}>Claude (AI)</button>
</div>
{#if settings.engine === 'ai'}<p class="dim small">The selection is sent to Anthropic. Needs your API key (AI Usage widget settings).</p>{/if}

<div class="head"><h4 class="dim">History</h4>{#if hist.v.length}<button class="link" onclick={() => ipc.ocrHistoryDelete(null)}>Clear</button>{/if}</div>
<div class="list">
  {#each hist.v as i (i.id)}
    <div class="card item">
      <pre>{i.text}</pre>
      <div class="meta dim small">
        <span>{relTime(i.at)} · {i.mode} · {i.engine === 'ai' ? 'Claude' : 'Windows'}</span>
        <button onclick={() => copy(i)} aria-label="Copy"><Copy size={13} />{copied === i.id ? ' Copied' : ''}</button>
        <button onclick={() => ipc.ocrHistoryDelete(i.id)} aria-label="Delete"><Trash2 size={13} /></button>
      </div>
    </div>
  {:else}
    <p class="dim small empty">Nothing captured yet.</p>
  {/each}
</div>

<style>
  .go { display: flex; align-items: center; justify-content: center; gap: 8px; width: 100%; padding: 10px; border-radius: 12px; background: var(--accent); color: #fff; font-weight: 600; }
  .k { font-size: 11px; font-weight: 500; opacity: 0.8; padding: 1px 6px; border-radius: 6px; background: rgba(255,255,255,0.2); }
  .seg { display: grid; grid-template-columns: repeat(3, 1fr); gap: 2px; padding: 3px; border-radius: 10px; background: var(--pill); margin-top: 10px; }
  .seg.two { grid-template-columns: 1fr 1fr; margin-top: 6px; }
  .seg button { padding: 6px; border-radius: 8px; font-size: 12px; }
  .seg .sel { background: var(--accent); color: #fff; }
  .head { display: flex; justify-content: space-between; align-items: center; margin: 12px 2px 6px; }
  h4 { margin: 0; font-size: 12px; font-weight: 500; }
  .link { color: var(--accent); font-size: 12px; }
  .list { display: flex; flex-direction: column; gap: 6px; }
  .item { padding: 8px 10px; }
  pre { margin: 0; max-height: 72px; overflow: hidden; white-space: pre-wrap; word-break: break-word; font: 12px/1.4 'Cascadia Code', Consolas, monospace; }
  .meta { display: flex; align-items: center; gap: 8px; margin-top: 6px; }
  .meta span { flex: 1; }
  .meta button { display: inline-flex; align-items: center; opacity: 0.7; }
  .meta button:hover { opacity: 1; }
  .small { font-size: 11px; }
  .err { color: #ff453a; margin: 6px 0 0; }
  .empty { text-align: center; padding: 20px 0; }
</style>
