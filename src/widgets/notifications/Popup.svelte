<script lang="ts">
  import { BellOff, X } from '@lucide/svelte';
  import { store, update } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import { relTime } from '../../lib/time';
  import type { PopupProps } from '../types';
  import { liveAlerts } from './alerts.svelte';

  let {}: PopupProps = $props();
  const alerts = liveAlerts();
  $effect(() => { ipc.alertsMarkRead(); });
</script>

<div class="head">
  <h3>Notifications</h3>
  {#if alerts.list.length}<button class="link" onclick={() => ipc.alertsClear()}>Clear all</button>{/if}
</div>

<label class="dnd card">
  <BellOff size={16} />
  <span>Do Not Disturb</span>
  <input type="checkbox" checked={store.config?.dnd} onchange={(e) => update((c) => (c.dnd = e.currentTarget.checked))} />
</label>

<div class="list">
  {#each alerts.list as a (a.id)}
    <div class="item card">
      <div class="top">
        <b class="ellipsis">{a.title}</b>
        <span class="dim small">{relTime(a.at)}</span>
        <button class="x" aria-label="Dismiss" onclick={() => ipc.alertsDismiss(a.id)}><X size={13} /></button>
      </div>
      {#if a.body}<div class="small">{a.body}</div>{/if}
      {#if a.missed}<span class="tag">missed</span>{/if}
    </div>
  {:else}
    <p class="dim empty">No notifications.</p>
  {/each}
</div>

<style>
  .head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  .link { color: var(--accent); font-size: 12px; }
  .dnd { display: flex; align-items: center; gap: 10px; padding: 10px 12px; margin-bottom: 10px; }
  .dnd span { flex: 1; }
  .list { display: flex; flex-direction: column; gap: 6px; }
  .item { padding: 10px 12px; }
  .top { display: flex; align-items: center; gap: 8px; }
  .top b { flex: 1; font-weight: 500; }
  .small { font-size: 12px; }
  .x { opacity: 0.5; display: grid; place-items: center; }
  .x:hover { opacity: 1; }
  .tag { display: inline-block; margin-top: 4px; font-size: 10px; padding: 1px 6px; border-radius: 6px; background: var(--track); }
  .empty { text-align: center; padding: 40px 0; }
</style>
