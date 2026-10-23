<script lang="ts">
  import { Aperture } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { OcrSettings } from './index';
  import { hotkeyLabel } from './hotkey';

  let { settings }: PillProps<OcrSettings> = $props();
</script>

<div class="pill" title="Capture: screenshot, recording, OCR ({hotkeyLabel(settings.hotkey)} for OCR)"><Aperture size={16} /></div>
