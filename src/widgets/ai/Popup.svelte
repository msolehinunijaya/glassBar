<script lang="ts">
  import { Copy, Send } from '@lucide/svelte';
  import { ipc, type AiUsage } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PopupProps } from '../types';
  import type { AiSettings } from './index';
  import { tokens } from './tokens';

  let { settings }: PopupProps<AiSettings> = $props();
  const u = live<AiUsage | null>('ai', ipc.aiUsage, null);
  let hasKey = $state<boolean | null>(null);
  let q = $state('');
  let answer = $state('');
  let error = $state('');
  let busy = $state(false);
  $effect(() => { ipc.aiKeyStatus().then((k) => (hasKey = k)); });

  async function ask() {
    if (!q.trim() || busy) return;
    busy = true; error = ''; answer = '';
    try { answer = await ipc.aiAsk(q); } catch (e) { error = String(e); }
    busy = false;
  }
</script>

<h3>Ask Claude <span class="dim model">{settings.model}</span></h3>
{#if hasKey === false}
  <p class="card note">Add your Anthropic API key in Settings › Widgets › Ask Claude. It's stored in Windows Credential Manager, not in GlassBar's files.</p>
{:else}
  <div class="ask">
    <textarea rows="3" placeholder="Ask anything… (Ctrl+Enter to send)" bind:value={q}
              onkeydown={(e) => { if (e.key === 'Enter' && (e.ctrlKey || !e.shiftKey)) { e.preventDefault(); ask(); } }}></textarea>
    <button class="send" disabled={busy || !q.trim()} onclick={ask} aria-label="Send"><Send size={15} /></button>
  </div>
  {#if busy}<p class="dim small">Thinking…</p>{/if}
  {#if error}<p class="err small">{error}</p>{/if}
  {#if answer}
    <div class="card ans">
      <pre>{answer}</pre>
      <button class="cp" onclick={() => ipc.ocrCopy(answer)} aria-label="Copy answer"><Copy size={13} /></button>
    </div>
  {/if}
{/if}

<h4 class="dim">Usage (from Anthropic's reported token counts)</h4>
{#if u.v}
  <div class="grid">
    <div class="card s"><span class="dim">Today</span><b>{tokens(u.v.dayIn + u.v.dayOut)}</b><span class="dim small">{tokens(u.v.dayIn)} in · {tokens(u.v.dayOut)} out · {u.v.dayRequests} req</span></div>
    <div class="card s"><span class="dim">This month</span><b>{tokens(u.v.monthIn + u.v.monthOut)}</b><span class="dim small">{tokens(u.v.monthIn)} in · {tokens(u.v.monthOut)} out</span></div>
  </div>
  <p class="dim small">Counts GlassBar's own requests only (quick-ask and AI OCR). See console.anthropic.com for billing.</p>
{/if}

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; display: flex; align-items: baseline; gap: 8px; }
  .model { font-size: 11px; font-weight: 400; }
  .ask { position: relative; }
  textarea { width: 100%; box-sizing: border-box; resize: none; padding: 10px 40px 10px 12px; border-radius: 12px; border: none; background: var(--pill); color: inherit; font: inherit; font-size: 13px; }
  .send { position: absolute; right: 8px; bottom: 10px; width: 28px; height: 28px; border-radius: 14px; display: grid; place-items: center; background: var(--accent); color: #fff; }
  .send:disabled { opacity: 0.4; }
  .ans { position: relative; margin-top: 8px; padding: 10px 12px; }
  .ans pre { margin: 0; white-space: pre-wrap; word-break: break-word; font: 13px/1.5 inherit; font-family: inherit; }
  .cp { position: absolute; top: 6px; right: 6px; opacity: 0.6; }
  .cp:hover { opacity: 1; }
  h4 { margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .s { display: flex; flex-direction: column; gap: 2px; padding: 8px 10px; font-size: 12px; }
  .s b { font-size: 18px; }
  .small { font-size: 11px; }
  .note { padding: 12px; font-size: 12px; line-height: 1.5; }
  .err { color: #ff453a; }
</style>
