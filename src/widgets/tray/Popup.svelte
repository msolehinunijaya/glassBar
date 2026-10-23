<script lang="ts">
  // Every real tray icon with its app name. Click / double-click / right-click go to the app,
  // exactly as in the Windows tray. Updates live as apps add or remove icons.
  import type { PopupProps } from '../types';
  import type { TraySettings } from './index';
  import { ipc, type TrayIcon } from '../../lib/ipc';
  import { liveTray } from './icons.svelte';
  import TrayButtons from './TrayButtons.svelte';

  let { settings }: PopupProps<TraySettings> = $props();
  const tray = liveTray();
  const name = (i: TrayIcon) => (i.tip.split('\n')[0] || i.exe.split(/[\\/]/).pop()?.replace(/\.exe$/i, '') || 'App').trim();
  const send = (e: MouseEvent, i: TrayIcon, b: 'left' | 'right' | 'double') => { e.preventDefault(); ipc.trayClick(i.id, b); };
</script>

{#if settings.native}
  <h3>System tray <span class="dim n">{tray.v.length}</span></h3>
  <div class="grid">
    {#each tray.v as i (i.id)}
      <button class="item" title={i.tip || i.exe} onclick={(e) => send(e, i, 'left')} ondblclick={(e) => send(e, i, 'double')} oncontextmenu={(e) => send(e, i, 'right')}>
        {#if i.icon}<img src={i.icon} alt="" width="24" height="24" draggable="false" />{:else}<span class="blank"></span>{/if}
        <span class="nm">{name(i)}</span>
        {#if i.promoted}<span class="pin" title="Always shown in Windows">•</span>{/if}
      </button>
    {:else}
      <p class="dim small empty">Waiting for apps to report their tray icons…</p>
    {/each}
  </div>
  <p class="dim small">Right-click an icon for the app's own menu. Esc or clicking outside closes this panel.</p>
{:else}
  <TrayButtons />
{/if}

<style>
  h3 { margin: 0 0 10px; font-size: 14px; font-weight: 600; display: flex; gap: 6px; align-items: baseline; }
  .n { font-size: 12px; font-weight: 400; }
  .grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 4px; max-height: 250px; overflow-y: auto; scrollbar-width: thin; }
  .item { position: relative; display: flex; flex-direction: column; align-items: center; gap: 5px; padding: 9px 4px 7px; border-radius: 12px; transition: background 0.15s, transform 0.2s var(--spring); }
  .item:hover { background: var(--pill-hover); }
  .item:active { transform: scale(0.94); }
  .nm { font-size: 10.5px; line-height: 1.2; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .blank { width: 24px; height: 24px; border-radius: 6px; background: var(--track); }
  .pin { position: absolute; top: 3px; right: 7px; color: var(--accent); font-size: 14px; line-height: 1; }
  .small { font-size: 11px; }
  .empty { grid-column: 1 / -1; text-align: center; padding: 30px 0; }
</style>
