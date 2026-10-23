<script lang="ts">
  // Horizontal level visualizer from the real Windows output meter: bars scroll right to
  // left, mirrored around the centre line. Paused / nothing playing: a calm row of dots.
  // The meter only runs while this is on screen and music is playing.
  import { listen } from '@tauri-apps/api/event';
  import { ipc } from '../../lib/ipc';
  import { barHeights, pushPeak } from './viz';

  let { active, width = 64, height = 22 }: { active: boolean; width?: number; height?: number } = $props();
  const BAR = 3, GAP = 2;
  const n = $derived(Math.max(4, Math.floor((width + GAP) / (BAR + GAP))));
  let hist = $state<number[]>([]);

  $effect(() => {
    if (!active) { hist = []; return; }
    const count = n;
    ipc.audioMeter();
    const lease = setInterval(() => ipc.audioMeter(), 2000);
    const off = listen<number>('audio:peak', (e) => { hist = pushPeak(hist, e.payload, count); });
    return () => { clearInterval(lease); off.then((f) => f()); };
  });
  const bars = $derived(barHeights([...Array(Math.max(0, n - hist.length)).fill(0), ...hist], height));
</script>

<span class="viz" class:idle={!active} style:width="{width}px" style:height="{height}px" aria-hidden="true">
  {#each bars as h, i (i)}
    <i style:height="{active ? h : 3}px" style:animation-delay="{(i % 8) * 0.12}s"></i>
  {/each}
</span>

<style>
  .viz { display: flex; align-items: center; justify-content: space-between; flex: none; overflow: hidden; }
  i {
    width: 3px; border-radius: 2px; flex: none;
    background: linear-gradient(to top, var(--accent), color-mix(in srgb, var(--accent) 45%, #fff));
    transition: height 90ms linear;
  }
  .idle i { background: color-mix(in srgb, var(--fg) 28%, transparent); animation: breathe 2.4s ease-in-out infinite; }
  @keyframes breathe { 50% { opacity: 0.35; } }
</style>
