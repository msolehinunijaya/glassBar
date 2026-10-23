<script lang="ts">
  // The card that pops up by the dock right after a screenshot or OCR snap: what was
  // captured, that it's on the clipboard, and quick actions. Closes by itself after a few
  // seconds unless the pointer is on it.
  import { Check, ExternalLink, ScanText, Trash2, TriangleAlert } from '@lucide/svelte';
  import { ipc, type CaptureDone } from '../../lib/ipc';

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
  .pic { width: 100%; max-height: 140px; object-fit: cover; object-position: top; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.25); }
  .row { display: flex; align-items: center; gap: 9px; }
  .ok { display: grid; place-items: center; width: 24px; height: 24px; border-radius: 12px; background: #30d158; color: #fff; flex: none; }
  .ok.bad { background: #ff9f0a; }
  .what { flex: 1; min-width: 0; display: flex; flex-direction: column; line-height: 1.25; }
  .what b { font-size: 13px; }
  .what .dim { font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ib { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 8px; }
  .ib:hover { background: var(--pill-hover); }
  .txt { margin: 0; max-height: 64px; overflow: hidden; white-space: pre-wrap; word-break: break-word; font: 12px/1.4 'Cascadia Code', Consolas, monospace;
         padding: 8px 10px; border-radius: 10px; background: var(--pill); mask-image: linear-gradient(#000 70%, transparent); }
</style>
