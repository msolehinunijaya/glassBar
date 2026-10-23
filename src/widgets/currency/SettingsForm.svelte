<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { CurrencySettings } from './index';

  let { settings, save }: SettingsProps<CurrencySettings> = $props();
  const code = (v: string) => v.trim().toUpperCase().slice(0, 3);
</script>

<label>Default from <input maxlength="3" value={settings.from} onchange={(e) => save({ from: code(e.currentTarget.value) })} /></label>
<label>Default to <input maxlength="3" value={settings.to} onchange={(e) => save({ to: code(e.currentTarget.value) })} /></label>
<p class="dim" style="font-size:12px">3-letter ISO codes (USD, MYR, EUR…). Rates refresh hourly; favourite pairs are saved from the widget panel.</p>
