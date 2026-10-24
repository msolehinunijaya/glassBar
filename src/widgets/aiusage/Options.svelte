<script lang="ts">
  // Alert switches (read by Rust when a limit crosses 80 % / 95 % or resets).
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { AiUsageSettings } from './index';

  let { settings, save }: { settings: AiUsageSettings; save: (p: Partial<AiUsageSettings>) => void } = $props();
</script>

<OptionsPopover title="Alerts" values={settings} save={(p) => save(p as Partial<AiUsageSettings>)} options={[
  { type: 'toggle', key: 'alertsClaude', label: 'Claude limit alerts', hint: 'At 80% and 95% of a limit' },
  { type: 'toggle', key: 'alertsChatgpt', label: 'ChatGPT limit alerts', hint: 'At 80% and 95% of a limit' },
  { type: 'toggle', key: 'alertReset', label: 'When a limit resets', hint: 'After it was past 80%' },
]} />
