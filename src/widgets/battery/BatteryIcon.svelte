<script lang="ts">
  // Battery drawn as a shape whose fill follows the charge level. While charging,
  // a soft highlight sweeps across it; at 15% or less (on battery) it pulses red.
  import { Plug, Zap } from '@lucide/svelte';
  import type { Battery } from '../../lib/ipc';

  let { b, size = 18 }: { b: Battery | null; size?: number } = $props();
  const level = $derived(b ? Math.max(4, b.percent) : 0);
  const low = $derived(!!b && !b.charging && b.percent <= 15);
  const color = $derived(!b ? 'currentColor' : low ? '#ff453a' : b.percent <= 30 && !b.charging ? '#ff9f0a' : '#30d158');
</script>

{#if !b}
  <Plug {size} />
{:else}
  <span class="bat" class:low class:charging={b.charging} style:--h="{size * 0.62}px" style:--w="{size * 1.35}px">
    <span class="body"><span class="fill" style:width="{level}%" style:background={color}></span></span>
    <span class="cap"></span>
    {#if b.charging}<span class="bolt"><Zap size={size * 0.55} fill="#fff" strokeWidth={0} /></span>{/if}
  </span>
{/if}

<style>
  .bat { position: relative; display: inline-flex; align-items: center; }
  .body {
    position: relative; width: var(--w); height: var(--h); border-radius: 4px;
    border: 1.5px solid currentColor; padding: 1.5px; overflow: hidden; opacity: 0.95;
  }
  .fill { display: block; height: 100%; border-radius: 2px; transition: width 0.8s var(--ease), background 0.4s; position: relative; overflow: hidden; }
  .cap { width: 2px; height: calc(var(--h) * 0.4); background: currentColor; border-radius: 0 1px 1px 0; margin-left: 1px; }
  .bolt { position: absolute; inset: 0; right: 3px; display: grid; place-items: center; filter: drop-shadow(0 0 1px rgba(0, 0, 0, 0.6)); }
  /* charging: a highlight sweeps across the fill (compositor-only animation) */
  .charging .fill::after {
    content: ''; position: absolute; inset: 0; width: 40%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.6), transparent);
    animation: sweep 1.8s ease-in-out infinite;
  }
  @keyframes sweep { from { transform: translateX(-100%); } to { transform: translateX(260%); } }
  .low .fill { animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.35; } }
</style>
