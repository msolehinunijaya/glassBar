<script lang="ts">
  // Hover chooser for an app with several windows (e.g. two Chrome windows).
  import { X } from '@lucide/svelte';
  import { onMount } from 'svelte';
  import { events, ipc, type AppWindow } from '../lib/ipc';
  import { keyOf, nameOf } from '../widgets/apps/groups';

  let { appKey, close }: { appKey: string; close: () => void } = $props();
  let windows = $state<AppWindow[]>([]);
  let icon = $state<string | null>(null);
  const mine = (all: AppWindow[]) => all.filter((w) => keyOf(w.exe) === appKey).sort((a, b) => a.id - b.id);

  onMount(() => {
    ipc.appsList().then((all) => {
      windows = mine(all);
      if (windows[0]) ipc.appIcon(windows[0].exe, windows[0].id).then((i) => (icon = i));
    });
    const off = events.appsChanged((all) => {
      windows = mine(all);
      if (!windows.length) close();
    });
    return () => { off.then((f) => f()); };
  });

  function pick(w: AppWindow) { ipc.appActivate(w.id); close(); }
</script>

<div class="head dim">{windows[0] ? nameOf(windows[0].exe) : ''} · {windows.length} windows</div>
{#each windows as w (w.id)}
  <div class="row" class:active={w.active}>
    <button class="pick" onclick={() => pick(w)} title={w.title}>
      {#if icon}<img src={icon} alt="" width="18" height="18" />{/if}
      <span class="ellipsis">{w.title}</span>
      {#if w.minimized}<span class="dim tag">minimized</span>{/if}
    </button>
    <button class="x" onclick={() => ipc.appClose(w.id)} aria-label="Close window"><X size={13} /></button>
  </div>
{/each}

<style>
  .head { font-size: 11px; padding: 0 6px 6px; }
  .row { display: flex; align-items: center; gap: 2px; border-radius: 10px; }
  .row:hover { background: var(--pill-hover); }
  .row.active { background: var(--pill); }
  .pick { flex: 1; min-width: 0; display: flex; align-items: center; gap: 8px; padding: 8px 8px; text-align: left; }
  .pick span { min-width: 0; }
  .tag { font-size: 10px; flex: none; }
  .x { width: 26px; height: 26px; display: grid; place-items: center; border-radius: 50%; opacity: 0; }
  .row:hover .x { opacity: 0.7; }
  .x:hover { background: var(--pill-hover); opacity: 1; }
</style>
