<script lang="ts">
  import type { SettingsProps } from '../types';
  import type { NotesSettings } from './index';

  let { settings, save }: SettingsProps<NotesSettings> = $props();
</script>

<label>
  Temporary notes: delete unpinned notes not edited for
  <input type="number" min="0" max="365" style="width:5em" value={settings.temporaryDays}
         onchange={(e) => save({ temporaryDays: Math.max(0, Math.round(+e.currentTarget.value || 0)) })} />
  days (0 = keep forever)
</label>
<p class="dim" style="font-size:12px;margin:0">Pinned notes are never deleted. Deleted notes stay in the trash for 7 days.</p>
