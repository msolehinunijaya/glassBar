<script lang="ts">
  import { ListChecks } from '@lucide/svelte';
  import { ipc, type Todo } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PillProps } from '../types';

  let { size }: PillProps = $props();
  const todos = live<Todo[]>('todos', ipc.todoList, []);
  const open = $derived(todos.v.filter((t) => !t.done));
  const next = $derived(open.filter((t) => t.dueAt).sort((a, b) => a.dueAt! - b.dueAt!)[0] ?? open[0]);
  const overdue = $derived(open.some((t) => t.dueAt && t.dueAt < Date.now()));
</script>

<div class="pill" title="{open.length} open to-dos">
  <ListChecks size={16} />
  <span class="n" class:overdue>{open.length}</span>
  {#if size !== 'small' && next}<span class="ellipsis next">{next.text}</span>{/if}
</div>

<style>
  .n { font-weight: 600; font-variant-numeric: tabular-nums; }
  .overdue { color: #ff453a; }
  .next { max-width: 150px; font-size: 12px; }
</style>
