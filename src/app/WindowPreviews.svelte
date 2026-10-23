<script lang="ts">
  // Hover preview for an app's windows: live thumbnails side by side (drawn by Windows/DWM
  // into the empty boxes below), each with its title and a close button. Click one to
  // switch to that exact window. Windows that can't be captured (e.g. minimised) show
  // the app icon + title instead; an app with no windows closes the preview, never an empty box.
  import { X } from '@lucide/svelte';
  import { onMount, tick } from 'svelte';
  import { events, ipc, type AppWindow } from '../lib/ipc';
  import { keyOf } from '../widgets/apps/groups';

  let { appKey, close }: { appKey: string; close: () => void } = $props();
  let windows = $state<AppWindow[]>([]);
  let live = $state<Set<number>>(new Set());
  let icon = $state<string | null>(null);
  const boxes: Record<number, HTMLElement> = {};
  let activeAtOpen: number | undefined;

  const mine = (all: AppWindow[]) => all.filter((w) => keyOf(w.exe) === appKey).sort((a, b) => a.id - b.id);

  async function place() {
    if (!windows.length) { close(); return; }
    await tick();
    const items = windows
      .map((w) => ({ w, el: boxes[w.id] }))
      .filter((x) => x.el)
      .map(({ w, el }) => { const r = el.getBoundingClientRect(); return { id: w.id, x: r.x, y: r.y, w: r.width, h: r.height }; });
    live = new Set(await ipc.thumbsShow(items));
  }

  onMount(() => {
    ipc.appsList().then((all) => {
      windows = mine(all);
      activeAtOpen = all.find((w) => w.active)?.id;
      if (windows[0]) ipc.appIcon(windows[0].exe, windows[0].id).then((i) => (icon = i));
      place();
    });
    const off = events.appsChanged((all) => {
      // Focus moved to another window: the preview has done its job.
      const active = all.find((w) => w.active)?.id;
      if (active !== activeAtOpen) { close(); return; }
      windows = mine(all);
      place();
    });
    return () => { off.then((f) => f()); ipc.thumbsClear(); };
  });

  function pick(w: AppWindow) { close(); ipc.appActivate(w.id); }
  function shut(e: Event, w: AppWindow) { e.stopPropagation(); ipc.appClose(w.id); }
</script>

{#if windows.length}
  <div class="row">
    {#each windows as w (w.id)}
      <div class="card win" class:active={w.active} role="button" tabindex="0"
           onclick={() => pick(w)} onkeydown={(e) => e.key === 'Enter' && pick(w)} title={w.title}>
        <div class="top">
          {#if icon}<img src={icon} alt="" width="14" height="14" />{/if}
          <span class="ellipsis">{w.title}</span>
          <button class="x" onclick={(e) => shut(e, w)} aria-label="Close {w.title}"><X size={12} /></button>
        </div>
        <div class="thumb" bind:this={boxes[w.id]}>
          {#if !live.has(w.id)}
            <div class="fallback">
              {#if icon}<img src={icon} alt="" width="40" height="40" />{/if}
              <span class="dim">{w.minimized ? 'Minimised' : ''}</span>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .row { display: flex; gap: 8px; }
  .win { width: 208px; flex: none; padding: 6px; display: flex; flex-direction: column; gap: 6px; cursor: default;
         transition: background 0.2s var(--ease), transform 0.25s var(--spring); }
  .win:hover { background: var(--pill-hover); }
  .win:active { transform: scale(0.97); }
  .win.active { border-color: var(--accent); }
  .top { display: flex; align-items: center; gap: 6px; font-size: 12px; min-width: 0; }
  .top span { flex: 1; }
  .x { width: 20px; height: 20px; display: grid; place-items: center; border-radius: 50%; opacity: 0.6; flex: none; }
  .x:hover { background: #ff453a; color: #fff; opacity: 1; }
  /* DWM draws the live window picture over this box */
  .thumb { height: 116px; border-radius: 8px; }
  .fallback { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px;
              border-radius: 8px; background: var(--pill); font-size: 11px; }
</style>
