<script lang="ts">
  // Details for one widget: its look at every size (click one to use it), on/off, its own
  // options, reset, remove — or "Add to dock" when it isn't there yet.
  import { untrack } from 'svelte';
  import { Plus, RotateCcw, Trash2, X } from '@lucide/svelte';
  import { profile, update, updateWidgetSettings } from '../../lib/config.svelte';
  import type { WidgetSize } from '../../lib/ipc';
  import { addWidget, settingsOf, widgets } from '../../widgets/registry';
  import { CATEGORIES, categoryOf } from '../../widgets/categories';
  import { isOn } from '../../widgets/layout';
  import Mini from './Mini.svelte';

  let { id, onclose }: { id: string; onclose: () => void } = $props();
  const m = $derived(widgets[id]);
  const w = $derived(profile().widgets.find((x) => x.id === id));
  const on = $derived(!!w && isOn(w));
  const SIZE: Record<WidgetSize, string> = { small: 'Small', medium: 'Medium', large: 'Large' };
  let confirm = $state<'reset' | 'remove' | null>(null);
  $effect(() => { id; untrack(() => (confirm = null)); });

  const edit = (f: (x: NonNullable<typeof w>) => void) =>
    update((c) => { const t = c.profiles[c.activeProfile].widgets.find((x) => x.id === id); if (t) f(t); });
  const remove = () => { update((c) => { const p = c.profiles[c.activeProfile]; p.widgets = p.widgets.filter((x) => x.id !== id); }); onclose(); };

  let closeBtn = $state<HTMLButtonElement>();
  $effect(() => { id; closeBtn?.focus(); });
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />
<button class="scrim" aria-label="Close details" onclick={onclose}></button>
<aside class="drawer" aria-label="{m.name} details">
  <header>
    <span class="wico"><m.icon size={16} /></span>
    <div class="title"><b>{m.name}</b><span class="hint m0">{CATEGORIES.find((c) => c.id === categoryOf(m))?.name}</span></div>
    <button class="icon-btn" bind:this={closeBtn} onclick={onclose} aria-label="Close"><X size={16} /></button>
  </header>
  <p class="desc">{m.description}</p>

  <h3>{m.sizes.length > 1 ? 'Size' : 'Look'}</h3>
  <div class="sizes">
    {#each m.sizes as s (s)}
      <button class="size" class:sel={(w?.size ?? m.defaultSize) === s} disabled={!w || m.sizes.length < 2}
              aria-pressed={(w?.size ?? m.defaultSize) === s} onclick={() => edit((t) => (t.size = s))}>
        <Mini {id} inst={w} size={s} height={40} />
        <span>{SIZE[s]}</span>
      </button>
    {/each}
  </div>

  {#if w}
    <label class="field row"><span>Show on the dock</span>
      <input type="checkbox" role="switch" class="switch" checked={on} onchange={(e) => edit((t) => (t.enabled = e.currentTarget.checked))} />
    </label>
    <p class="hint">Off keeps its place and settings, and stops its background work.</p>

    {#if m.SettingsForm}
      <h3>Options</h3>
      <div class="wform"><m.SettingsForm instance={w.instance} settings={settingsOf(m, w)} save={(p) => updateWidgetSettings(w.instance, p)} /></div>
    {/if}

    <div class="foot">
      {#if confirm === 'reset'}
        <span class="hint m0">Reset all of {m.name}'s options?</span>
        <button class="btn" onclick={() => { edit((t) => (t.settings = {})); confirm = null; }}>Reset</button>
        <button class="btn plain" onclick={() => (confirm = null)}>Cancel</button>
      {:else if confirm === 'remove'}
        <span class="hint m0">Remove it and forget its options? Turning it off keeps them.</span>
        <button class="btn danger" onclick={remove}>Remove</button>
        <button class="btn plain" onclick={() => (confirm = null)}>Cancel</button>
      {:else}
        <button class="btn plain" onclick={() => (confirm = 'reset')}><RotateCcw size={14} /> Reset to defaults</button>
        <button class="btn plain" onclick={() => (confirm = 'remove')}><Trash2 size={14} /> Remove from dock</button>
      {/if}
    </div>
  {:else}
    <button class="btn add" onclick={() => addWidget(id)}><Plus size={14} /> Add to dock</button>
    <p class="hint">It goes at the end of the dock. Drag it in the preview to move it.</p>
  {/if}
</aside>

<style>
  .scrim { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.25); z-index: 20; animation: fade 0.18s var(--ease); }
  .drawer {
    position: fixed; top: 0; right: 0; bottom: 0; width: min(400px, 100%); z-index: 21; overflow: auto;
    background: var(--panel); border-left: 1px solid var(--line-soft); box-shadow: var(--shadow);
    padding: 16px 18px 24px; display: flex; flex-direction: column; gap: 8px;
    animation: slide 0.24s var(--ease);
  }
  @keyframes slide { from { translate: 24px 0; opacity: 0; } }
  @keyframes fade { from { opacity: 0; } }
  @media (prefers-reduced-motion: reduce) { .drawer, .scrim { animation: none; } }
  header { display: flex; align-items: center; gap: 10px; }
  .title { flex: 1; display: flex; flex-direction: column; }
  .title b { font-size: 16px; font-weight: 600; }
  .m0 { margin: 0; }
  .desc { margin: 2px 0 4px; color: var(--fg-dim); }
  h3 { font-size: 13px; font-weight: 600; margin: 10px 0 2px; }
  .sizes { display: flex; flex-direction: column; gap: 6px; }
  .size { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 6px 10px 6px 6px; border-radius: 12px; background: var(--settings-bg); border: 1px solid transparent; min-width: 0; }
  .size > span { flex: none; font-size: 12px; color: var(--fg-dim); }
  .size.sel { border-color: var(--accent); }
  .size.sel > span { color: var(--accent); font-weight: 600; }
  .size:disabled { opacity: 1; }
  .row { border-top: 1px solid var(--line-soft); margin-top: 6px; }
  .foot { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-top: 12px; padding-top: 12px; border-top: 1px solid var(--line-soft); }
  .btn.plain { background: var(--pill); color: var(--fg); }
  .btn.danger { background: #ff453a; }
  .add { align-self: flex-start; margin-top: 8px; }
  .drawer :global(.wform) { padding-left: 0; }
</style>
