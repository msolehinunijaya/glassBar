<script lang="ts">
  // With "Show tray icons on the dock" on: real tray icons (always-shown ones inline, the
  // rest behind the chevron). Otherwise: a button for Windows' own hidden-icons flyout.
  import { ChevronDown, ChevronLeft, ChevronRight, ChevronUp } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { profile, store } from '../../lib/config.svelte';
  import type { PillProps } from '../types';
  import type { TraySettings } from './index';
  import { liveTray, split } from './icons.svelte';
  import TrayIconButton from './TrayIconButton.svelte';

  let { settings, vertical }: PillProps<TraySettings> = $props();
  const nativeVisible = $derived(!!store.config && profile().bar.nativeTaskbar !== 'hidden' && !profile().bar.autoHide);
  const tray = liveTray();
  const parts = $derived(split(tray.v, settings));
  const edge = $derived(store.config ? profile().bar.edge : 'bottom');
  // The chevron points to where the overflow panel opens.
  const Chevron = $derived(edge === 'top' ? ChevronDown : edge === 'left' ? ChevronRight : edge === 'right' ? ChevronLeft : ChevronUp);
</script>

{#if settings.native}
  <div class="pill tray" class:vertical class:compact={settings.compact} title={parts.overflow.length ? `${parts.overflow.length} more icons` : 'Tray'}>
    <!-- The chevron (or anywhere off the icons) opens the panel with every tray icon. -->
    <span class="chev"><Chevron size={14} /></span>
    {#each parts.inline as i (i.id)}<TrayIconButton icon={i} size={settings.compact ? 16 : 18} />{/each}
  </div>
{:else}
  <button class="pill" title={nativeVisible ? 'Show hidden icons (right-click: tray settings)' : 'Taskbar hidden: opens tray icon settings'}
          onclick={(e) => { e.stopPropagation(); ipc.trayOverflow(nativeVisible); }}
          oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); ipc.traySettings(); }}>
    <ChevronUp size={16} />
  </button>
{/if}

<style>
  .tray { gap: 0; padding: 0 4px; }
  .tray.vertical { flex-direction: column; padding: 4px 0; height: auto; }
  .tray.compact { padding: 0 2px; }
  .chev { display: grid; place-items: center; padding: 0 4px; opacity: 0.8; }
  .small { font-size: 11px; }
</style>
