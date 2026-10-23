<script lang="ts">
  import { ScanText } from '@lucide/svelte';
  import type { PillProps } from '../types';
  import type { OcrSettings } from './index';
  import { hotkeyLabel } from './hotkey';

  let { settings }: PillProps<OcrSettings> = $props();
</script>

<div class="pill" title="OCR Snap ({hotkeyLabel(settings.hotkey)})"><ScanText size={16} /></div>
