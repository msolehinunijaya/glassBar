<script lang="ts">
  import type { PomoCfg } from '../../lib/ipc';
  import type { SettingsProps } from '../types';

  let { settings, save }: SettingsProps<PomoCfg> = $props();
  const fields: [keyof PomoCfg, string, number][] = [
    ['workMin', 'Work session (minutes)', 180], ['shortMin', 'Short break (minutes)', 60],
    ['longMin', 'Long break (minutes)', 120], ['every', 'Work sessions before a long break', 12],
  ];
</script>

{#each fields as [key, label, max] (key)}
  <label>{label}
    <input type="number" min="1" {max} style="width:5em" value={settings[key]}
           onchange={(e) => save({ [key]: Math.min(max, Math.max(1, Math.round(+e.currentTarget.value || 1))) })} />
  </label>
{/each}
<p class="dim" style="font-size:12px;margin:0">Phases never start by themselves: when one ends you get an alert (sound per your Notifications/Do Not Disturb setting) and a button to start the next.</p>
