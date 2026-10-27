<script lang="ts">
  // Settings › Performance: what GlassBar itself uses (its whole process tree), an hour of
  // history, and the Low / Normal / High modes with what each one changes, widget by widget.
  import { onMount } from 'svelte';
  import Sparkline from '../lib/Sparkline.svelte';
  import { update } from '../lib/config.svelte';
  import { events, ipc, type PerfMode, type PerfSample } from '../lib/ipc';
  import { perf } from '../lib/perf.svelte';
  import { widgets } from '../widgets/registry';
  import { APP_ROWS, MODE_CARDS, changeList, fmtBytes, fmtPct, netLastHour } from './perfPage';

  type Metric = 'ram' | 'cpu' | 'gpu' | 'net';
  let samples = $state<PerfSample[]>([]);
  let metric = $state<Metric>('ram');
  let graphW = $state(520);

  onMount(() => {
    ipc.perfHistory().then((h) => (samples = h)).catch(() => {});
    const off = events.perfSample((s) => { samples = [...samples.filter((x) => x.at > s.at - 3_600_000), s]; });
    // The 1-second sampling is a lease: renew it while this page is open.
    ipc.perfWatch().catch(() => {});
    const t = setInterval(() => ipc.perfWatch().catch(() => {}), 2000);
    return () => { clearInterval(t); off.then((f) => f()); };
  });

  const last = $derived(samples.at(-1));
  const series = $derived.by(() => {
    if (metric === 'ram') return samples.map((s) => s.ramMb);
    if (metric === 'cpu') return samples.map((s) => s.cpu);
    if (metric === 'gpu') return samples.map((s) => s.gpu ?? 0);
    return samples.map((s, i) => (i ? Math.max(0, s.netBytes - samples[i - 1].netBytes) : 0));
  });
  const floor: Record<Metric, number> = { ram: 50, cpu: 5, gpu: 5, net: 1024 };
  const max = $derived(Math.max(floor[metric], ...series) * 1.2);
  const changes = $derived(changeList(perf.mode, [...APP_ROWS, ...Object.values(widgets).map((m) => ({ name: m.name, perf: m.perf }))]));
  const title = (m: PerfMode) => MODE_CARDS.find((c) => c.mode === m)!.title;

  const pick = (mode: PerfMode) => update((c) => { c.performance = { mode, auto: false }; });
  const setAuto = (auto: boolean) => update((c) => { c.performance.auto = auto; });
</script>

<div class="group">
  <div class="strip">
    <span>RAM <b>{last ? `${Math.round(last.ramMb)} MB` : '—'}</b></span>
    <span>CPU <b>{last ? fmtPct(last.cpu) : '—'}</b></span>
    <span>GPU <b>{last ? fmtPct(last.gpu) : '—'}</b></span>
    <span>Net <b>{fmtBytes(netLastHour(samples))}/hr</b> <span class="dim">({fmtBytes(last?.netBytes ?? 0)} since start)</span></span>
  </div>
  <div class="seg chips">
    {#each [['ram', 'RAM'], ['cpu', 'CPU'], ['gpu', 'GPU'], ['net', 'Network']] as const as [k, label] (k)}
      <button class:sel={metric === k} onclick={() => (metric = k)}>{label}</button>
    {/each}
    <span class="dim span">Last hour</span>
  </div>
  <div class="graph" bind:clientWidth={graphW}>
    <Sparkline values={series} {max} width={graphW} height={70} />
  </div>
  <p class="hint">Includes GlassBar's WebView2 helpers and this Settings window. Network counts data GlassBar downloaded.</p>
</div>

<div class="group">
  <label class="field"><span>Auto</span>
    <input type="checkbox" checked={perf.auto} onchange={(e) => setAuto(e.currentTarget.checked)} />
  </label>
  <p class="hint">Drops to Low while your PC is short on memory or CPU, or battery saver is on, and back to Normal once it's calm.</p>
  <div class="cards">
    {#each MODE_CARDS as c (c.mode)}
      <button class="card" class:sel={perf.mode === c.mode} onclick={() => pick(c.mode)}>
        <b>{c.title}</b><span>{c.summary}</span>
      </button>
    {/each}
  </div>
  {#if perf.auto && perf.reason}<p class="reason">Auto chose {title(perf.mode)} · {perf.reason}</p>{/if}
</div>

<h2>{title(perf.mode)} {perf.mode === 'normal' ? 'compared to High' : 'compared to Normal'}</h2>
<div class="group">
  {#each changes.rows as r (r.name)}
    <div class="field"><span>{r.name}</span><span class="dim">{r.text}</span></div>
  {/each}
  <p class="hint">Unchanged: {changes.unchanged.join(', ')}</p>
</div>

<style>
  .strip { display: flex; flex-wrap: wrap; gap: 6px 18px; padding: 8px 0 2px; font-size: 13px; }
  .strip b { font-variant-numeric: tabular-nums; }
  .dim { color: var(--fg-dim); }
  .chips { align-self: flex-start; align-items: center; }
  .chips button { text-transform: none; }
  .chips .span { padding: 0 8px; font-size: 12px; }
  .graph { width: 100%; height: 70px; }
  .cards { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; padding: 4px 0 8px; }
  .card { display: flex; flex-direction: column; gap: 3px; text-align: left; padding: 10px 12px; border-radius: calc(10px * var(--rs));
          background: var(--pill); border: 2px solid transparent; }
  .card:hover { background: var(--pill-hover); }
  .card.sel { border-color: var(--accent); }
  .card span { font-size: 12px; color: var(--fg-dim); }
  .reason { margin: 0 0 8px; padding: 6px 10px; border-radius: calc(8px * var(--rs)); font-size: 12px;
            background: color-mix(in srgb, #ffb340 18%, transparent); }
</style>
