<script lang="ts">
  import type { PomoCfg } from '../../lib/ipc';
  import type { SettingsProps } from '../types';

  let { settings, save }: SettingsProps<PomoCfg> = $props();
  const fields: [keyof PomoCfg, string, number][] = [
    ['workMin', 'Focus (minutes)', 180], ['shortMin', 'Short break (minutes)', 60],
    ['longMin', 'Long break (minutes)', 120], ['every', 'Long break after sessions', 12],
  ];
</script>

{#each fields as [key, label, max] (key)}
  <label>{label}
    <input type="number" min="1" {max} style="width:5em" value={settings[key]}
           onchange={(e) => save({ [key]: Math.min(max, Math.max(1, Math.round(+e.currentTarget.value || 1))) })} />
  </label>
{/each}
<label><input type="checkbox" checked={settings.autoStart} onchange={(e) => save({ autoStart: e.currentTarget.checked })} /> Start the next phase automatically</label>
