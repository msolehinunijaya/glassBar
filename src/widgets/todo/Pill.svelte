<script lang="ts">
  // Shows the next 2 (medium) or 3 (large) open to-dos with tick boxes right on the bar.
  // Click anywhere else on the pill to open the full list.
  import { ListChecks } from '@lucide/svelte';
  import { ipc, type Todo } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';
  import { progress } from './progress';

  let { size }: PillProps = $props();
  const todos = live<Todo[]>('todos', ipc.todoList, []);
  const open = $derived(todos.v.filter((t) => !t.done));
  // Due-dated first (soonest first), then the rest in list order.
  const ordered = $derived([...open].sort((a, b) => (a.dueAt ?? Infinity) - (b.dueAt ?? Infinity)));
  const shown = $derived(size === 'small' ? [] : ordered.slice(0, size === 'large' ? 3 : 2));
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

  function tick(e: Event, t: Todo) {
    e.stopPropagation();
    ipc.todoUpdate({ ...t, done: true });
  }
</script>

<div class="pill todo" class:complete={done === 1} class:celebrate title="{open.length} open to-dos">
  {#if done !== null}
    <svg class="ring" aria-hidden="true" preserveAspectRatio="none">
      <rect x="1" y="1" rx="13" ry="13" pathLength="100" style="width: calc(100% - 2px); height: calc(100% - 2px)"
            stroke-dasharray="{done * 100} 100" />
    </svg>
  {/if}
  <ListChecks size={16} />
  {#if size === 'small' || !open.length}
    <span class="n">{open.length || '✓'}</span>
  {/if}
  {#each shown as t (t.id)}
    <span class="item" class:late={t.dueAt && t.dueAt < now}>
      <input type="checkbox" onchange={(e) => tick(e, t)} onclick={(e) => e.stopPropagation()}
             onpointerdown={(e) => e.stopPropagation()} aria-label="Done: {t.text}" />
      <span class="ellipsis">{t.text}</span>
    </span>
  {/each}
  {#if shown.length && open.length > shown.length}<span class="dim more">+{open.length - shown.length}</span>{/if}
</div>

<style>
  .todo { gap: 6px; position: relative; }
  .ring { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }
  .ring rect { fill: none; stroke: var(--accent); stroke-width: 2; stroke-linecap: round; transition: stroke-dasharray 0.3s var(--ease), stroke 0.3s; }
  .complete .ring rect { stroke: #30d158; }
  .celebrate { animation: celebrate 0.9s var(--spring); }
  @keyframes celebrate { 30% { transform: scale(1.08); box-shadow: 0 0 0 4px rgba(48, 209, 88, 0.35); } }
  .n { font-weight: 600; font-variant-numeric: tabular-nums; }
  .item { display: flex; align-items: center; gap: 5px; max-width: 130px; padding: 3px 8px 3px 5px; border-radius: 9px; background: var(--pill); font-size: 12px; }
  .item:hover { background: var(--pill-hover); }
  .item input { width: 14px; height: 14px; margin: 0; accent-color: var(--accent); flex: none; }
  .late { color: #ff453a; }
  .more { font-size: 11px; }
</style>
