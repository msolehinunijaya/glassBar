<script lang="ts">
  // Privacy Monitor panel: an honest one-line status, then a card per resource.
  import { RefreshCw, Settings2, X } from '@lucide/svelte';
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { ipc } from '../../lib/ipc';
  import type { PopupProps } from '../types';
  import type { PrivacySettings } from './settings';
  import Card from './Card.svelte';
  import { monitor } from './monitor.svelte';
  import { clock, summary, visible } from './status';

  let { settings, close }: PopupProps<PrivacySettings> = $props();
  const m = monitor();
  const list = $derived(visible(m.v, settings));
  const active = $derived(list.filter((r) => r.state === 'active').length);
</script>

<PanelHeader title="Privacy Monitor">
  <button class="icon-btn" onclick={() => m.reload()} disabled={m.busy} aria-label="Check again" title="Check again"><span class:spin={m.busy}><RefreshCw size={15} /></span></button>
  <button class="icon-btn" onclick={() => { ipc.openSettings('Widgets').catch(() => {}); close(); }} aria-label="Privacy Monitor settings" title="Choose what to monitor and notify"><Settings2 size={15} /></button>
  <button class="icon-btn" onclick={close} aria-label="Close" title="Close"><X size={15} /></button>
</PanelHeader>

<div class="status" class:on={active > 0}>
  <span>{m.v || m.error ? summary(list, !!m.error) : 'Checking…'}</span>
  {#if active}<span class="count">{active} active</span>{/if}
</div>
{#if m.error}<p class="err">{m.error}</p>{/if}

<div class="cards">
  {#each list as r (r.key)}<Card {r} {settings} muted={m.v?.micMuted ?? null} reload={m.reload} />{/each}
</div>

<p class="foot dim">{m.v ? `Checked ${clock(m.v.checked)} · ` : ''}Read from Windows. GlassBar records no audio, video, screen or location and keeps no history.</p>

<style>
  .status { display: flex; align-items: center; justify-content: space-between; gap: var(--s-2); margin: 0 2px var(--s-3); font-size: 12.5px; color: var(--fg-dim); }
  .status.on { color: var(--fg); font-weight: 500; }
  .count { flex: none; padding: 2px 8px; border-radius: var(--r-chip); background: var(--accent); color: var(--on-accent); font-size: 11px; font-weight: 600; }
  .cards { display: flex; flex-direction: column; gap: var(--s-2); }
  .err { margin: 0 2px var(--s-2); color: #ff453a; font-size: 12px; }
  .foot { margin: var(--s-3) 2px 0; font-size: 11px; line-height: 1.4; }
  .spin { display: grid; animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
