<script lang="ts">
  // Clock & date format settings, shown in the clock panel and in the Settings window.
  // Date styles show a live preview of today; a custom pattern is saved only when valid.
  import OptionsPopover from '../../lib/OptionsPopover.svelte';
  import { formatDate, isValidPattern, type DateStyle } from './format';
  import type { ClockSettings } from './index';
  import { dateOpts, is12h, timeText } from './prefs.svelte';

  let { settings, save }: { settings: ClockSettings; save: (p: Partial<ClockSettings>) => void } = $props();
  const now = new Date();
  const values = $derived({ ...settings, timeFormat: settings.timeFormat ?? (settings.hour12 ? '12' : 'system') });
  const preview = (style: DateStyle) => formatDate(now, { ...dateOpts(settings), style });
  const styles: [DateStyle, string][] = [
    ['short', 'Short day/month'], ['dmy', 'DD/MM/YYYY'], ['mdy', 'MM/DD/YYYY'], ['iso', 'YYYY-MM-DD (ISO)'],
    ['medium', 'Medium'], ['weekday', 'With weekday'], ['long', 'Long'], ['custom', 'Custom pattern'],
  ];
  let pattern = $state(settings.datePattern);
  const valid = $derived(isValidPattern(pattern));
  const sysLabel = $derived(`Follow Windows (${is12h({ ...settings, timeFormat: 'system' }) ? '12-hour' : '24-hour'})`);
</script>

<OptionsPopover title="Time" values={values} save={(p) => save(p as Partial<ClockSettings>)} options={[
  { type: 'choice', key: 'timeFormat', label: 'Format', choices: [
    { value: 'system', label: sysLabel }, { value: '24', label: `24-hour (${timeText(now, { ...settings, timeFormat: '24' })})` },
    { value: '12', label: `12-hour (${timeText(now, { ...settings, timeFormat: '12' })})` }] },
  { type: 'choice', key: 'ampm', label: 'AM/PM', choices: [{ value: 'PM', label: 'PM' }, { value: 'pm', label: 'pm' }, { value: 'P.M.', label: 'P.M.' }, { value: 'p.m.', label: 'p.m.' }] },
  { type: 'choice', key: 'ampmSize', label: 'AM/PM size', choices: [{ value: 'small', label: 'Small' }, { value: 'same', label: 'Same' }, { value: 'hidden', label: 'Hide' }] },
  { type: 'choice', key: 'leadingZero', label: 'Hour', choices: [{ value: 'auto', label: 'Auto' }, { value: 'on', label: '02:05' }, { value: 'off', label: '2:05' }] },
  { type: 'toggle', key: 'showSeconds', label: 'Show seconds' },
  { type: 'toggle', key: 'worldSeconds', label: 'Show seconds in world clocks' },
]} />

<OptionsPopover title="Date" values={settings} save={(p) => save(p as Partial<ClockSettings>)} options={[
  { type: 'choice', key: 'dateStyle', label: 'Style', choices: styles.map(([value, label]) => ({ value, label: `${label} — ${preview(value)}` })) },
  { type: 'choice', key: 'dateSep', label: 'Separator', choices: [{ value: '/', label: '/' }, { value: '-', label: '-' }, { value: '.', label: '.' }] },
  { type: 'choice', key: 'weekday', label: 'Weekday', choices: [{ value: 'none', label: 'As style' }, { value: 'short', label: 'Fri' }, { value: 'full', label: 'Friday' }] },
]} />
{#if settings.dateStyle === 'custom'}
  <label class="pat">Pattern
    <input value={pattern} oninput={(e) => { pattern = e.currentTarget.value; if (isValidPattern(pattern)) save({ datePattern: pattern }); }} spellcheck="false" />
  </label>
  <p class="hint" class:bad={!valid}>
    {#if valid}{formatDate(now, { ...dateOpts(settings), pattern })}{:else}Not a valid pattern: keeping “{formatDate(now, dateOpts(settings))}”.{/if}
    <br /><span class="dim">D DD · M MM MMM MMMM · YY YYYY · ddd dddd · [text]</span>
  </p>
{/if}

<OptionsPopover title="On the bar" values={settings} save={(p) => save(p as Partial<ClockSettings>)} options={[
  { type: 'choice', key: 'show', label: 'Show', choices: [{ value: 'time', label: 'Time' }, { value: 'date', label: 'Date' }, { value: 'both', label: 'Both' }] },
  { type: 'choice', key: 'layout', label: 'Both as', choices: [{ value: 'inline', label: 'Side by side' }, { value: 'stacked', label: 'Stacked' }] },
  { type: 'choice', key: 'appearance', label: 'Appearance', menu: true, choices: [
    { value: 'standard', label: 'Standard' }, { value: 'prominent', label: 'Prominent', hint: 'Bigger, bolder time on an accent panel' },
    { value: 'popup', label: 'Popup', hint: 'Hover the clock to see a large clock and world clocks' }] },
  { type: 'toggle', key: 'dayNight', label: 'Day/night icon on world clocks' },
]} />

<style>
  .pat { display: flex; align-items: center; gap: 8px; font-size: 12px; margin-top: 6px; }
  .pat input { flex: 1; font-family: 'Cascadia Code', Consolas, monospace; }
  .hint { font-size: 12px; margin: 4px 0 8px; }
  .bad { color: #ff9f0a; }
</style>
