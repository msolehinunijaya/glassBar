<script lang="ts">
  // Windows 11 doesn't let other programs draw its tray icons, so GlassBar opens Windows'
  // own hidden-icons flyout instead (only possible while the native taskbar is shown).
  import { ChevronUp, Settings2 } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';
  import { profile, store } from '../../lib/config.svelte';

  const nativeVisible = $derived(!!store.config && profile().bar.nativeTaskbar !== 'hidden' && !profile().bar.autoHide);
</script>

<div class="row">
  <button class="b" onclick={() => ipc.trayOverflow(nativeVisible)}><ChevronUp size={14} />{nativeVisible ? 'Hidden icons' : 'Tray icons…'}</button>
  <button class="b" onclick={() => ipc.traySettings()}><Settings2 size={14} />Tray settings</button>
</div>
{#if !nativeVisible}<p class="dim small">The Windows taskbar is hidden, so its tray flyout can't open; this opens tray icon settings instead.</p>{/if}

<style>
  .row { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .b { display: flex; align-items: center; justify-content: center; gap: 6px; padding: 8px; border-radius: 10px; background: var(--pill); font-size: 12px; }
  .b:hover { background: var(--pill-hover); }
  .small { font-size: 11px; margin: 6px 0 0; }
</style>
