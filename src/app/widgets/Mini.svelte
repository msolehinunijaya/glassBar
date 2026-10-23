<script lang="ts">
  // One widget drawn by its real dock component (Pill), at a given size and height.
  // Inert: in the Settings window it only shows, it never opens panels or launches apps.
  import type { WidgetInst, WidgetSize } from '../../lib/ipc';
  import { settingsOf, widgets } from '../../widgets/registry';

  let { id, inst, size, height = 38, vertical = false }:
    { id: string; inst?: WidgetInst; size?: WidgetSize; height?: number; vertical?: boolean } = $props();
  const m = $derived(widgets[id]);
  const s = $derived(size ?? inst?.size ?? m.defaultSize);
  // Not on the dock yet: its defaults, under a preview-only instance id.
  const w = $derived(inst ?? { id, instance: `preview-${id}`, size: s, settings: {} });
</script>

<div class="mini dock" class:vertical style:height={vertical ? 'auto' : `${height}px`} inert>
  <div class="slot"><m.Pill instance={w.instance} size={vertical ? 'small' : s} settings={settingsOf(m, w)} {vertical} /></div>
</div>

<style>
  .mini { display: flex; align-items: stretch; min-width: 0; max-width: 100%; overflow: hidden; pointer-events: none; }
  .mini > .slot { max-width: 100%; }
  .mini.vertical { width: 56px; }
</style>
