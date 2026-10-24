<script lang="ts">
  // One provider in the flyout: plan limits (or tokens today), usage windows, token split, 30-day trend, by model,
  // by project (Claude Code), cost estimate, footer with source and age.
  import { TriangleAlert } from '@lucide/svelte';
  import type { AiuSource } from '../../lib/ipc';
  import { relTime } from '../../lib/time';
  import Bars from './Bars.svelte';
  import { at, limitName, modelName, money, pct, resetShort, SOURCE_LABEL, tokens, tone, TONE, totalOf, until, windows, type View } from './model';

  let { v, now, myr, manage, prefer }: { v: View; now: number; myr: number | null; manage: () => void; prefer: (s: AiuSource) => void } = $props();
  const u = $derived(v.usage);
  // Plan limits get their own block, so the window rows below show tokens only.
  const rows = $derived(windows(v).filter((w) => !v.limits.length || w.tokens != null).map((w) => (v.limits.length ? { ...w, limit: null } : w)));
  const top = $derived(u?.models[0]);
  const cur = $derived(u ? (u.session?.totals ?? u.today) : null);
  const modelsTotal = $derived(u ? u.models.reduce((a, m) => a + m.tokens, 0) : 0);
  const offline = $derived(v.problems.some((p) => p.kind === 'offline'));
  const clock = (ms: number) => new Date(ms).toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' }).toLowerCase();
</script>

{#each v.problems.filter((p) => p.kind === 'auth' || p.kind === 'broken') as p (p.source)}
  <div class="banner" role="alert">
    <TriangleAlert size={16} />
    {#if p.kind === 'auth'}
      <div class="col"><b>Reconnect {v.name}</b><span>{p.text}</span></div>
      <button class="ghost" onclick={manage}>Reconnect</button>
    {:else}
      <div class="col"><b>{p.text}</b>{#if u?.updatedAt && !u.error}<span>Showing {SOURCE_LABEL[u.source]} data instead.</span>{/if}</div>
    {/if}
  </div>
{/each}

<div class="body" class:stale={v.stale}>
  {#if v.loading}
    <div class="hero"><div class="col"><span class="sk l1"></span><span class="sk l2"></span></div></div>
    <div class="sec">{#each [0, 1, 2, 3] as i (i)}<span class="sk rowsk"></span>{/each}</div>
  {:else}
    {#if v.limits.length}
      <!-- Plan limits: one row each (bar, time to reset, % used), then the most used model. -->
      <section class="plan" aria-label="Plan usage limits">
        {#each v.limits as l (l.key)}
          <div class="pl">
            <span class="pn">{limitName(l)}</span>
            <span class="pbar"><i style:width="{Math.min(100, l.pct)}%" style:background={TONE[tone(l.pct)]}></i></span>
            <span class="pr" title={l.resetsAt ? `Resets ${at(l.resetsAt, now)}` : ''}>{l.resetsAt ? resetShort(l.resetsAt, now) : ''}</span>
            <b class="pp" style:color={tone(l.pct) === 'accent' ? null : TONE[tone(l.pct)]}>{pct(l.pct)}</b>
          </div>
        {/each}
        {#if top}
          <div class="pl">
            <span class="pn">{modelName(top.name)}</span><span></span>
            <span class="pr">most used</span>
            <b class="pp">{tokens(top.tokens)}</b>
          </div>
        {/if}
      </section>
    {:else if u}
      <div class="hero">
        <div class="col">
          <span class="huge">{tokens(totalOf(u.today))}</span>
          <span class="t2">tokens today{#if u.costToday != null && myr != null} · ≈ {money(u.costToday, 'MYR', myr)}{:else if u.costToday != null} · ≈ {money(u.costToday, 'USD')}{/if}</span>
        </div>
      </div>
    {/if}

    <section class="sec">
      {#each rows as w (w.key)}
        <div class="win" class:cur={w.current}>
          <span class="lbl">{w.label}</span>
          {#if w.limit}<span class="bar"><i style:width="{Math.min(100, w.limit.pct)}%" style:background={TONE[tone(w.limit.pct)]}></i></span>{:else}<span></span>{/if}
          <span class="val">
            {#if w.tokens != null}{tokens(w.tokens)}{/if}
            {#if w.limit}<span class:sub={w.tokens != null}>{pct(w.limit.pct)}{#if w.limit.resetsAt && w.key !== 'session'} · {until(w.limit.resetsAt, now)}{/if}</span>{/if}
          </span>
        </div>
      {/each}
    </section>

    {#if u && cur}
      <section class="sec">
        <h4>Tokens {u.session ? 'this session' : 'today'}</h4>
        <div class="tiles">
          {#each [['Input', cur.input], ['Output', cur.output], ['Cache read', cur.cacheRead], ['Cache write', cur.cacheWrite]] as [k, n] (k)}
            <div class="tile"><span class="k">{k}</span><b>{tokens(n as number)}</b></div>
          {/each}
        </div>
      </section>

      <section class="sec"><Bars series={[{ name: v.name, color: 'var(--accent)', data: u.daily }]} /></section>

      {#if u.models.length}
        <section class="sec">
          <h4>By model <span class="k">· last 30 days</span></h4>
          {#each u.models.slice(0, 5) as m (m.name)}
            <div class="li"><span class="ellipsis">{modelName(m.name)}</span><span class="val">{tokens(m.tokens)} <span class="sub">{modelsTotal ? Math.round((m.tokens / modelsTotal) * 100) : 0}%</span></span></div>
          {/each}
        </section>
      {/if}

      {#if u.source === 'claudecode' && u.projects.length}
        <section class="sec">
          <h4>By project <span class="k">· last 30 days</span></h4>
          {#each u.projects as p (p.full)}
            <div class="li" title={p.full}><span class="ellipsis">{p.name}</span><span class="val">{tokens(p.tokens)}</span></div>
          {/each}
        </section>
      {/if}

      {#if u.costToday != null}
        <section class="sec">
          <h4>Cost <span class="k">· estimate{u.costEstimated ? ' from list prices' : ''}</span></h4>
          {#each [['Today', u.costToday], ['This month', u.costMonth ?? 0]] as [k, c] (k)}
            <div class="li"><span>{k}</span><span class="val">{money(c as number, 'USD')}{#if myr != null} <span class="sub">{money(c as number, 'MYR', myr)}</span>{/if}</span></div>
          {/each}
        </section>
      {/if}
    {:else if !u}
      <p class="note">Only the plan limit is available. Connect {v.provider === 'claude' ? 'Claude Code or the Claude API' : 'the OpenAI API'} for tokens by day and model.</p>
    {/if}
  {/if}
</div>

<footer>
  {#if v.usageSources.length > 1}
    <span class="seg" role="group" aria-label="Usage source">
      {#each v.usageSources as s (s.source)}<button class:sel={s === u} onclick={() => prefer(s.source)}>{SOURCE_LABEL[s.source]}</button>{/each}
    </span>
  {:else if u}<span>{SOURCE_LABEL[u.source]}</span>{/if}
  {#if v.web}<span>{u ? '+ ' : ''}{SOURCE_LABEL[v.web.source]}</span>{/if}
  <span class="dot">·</span>
  <span class:off={offline}>{offline ? `offline · last updated ${clock(v.updatedAt)}` : v.updatedAt ? `updated ${relTime(v.updatedAt, now)}` : 'not read yet'}</span>
  <button class="link" onclick={manage}>Manage accounts</button>
</footer>

<style>
  .body { transition: opacity 0.2s; }
  .stale { opacity: 0.55; }
  .banner { display: flex; align-items: center; gap: var(--s-3); padding: var(--s-3); margin-bottom: var(--s-3); border-radius: var(--r-inner);
            background: rgba(255, 159, 10, 0.14); border: 1px solid rgba(255, 159, 10, 0.35); color: var(--fg); }
  .banner :global(svg) { color: #ff9f0a; flex: none; }
  .banner .col span { font-size: 12px; color: var(--fg-dim); }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .plan { display: flex; flex-direction: column; gap: var(--s-1); padding: var(--s-1) 0 var(--s-4); }
  .pl { display: grid; grid-template-columns: 84px 1fr auto 52px; align-items: center; gap: var(--s-3); height: 28px; }
  .pn { font-size: 14px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .pbar { height: 8px; border-radius: var(--r-chip); background: var(--track); overflow: hidden; }
  .pbar i { display: block; height: 100%; border-radius: inherit; transition: width 0.6s var(--ease); }
  .pr { font-size: 12px; color: var(--fg-3); white-space: nowrap; text-align: right; }
  .pp { font-size: 14px; font-weight: 600; text-align: right; font-variant-numeric: tabular-nums; }
  .hero { display: flex; align-items: center; gap: var(--s-4); padding: var(--s-2) 0 var(--s-4); }
  .huge { font-size: 32px; font-weight: 600; letter-spacing: -0.02em; line-height: 1.1; }
  .t2 { font-size: 12px; color: var(--fg-dim); }
  .sec { padding: var(--s-3) 0; border-top: 1px solid var(--border); display: flex; flex-direction: column; gap: 2px; }
  h4 { margin: 0 0 var(--s-2); font-size: 12px; font-weight: 500; color: var(--fg-dim); }
  .k { color: var(--fg-3); font-weight: 400; }
  .win { position: relative; display: grid; grid-template-columns: 96px 1fr auto; align-items: center; gap: var(--s-3); height: 32px; padding: 0 var(--s-3); border-radius: var(--r-inner); font-size: 12px; }
  .win.cur { background: var(--bg-2); font-weight: 600; }
  .win.cur::before { content: ''; position: absolute; left: 0; top: 8px; bottom: 8px; width: 3px; border-radius: 2px; background: var(--accent); }
  .bar { height: 4px; border-radius: 2px; background: var(--track); overflow: hidden; }
  .bar i { display: block; height: 100%; border-radius: inherit; transition: width 0.6s var(--ease); }
  .val { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .sub { color: var(--fg-3); font-weight: 400; margin-left: 4px; }
  .tiles { display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--s-2); }
  .tile { display: flex; flex-direction: column; gap: 2px; padding: var(--s-2) 10px; border-radius: var(--r-inner); background: var(--bg-2); }
  .tile .k { font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tile b { font-size: 14px; font-weight: 600; }
  .li { display: flex; justify-content: space-between; gap: var(--s-3); align-items: center; height: 28px; font-size: 12px; padding: 0 var(--s-1); }
  .note { font-size: 12px; color: var(--fg-dim); line-height: 1.5; margin: 0; padding: var(--s-3) 0; border-top: 1px solid var(--border); }
  footer { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; padding-top: var(--s-3); border-top: 1px solid var(--border); font-size: 11px; color: var(--fg-3); }
  footer .off { color: #ff9f0a; }
  .link { margin-left: auto; color: var(--accent); font-size: 11px; padding: 2px 4px; border-radius: 6px; }
  .link:hover { text-decoration: underline; }
  .link:focus-visible, .seg button:focus-visible, .ghost:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .seg { display: inline-flex; gap: 2px; padding: 2px; border-radius: var(--r-inner); background: var(--bg-2); }
  .seg button { padding: 2px 8px; border-radius: calc(8px * var(--rs)); font-size: 11px; }
  .seg button:hover:not(.sel) { background: var(--pill-hover); }
  .seg .sel { background: var(--accent); color: #fff; }
  .ghost { height: 32px; padding: 0 var(--s-3); border-radius: var(--r-inner); background: var(--bg-2); font-size: 12px; flex: none; }
  .ghost:hover { background: var(--pill-hover); }
  /* Skeleton shaped like the ring and rows. */
  .sk { display: block; background: var(--bg-2); border-radius: var(--r-inner); animation: sk 1.2s ease-in-out infinite; }
  .sk.l1 { width: 70%; height: 16px; }
  .sk.l2 { width: 50%; height: 12px; margin-top: 6px; }
  .sk.rowsk { height: 32px; margin: 2px 0; }
  @keyframes sk { 50% { opacity: 0.5; } }
  @media (prefers-reduced-motion: reduce) { .sk { animation: none; } }
</style>
