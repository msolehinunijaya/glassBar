<script lang="ts">
  // The battery's "Fill" design: its whole box fills with liquid up to the charge level --
  // green above 50 %, amber from 20 %, red below (as the Liquid design) -- with the percentage
  // in the middle and a bolt beside it while plugged in. The surface moves only while
  // charging, so it costs nothing otherwise. Put it in a box that is `position: relative`.
  import { Zap } from '@lucide/svelte';
  import type { Battery } from '../../lib/ipc';
  import Tank from '../../lib/Tank.svelte';

  let { b, big = false }: { b: Battery; big?: boolean } = $props();
  const level = $derived(Math.max(0, Math.min(100, Math.round(b.percent))));
  const tone = $derived(level > 50 ? '#30d158' : level >= 20 ? '#ffb340' : '#ff453a');
</script>

<Tank fill={level / 100} color={tone} live={b.charging} />
<span class="v" class:big aria-label="{level}% battery{b.onAc ? ', plugged in' : ''}">
  {#if b.onAc}<span class="bolt" class:charging={b.charging}><Zap size={big ? 16 : 11} fill="currentColor" strokeWidth={1.5} /></span>{/if}
  {level}<small>%</small>
</span>

<style>
  .v { position: relative; z-index: 1; display: inline-flex; align-items: center; gap: 2px; font-size: 12px; font-weight: 600; font-variant-numeric: tabular-nums; }
  .v small { font-size: 0.75em; font-weight: 500; opacity: 0.8; }
  .v.big { font-size: 22px; gap: 4px; }
  .bolt { display: inline-flex; color: #ffd60a; filter: drop-shadow(0 0 2px rgba(0, 0, 0, 0.35)); }
  .bolt.charging { animation: pulse 1.6s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.55; transform: scale(0.9); } }
  @media (prefers-reduced-motion: reduce) { .bolt.charging { animation: none; } }
</style>
