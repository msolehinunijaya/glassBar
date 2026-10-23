<script lang="ts">
  import { BatteryCharging, BatteryFull, BatteryLow, BatteryMedium, BatteryWarning, Plug } from '@lucide/svelte';
  import type { Battery } from '../../lib/ipc';

  let { b, size = 18 }: { b: Battery | null; size?: number } = $props();
  const Icon = $derived(
    !b ? Plug : b.charging ? BatteryCharging : b.percent <= 10 ? BatteryWarning : b.percent <= 35 ? BatteryLow : b.percent <= 75 ? BatteryMedium : BatteryFull,
  );
</script>

<Icon {size} color={b && !b.charging && b.percent <= 15 ? '#ff453a' : 'currentColor'} />
