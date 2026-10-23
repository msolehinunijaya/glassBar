<script lang="ts">
  import { Bell, BellOff } from '@lucide/svelte';
  import { store } from '../../lib/config.svelte';
  import type { PillProps } from '../types';
  import type { NotifSettings } from './index';
  import { liveAlerts } from './alerts.svelte';

  let { settings }: PillProps<NotifSettings> = $props();
  const alerts = liveAlerts();
  const unread = $derived(alerts.list.filter((a) => !a.read).length);
</script>

<div class="pill" title={store.config?.dnd ? 'Do Not Disturb is on' : `${unread} unread`}>
  <span class="bell">
    {#if store.config?.dnd}<BellOff size={16} />{:else}<Bell size={16} />{/if}
    {#if unread && settings.badge}<span class="badge">{unread > 99 ? '99+' : unread}</span>{/if}
  </span>
</div>

<style>
  .bell { position: relative; display: grid; place-items: center; }
  .badge {
    position: absolute; top: -7px; right: -9px; min-width: 15px; height: 15px; padding: 0 4px;
    border-radius: 8px; background: #ff453a; color: #fff; font-size: 9px; font-weight: 600;
    display: grid; place-items: center;
  }
</style>
