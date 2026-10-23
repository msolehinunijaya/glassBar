<script lang="ts">
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { DiskSettings } from './index';
  import { bytes, letter, level, levelColor, pickDrives, usedPct } from './level';
  import { liveDrives } from './drives.svelte';

  let { instance, settings }: PopupProps<DiskSettings> = $props();
  const drives = liveDrives(15_000);
  const onBar = $derived(new Set(pickDrives(drives.list, settings.drives).map((d) => d.mount)));
  function toggle(mount: string) {
    const next = new Set(onBar);
    if (next.has(mount)) { if (next.size === 1) return; next.delete(mount); } else next.add(mount);
    updateWidgetSettings(instance, { drives: [...next] });
  }
</script>

<h3>Drives</h3>
{#each drives.list as d (d.mount)}
  {@const p = usedPct(d)}
  {@const lv = level(p, +settings.high, +settings.critical)}
  <div class="card drive">
    <div class="row">
      <b>{letter(d.mount)}</b><span class="dim">{d.label}{d.removable ? ' · removable' : ''} · {d.fs}</span>
      <label class="dim on" title="Show on the bar"><input type="checkbox" checked={onBar.has(d.mount)} onchange={() => toggle(d.mount)} /> bar</label>
    </div>
    <div class="track"><span class="fill" style:width="{p}%" style:background={levelColor[lv]}></span></div>
    <div class="row dim small">
      <span>{bytes(d.total - d.available)} used ({p}%)</span><span>{bytes(d.available)} free of {bytes(d.total)}</span>
    </div>
    {#if lv !== 'normal'}<div class="warn" style:color={levelColor[lv]}>{lv === 'critical' ? 'Almost full' : 'Getting full'}</div>{/if}
  </div>
{:else}
  <p class="dim">{drives.loaded ? 'Windows reported no drives.' : 'Loading…'}</p>
{/each}

<style>
  h3 { margin: 0 0 10px; font-size: 14px; font-weight: 600; }
  .drive { padding: 10px 12px; margin-bottom: 8px; display: flex; flex-direction: column; gap: 6px; }
  .row { display: flex; align-items: center; gap: 8px; justify-content: space-between; }
  .row .dim { flex: 1; font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .on { flex: none !important; display: inline-flex; gap: 4px; align-items: center; }
  .track { height: 8px; border-radius: 4px; background: var(--track); overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 4px; }
  .small { font-size: 11px; }
  .small span { flex: none; }
  .warn { font-size: 11px; font-weight: 600; }
</style>
