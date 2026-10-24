<script lang="ts">
  import { ipc, type AiuReading } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import type { SettingsProps } from '../types';
  import Connect from './Connect.svelte';
  import type { AiUsageSettings } from './index';
  import Options from './Options.svelte';

  let { settings, save }: SettingsProps<AiUsageSettings> = $props();
  const st = live<AiuReading[]>('aiusage', ipc.aiuState, []);
</script>

<Connect readings={st.v} />
<div style="margin-top:12px"><Options {settings} {save} /></div>
