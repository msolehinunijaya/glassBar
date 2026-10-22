<script lang="ts">
  import { X } from '@lucide/svelte';
  import type { SettingsProps } from '../types';
  import type { AppsSettings } from './index';
  import { nameOf } from './groups';

  let { settings, save }: SettingsProps<AppsSettings> = $props();
  let path = $state('');

  function add() {
    const p = path.trim().replace(/^"|"$/g, '');
    if (p) save({ pinned: [...settings.pinned, { path: p }] });
    path = '';
  }
</script>

<label><input type="checkbox" checked={settings.showTitles} onchange={(e) => save({ showTitles: e.currentTarget.checked })} /> Show active window title (large size)</label>
<p class="dim">Pinned apps — tip: right-click a running app in the bar and choose "Pin to bar".</p>
<ul>
  {#each settings.pinned as p, i (p.path + i)}
    <li>
      <span class="ellipsis" title={p.path}><b>{nameOf(p.path)}</b> <span class="dim">{p.path}</span></span>
      <button class="icon-btn" aria-label="Unpin" onclick={() => save({ pinned: settings.pinned.filter((_, j) => j !== i) })}><X size={14} /></button>
    </li>
  {/each}
</ul>
<div class="row">
  <input placeholder="C:\Program Files\App\app.exe" bind:value={path} onkeydown={(e) => e.key === 'Enter' && add()} />
  <button class="btn" onclick={add}>Pin</button>
</div>

<style>
  ul { list-style: none; padding: 0; margin: 4px 0; }
  li { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .row { display: flex; gap: 6px; }
  .row input { flex: 1; }
</style>
