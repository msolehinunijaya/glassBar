<script lang="ts">
  import { Plus } from '@lucide/svelte';
  import { addWidget, widgets } from '../widgets/registry';

  let { close = () => {} }: { close?: () => void } = $props();
</script>

<h3>Add widget</h3>
<div class="list">
  {#each Object.values(widgets) as m (m.id)}
    <button class="row" onclick={() => { addWidget(m.id); close(); }}>
      <span class="ico"><m.icon size={18} /></span>
      <span class="text">
        <b>{m.name}</b>
        <span class="dim">{m.description}</span>
      </span>
      <Plus size={16} />
    </button>
  {/each}
</div>

<style>
  h3 { margin: 0 0 10px; font-size: 15px; font-weight: 600; }
  .list { display: flex; flex-direction: column; gap: 6px; }
  .row {
    display: flex; align-items: center; gap: 10px; text-align: left;
    padding: 8px 10px; border-radius: 14px; background: var(--pill);
    border: 1px solid var(--line-soft);
    transition: background 0.2s var(--ease), transform 0.3s var(--spring);
  }
  .row:hover { background: var(--pill-hover); }
  .row:active { transform: scale(0.97); }
  .ico { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 10px; background: var(--accent); color: #fff; flex: none; }
  .text { display: flex; flex-direction: column; flex: 1; min-width: 0; font-size: 12px; }
  .text b { font-weight: 500; font-size: 13px; }
</style>
