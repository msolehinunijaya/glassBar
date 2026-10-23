<script lang="ts">
  // Settings › Widgets: see the dock, organise widgets by purpose, change size, order and
  // options. Everything saves at once and the real bar follows.
  import { flip } from 'svelte/animate';
  import { ArrowDown, ArrowUp, Plus, RotateCcw, Search, Settings2, Trash2, Undo2 } from '@lucide/svelte';
  import { profile, update } from '../../lib/config.svelte';
  import type { Edge } from '../../lib/geometry';
  import type { WidgetInst, WidgetSize } from '../../lib/ipc';
  import { addWidget, widgets } from '../../widgets/registry';
  import { CATEGORIES, categoryOf, type CategoryId } from '../../widgets/categories';
  import { applyLayout, counts, DEFAULT_LAYOUT, isOn, matches, PRESETS, presetItems, stateOf, visibleOnDock, type Filter, type LayoutItem } from '../../widgets/layout';
  import { reorder } from '../../lib/geometry';
  import DockPreview from './DockPreview.svelte';
  import Mini from './Mini.svelte';
  import WidgetDrawer from './WidgetDrawer.svelte';
  import { dragSort, dragged } from './drag';

  const all = Object.values(widgets).sort((a, b) => a.name.localeCompare(b.name));
  const bar = $derived(profile().bar);
  const placed = $derived(profile().widgets.filter((w) => widgets[w.id]));
  const setWidgets = (f: (list: WidgetInst[]) => WidgetInst[]) =>
    update((c) => { const p = c.profiles[c.activeProfile]; p.widgets = f(p.widgets); });
  const edit = (instance: string, f: (w: WidgetInst) => void) =>
    update((c) => { const t = c.profiles[c.activeProfile].widgets.find((w) => w.instance === instance); if (t) f(t); });

  // ---- preview: orientation, presets (shown first, applied on confirm), undo ----
  let edge = $state<Edge>(profile().bar.edge);
  let pending = $state<{ name: string; list: WidgetInst[] } | null>(null);
  let undo = $state<WidgetInst[] | null>(null);
  const previewItems = $derived(visibleOnDock(pending?.list ?? placed, widgets));
  function propose(name: string, items: LayoutItem[]) { pending = { name, list: applyLayout(profile().widgets, items, widgets) }; }
  function applyPending() {
    if (!pending) return;
    undo = $state.snapshot(profile().widgets);
    const list = pending.list;
    setWidgets(() => list);
    pending = null;
  }
  function doUndo() { if (undo) { const u = undo; setWidgets(() => u); undo = null; } }
  // Preview reorder: the widgets that are on take the new order; the ones that are off follow.
  function reorderOn(instances: string[]) {
    setWidgets((l) => [...instances.map((i) => l.find((w) => w.instance === i)!).filter(Boolean), ...l.filter((w) => !instances.includes(w.instance))]);
  }

  // ---- browsing ----
  let q = $state<{ text: string; category: CategoryId | 'all'; filter: Filter }>({ text: '', category: 'all', filter: 'all' });
  const tally = $derived(counts(all, placed));
  const shownCats = CATEGORIES.filter((c) => all.some((m) => categoryOf(m) === c.id));
  const onDock = $derived(placed.filter((w) => matches(widgets[w.id], placed, q)));
  const toAdd = $derived(all.filter((m) => stateOf(m.id, placed) === 'available' && matches(m, placed, q)));
  const FILTERS: [Filter, string][] = [['all', 'All'], ['enabled', 'On'], ['disabled', 'Off'], ['available', 'Not added']];
  const SIZE: Record<WidgetSize, string> = { small: 'S', medium: 'M', large: 'L' };
  const SIZE_NAME: Record<WidgetSize, string> = { small: 'Small', medium: 'Medium', large: 'Large' };

  let selected = $state<string | null>(null);

  // ---- card grid drag ----
  let grid = $state<HTMLDivElement>();
  let order = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  const cards = $derived(order ? order.map((i) => onDock.find((w) => w.instance === i)!).filter(Boolean) : onDock);
  function down(e: PointerEvent, inst: string) {
    if (!grid || q.text || q.filter !== 'all' || q.category !== 'all') return; // reorder the full list only
    dragSort(e, inst, placed.map((w) => w.instance), grid, 'grid',
      (o, d) => { order = o; lifted = d; if (d) dragged.at = Date.now(); },
      (o) => setWidgets((l) => o.map((i) => l.find((w) => w.instance === i)!).filter(Boolean).concat(l.filter((w) => !widgets[w.id]))));
  }
  const move = (i: number, d: number) => setWidgets((l) => {
    const from = l.findIndex((w) => w.instance === placed[i].instance), to = l.findIndex((w) => w.instance === placed[i + d]?.instance);
    return to < 0 ? l : reorder(l, from, to);
  });
  const open = (id: string) => { if (Date.now() - dragged.at > 250) selected = id; };
</script>

<!-- Preview -->
<DockPreview items={previewItems} {edge} size={bar.size} {selected} onselect={(id) => (selected = id)}
             banner={pending ? `Preview: ${pending.name}` : undefined} onreorder={pending ? undefined : reorderOn} />

<div class="bar-tools">
  <div class="seg" role="group" aria-label="Preview position">
    {#each ['top', 'bottom', 'left', 'right'] as const as e (e)}
      <button class:sel={edge === e} onclick={() => (edge = e)}>{e}</button>
    {/each}
  </div>
  {#if edge !== bar.edge || bar.floating}
    <button class="btn" onclick={() => update((c) => { const b = c.profiles[c.activeProfile].bar; b.edge = edge; b.floating = null; })}>Move my bar to the {edge}</button>
  {/if}
  <span class="spacer"></span>
  <select aria-label="Layout presets" value="" onchange={(e) => { const p = PRESETS.find((x) => x.id === e.currentTarget.value); if (p) propose(p.name, presetItems(p, all.map((m) => m.id))); e.currentTarget.value = ''; }}>
    <option value="" disabled>Layout presets…</option>
    {#each PRESETS as p (p.id)}<option value={p.id}>{p.name}: {p.description}</option>{/each}
  </select>
  <button class="btn plain" onclick={() => propose('Default layout', DEFAULT_LAYOUT)}><RotateCcw size={14} /> Restore default</button>
</div>

{#if pending}
  <div class="confirm" role="status">
    <span><b>{pending.name}</b> is shown above. Apply it? Widgets it leaves out are turned off, and every widget keeps its options.</span>
    <button class="btn" onclick={applyPending}>Apply layout</button>
    <button class="btn plain" onclick={() => (pending = null)}>Cancel</button>
  </div>
{:else if undo}
  <div class="confirm" role="status">
    <span>Layout applied.</span>
    <button class="btn plain" onclick={doUndo}><Undo2 size={14} /> Undo</button>
    <button class="btn plain" onclick={() => (undo = null)}>Dismiss</button>
  </div>
{/if}

<!-- Find -->
<div class="find">
  <label class="search"><Search size={14} /><input placeholder="Search widgets" bind:value={q.text} aria-label="Search widgets" /></label>
  <div class="seg plain-case" role="group" aria-label="Show">
    {#each FILTERS as [f, label] (f)}<button class:sel={q.filter === f} onclick={() => (q.filter = f)}>{label}</button>{/each}
  </div>
</div>
<div class="chips" role="group" aria-label="Categories">
  <button class="chip" class:sel={q.category === 'all'} onclick={() => (q.category = 'all')}>All widgets <span>{all.length}</span></button>
  {#each shownCats as c (c.id)}
    <button class="chip" class:sel={q.category === c.id} onclick={() => (q.category = c.id)}
            title="{tally[c.id]?.on ?? 0} of {tally[c.id]?.total ?? 0} on the dock">{c.name} <span>{tally[c.id]?.on ?? 0}/{tally[c.id]?.total ?? 0}</span></button>
  {/each}
</div>

<!-- On the dock -->
{#if q.filter !== 'available'}
  <h2>On the dock <span class="count">{placed.filter(isOn).length} on{placed.some((w) => !isOn(w)) ? `, ${placed.filter((w) => !isOn(w)).length} off` : ''}</span></h2>
  {#if cards.length}
    <div class="grid" bind:this={grid}>
      {#each cards as w (w.instance)}
        {@const m = widgets[w.id]}
        {@const i = placed.findIndex((x) => x.instance === w.instance)}
        <div class="card" role="group" aria-label={m.name} class:off={!isOn(w)} class:lifted={lifted === w.instance} data-id={w.instance} animate:flip={{ duration: 220 }}
             onpointerdown={(e) => down(e, w.instance)}>
          <button class="look" onclick={() => open(w.id)} aria-label="{m.name} details"><Mini id={w.id} inst={w} /></button>
          <div class="meta">
            <b>{m.name}</b>
            <input type="checkbox" role="switch" class="switch" checked={isOn(w)} aria-label="Show {m.name} on the dock"
                   onchange={(e) => edit(w.instance, (t) => (t.enabled = e.currentTarget.checked))} />
          </div>
          <div class="ctl">
            {#if m.sizes.length > 1}
              <div class="seg small" role="group" aria-label="{m.name} size">
                {#each m.sizes as s (s)}<button class:sel={w.size === s} title={SIZE_NAME[s]} aria-label={SIZE_NAME[s]} onclick={() => edit(w.instance, (t) => (t.size = s))}>{SIZE[s]}</button>{/each}
              </div>
            {:else}<span class="hint m0 nowrap">{SIZE_NAME[m.sizes[0]]} only</span>{/if}
            <span class="spacer"></span>
            <button class="icon-btn" disabled={i <= 0} onclick={() => move(i, -1)} aria-label="Move {m.name} earlier"><ArrowUp size={14} /></button>
            <button class="icon-btn" disabled={i >= placed.length - 1} onclick={() => move(i, 1)} aria-label="Move {m.name} later"><ArrowDown size={14} /></button>
            <button class="icon-btn" onclick={() => (selected = w.id)} aria-label="{m.name} options"><Settings2 size={14} /></button>
            <button class="icon-btn" onclick={() => (selected = w.id)} aria-label="Remove {m.name}…"><Trash2 size={14} /></button>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="hint">No widget on the dock matches. Clear the search or pick another category.</p>
  {/if}
{/if}

<!-- Add -->
{#if q.filter === 'all' || q.filter === 'available'}
  <h2>Add widgets</h2>
  {#each shownCats as c (c.id)}
    {@const list = toAdd.filter((m) => categoryOf(m) === c.id)}
    {#if list.length}
      <h3>{c.name}</h3>
      <div class="grid">
        {#each list as m (m.id)}
          <div class="card avail">
            <button class="look" onclick={() => (selected = m.id)} aria-label="{m.name} details"><Mini id={m.id} /></button>
            <b>{m.name}</b>
            <span class="hint m0 desc">{m.description}</span>
            <button class="btn add" onclick={() => addWidget(m.id)}><Plus size={14} /> Add to dock</button>
          </div>
        {/each}
      </div>
    {/if}
  {/each}
  {#if !toAdd.length}<p class="hint">{q.text || q.category !== 'all' ? 'Nothing to add matches.' : 'Every widget is on the dock.'}</p>{/if}
{/if}

{#if selected}<WidgetDrawer id={selected} onclose={() => (selected = null)} />{/if}

<style>
  .bar-tools, .find, .confirm { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  .bar-tools { margin: 10px 0 12px; }
  .bar-tools select { max-width: 220px; }
  .spacer { flex: 1; }
  .btn.plain { background: var(--pill); color: var(--fg); }
  .confirm { padding: 10px 14px; border-radius: 12px; background: color-mix(in srgb, var(--accent) 14%, var(--panel)); border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent); margin-bottom: 12px; }
  .confirm span { flex: 1 1 260px; font-size: 13px; }
  .find { margin-top: 6px; }
  .search { flex: 1 1 220px; display: flex; align-items: center; gap: 6px; padding: 0 10px; border-radius: 9px; border: 1px solid var(--line-soft); background: var(--panel); color: var(--fg-dim); }
  .search input { flex: 1; border: 0; background: none; padding: 7px 0; outline: none; color: var(--fg); }
  .search:focus-within { border-color: var(--accent); }
  .chips { display: flex; flex-wrap: wrap; gap: 6px; margin: 10px 0 4px; }
  .chip { padding: 4px 10px; border-radius: 999px; font-size: 12px; background: var(--pill); }
  .chip span { color: var(--fg-dim); margin-left: 2px; font-variant-numeric: tabular-nums; }
  .chip.sel { background: var(--accent); color: #fff; }
  .chip.sel span { color: rgba(255, 255, 255, 0.8); }
  .count { font-weight: 400; font-size: 12px; color: var(--fg-dim); margin-left: 6px; }
  h3 { font-size: 12px; font-weight: 600; color: var(--fg-dim); margin: 14px 0 6px; }
  .m0 { margin: 0; }
  .nowrap { white-space: nowrap; }

  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 8px; }
  .card { display: flex; flex-direction: column; gap: 6px; padding: 8px 10px 8px; border-radius: 14px; background: var(--panel); border: 1px solid var(--line-soft); min-width: 0; touch-action: none; }
  .card.off { background: transparent; border-style: dashed; }
  .card.off .look, .card.off .meta b { opacity: 0.45; }
  .card.lifted { opacity: 0.6; outline: 2px dashed color-mix(in srgb, var(--accent) 70%, transparent); }
  .look { display: flex; justify-content: flex-start; padding: 6px; border-radius: 10px; background: rgba(var(--tint), 0.55); border: 1px solid var(--line-soft); overflow: hidden; min-width: 0;
          -webkit-mask-image: linear-gradient(90deg, #000 85%, transparent); mask-image: linear-gradient(90deg, #000 85%, transparent); }
  .look:focus-visible, .chip:focus-visible, .card button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .meta { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .meta b, .avail b { font-weight: 500; }
  .ctl { display: flex; align-items: center; gap: 2px; }
  .plain-case button { text-transform: none; }
  .seg.small button { padding: 2px 9px; font-size: 12px; text-transform: none; }
  .avail .desc { display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
  .add { align-self: flex-start; padding: 5px 10px; font-size: 12px; }
</style>
