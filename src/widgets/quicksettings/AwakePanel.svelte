<script lang="ts">
  // Keep awake: how long, and whether the screen stays on too. Choosing a length turns it
  // on (or changes it); the switch at the top turns it off.
  import { Check, ChevronLeft, Coffee } from '@lucide/svelte';
  import { updateWidgetSettings } from '../../lib/config.svelte';
  import { ipc } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import { awakeText } from './extras';
  import type { QuickSettings } from './index';

  let { back, instance, settings }: { back: () => void; instance: string; settings: QuickSettings } = $props();
  const awake = live('awake', ipc.awakeGet, { on: false, until: null, screen: true });
  const t = tickingNow(() => awake.v.on && awake.v.until != null, 15_000);
  let error = $state('');
  const LENGTHS: [number, string][] = [[30, '30 minutes'], [60, '1 hour'], [120, '2 hours'], [240, '4 hours'], [0, 'Until I turn it off']];
  const current = $derived(awake.v.on ? settings.awakeMinutes ?? 0 : null);

  async function choose(min: number, screen = settings.awakeScreen ?? true) {
    updateWidgetSettings(instance, { awakeMinutes: min, awakeScreen: screen });
    try { awake.v = await ipc.awakeSet(min || null, screen); error = ''; } catch (e) { error = String(e); }
  }
  async function toggle() {
    try { awake.v = awake.v.on ? await ipc.awakeOff() : await ipc.awakeSet((settings.awakeMinutes ?? 0) || null, settings.awakeScreen ?? true); error = ''; }
    catch (e) { error = String(e); }
  }
  function screen(on: boolean) {
    updateWidgetSettings(instance, { awakeScreen: on });
    if (awake.v.on) choose(settings.awakeMinutes ?? 0, on);
  }
</script>

<div class="head">
  <button class="icon-btn" onclick={back} aria-label="Back"><ChevronLeft size={16} /></button>
  <h3>Keep awake</h3>
  <input type="checkbox" class="switch" checked={awake.v.on} onchange={toggle} aria-label="Keep awake on or off" />
</div>

<div class="hero" class:on={awake.v.on}>
  <span class="cup"><Coffee size={20} /></span>
  <span class="col"><b>{awake.v.on ? 'Your PC stays awake' : 'Your PC sleeps as usual'}</b><span class="dim">{awakeText(awake.v, t.now)}</span></span>
</div>

<h4 class="dim">For how long</h4>
<div class="list" role="radiogroup" aria-label="How long to keep awake">
  {#each LENGTHS as [min, label] (min)}
    <button class="row" role="radio" aria-checked={current === min} class:sel={current === min} onclick={() => choose(min)}>
      <span>{label}</span>{#if current === min}<Check size={16} />{/if}
    </button>
  {/each}
</div>

<label class="row sw">
  <span class="col"><span>Keep the screen on</span><span class="dim small">Off: the PC stays awake, the screen may turn off (good for downloads)</span></span>
  <input type="checkbox" class="switch" checked={settings.awakeScreen ?? true} onchange={(e) => screen(e.currentTarget.checked)} />
</label>
{#if error}<p class="err">{error}</p>{/if}
<p class="dim small foot">Ends by itself, and always when GlassBar closes.</p>

<style>
  .head { display: flex; align-items: center; gap: 6px; margin-bottom: 10px; }
  h3 { flex: 1; margin: 0; font-size: 15px; font-weight: 600; }
  h4 { margin: 14px 2px 6px; font-size: 12px; font-weight: 500; }
  .hero { display: flex; align-items: center; gap: 12px; padding: 12px; border-radius: var(--r-inner); background: var(--bg-2); }
  .cup { display: grid; place-items: center; width: 36px; height: 36px; border-radius: 50%; background: var(--pill); flex: none; transition: background 0.2s var(--ease), color 0.2s; }
  .hero.on .cup { background: #ff9f0a; color: #fff; }
  .col { display: flex; flex-direction: column; min-width: 0; line-height: 1.3; }
  .col b { font-size: 13px; }
  .col .dim { font-size: 12px; font-variant-numeric: tabular-nums; }
  .list { display: flex; flex-direction: column; gap: 2px; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 10px; min-height: 40px; padding: 0 var(--s-3); border-radius: var(--r-inner); font-size: 13px; text-align: left; }
  button.row:hover { background: var(--bg-2); }
  .row.sel { background: var(--bg-2); box-shadow: inset 3px 0 0 var(--accent); font-weight: 600; }
  .row:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .sw { margin-top: 10px; padding-top: 8px; padding-bottom: 8px; border-top: 1px solid var(--border); border-radius: 0; }
  .small { font-size: 11px; }
  .foot { margin: 10px 2px 0; }
  .err { color: #ff453a; font-size: 12px; }
</style>
