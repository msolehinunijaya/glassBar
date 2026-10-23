<script lang="ts">
  // Compact battery: the percentage sits inside the battery body, so no separate label is
  // needed. States: charging (⚡ + green, sweeping highlight), full (✓ on mains), plugged in
  // (green cap), discharging (green/orange/red by level, red pulse at ≤15%), unavailable (–).
  import type { Battery } from '../../lib/ipc';

  let { b, size = 18, showPct = true }: { b: Battery | null; size?: number; showPct?: boolean } = $props();
  const state = $derived(!b ? 'none' : b.charging ? 'charging' : b.onAc && b.percent >= 95 ? 'full' : b.onAc ? 'plugged' : 'battery');
  const level = $derived(b ? Math.max(3, b.percent) : 0);
  const low = $derived(state === 'battery' && !!b && b.percent <= 15);
  const color = $derived(state === 'charging' || state === 'full' || state === 'plugged' ? '#30d158'
    : !b ? 'transparent' : b.percent <= 15 ? '#ff453a' : b.percent <= 30 ? '#ff9f0a' : '#30d158');
  // Width grows with the size so 3 digits always fit.
  const w = $derived(Math.round(size * 1.9));
  const fs = $derived(Math.max(8.5, Math.round(size * 0.56)));
</script>

<span class="bat {state}" class:low style:--h="{size}px" style:--w="{w}px" style:--fs="{fs}px"
      title={!b ? 'No battery' : `${b.percent}% · ${state === 'charging' ? 'charging' : state === 'full' ? 'fully charged' : state === 'plugged' ? 'plugged in' : 'on battery'}`}>
  <span class="body">
    <span class="fill" style:width="{level}%" style:background={color}></span>
    {#if showPct}
      <span class="pct">
        {#if state === 'charging'}<svg viewBox="0 0 10 14" class="glyph"><path d="M6 0 0 8h4l-1 6 6-8H5z" /></svg>{/if}
        {#if state === 'full'}<svg viewBox="0 0 12 12" class="glyph"><path d="M2 6.5 5 9.5 10 3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>{/if}
        {b ? b.percent : '–'}
      </span>
    {/if}
  </span>
  <span class="cap"></span>
</span>

<style>
  .bat { position: relative; display: inline-flex; align-items: center; flex: none; }
  .body {
    position: relative; width: var(--w); height: var(--h); border-radius: calc(var(--h) * 0.28);
    border: 1.5px solid currentColor; padding: 1.5px; overflow: hidden;
  }
  .fill { display: block; height: 100%; border-radius: 2px; opacity: 0.85; transition: width 0.8s var(--ease), background 0.4s; position: relative; overflow: hidden; }
  .pct {
    position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; gap: 1px;
    font-size: var(--fs); font-weight: 700; line-height: 1; font-variant-numeric: tabular-nums;
    color: #fff; text-shadow: 0 0 2px rgba(0, 0, 0, 0.8);
  }
  .glyph { width: calc(var(--fs) * 0.75); height: calc(var(--fs) * 0.95); fill: currentColor; }
  .cap { width: 2px; height: calc(var(--h) * 0.42); background: currentColor; border-radius: 0 1px 1px 0; margin-left: 1px; }
  .plugged .cap, .full .cap, .charging .cap { background: #30d158; width: 3px; }
  .none .body { opacity: 0.5; border-style: dashed; }
  .charging .fill::after {
    content: ''; position: absolute; inset: 0; width: 40%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.55), transparent);
    animation: sweep 1.8s ease-in-out infinite;
  }
  @keyframes sweep { from { transform: translateX(-100%); } to { transform: translateX(260%); } }
  .low .fill { animation: pulse 1.4s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.35; } }
</style>
