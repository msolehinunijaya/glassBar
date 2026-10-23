<script lang="ts">
  // Half-ellipse from sunrise (left) to sunset (right); the sun dot sits at the current
  // fraction of daylight. Before sunrise / after sunset it rests at the end, dimmed.
  import { daylight, sunArc } from './sun';

  let { rise, set, now }: { rise?: number | null; set?: number | null; now: number } = $props();
  const W = 280, H = 90, R = 120, CX = W / 2, BASE = H - 10;
  const s = $derived(sunArc(now, rise, set));
  const a = $derived(Math.PI * (1 - s.frac)); // π at sunrise, 0 at sunset
  const x = $derived(CX + R * Math.cos(a));
  const y = $derived(BASE - (H - 22) * Math.sin(a));
  const t = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(ms);
  const len = $derived(daylight(rise, set));
</script>

{#if s.phase === 'none'}
  <p class="dim small">No sunrise or sunset today at this location (polar day or night).</p>
{:else}
  <svg viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="Sun position">
    <path d="M {CX - R} {BASE} A {R} {H - 22} 0 0 1 {CX + R} {BASE}" fill="none" stroke="currentColor" stroke-opacity="0.25" stroke-dasharray="3 4" />
    <path d="M {CX - R} {BASE} A {R} {H - 22} 0 0 1 {x} {y}" fill="none" stroke="#ffb340" stroke-width="2.5" stroke-linecap="round" />
    <line x1="8" x2={W - 8} y1={BASE} y2={BASE} stroke="currentColor" stroke-opacity="0.2" />
    <circle cx={x} cy={y} r="7" fill="#ffcc4d" opacity={s.phase === 'day' ? 1 : 0.45} />
    {#if s.phase === 'day'}<circle cx={x} cy={y} r="12" fill="#ffcc4d" opacity="0.25" />{/if}
  </svg>
  <div class="row small">
    <span>☀︎ {t(rise!)}<br /><span class="dim">Sunrise</span></span>
    <span class="mid">{len}<br /><span class="dim">Daylight</span></span>
    <span class="end">{t(set!)}<br /><span class="dim">Sunset</span></span>
  </div>
{/if}

<style>
  svg { display: block; }
  .row { display: grid; grid-template-columns: 1fr 1fr 1fr; margin-top: 2px; line-height: 1.35; }
  .mid { text-align: center; }
  .end { text-align: right; }
  .small { font-size: 11px; }
</style>
