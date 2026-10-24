<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { SettingsProps } from '../types';
  import type { OcrSettings } from './index';
  import { fromEvent, hotkeyLabel } from './hotkey';

  let { settings, save }: SettingsProps<OcrSettings> = $props();
  let recording = $state(false);
</script>

<label>Hotkey
  <input readonly value={recording ? 'Press keys…' : hotkeyLabel(settings.hotkey)}
         onfocus={() => (recording = true)} onblur={() => (recording = false)}
         onkeydown={(e) => { e.preventDefault(); const k = fromEvent(e); if (k) { save({ hotkey: k }); (e.currentTarget as HTMLInputElement).blur(); } }} />
</label>
<p class="dim" style="font-size:12px">Click the box and press a combination with Ctrl, Alt, Shift or Win.</p>
<OptionsPopover title="OCR Snap" values={settings} save={(p) => save(p as Partial<OcrSettings>)} options={[
  { type: 'choice', key: 'mode', label: 'Mode', choices: [{ value: 'text', label: 'Text' }, { value: 'code', label: 'Code' }, { value: 'terminal', label: 'Terminal' }] },
  { type: 'choice', key: 'engine', label: 'Engine', choices: [{ value: 'windows', label: 'Windows OCR', hint: 'Offline' }, { value: 'ai', label: 'Claude', hint: 'Sends the selection to Anthropic' }] },
  { type: 'toggle', key: 'autoCopy', label: 'Copy to clipboard automatically' },
  { type: 'toggle', key: 'toast', label: 'Show a notification' },
]} />
<OptionsPopover title="Icons" values={{ iconStyle: settings.iconStyle ?? 'tile' }} save={(p) => save(p as Partial<OcrSettings>)} options={[
  { type: 'choice', key: 'iconStyle', label: 'Style', choices: [
    { value: 'tile', label: 'Colour', hint: 'White symbol on a colour tile' },
    { value: 'plain', label: 'Simple', hint: 'Plain symbol, like the rest of the dock' },
    { value: 'outline', label: 'Outline', hint: 'The symbol drawn in its colour' }] },
]} />
