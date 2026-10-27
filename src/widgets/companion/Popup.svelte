<script lang="ts">
  // What it looks like and what it reacts to, with every mood on show so the colours and the
  // size can be judged against the real faces rather than guessed at.
  import PanelHeader from '../../lib/PanelHeader.svelte';
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import type { PopupProps } from '../types';
  import Face from './Face.svelte';
  import { LOOKS, TRIGGERS, type Mood } from './mood';
  import { COMPANION_DEFAULTS, triggersOf, type CompanionSettings } from './settings';

  let { instance, settings }: PopupProps<CompanionSettings> = $props();
  const on = $derived(triggersOf(settings));
  const saveOn = (p: Record<string, unknown>) => updateWidgetSettings(instance, { on: { ...on, ...p } });
  const moods = Object.keys(LOOKS) as Mood[];
</script>

<PanelHeader title="Companion" />

<div class="sheet" data-back={settings.backdrop}
     style:--comp-c={settings.backdrop === 'solid' ? settings.color : 'var(--accent)'}>
  {#each moods as m (m)}
    <span class="one" title={m}>
      <Face mood={m} hands={settings.hands} eye={settings.eyeColor} size={settings.size} />
      <small class="dim">{m}</small>
    </span>
  {/each}
</div>

<OptionsPopover title="Look" values={settings} save={(p) => updateWidgetSettings(instance, p)} options={[
  { type: 'range', key: 'size', label: 'Size', min: 14, max: 40, unit: 'px' },
  { type: 'choice', key: 'backdrop', label: 'Background', choices: [
    { value: 'none', label: 'None' }, { value: 'glass', label: 'Glass' },
    { value: 'solid', label: 'Colour' }, { value: 'accent', label: 'Accent' }] },
  { type: 'color', key: 'color', label: 'Background colour' },
  { type: 'color', key: 'eyeColor', label: 'Eye colour' },
  { type: 'toggle', key: 'hands', label: 'Hands', hint: 'Two little hands over the pill edge' },
  { type: 'range', key: 'blinkEvery', label: 'Blink about every', min: 2, max: 12, unit: 's' },
]} />

<OptionsPopover title="What it reacts to" values={on} save={saveOn} options={[
  { type: 'checks', label: 'Triggers', items: TRIGGERS.map((t) => ({ key: t.key, label: t.label })) },
]} />

<p class="foot dim">
  Tickle it by wiggling the pointer across it, and click it for a wink. Otherwise it reacts to the
  charger going in or coming out, a battery under 15 %, music playing, a prayer time arriving, a
  Pomodoro running or on a break, Do Not Disturb, and being left alone for a minute and a half.
  It holds still whenever the dock isn't on screen.
</p>

<style>
  .sheet {
    display: flex; flex-wrap: wrap; gap: 10px 12px; justify-content: center;
    padding: 14px 10px; margin: 4px 0 12px;
    border-radius: var(--r-outer); background: var(--bg-2, var(--pill));
  }
  .sheet[data-back='solid'], .sheet[data-back='accent'] { background: var(--comp-c); }
  .one { display: flex; flex-direction: column; align-items: center; gap: 3px; }
  .one small { font-size: 9.5px; }
  .sheet[data-back='solid'] .one small, .sheet[data-back='accent'] .one small { color: #fff; opacity: 0.75; }
  .foot { font-size: 11px; line-height: 1.5; margin: 12px 2px 0; }
</style>
