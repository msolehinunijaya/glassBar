<script lang="ts">
  // A small "screen" with the dock on its edge, drawn by the widgets' real dock components.
  // Scaled down to fit. Drag a widget to reorder; click one to open its details.
  import { flip } from 'svelte/animate';
  import type { Edge } from '../../lib/geometry';
  import { SIDE_PX, type SideWidth, type WidgetInst } from '../../lib/ipc';
  import { settingsOf, widgets } from '../../widgets/registry';
  import { dragSort, dragged } from './drag';

  let { items, edge, size, side = 'regular', selected, banner, onselect, onreorder }: {
    items: WidgetInst[]; edge: Edge; size: 'compact' | 'normal' | 'large'; side?: SideWidth; selected: string | null;
    banner?: string; onselect: (id: string) => void; onreorder?: (instances: string[]) => void;
  } = $props();

  const vertical = $derived(edge === 'left' || edge === 'right');
  const thick = $derived(vertical ? SIDE_PX[side] : { compact: 36, normal: 48, large: 60 }[size]);
  let order = $state<string[] | null>(null);
  let dragging = $state<string | null>(null);
  const shown = $derived(order ? order.map((i) => items.find((w) => w.instance === i)!).filter(Boolean) : items);

  let stage = $state<HTMLDivElement>();
  let dock = $state<HTMLDivElement>();
  let sw = $state(0), sh = $state(0), dw = $state(0), dh = $state(0);
  // Shrink to fit, but never below 60% (still readable); past that the stage scrolls.
  const k = $derived(Math.max(0.6, Math.min(1, vertical ? (sh - 24) / Math.max(dh, 1) : (sw - 24) / Math.max(dw, 1))));

  function down(e: PointerEvent, inst: string) {
    if (!onreorder || !dock) return;
    dragSort(e, inst, items.map((w) => w.instance), dock, vertical ? 'y' : 'x',
      (o, d) => { order = o; dragging = d; if (d) dragged.at = Date.now(); },
      (o) => onreorder!(o));
  }
  const click = (id: string) => { if (Date.now() - dragged.at > 250) onselect(id); };
</script>

<div class="wrap {edge}">
{#if banner}<div class="banner">{banner}</div>{/if}
<div class="stage {edge}" bind:this={stage} bind:clientWidth={sw} bind:clientHeight={sh}>
  <div class="fit" style:zoom={k}>
    <div class="pv dock" class:vertical data-size={size} data-side={side} bind:this={dock} bind:offsetWidth={dw} bind:offsetHeight={dh}
         style:height={vertical ? 'auto' : `${thick}px`} style:width={vertical ? `${thick}px` : 'auto'}>
      {#each shown as w (w.instance)}
        {@const m = widgets[w.id]}
        <div class="slot" class:sel={selected === w.id} class:lifted={dragging === w.instance} data-id={w.instance}
             role="button" tabindex="-1" title={m.name} animate:flip={{ duration: 220 }}
             onpointerdown={(e) => down(e, w.instance)} onclick={() => click(w.id)} onkeydown={() => {}}>
          <div class="inert" inert><m.Pill instance={w.instance} size={vertical ? 'small' : w.size} settings={settingsOf(m, w)} {vertical} /></div>
        </div>
      {:else}
        <span class="empty">No widgets on the dock</span>
      {/each}
    </div>
  </div>
</div>

</div>
<style>
  /* The "screen": the settings background, a faint frame, the dock on one edge. */
  .stage {
    position: relative; height: 132px; border-radius: 16px; overflow: auto;
    background:
      radial-gradient(120% 90% at 20% 0%, color-mix(in srgb, var(--accent) 22%, transparent), transparent 60%),
      radial-gradient(90% 80% at 100% 100%, color-mix(in srgb, var(--accent) 12%, transparent), transparent 55%),
      var(--settings-bg);
    border: 1px solid var(--line-soft);
    display: flex; justify-content: safe center;
  }
  .stage.top { align-items: flex-start; }
  .stage.bottom { align-items: flex-end; }
  .stage.top, .stage.bottom { overflow-y: hidden; }
  .stage.left, .stage.right { height: 360px; align-items: safe center; overflow-x: hidden; }
  .stage.left { justify-content: flex-start; }
  .stage.right { justify-content: flex-end; }
  .fit { margin: 12px; flex: none; }

  /* The dock itself: the bar's tint and radius, solid enough to read on any background. */
  .pv {
    display: flex; align-items: stretch; gap: var(--gap); padding: 5px;
    background: rgba(var(--tint), max(var(--tint-alpha), 0.72));
    border: 1px solid var(--line); border-radius: var(--radius-bar); box-shadow: var(--shadow);
  }
  .pv[data-size='compact'] { padding: 4px; }
  .pv.vertical { flex-direction: column; }
  .slot { border-radius: var(--radius-pill); cursor: grab; touch-action: none; }
  .slot .inert { display: flex; width: 100%; }
  .slot.sel { outline: 2px solid var(--accent); outline-offset: 1px; }
  .slot.lifted { opacity: 0.55; cursor: grabbing; }
  .empty { align-self: center; padding: 0 14px; color: var(--fg-dim); font-size: 12px; }
  .banner {
    position: absolute; left: 50%; top: 50%; translate: -50% -50%; z-index: 1; pointer-events: none;
    font-size: 12px; padding: 4px 12px; border-radius: 999px; background: var(--panel); color: var(--fg-dim); border: 1px solid var(--line-soft);
  }
  .wrap { position: relative; }
  .left .banner, .right .banner { top: auto; bottom: 10px; translate: -50% 0; }
  @media (prefers-reduced-motion: reduce) { .slot { transition: none; } }
</style>
