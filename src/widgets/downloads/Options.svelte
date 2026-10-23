<script lang="ts">
  // Downloads options: shown in the panel (gear) and in Settings › Widgets. Saved at once.
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import type { DownloadsSettings } from './index';

  let { settings, save }: { settings: DownloadsSettings; save: (p: Partial<DownloadsSettings>) => void } = $props();
</script>

<OptionsPopover title="Downloads" values={{ ...settings, limit: String(settings.limit) }}
  save={(p) => save((p.limit !== undefined ? { limit: +(p.limit as string) } : p) as Partial<DownloadsSettings>)} options={[
  { type: 'choice', key: 'limit', label: 'Items in the panel', choices: [{ value: '5', label: '5' }, { value: '10', label: '10' }, { value: '20', label: '20' }] },
  { type: 'toggle', key: 'newDot', label: '"New" dot on the dock', hint: 'Until you hover or open it' },
  { type: 'toggle', key: 'subfolders', label: 'Include subfolders' },
  { type: 'toggle', key: 'toast', label: 'Notify when a download finishes' },
]} />
<label class="folder">Folder to watch
  <input placeholder="Downloads (default)" value={settings.folder} onchange={(e) => save({ folder: e.currentTarget.value.trim().replace(/^"|"$/g, '') })} />
  {#if settings.folder}<button onclick={() => save({ folder: '' })}>Use Downloads</button>{/if}
</label>

<style>
  .folder { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; font-size: 12px; margin-top: 8px; }
  .folder input { flex: 1; min-width: 140px; font-size: 12px; padding: 6px 8px; border-radius: 8px; border: 1px solid var(--line-soft); background: var(--pill); color: inherit; }
  .folder button { font-size: 11.5px; padding: 5px 9px; border-radius: 8px; background: var(--pill); }
</style>
