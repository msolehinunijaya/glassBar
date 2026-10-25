<script lang="ts">
  // Click the Equalizer: its settings, with the bar behind it as the live preview.
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import type { EqSettings } from './index';
  import SettingsForm from './SettingsForm.svelte';

  let { instance, settings }: PopupProps<EqSettings> = $props();
</script>

<PanelHeader title="Equalizer" />
<SettingsForm {instance} {settings} save={(p) => updateWidgetSettings(instance, p)} />
