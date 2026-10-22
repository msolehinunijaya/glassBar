<script lang="ts">
  import type { PillProps } from '../types';
  import type { ClockSettings } from './index';
  import { fmtTime, ticker } from './time.svelte';

  let { size, settings, vertical }: PillProps<ClockSettings> = $props();
  const clock = ticker(() => settings.showSeconds);
  const date = $derived(new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric', month: 'short' }).format(clock.now));
  const zoneLabel = $derived(settings.secondTz.split('/').pop()?.replace(/_/g, ' ') ?? '');
</script>

<div class="pill clock" class:vertical>
  {#if size !== 'small'}<span class="dim">{date}</span>{/if}
  <span class="time">{fmtTime(clock.now, settings.hour12, settings.showSeconds)}</span>
  {#if size === 'large' && settings.secondTz}
    <span class="dim">{zoneLabel} {fmtTime(clock.now, settings.hour12, false, settings.secondTz)}</span>
  {/if}
</div>

<style>
  .time { font-weight: 600; font-variant-numeric: tabular-nums; }
  .vertical { padding: 0 4px; font-size: 11px; }
</style>
