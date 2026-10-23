<script lang="ts">
  import { ipc } from '../../lib/ipc';
  import type { SettingsProps } from '../types';
  import type { AiSettings } from './index';

  let { settings, save }: SettingsProps<AiSettings> = $props();
  let hasKey = $state<boolean | null>(null);
  let key = $state('');
  let msg = $state('');
  const refresh = () => ipc.aiKeyStatus().then((k) => (hasKey = k));
  $effect(() => { refresh(); });

  async function saveKey() {
    try { await ipc.aiKeySet(key); key = ''; msg = 'Saved to Windows Credential Manager.'; } catch (e) { msg = String(e); }
    refresh();
  }
  async function clearKey() { await ipc.aiKeyClear(); msg = 'Key removed.'; refresh(); }
</script>

<label>Anthropic API key
  <input type="password" autocomplete="off" placeholder={hasKey ? '•••••••• (saved)' : 'sk-ant-…'} bind:value={key} />
</label>
<div style="display:flex;gap:8px;margin:6px 0">
  <button class="btn" disabled={!key.trim()} onclick={saveKey}>Save key</button>
  {#if hasKey}<button class="btn" onclick={clearKey}>Remove key</button>{/if}
</div>
{#if msg}<p class="dim" style="font-size:12px">{msg}</p>{/if}
<p class="dim" style="font-size:12px">Get a key at console.anthropic.com. It never leaves this PC except in requests to Anthropic.</p>

<label>Model
  <input value={settings.model} onchange={(e) => save({ model: e.currentTarget.value.trim() || 'claude-opus-5-5' })} />
</label>
<label class="row"><input type="checkbox" checked={settings.showTokens} onchange={(e) => save({ showTokens: e.currentTarget.checked })} /> Show today's tokens on the bar</label>
