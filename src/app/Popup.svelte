<script lang="ts">
  import { onMount } from 'svelte';
  import { profile } from '../lib/config.svelte';
  import { events, ipc } from '../lib/ipc';
  import { GALLERY } from '../lib/popup';
  import { settingsOf, widgets } from '../widgets/registry';
  import Gallery from './Gallery.svelte';
  import WeatherPanel from '../widgets/weather/Popup.svelte';
  import SoundPanel from '../widgets/sound/SoundPanel.svelte';
  import { BATTERY_PANEL, SOUND_PANEL, soundOf } from '../widgets/sound/settings';
  import BatterySection from '../widgets/battery/BatterySection.svelte';
  import { parsePartPanel } from '../widgets/productivity/parts';
  import { updateWidgetSettings } from '../lib/config.svelte';
  import WindowPreviews from './WindowPreviews.svelte';
  import Feedback from '../widgets/ocr/Feedback.svelte';
  import type { CaptureDone } from '../lib/ipc';

  // Which widget's panel is showing. Cleared when Rust hides the popup, so hidden
  // panels stop their timers. (Rust sends hide before the next show, so no race.)
  let instance = $state<string | null>(null);
  /** "__feedback": the card shown right after a screenshot / OCR snap. */
  const FEEDBACK = '__feedback';
  let feedback = $state<CaptureDone | null>(null);
  const close = () => ipc.popupHide();
  const WINDOWS = '__windows:';
  /** "__hover:<instance>" = that widget's panel opened by hovering (closes on mouse-out). */
  const HOVER = '__hover:';
  /** "__weather:<clock instance>" = the weather forecast, opened from the clock's weather part. */
  const WEATHER = '__weather:';
  const hover = $derived(!!instance && (instance.startsWith(WINDOWS) || instance.startsWith(HOVER)));
  const target = $derived(instance?.startsWith(HOVER) ? instance.slice(HOVER.length) : instance);
  const w = $derived(target ? profile().widgets.find((x) => x.instance === target) : undefined);
  const m = $derived(w ? widgets[w.id] : undefined);

  // Hover popups close shortly after the pointer leaves both the icon and the popup.
  let leaveTimer: ReturnType<typeof setTimeout>;
  const leaveSoon = () => { clearTimeout(leaveTimer); if (hover) leaveTimer = setTimeout(close, 150); };
  const stay = () => clearTimeout(leaveTimer);

  // Size the window to the panel's content (no empty space below, no needless scrollbar):
  // measure the content whenever it changes size and ask Rust to fit the popup to it.
  let surface = $state<HTMLElement>();
  let content = $state<HTMLElement>();
  let lastFit = 0;
  $effect(() => {
    if (!content || !surface || !instance) return;
    const cs = getComputedStyle(surface);
    const chrome = ['paddingTop', 'paddingBottom', 'borderTopWidth', 'borderBottomWidth'].reduce((s, k) => s + parseFloat(cs[k as any] as string), 0);
    const fit = () => {
      const h = Math.ceil(content!.offsetHeight + chrome);
      if (Math.abs(h - lastFit) > 1) { lastFit = h; ipc.popupFit(h); }
    };
    lastFit = 0;
    fit();
    const ro = new ResizeObserver(fit);
    ro.observe(content);
    return () => ro.disconnect();
  });

  onMount(() => {
    const offs = [
      events.popupShow((i) => { instance = i; stay(); }),
      events.popupHide(() => { instance = null; }),
      events.popupLeave(leaveSoon),
      events.captureDone((d) => (feedback = d)),
    ];
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
    addEventListener('keydown', onKey);
    return () => { offs.forEach((p) => p.then((off) => off())); removeEventListener('keydown', onKey); };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="surface popup" class:hover bind:this={surface} onpointerenter={stay} onpointerleave={leaveSoon}>
  <div class="content" bind:this={content}>
  {#if instance?.startsWith(WINDOWS)}
    {#key instance}<WindowPreviews appKey={instance.slice(WINDOWS.length)} {close} />{/key}
  {:else if instance === FEEDBACK && feedback}
    {#key feedback}<Feedback data={feedback} {close} />{/key}
  {:else if instance && parsePartPanel(instance)}
    {@const pp = parsePartPanel(instance)!}
    {@const pw = profile().widgets.find((x) => x.instance === pp.instance)}
    {#if pw}
      {#key instance}<pp.part.Popup instance={pw.instance} settings={settingsOf(widgets.productivity, pw)} {close} />{/key}
    {/if}
  {:else if instance?.startsWith(BATTERY_PANEL)}
    {@const qs = profile().widgets.find((x) => x.instance === instance!.slice(BATTERY_PANEL.length))}
    {#if qs}
      <div class="solo"><BatterySection instance={qs.instance} style={(qs.settings as { batteryStyle?: 'classic' | 'liquid' })?.batteryStyle ?? 'classic'} /></div>
    {/if}
  {:else if instance?.startsWith(SOUND_PANEL)}
    {@const qs = profile().widgets.find((x) => x.instance === instance!.slice(SOUND_PANEL.length))}
    {#if qs}
      {@const snd = soundOf(qs.settings as never)}
      <SoundPanel settings={snd} save={(p) => updateWidgetSettings(qs.instance, { sound: { ...snd, ...p } })} />
    {/if}
  {:else if instance?.startsWith(WEATHER)}
    <WeatherPanel />
  {:else if instance === GALLERY}
    <Gallery {close} />
  {:else if w && m?.Popup}
    {#key instance}
      <m.Popup.component instance={w.instance} settings={settingsOf(m, w)} {close} />
    {/key}
  {/if}
  </div>
</div>

<style>
  .popup { padding: 14px; overflow: auto; }
  /* flow-root: a last child's bottom margin stays inside, so the measured height is exact */
  .content { display: flow-root; }
  .popup.hover { padding: 8px; }
  /* A section shown as its own panel: no gap above its heading. */
  .solo :global(.bhead) { margin-top: 0; }
</style>
