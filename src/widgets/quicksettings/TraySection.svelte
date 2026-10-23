<script lang="ts">
  // System tray inside Quick Settings: every real tray icon (click / double-click /
  // right-click go to the app as in Windows) and the switch for showing them on the dock.
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc, type TrayIcon } from '../../lib/ipc';
  import { liveTray } from '../tray/icons.svelte';
  import { RefreshCw } from '@lucide/svelte';

  let { instance, onDock }: { instance: string; onDock: boolean } = $props();
  const tray = liveTray();
  const name = (i: TrayIcon) => (i.tip.split('\n')[0] || i.exe.split(/[\\/]/).pop()?.replace(/\.exe$/i, '') || 'App').trim();
  const send = (e: MouseEvent, i: TrayIcon, b: 'left' | 'right' | 'double') => { e.preventDefault(); ipc.trayClick(i.id, b); };
</script>

<div class="head">
  <h4 class="dim">System tray <span class="n">{tray.v.length}</span>
    <button class="refresh" title="Ask apps to show their tray icons again" onclick={() => ipc.trayRefresh()}><RefreshCw size={11} /></button></h4>
  <label class="dim sw">Show on dock
    <input type="checkbox" class="switch" checked={onDock} onchange={(e) => updateWidgetSettings(instance, { trayOnDock: e.currentTarget.checked })} />
  </label>
</div>
<div class="grid">
  {#each tray.v as i (i.id)}
    <button class="item" title={i.tip || i.exe} onclick={(e) => send(e, i, 'left')} ondblclick={(e) => send(e, i, 'double')} oncontextmenu={(e) => send(e, i, 'right')}>
      {#if i.icon}<img src={i.icon} alt="" width="20" height="20" draggable="false" />{:else}<span class="blank"></span>{/if}
      <span class="nm">{name(i)}</span>
    </button>
  {:else}
    <p class="dim small empty">Waiting for apps to report their tray icons…</p>
  {/each}
</div>

<style>
  .head { display: flex; align-items: center; justify-content: space-between; margin: 14px 2px 6px; }
  h4 { margin: 0; font-size: 12px; font-weight: 500; display: flex; align-items: center; gap: 5px; }
  .refresh { display: grid; place-items: center; width: 20px; height: 20px; border-radius: 6px; opacity: 0.7; }
  .refresh:hover { opacity: 1; background: var(--pill-hover); }
  .n { opacity: 0.7; }
  .sw { display: flex; align-items: center; gap: 6px; font-size: 11px; }
  .grid { display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); gap: 2px; max-height: 150px; overflow-x: hidden; overflow-y: auto; scrollbar-width: thin; }
  .item { min-width: 0; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 7px 2px 5px; border-radius: 10px; transition: background 0.15s, transform 0.2s var(--spring); }
  .item:hover { background: var(--pill-hover); }
  .item:active { transform: scale(0.94); }
  .nm { font-size: 10px; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .blank { width: 20px; height: 20px; border-radius: 5px; background: var(--track); }
  .small { font-size: 11px; }
  .empty { grid-column: 1 / -1; text-align: center; padding: 14px 0; }
</style>
