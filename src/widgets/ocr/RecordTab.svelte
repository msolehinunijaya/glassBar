<script lang="ts">
  // Screen recording uses the recorders built into Windows; GlassBar just starts them.
  import { AppWindow, SquareDashed } from '@lucide/svelte';
  import { ipc } from '../../lib/ipc';

  async function rec(kind: 'area' | 'app') {
    await ipc.popupHide();
    setTimeout(() => ipc.recordScreen(kind).catch(() => {}), 180);
  }
</script>

<div class="grp act">
<div class="opts">
  <button class="opt" onclick={() => rec('area')}>
    <SquareDashed size={22} />
    <span class="t">Record an area or screen</span>
    <span class="dim d">Snipping Tool (Win+Shift+R): pick the area, then Start. Stop from its toolbar; the video is saved in Videos › Screen Recordings and can be shared or saved from there.</span>
  </button>
  <button class="opt" onclick={() => rec('app')}>
    <AppWindow size={22} />
    <span class="t">Record the current app</span>
    <span class="dim d">Xbox Game Bar (Win+Alt+R): records the app in front, with its sound. Press Win+Alt+R again to stop; saved in Videos › Captures.</span>
  </button>
</div>
<p class="dim small">Uses Windows' own recorders. If nothing happens, update Snipping Tool from the Microsoft Store, or turn on Game Bar in Settings › Gaming.</p>
</div>

<style>
  .opts { display: flex; flex-direction: column; gap: 8px; }
  .opt { display: grid; grid-template-columns: 30px 1fr; grid-template-rows: auto auto; column-gap: 10px; row-gap: 3px; align-items: center; text-align: left;
         padding: 12px; border-radius: calc(14px * var(--rs)); background: var(--pill); transition: background 0.15s, transform 0.2s var(--spring); }
  .opt:hover { background: var(--pill-hover); }
  .opt:active { transform: scale(0.98); }
  .opt :global(svg) { grid-row: 1 / 3; color: #ff453a; }
  .t { font-size: 13px; font-weight: 600; }
  .d { font-size: 11px; line-height: 1.4; }
  .small { font-size: 11px; margin: 10px 2px 0; }
</style>
