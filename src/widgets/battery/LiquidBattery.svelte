<script lang="ts">
  // Liquid battery: water level = charge %, a moving sine wave on top, colour by level
  // (green > 50, amber 20–50, red < 20), % centred, pulsing bolt and rising bubbles while charging.
  // The waves move only while charging (always in High, never in Low): moving SVG groups
  // repaints the whole bar every frame. Low also stills the bubbles and the bolt.
  import type { Battery } from '../../lib/ipc';

  let { b, height = 24 }: { b: Battery; height?: number } = $props();
  const W = 46, H = 22;
  const level = $derived(Math.max(0, Math.min(100, b.percent)));
  const color = $derived(level > 50 ? '#30d158' : level >= 20 ? '#ffb340' : '#ff453a');
  // y of the water surface inside the body (0 = top).
  const surface = $derived(H * (1 - level / 100));
  // One wavelength repeated so a -24px translate loops seamlessly.
  const wave = `M0 2 Q 6 0 12 2 T 24 2 T 36 2 T 48 2 T 60 2 T 72 2 V 30 H 0 Z`;
  const id = `clip-${Math.random().toString(36).slice(2)}`;
</script>

<span class="liquid" class:charging={b.charging} style:height="{height}px">
  <svg viewBox="0 0 {W + 4} {H + 2}" height={height} aria-label="{level}% battery">
    <defs><clipPath id={id}><rect x="1" y="1" width={W} height={H} rx="5" /></clipPath></defs>
    <g clip-path="url(#{id})">
      <rect x="1" y="1" width={W} height={H} fill="currentColor" opacity="0.12" />
      <g class="water" style:transform="translateY({surface - 2}px)">
        <g class="wave back"><path d={wave} fill={color} opacity="0.45" transform="translate(1 1)" /></g>
        <g class="wave"><path d={wave} fill={color} transform="translate(1 1.5)" /></g>
      </g>
      {#if b.charging}
        {#each [8, 20, 31, 40] as x, i (i)}
          <circle class="bubble" cx={x} cy={H} r={i % 2 ? 1.2 : 1.6} style:animation-delay="{i * 0.45}s" />
        {/each}
      {/if}
    </g>
    <rect x="1" y="1" width={W} height={H} rx="5" fill="none" stroke="currentColor" stroke-width="1.6" opacity="0.9" />
    <rect x={W + 1.6} y={H / 2 - 3} width="2.2" height="8" rx="1" fill="currentColor" opacity="0.9" />
  </svg>
  <span class="pct">
    {#if b.charging}<svg class="bolt" viewBox="0 0 10 14" width="7" height="10"><path d="M6 0 0 8h4l-1 6 6-8H5z" fill="currentColor" /></svg>{/if}
    {level}
  </span>
</span>

<style>
  .liquid { position: relative; display: inline-grid; place-items: center; }
  .liquid > svg { display: block; overflow: visible; }
  .water { transition: transform 0.9s var(--ease); }
  .wave { animation: wave 2.6s linear infinite paused; }
  .wave.back { animation-duration: 3.8s; animation-direction: reverse; }
  :global(:root:not([data-perf='low'])) .charging .wave, :global(:root[data-perf='high']) .wave { animation-play-state: running; }
  :global(:root[data-perf='low']) .bolt { animation: none; }
  :global(:root[data-perf='low']) .bubble { display: none; }
  @keyframes wave { from { transform: translateX(0); } to { transform: translateX(-24px); } }
  .pct {
    position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; gap: 1px;
    font-size: 10.5px; font-weight: 700; font-variant-numeric: tabular-nums; color: #fff;
    text-shadow: 0 0 2px rgba(0, 0, 0, 0.75); padding-right: 3px;
  }
  .bolt { animation: pulse 1.6s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.45; transform: scale(0.9); } }
  .bubble { fill: rgba(255, 255, 255, 0.75); animation: rise 1.8s ease-in infinite; transform-box: fill-box; }
  @keyframes rise {
    from { transform: translateY(0); opacity: 0; }
    20% { opacity: 1; }
    to { transform: translateY(-18px); opacity: 0; }
  }
</style>
