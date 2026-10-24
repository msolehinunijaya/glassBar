<script lang="ts">
  import { BellOff, ExternalLink, X } from '@lucide/svelte';
  import { store, update, updateWidgetSettings } from '../../lib/config.svelte';
  import type { NotifSettings } from './index';
  import { ipc } from '../../lib/ipc';
  import { relTime } from '../../lib/time';
  import type { PopupProps } from '../types';
  import { liveAlerts } from './alerts.svelte';

  let { instance, settings }: PopupProps<NotifSettings> = $props();
  // Tab starts on the remembered one (if enabled), otherwise on GlassBar's own list.
  let tab = $state<'dock' | 'windows'>('dock');
  $effect.pre(() => { tab = settings.windowsTab && settings.rememberTab ? settings.lastTab : 'dock'; });
  function setTab(t: 'dock' | 'windows') {
    tab = t;
    if (settings.rememberTab) updateWidgetSettings(instance, { lastTab: t });
  }
  const alerts = liveAlerts();
  $effect(() => { ipc.alertsMarkRead(); });
</script>

<div class="head">
  <h3>Notifications</h3>
  {#if tab === 'dock' && alerts.list.length}<button class="link" onclick={() => ipc.alertsClear()}>Clear all</button>{/if}
</div>

{#if settings.windowsTab}
  <div class="seg">
    <button class:sel={tab === 'dock'} onclick={() => setTab('dock')}>Dock</button>
    <button class:sel={tab === 'windows'} onclick={() => setTab('windows')}>Windows</button>
  </div>
{/if}

{#if tab === 'windows'}
  <div class="card win">
    <p>Windows only lets packaged (Microsoft Store / MSIX) apps read other apps' notifications, so GlassBar can't list them here.</p>
    <button class="open" onclick={() => ipc.windowsNotifications()}><ExternalLink size={14} /> Open Windows Notification Center</button>
    <p class="dim small">Shortcut: Win + N</p>
  </div>
{:else}

<label class="dnd card">
  <BellOff size={16} />
  <span>Do Not Disturb</span>
  <input type="checkbox" checked={store.config?.dnd} onchange={(e) => update((c) => (c.dnd = e.currentTarget.checked))} />
</label>

<div class="list">
  {#each alerts.list as a (a.id)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="item card" role="button" tabindex="-1" title="Go to the widget" onclick={() => ipc.jumpTo(a.source, a.data)}>
      <div class="top">
        <b class="ellipsis">{a.title}</b>
        <span class="dim small">{relTime(a.at)}</span>
        <button class="x" aria-label="Dismiss" onclick={(e) => { e.stopPropagation(); ipc.alertsDismiss(a.id); }}><X size={13} /></button>
      </div>
      {#if a.body}<div class="small">{a.body}</div>{/if}
      {#if a.missed}<span class="tag">missed</span>{/if}
    </div>
  {:else}
    <p class="dim empty">No notifications.</p>
  {/each}
</div>
{/if}

<style>
  .head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  h3 { margin: 0; font-size: 15px; font-weight: 600; }
  .link { color: var(--accent); font-size: 12px; }
  .dnd { display: flex; align-items: center; gap: 10px; padding: 10px 12px; margin-bottom: 10px; }
  .dnd span { flex: 1; }
  .list { display: flex; flex-direction: column; gap: 6px; }
  .item { padding: 10px 12px; transition: background 0.15s; }
  .item:hover { background: var(--pill-hover); }
  .top { display: flex; align-items: center; gap: 8px; }
  .top b { flex: 1; font-weight: 500; }
  .small { font-size: 12px; }
  .x { opacity: 0.5; display: grid; place-items: center; }
  .x:hover { opacity: 1; }
  .tag { display: inline-block; margin-top: 4px; font-size: 10px; padding: 1px 6px; border-radius: calc(6px * var(--rs)); background: var(--track); }
  .seg { display: grid; grid-template-columns: 1fr 1fr; gap: 2px; padding: 3px; border-radius: calc(10px * var(--rs)); background: var(--pill); margin-bottom: 10px; }
  .seg button { padding: 6px; border-radius: calc(8px * var(--rs)); font-size: 12px; }
  .seg .sel { background: var(--accent); color: #fff; }
  .win { padding: 14px; font-size: 12px; line-height: 1.5; }
  .win p { margin: 0 0 10px; }
  .open { display: flex; align-items: center; gap: 6px; justify-content: center; width: 100%; padding: 9px; border-radius: calc(10px * var(--rs)); background: var(--accent); color: #fff; font-size: 12px; margin-bottom: 6px; }
  .empty { text-align: center; padding: 40px 0; }
</style>
