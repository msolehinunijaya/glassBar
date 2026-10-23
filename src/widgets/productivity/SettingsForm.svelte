<script lang="ts">
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import PomoSettings from '../pomodoro/SettingsForm.svelte';
  import NotesSettings from '../notes/SettingsForm.svelte';
  import type { SettingsProps } from '../types';
  import type { ProductivitySettings } from './index';

  let { instance, settings, save }: SettingsProps<ProductivitySettings> = $props();
  const allHidden = $derived(!settings.showPomo && !settings.showTodo && !settings.showNotes);
</script>

<OptionsPopover title="Show on the dock" values={settings} save={(p) => save(p as Partial<ProductivitySettings>)} options={[
  { type: 'toggle', key: 'showPomo', label: 'Pomodoro' },
  { type: 'toggle', key: 'showTodo', label: 'To-Do' },
  { type: 'toggle', key: 'showNotes', label: 'Notes' },
]} />
{#if allHidden}
  <p class="warn">All three are hidden, so this widget only shows a reminder on the dock. Turn one on, or remove the widget.</p>
{/if}
<h4>Pomodoro</h4>
<PomoSettings {instance} {settings} save={(p) => save(p as Partial<ProductivitySettings>)} />
<h4>Notes</h4>
<NotesSettings {instance} {settings} save={(p) => save(p as Partial<ProductivitySettings>)} />

<style>
  h4 { margin: 12px 0 4px; font-size: 12px; font-weight: 600; }
  .warn { color: #ff9f0a; font-size: 12px; margin: 6px 0 0; }
</style>
