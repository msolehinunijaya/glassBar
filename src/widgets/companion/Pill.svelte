<script lang="ts">
  // A small creature on the dock. It blinks on its own, can be tickled, and changes mood from
  // things the dock already knows: the charger, the battery, music, prayer time, a Pomodoro,
  // Do Not Disturb, and being left alone.
  //
  // Nothing here polls. Blinking is a chained timeout, not an interval; the reactions and
  // falling asleep are single timeouts; everything else rides events the app already emits.
  // While the dock is hidden, or the PC is on battery with the saver on, it holds still.
  import { events, ipc, type Battery, type PomoState } from '../../lib/ipc';
  import { store } from '../../lib/config.svelte';
  import { live } from '../../lib/live.svelte';
  import { liveMedia } from '../music/media.svelte';
  import type { PillProps } from '../types';
  import Face from './Face.svelte';
  import { canBlink, moodFor, type Mood } from './mood';
  import { IDLE_MS, REACT_MS, triggersOf, type CompanionSettings } from './settings';
  import { NO_TICKLE, level, nextChangeAt, onMove, reset, type Tickle } from './tickle';

  let { settings: s, vertical }: PillProps<CompanionSettings> = $props();

  const on = $derived(triggersOf(s));
  const bat = live<Battery | null>('battery', ipc.batteryGet, null);
  const media = liveMedia();
  // The phase only, not livePomo(): that carries a one-second clock for the countdown, and a
  // face that just needs "working or resting" shouldn't start a timer to find out.
  const pomo = live<PomoState | null>('pomodoro', ipc.pomoGet, null);

  let reaction = $state<'click' | 'plugged' | 'unplugged' | null>(null);
  let tickle = $state<Tickle>(NO_TICKLE);
  let tickled = $state(0);
  let asleep = $state(false);
  let blinking = $state(false);
  let awake = $state(true); // the dock is on screen and we are allowed to move

  const dnd = $derived(!!store.config?.dnd);
  const pomoRunning = $derived(!!pomo.v?.pomo.endsAt);
  const pomoPhase = $derived(!pomoRunning ? null : pomo.v?.pomo.phase === 'work' ? 'work' as const : 'rest' as const);

  // Prayer time is only read when that trigger is on: it loads prayer times, which Rust
  // caches per zone, and nobody should pay for data they switched off.
  let prayerNow = $state(false);
  /** The times list carries two markers that aren't prayers: sunrise, and the pre-dawn cut-off. */
  const NOT_A_PRAYER = new Set(['Syuruk', 'Imsak']);
  $effect(() => {
    if (!on.prayer) { prayerNow = false; return; }
    let stop = false;
    const look = () => ipc.solatDays().then((i) => {
      if (stop || !i) return;
      const now = Date.now();
      // "Just now" is the 10 minutes after a prayer time came.
      prayerNow = i.days.some((d) => d.times.some(([name, at]) =>
        !NOT_A_PRAYER.has(name) && now >= at && now - at <= 10 * 60_000));
    }).catch(() => {});
    look();
    const t = setInterval(look, 60_000);
    return () => { stop = true; clearInterval(t); };
  });

  const mood = $derived.by<Mood>(() => {
    const m = moodFor({
      reaction, tickle: tickled, plugged: !!bat.v?.onAc, percent: bat.v?.percent ?? null,
      playing: !!media.m?.playing, prayerNow, pomo: pomoPhase, dnd, asleep,
    }, on);
    return blinking && canBlink(m) ? 'blink' : m;
  });

  /** One timeout to clear a reaction, replacing any earlier one. */
  let reactTimer: ReturnType<typeof setTimeout>;
  function react(kind: 'click' | 'plugged' | 'unplugged') {
    clearTimeout(reactTimer);
    reaction = kind;
    wake();
    reactTimer = setTimeout(() => (reaction = null), REACT_MS);
  }

  // The charger moving is the same event the dock's charge flash rides.
  $effect(() => {
    const off = events.chargerFlip((f) => { if (on.charger) react(f.plugged ? 'plugged' : 'unplugged'); });
    return () => { off.then((f) => f()).catch(() => {}); };
  });

  // Blinking: a chained timeout at a jittered interval, so it never looks metronomic, and it
  // stops dead while the dock is hidden instead of ticking away unseen.
  $effect(() => {
    if (!awake) return;
    const every = Math.max(1, s.blinkEvery) * 1000;
    let t: ReturnType<typeof setTimeout>;
    const shut = () => {
      blinking = true;
      t = setTimeout(() => { blinking = false; t = setTimeout(shut, every * (0.6 + Math.random() * 0.8)); }, 130);
    };
    t = setTimeout(shut, every * (0.6 + Math.random() * 0.8));
    return () => { blinking = false; clearTimeout(t); };
  });

  // Hold still while the dock isn't on screen. Same discipline as the Equalizer.
  $effect(() => {
    const sync = () => (awake = document.visibilityState === 'visible');
    sync();
    document.addEventListener('visibilitychange', sync);
    return () => document.removeEventListener('visibilitychange', sync);
  });

  // Nodding off: one timeout, re-armed whenever the pointer comes near.
  let idleTimer: ReturnType<typeof setTimeout>;
  function wake() {
    asleep = false;
    clearTimeout(idleTimer);
    if (on.idle) idleTimer = setTimeout(() => (asleep = true), IDLE_MS);
  }
  $effect(() => { wake(); return () => clearTimeout(idleTimer); });
  // Reads nothing, so this runs once and its cleanup is unmount: the reaction and fade timers
  // would otherwise fire against a widget the user had just taken off the dock.
  $effect(() => () => { clearTimeout(reactTimer); clearTimeout(fade); });

  // Tickling: direction changes, not movement, so walking the pointer across is not a tickle.
  //
  // The level fades as the flips age out of their window, which is a clock nothing else
  // watches: without re-arming until it reaches zero, a tickle followed by the pointer leaving
  // (or just holding still) left the face stuck giggling for good, hiding every other mood.
  let fade: ReturnType<typeof setTimeout>;
  function scheduleFade() {
    clearTimeout(fade);
    const now = performance.now();
    const at = nextChangeAt(tickle, now);
    if (at === null) return;
    fade = setTimeout(() => { tickled = level(tickle, performance.now()); scheduleFade(); }, Math.max(60, at - now + 20));
  }
  function move(e: PointerEvent) {
    if (!on.tickle) return;
    const now = performance.now();
    tickle = onMove(tickle, e.clientX, now);
    tickled = level(tickle, now);
    wake();
    scheduleFade();
  }
  function leave() {
    tickle = reset(tickle);
    scheduleFade();
  }

  const label = $derived(`Companion: ${mood}`);
</script>

<!-- A button, not a decorated div: you really do poke it, and this way a keyboard gets the
     wink too. The click still reaches the dock slot, so it opens the panel as any widget does.
     On a left/right dock it becomes a side tile, because a 56-96 px column has no room for a
     horizontal pill; the face letterboxes into whatever width is left. -->
<button type="button" class="comp" class:pill={!vertical} class:side={vertical} class:vert={vertical}
        data-back={s.backdrop} class:plain={s.backdrop === 'none'}
        style:--comp-c={s.backdrop === 'solid' ? s.color : 'var(--accent)'}
        onpointermove={move} onpointerleave={leave} onclick={() => react('click')}
        aria-label={label} title={label}>
  <Face {mood} hands={s.hands} eye={s.eyeColor} size={s.size} />
</button>

<style>
  .comp { padding: 0 8px; cursor: default; }
  .comp.vert { padding: 6px 4px; }
  .comp:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  /* No pill at all: the creature sits straight on the dock. */
  .comp.plain { background: none; border-color: transparent; box-shadow: none; }
  .comp[data-back='solid'], .comp[data-back='accent'] { background: var(--comp-c); border-color: transparent; }
</style>
