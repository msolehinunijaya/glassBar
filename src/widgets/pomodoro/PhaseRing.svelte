<script lang="ts">
  // The Pomodoro ring: time remaining, drained clockwise from 12 o'clock. Track = 10 % of
  // the text colour; progress in the phase colour (`--c`) with a round cap and a soft glow.
  // Paused: dimmed and dashed, no glow. `urgent` (last minute) thickens the stroke. The arc is
  // a solid mask over a (dashed when paused) circle, so dashes and progress don't fight.
  import type { Snippet } from 'svelte';

  let { size, stroke, left, paused = false, urgent = false, ticking = false, glow = 8, children }: {
    size: number; stroke: number; left: number; paused?: boolean; urgent?: boolean;
    /** Running: follow the seconds linearly; otherwise jumps sweep in 400 ms. */
    ticking?: boolean; glow?: number; children?: Snippet;
  } = $props();
  const id = $props.id();
  const w = $derived(urgent ? stroke * 1.3 : stroke);
  const r = $derived((size - stroke * 1.3) / 2);
  const C = $derived(2 * Math.PI * r);
  const f = $derived(Math.min(1, Math.max(0, left)));
  const mid = $derived(size / 2);
  const dash = $derived(Math.max(2, stroke * 0.6));
</script>

<span class="ring" class:paused style:width="{size}px" style:height="{size}px" style:--glow="{glow}px">
  <svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
    <mask id="m{id}" maskUnits="userSpaceOnUse" x="0" y="0" width={size} height={size}>
      <circle class="prog" class:ticking cx={mid} cy={mid} {r} fill="none" stroke="#fff" stroke-width={w} stroke-linecap="round"
              stroke-dasharray="{C} {C}" stroke-dashoffset={-(1 - f) * C} transform="rotate(-90 {mid} {mid})" opacity={f > 0.002 ? 1 : 0} />
    </mask>
    <circle class="track" cx={mid} cy={mid} {r} fill="none" stroke-width={stroke} />
    <g class="arcwrap">
      <circle class="arc" cx={mid} cy={mid} {r} fill="none" stroke-width={w + 1}
              stroke-dasharray={paused ? `${dash} ${dash * 1.2}` : 'none'} mask="url(#m{id})" />
    </g>
  </svg>
  {#if children}<span class="mid">{@render children()}</span>{/if}
</span>

<style>
  .ring { position: relative; display: inline-grid; place-items: center; flex: none; }
  .mid { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; }
  .track { stroke: color-mix(in srgb, var(--fg) 10%, transparent); }
  .arc { stroke: var(--c); transition: opacity 0.2s; }
  .arcwrap { filter: drop-shadow(0 0 var(--glow) color-mix(in srgb, var(--c) 40%, transparent)); }
  .paused .arcwrap { filter: none; }
  .paused .arc { opacity: 0.5; }
  .prog { transition: stroke-dashoffset 400ms var(--ease), stroke-width 0.3s; }
  .prog.ticking { transition: stroke-dashoffset 1s linear, stroke-width 0.3s; }
  @media (prefers-reduced-motion: reduce) { .prog, .prog.ticking { transition: none; } }
  :global(:root[data-motion='reduced']) .prog { transition: none; }
</style>
