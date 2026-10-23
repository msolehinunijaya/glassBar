<script lang="ts">
  // Medium/large: a one-row window onto the open to-dos. Mouse-wheel (or the arrows) scrolls
  // through every task right on the dock; "2/5" shows where you are. Small: icon + count.
  // Click the pill outside the list to open the full panel.
  import { ChevronDown, ChevronUp } from '@lucide/svelte';
  import TodoIcon3D from '../../lib/icons/TodoIcon3D.svelte';
  import { ipc, type Todo } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';
  import { progress } from './progress';

  let { size, vertical }: PillProps = $props();
  const todos = live<Todo[]>('todos', ipc.todoList, []);
  const open = $derived(todos.v.filter((t) => !t.done));
  // Due-dated first (soonest first), then the rest in list order.
  const ordered = $derived([...open].sort((a, b) => (a.dueAt ?? Infinity) - (b.dueAt ?? Infinity)));
  const list = $derived(size !== 'small' && !vertical);
  let at = $state(0); // index of the task in view
  $effect(() => { if (at > Math.max(0, ordered.length - 1)) at = Math.max(0, ordered.length - 1); });
  const now = Date.now();
  // Outer ring = done / total. Hidden with no to-dos; green with a little pulse at 100%.
  const done = $derived(progress(todos.v));
  let celebrate = $state(false);
  let wasComplete = false;
  $effect(() => {
    const complete = done === 1;
    if (complete && !wasComplete) { celebrate = true; setTimeout(() => (celebrate = false), 900); }
    wasComplete = complete;
  });

  let lastWheel = 0;
  function wheel(e: WheelEvent) {
    if (ordered.length < 2) return;
    e.preventDefault(); // keep the bar itself from scrolling
    e.stopPropagation();
    const t = performance.now();
    if (t - lastWheel < 120) return; // one task per notch, even on smooth-scroll mice
    lastWheel = t;
    at = Math.max(0, Math.min(ordered.length - 1, at + Math.sign(e.deltaY || e.deltaX)));
  }
  const step = (e: Event, by: number) => { e.stopPropagation(); at = Math.max(0, Math.min(ordered.length - 1, at + by)); };
  function tick(e: Event, t: Todo) {
    e.stopPropagation();
    ipc.todoUpdate({ ...t, done: true });
  }
  const stop = (e: Event) => e.stopPropagation();
</script>

{#if vertical}
<div class="side" title="{open.length} open to-dos">
  <TodoIcon3D size={20} />
  <span class="side-v">{open.length || '✓'}</span>
  {#if done !== null}<span class="side-meter"><i style:width="{done * 100}%" style:--c={done === 1 ? '#30d158' : 'var(--accent)'}></i></span>{/if}
</div>
{:else}
<div class="pill todo" class:complete={done === 1} class:celebrate title="{open.length} open to-dos">
  {#if done !== null}
    <svg class="ring" aria-hidden="true" preserveAspectRatio="none">
      <rect x="1" y="1" rx="13" ry="13" pathLength="100" style="width: calc(100% - 2px); height: calc(100% - 2px)"
            stroke-dasharray="{done * 100} 100" />
    </svg>
  {/if}
  <TodoIcon3D size={20} />
  {#if !list || !ordered.length}
    <span class="n">{open.length || '✓'}</span>
  {:else}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="view" class:more-up={at > 0} class:more-down={at < ordered.length - 1} onwheel={wheel} onpointerdown={stop}
         style:--w="{size === 'large' ? 190 : 140}px">
      <div class="track" style:transform="translateY({-at * 22}px)">
        {#each ordered as t (t.id)}
          <label class="item" class:late={t.dueAt && t.dueAt < now} title={t.text}>
            <input type="checkbox" onchange={(e) => tick(e, t)} onclick={stop} aria-label="Done: {t.text}" />
            <span class="ellipsis">{t.text}</span>
          </label>
        {/each}
      </div>
    </div>
    {#if ordered.length > 1}
      <span class="nav" onpointerdown={stop}>
        <button class="arrow" disabled={at === 0} onclick={(e) => step(e, -1)} aria-label="Previous task"><ChevronUp size={11} /></button>
        <span class="pos">{at + 1}/{ordered.length}</span>
        <button class="arrow" disabled={at === ordered.length - 1} onclick={(e) => step(e, 1)} aria-label="Next task"><ChevronDown size={11} /></button>
      </span>
    {/if}
  {/if}
</div>

{/if}
<style>
  .todo { gap: 7px; position: relative; }
  .ring { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }
  .ring rect { fill: none; stroke: var(--accent); stroke-width: 2; stroke-linecap: round; transition: stroke-dasharray 0.3s var(--ease), stroke 0.3s; }
  .complete .ring rect { stroke: #30d158; }
  .celebrate { animation: celebrate 0.9s var(--spring); }
  @keyframes celebrate { 30% { transform: scale(1.08); box-shadow: 0 0 0 4px rgba(48, 209, 88, 0.35); } }
  .n { font-weight: 600; font-variant-numeric: tabular-nums; }
  /* fixed-size window: the dock never stretches, however many tasks there are */
  .view { position: relative; width: var(--w); height: 22px; overflow: hidden; border-radius: 9px; background: var(--pill); }
  .view::before, .view::after { content: ''; position: absolute; left: 0; right: 0; height: 4px; z-index: 1; pointer-events: none; opacity: 0; transition: opacity 0.2s; }
  .view::before { top: 0; background: linear-gradient(var(--fg), transparent); }
  .view::after { bottom: 0; background: linear-gradient(transparent, var(--fg)); }
  .view.more-up::before, .view.more-down::after { opacity: 0.18; }
  .track { transition: transform 0.28s var(--spring); }
  .item { display: flex; align-items: center; gap: 6px; height: 22px; padding: 0 8px 0 5px; font-size: 12px; cursor: default; }
  .item:hover { background: var(--pill-hover); }
  .item input { width: 14px; height: 14px; margin: 0; accent-color: var(--accent); flex: none; }
  .late { color: #ff453a; }
  .nav { display: flex; flex-direction: column; align-items: center; line-height: 1; }
  .arrow { display: grid; place-items: center; height: 11px; width: 18px; border-radius: 4px; opacity: 0.7; }
  .arrow:hover:not(:disabled) { opacity: 1; background: var(--pill-hover); }
  .arrow:disabled { opacity: 0.2; }
  .pos { font-size: 9.5px; font-variant-numeric: tabular-nums; color: var(--fg-dim); }
</style>
