<script lang="ts">
  // "C: 68%" with a slim usage bar, one block per selected drive.
  import type { PillProps } from '../types';
  import type { DiskSettings } from './index';
  import { bytes, letter, level, levelColor, pickDrives, usedPct } from './level';
  import { liveDrives } from './drives.svelte';

  let { size, settings }: PillProps<DiskSettings> = $props();
  const drives = liveDrives();
  const shown = $derived(pickDrives(drives.list, settings.drives));
</script>

<div class="pill disk" title={shown.map((d) => `${letter(d.mount)} ${bytes(d.available)} free of ${bytes(d.total)}`).join('\n') || 'No drives found'}>
  {#each shown as d (d.mount)}
    {@const p = usedPct(d)}
    <div class="col">
      <span class="txt"><span class="dim">{letter(d.mount)}</span> <b>{size !== 'small' && settings.showFree ? bytes(d.available) : `${p}%`}</b></span>
      <span class="track"><span class="fill" style:width="{p}%" style:background={levelColor[level(p, +settings.high, +settings.critical)]}></span></span>
    </div>
  {:else}
    <span class="dim txt">{drives.loaded ? 'No drives' : '…'}</span>
  {/each}
</div>

<style>
  .disk { gap: 10px; }
  .col { display: flex; flex-direction: column; gap: 3px; min-width: 48px; }
  .txt { font-size: 12px; font-variant-numeric: tabular-nums; white-space: nowrap; }
  .track { height: 4px; border-radius: 2px; background: var(--track); overflow: hidden; }
  .fill { display: block; height: 100%; border-radius: 2px; transition: width 0.6s var(--ease); }
</style>
