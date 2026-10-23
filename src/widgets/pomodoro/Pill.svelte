<script lang="ts">
  import Ring from '../../lib/Ring.svelte';
  import { clock } from '../../lib/time';
  import type { PillProps } from '../types';
  import { PHASE } from './index';
  import { livePomo } from './pomo.svelte';

  let { size }: PillProps = $props();
  const p = livePomo();
  const phase = $derived(p.state ? PHASE[p.state.pomo.phase] : PHASE.work);
</script>

<div class="pill" title="{phase.name} · {p.state?.label ?? ''}">
  <Ring value={p.progress} color={phase.color} />
  <span class="t">{clock(p.remaining)}</span>
  {#if size !== 'small' && p.state}<span class="dim">{p.state.label}</span>{/if}
</div>

<style>
  .t { font-variant-numeric: tabular-nums; font-weight: 600; }
</style>
