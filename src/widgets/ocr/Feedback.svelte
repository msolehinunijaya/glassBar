<script lang="ts">
  // The card that pops up by the dock right after a screenshot or OCR snap: what was
  // captured, that it's on the clipboard, and quick actions. Closes by itself after a few
  // seconds unless the pointer is on it.
  import { Check, ExternalLink, ScanText, Trash2, TriangleAlert } from '@lucide/svelte';
  import { ipc, type CaptureDone } from '../../lib/ipc';
  import { formatColor, textOn, type ColorFormat } from './color';
  let chipCopied = $state<string | null>(null);
  const copyAs = (hex: string, f: ColorFormat) => ipc.ocrCopy(formatColor(hex, f)).then(() => (chipCopied = f)).catch(() => {});

  let { data, close }: { data: CaptureDone; close: () => void } = $props();
  let timer: ReturnType<typeof setTimeout>;
  const arm = () => { clearTimeout(timer); timer = setTimeout(close, data.kind === 'error' ? 6000 : 4000); };
  $effect(() => { data; arm(); return () => clearTimeout(timer); });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fb" class:err={data.kind === 'error'} onpointerenter={() => clearTimeout(timer)} onpointerleave={arm}>
  {#if data.kind === 'shot'}
    <img class="pic" src={data.thumb} alt="Screenshot preview" />
    <div class="row">
      <span class="ok"><Check size={13} strokeWidth={3} /></span>
      <span class="what"><b>Screenshot copied</b><span class="dim">{data.w} × {data.h}{data.shotKind === 'scroll' ? ' · whole page' : ''} · in history</span></span>
      <button class="ib" title="Open" onclick={() => { ipc.shotsOpen(data.kind === 'shot' ? data.id : null); close(); }}><ExternalLink size={14} /></button>
      <button class="ib" title="Discard" onclick={() => { if (data.kind === 'shot') ipc.shotsDelete(data.id); close(); }}><Trash2 size={14} /></button>
    </div>
  {:else if data.kind === 'ocr'}
    <div class="row">
      <span class="ok"><ScanText size={13} strokeWidth={2.6} /></span>
      <span class="what"><b>{data.copied ? 'Text copied' : 'Text captured'}</b><span class="dim">{data.lines} line{data.lines === 1 ? '' : 's'} · in OCR history</span></span>
    </div>
    <pre class="txt">{data.text}</pre>
  {:else if data.kind === 'color'}
    <div class="swatch" style:background={data.hex} style:color={textOn(data.hex)}><b>{data.hex}</b></div>
    <div class="row">
      <span class="ok"><Check size={13} strokeWidth={3} /></span>
      <span class="what"><b>Colour copied</b><span class="dim">{data.text}</span></span>
    </div>
    <div class="fmts">
      {#each [['hex', 'HEX'], ['rgb', 'RGB'], ['hsl', 'HSL']] as const as [f, label] (f)}
        <button class="fmt" class:sel={chipCopied === f} onclick={() => copyAs(data.kind === 'color' ? data.hex : '', f)} title="Copy {formatColor(data.hex, f)}">
          {#if chipCopied === f}<Check size={11} strokeWidth={3} />{/if}{label}
        </button>
      {/each}
    </div>
  {:else}
    <div class="row">
      <span class="ok bad"><TriangleAlert size={13} strokeWidth={2.6} /></span>
      <span class="what"><b>Couldn't capture</b><span class="dim">{data.text}</span></span>
    </div>
  {/if}
</div>

<style>
  .fb { display: flex; flex-direction: column; gap: 8px; animation: pop 0.32s var(--spring); }
  @keyframes pop { from { transform: translateY(8px) scale(0.96); opacity: 0; } }
  .pic { width: 100%; max-height: 140px; object-fit: cover; object-position: top; border-radius: calc(10px * var(--rs)); box-shadow: 0 2px 10px rgba(0,0,0,0.25); }
  .row { display: flex; align-items: center; gap: 9px; }
  .ok { display: grid; place-items: center; width: 24px; height: 24px; border-radius: calc(12px * var(--rs)); background: #30d158; color: #fff; flex: none; }
  .ok.bad { background: #ff9f0a; }
  .swatch { display: flex; align-items: flex-end; height: 64px; padding: 8px 10px; border-radius: calc(10px * var(--rs)); box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.12); }
  .swatch b { font-size: 15px; font-variant-numeric: tabular-nums; letter-spacing: 0.02em; }
  .fmts { display: flex; gap: 4px; }
  .fmt { flex: 1; display: inline-flex; align-items: center; justify-content: center; gap: 3px; height: 28px; border-radius: calc(8px * var(--rs)); background: var(--pill); font-size: 11.5px; font-weight: 600; }
  .fmt:hover { background: var(--pill-hover); }
  .fmt.sel { color: #30d158; }
  .what { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 1.25; }
  .what b { font-size: 13px; }
  .what .dim { font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ib { display: grid; place-items: center; width: 28px; height: 28px; border-radius: calc(8px * var(--rs)); }
  .ib:hover { background: var(--pill-hover); }
  .txt { margin: 0; max-height: 64px; overflow: hidden; white-space: pre-wrap; word-break: break-word; font: 12px/1.4 'Cascadia Code', Consolas, monospace;
         padding: 8px 10px; border-radius: calc(10px * var(--rs)); background: var(--pill); mask-image: linear-gradient(#000 70%, transparent); }
</style>
