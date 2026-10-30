<script lang="ts">
  // Marks on the Quick Settings pill that only appear when they carry news: the microphone
  // (muted = red, in use = orange dot), the camera in use (green dot), and Keep awake (a cup).
  // Clicking the mic mutes/unmutes; clicking the cup turns Keep awake off. Neither opens the panel.
  import { Camera, Coffee, Mic, MicOff } from '@lucide/svelte';
  import { ipc, type InUse, type MicState } from '../../lib/ipc';
  import { live } from '../../lib/live.svelte';
  import { tickingNow } from '../../lib/now.svelte';
  import { awakeText, micDock, usingText, type MicOnDock } from './extras';

  let { mode = 'auto' }: { mode?: MicOnDock } = $props();
  const mic = live<MicState | null>('mic', ipc.micState, null);
  const using = live<InUse>('privacy', ipc.privacyInUse, { mic: [], cam: [] });
  const awake = live('awake', ipc.awakeGet, { on: false, until: null, screen: true });
  const t = tickingNow(() => awake.v.on && awake.v.until != null, 30_000);
  const m = $derived(mic.v ? micDock(mode, mic.v.muted, using.v.mic.length > 0) : null);
  const cam = $derived(mode !== 'never' && using.v.cam.length > 0);

  const stop = (e: Event) => e.stopPropagation();
  function toggleMic(e: MouseEvent) {
    e.stopPropagation();
    if (!mic.v) return;
    const muted = !mic.v.muted;
    mic.v = { ...mic.v, muted };
    ipc.micSetMute(muted).catch(() => {});
  }
  const micTip = $derived(!mic.v ? '' : mic.v.muted ? 'Microphone muted · click to unmute'
    : using.v.mic.length ? `Microphone in use by ${using.v.mic.map((a) => a.name).join(', ')} · click to mute` : 'Microphone on · click to mute');
</script>

{#if m}
  <button class="mk mic {m}" title={micTip} aria-label={micTip} onpointerdown={stop} onclick={toggleMic}>
    {#if m === 'muted'}<MicOff size={15} />{:else}<Mic size={15} />{/if}
    {#if m === 'live'}<i class="dot"></i>{/if}
  </button>
{/if}
{#if cam}
  <span class="mk cam" title="Camera in use by {usingText(using.v.cam)}" role="img" aria-label="Camera in use"><Camera size={15} /><i class="dot"></i></span>
{/if}
{#if awake.v.on}
  <button class="mk cup" title="Keep awake · {awakeText(awake.v, t.now).replace('On · ', '')} · click to turn off" aria-label="Turn Keep awake off"
          onpointerdown={stop} onclick={(e) => { e.stopPropagation(); ipc.awakeOff().then((s) => (awake.v = s)).catch(() => {}); }}>
    <Coffee size={15} />
  </button>
{/if}

<style>
  .mk { position: relative; display: grid; place-items: center; width: 22px; height: 22px; border-radius: calc(7px * var(--rs)); flex: none;
        transition: background 0.15s var(--ease), transform 0.2s var(--spring); }
  button.mk:hover { background: var(--pill-hover); }
  button.mk:active { transform: scale(0.88); }
  button.mk:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
  .mic.muted { color: #ff453a; }
  .cup { color: #ff9f0a; }
  /* The "on air" dot: orange for the mic, green for the camera (iOS's convention). */
  .dot { position: absolute; top: 1px; right: 1px; width: 6px; height: 6px; border-radius: 50%; background: #ff9f0a; box-shadow: 0 0 5px #ff9f0a; animation: air 2s ease-in-out infinite; }
  .cam .dot { background: #30d158; box-shadow: 0 0 5px #30d158; }
  @keyframes air { 50% { opacity: 0.55; } }
  :global(:root[data-perf='low']) .dot { animation: none; }
  @media (prefers-reduced-motion: reduce) { .dot { animation: none; } }
</style>
