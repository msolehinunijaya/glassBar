<script lang="ts">
  import { CalendarClock, Plus, Trash2 } from '@lucide/svelte';
  import { ipc, type Todo } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { PopupProps } from '../types';

  let {}: PopupProps = $props();
  const todos = live<Todo[]>('todos', ipc.todoList, []);
  let text = $state('');
  let due = $state('');
  let showDue = $state(false);
  let error = $state('');

  // <input type="datetime-local"> works in local time without a zone.
  const toLocalInput = (ms: number) => {
    const d = new Date(ms);
    d.setMinutes(d.getMinutes() - d.getTimezoneOffset());
    return d.toISOString().slice(0, 16);
  };
  const fmtDue = (ms: number) =>
    new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' }).format(ms);

  async function add() {
    if (!text.trim()) return;
    try {
      await ipc.todoAdd(text, due ? new Date(due).getTime() : null);
      text = ''; due = ''; showDue = false; error = '';
    } catch (e) { error = String(e); }
  }
  const save = (t: Todo, patch: Partial<Todo>) => ipc.todoUpdate({ ...t, ...patch });
</script>

<h3>To-Do</h3>
<div class="add card">
  <input placeholder="Add a task…" bind:value={text} onkeydown={(e) => e.key === 'Enter' && add()} />
  <button class="icon-btn" class:on={showDue} onclick={() => (showDue = !showDue)} aria-label="Set due date"><CalendarClock size={16} /></button>
  <button class="icon-btn" onclick={add} aria-label="Add"><Plus size={16} /></button>
</div>
{#if showDue}<input class="due-in" type="datetime-local" bind:value={due} />{/if}
{#if error}<p class="err">{error}</p>{/if}

<div class="list">
  {#each todos.v as t (t.id)}
    <div class="row" class:done={t.done}>
      <input type="checkbox" checked={t.done} onchange={(e) => save(t, { done: e.currentTarget.checked })} aria-label="Done" />
      <div class="col">
        <input class="text" value={t.text} onchange={(e) => save(t, { text: e.currentTarget.value })} aria-label="Task" />
        <label class="due" class:late={!t.done && t.dueAt && t.dueAt < Date.now()}>
          {#if t.dueAt}{fmtDue(t.dueAt)}{:else}<span class="dim">No due date</span>{/if}
          <input type="datetime-local" value={t.dueAt ? toLocalInput(t.dueAt) : ''}
                 onchange={(e) => save(t, { dueAt: e.currentTarget.value ? new Date(e.currentTarget.value).getTime() : null })} />
        </label>
      </div>
      <button class="icon-btn del" onclick={() => ipc.todoDelete(t.id)} aria-label="Delete"><Trash2 size={14} /></button>
    </div>
  {:else}
    <p class="dim empty">Nothing to do. Nice.</p>
  {/each}
</div>
{#if todos.v.some((t) => t.done)}
  <button class="link" onclick={() => ipc.todoClearDone()}>Clear completed</button>
{/if}

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; }
  .add { display: flex; align-items: center; gap: 4px; padding: 4px 4px 4px 12px; }
  .add input { flex: 1; border: 0; background: none; outline: none; padding: 6px 0; }
  .icon-btn.on { color: var(--accent); }
  .due-in { margin-top: 6px; width: 100%; padding: 6px 10px; border-radius: calc(10px * var(--rs)); border: 1px solid var(--line-soft); background: var(--pill); }
  .list { display: flex; flex-direction: column; gap: 2px; margin-top: 10px; }
  .row { display: flex; align-items: flex-start; gap: 8px; padding: 6px 4px; border-radius: calc(10px * var(--rs)); }
  .row:hover { background: var(--pill); }
  .row input[type='checkbox'] { margin-top: 4px; accent-color: var(--accent); width: 16px; height: 16px; }
  .col { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .text { border: 0; background: none; outline: none; padding: 1px 0; }
  .done .text { text-decoration: line-through; opacity: 0.55; }
  .due { position: relative; font-size: 11px; color: var(--fg-dim); cursor: pointer; width: fit-content; }
  .due input { position: absolute; inset: 0; opacity: 0; cursor: pointer; }
  .due.late { color: #ff453a; }
  .del { width: 26px; height: 26px; opacity: 0; }
  .row:hover .del { opacity: 0.7; }
  .link { color: var(--accent); font-size: 12px; margin-top: 8px; }
  .empty { text-align: center; padding: 40px 0; }
  .err { color: #ff453a; font-size: 12px; }
</style>
