<script lang="ts">
  // Pomodoro | To-Do | Notes as joined segments; each opens its own panel. Right-click to
  // choose which show. With all three hidden, the widget says how to bring them back.
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import { showMenu } from '../../lib/menu';
  import type { PillProps } from '../types';
  import type { ProductivitySettings } from './index';
  import { PARTS, partPanel, type Part } from './parts';

  let { instance, size, settings, vertical }: PillProps<ProductivitySettings> = $props();
  const shown = $derived(PARTS.filter((p) => settings[p.show] !== false));

  function open(e: MouseEvent, p: Part) {
    e.stopPropagation();
    ipc.popupOpen(partPanel(p.key, instance), (e.currentTarget as HTMLElement).getBoundingClientRect(), p.width, p.height);
  }
  async function menu(e: MouseEvent) {
    e.preventDefault(); e.stopPropagation();
    await showMenu([{ text: 'Show on the dock', items: PARTS.map((p) => ({
      text: p.label, checked: settings[p.show] !== false,
      action: () => updateWidgetSettings(instance, { [p.show]: settings[p.show] === false }),
    })) }]);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="segwrap" class:vertical oncontextmenu={menu}>
  {#each shown as p (p.key)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="seg" role="button" tabindex="-1" onclick={(e) => open(e, p)}>
      <!-- Notes stays compact (icon + count) unless the widget is Large: it would show a long title. -->
      <p.Pill {instance} size={p.key === 'notes' && size !== 'large' ? 'small' : size} {settings} {vertical} />
    </div>
  {:else}
    {#if vertical}<div class="side dim" title="Pomodoro, To-Do and Notes are hidden · right-click to show"><span class="side-k">Hidden</span></div>
    {:else}<div class="pill empty dim" title="Right-click to choose what shows">Pomodoro, To-Do and Notes are hidden · right-click to show</div>{/if}
  {/each}
</div>

<style>
  .segwrap { display: flex; align-items: stretch; gap: 2px; height: 100%; }
  .segwrap.vertical { flex-direction: column; height: auto; }
  .seg { display: flex; height: 100%; }
  .vertical .seg { height: auto; width: 100%; }
  /* Joined segments: inner corners nearly square. */
  .segwrap:not(.vertical) > .seg:not(:first-child) > :global(.pill) { border-top-left-radius: calc(4px * var(--rs)); border-bottom-left-radius: calc(4px * var(--rs)); }
  .segwrap:not(.vertical) > .seg:not(:last-child) > :global(.pill) { border-top-right-radius: calc(4px * var(--rs)); border-bottom-right-radius: calc(4px * var(--rs)); }
  .segwrap.vertical > .seg:not(:first-child) > :global(:is(.pill, .side)) { border-top-left-radius: calc(4px * var(--rs)); border-top-right-radius: calc(4px * var(--rs)); }
  .segwrap.vertical > .seg:not(:last-child) > :global(:is(.pill, .side)) { border-bottom-left-radius: calc(4px * var(--rs)); border-bottom-right-radius: calc(4px * var(--rs)); }
  .segwrap.vertical { width: 100%; }
  .empty { font-size: 11px; white-space: nowrap; padding: 0 12px; }
</style>
