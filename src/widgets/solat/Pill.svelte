<script lang="ts">
  import { Moon } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { SolatSettings } from './index';
  import { countdown } from './next';
  import { liveSolat } from './solat.svelte';

  let { size, settings }: PillProps<SolatSettings> = $props();
  const s = liveSolat(() => settings);
  const at = (ms: number) => new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' }).format(ms);
  const soon = $derived(s.next ? s.next.at - s.now <= 600_000 : false);
</script>

<div class="pill" class:soon title={s.error || (s.next ? `${s.next.name} at ${at(s.next.at)}` : 'Waktu Solat')}>
  <Moon size={15} />
  {#if s.next}
    <span class="name">{s.next.name}</span>
    {#if size === 'large'}<span class="dim">{at(s.next.at)}</span>{/if}
    {#if size !== 'small'}<span class="t">{countdown(s.next.at - s.now)}</span>{/if}
  {:else}
    <span class="dim">–</span>
  {/if}
</div>

<style>
  .name { font-weight: 600; }
  .t { font-variant-numeric: tabular-nums; }
  .soon { background: color-mix(in srgb, var(--accent) 35%, var(--pill)); }
</style>
