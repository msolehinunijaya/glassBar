<script lang="ts">
  // A formatted time with its AM/PM marker shown small, same size, or not at all.
  // Tabular numbers keep the width steady as seconds tick.
  import type { ClockSettings } from './index';
  import { time } from './prefs.svelte';

  let { now, settings, seconds = settings.showSeconds, tz }: { now: Date; settings: ClockSettings; seconds?: boolean; tz?: string } = $props();
  const t = $derived(time(now, settings, seconds, tz));
</script>

<span class="t">{t.main}{#if t.ampm && settings.ampmSize !== 'hidden'}<span class="ap" class:small={settings.ampmSize === 'small'}>{t.ampm}</span>{/if}</span>

<style>
  .t { font-variant-numeric: tabular-nums; white-space: nowrap; }
  .ap { margin-left: 0.25em; }
  .ap.small { font-size: 0.62em; font-weight: 600; letter-spacing: 0.02em; opacity: 0.85; }
</style>
