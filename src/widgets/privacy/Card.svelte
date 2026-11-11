<script lang="ts">
  // One resource in the Privacy Monitor panel: its state at a glance, the apps using it, and
  // (opened) its devices, details, Windows' recent record and the controls. Each control does
  // one named thing; mute, app permission and turning a device off stay separate actions.
  import { AppWindow, ChevronDown, ExternalLink, Mic, MicOff, Power, PowerOff } from '@lucide/svelte';
  import { ipc, type PrivacyDevice, type PrivacyKey, type PrivacyResource } from '../../lib/ipc';
  import type { PrivacySettings } from './settings';
  import { ICON } from './monitor.svelte';
  import { ago, clock, LABEL, STATE_LABEL } from './status';

  let { r, settings, muted, reload }: { r: PrivacyResource; settings: PrivacySettings; muted: boolean | null; reload: () => Promise<void> } = $props();

  let opened = $state<boolean | null>(null);
  const open = $derived(opened ?? settings.details);
  const Icon = $derived(ICON[r.key]);
  let msg = $state<{ ok: boolean; text: string } | null>(null);
  let busy = $state('');
  let confirm = $state<PrivacyDevice | null>(null);
  const word = $derived(r.key === 'mic' ? 'microphone' : 'camera');

  type Link = { label: string; uri?: string; tool?: 'devmgmt.msc' | 'services.msc'; hint: string };
  const CAPTURE: Link[] = [
    { label: 'Screen capture permissions', uri: 'ms-settings:privacy-graphicsCaptureProgrammatic', hint: 'Which Store apps may capture the screen' },
    { label: 'Screenshot borders', uri: 'ms-settings:privacy-graphicsCaptureWithoutBorder', hint: 'Which apps may capture without the yellow border' },
  ];
  const LINKS: Record<PrivacyKey, Link[]> = {
    camera: [
      { label: 'App permissions', uri: 'ms-settings:privacy-webcam', hint: 'Choose which apps may use the camera' },
      { label: 'Camera settings', uri: 'ms-settings:camera', hint: 'Windows\' camera settings' },
      { label: 'Device Manager', tool: 'devmgmt.msc', hint: 'Drivers and device details' },
    ],
    mic: [
      { label: 'App permissions', uri: 'ms-settings:privacy-microphone', hint: 'Choose which apps may use the microphone' },
      { label: 'Sound settings', uri: 'ms-settings:sound', hint: 'Input devices and levels' },
      { label: 'Device Manager', tool: 'devmgmt.msc', hint: 'Drivers and device details' },
    ],
    location: [
      { label: 'Location settings', uri: 'ms-settings:privacy-location', hint: 'Location services and which apps may use them' },
      { label: 'Services', tool: 'services.msc', hint: 'Windows Geolocation Service (lfsvc): its startup type should be Manual' },
    ],
    recording: CAPTURE,
    sharing: CAPTURE,
  };
  const openLink = (l: Link) => run(l.label, () => (l.tool ? ipc.privacyOpenTool(l.tool) : ipc.openSettingsUri(l.uri!)).then(() => ''));

  /** Run an action; show what Windows confirmed, or why it didn't happen. Then read again. */
  async function run(label: string, f: () => Promise<string | void>) {
    busy = label;
    msg = null;
    try {
      const text = await f();
      if (text) msg = { ok: true, text };
    } catch (e) {
      msg = { ok: false, text: String(e) };
    } finally {
      busy = '';
      await reload();
    }
  }

  const setDevice = (d: PrivacyDevice, on: boolean) => { confirm = null; run(`dev:${d.instance}`, () => ipc.privacySetDevice(d.instance, on)); };

  const makeDefault = (d: PrivacyDevice) => run(`def:${d.id}`, async () => {
    await ipc.micSetDefault(d.id);
    const s = await ipc.micState();
    if (s.devices.find((x) => x.isDefault)?.id !== d.id) throw new Error('Windows didn\'t switch the default microphone.');
    return `${d.name} is now the default microphone.`;
  });

  const toggleMute = () => run('mute', async () => {
    const want = !muted;
    await ipc.micSetMute(want);
    const s = await ipc.micState();
    if (s.muted !== want) throw new Error('Windows didn\'t apply it.');
    return want ? 'Muted. Apps keep their access but get silence.' : 'Unmuted.';
  });
</script>

<section class="pc" data-state={r.state} data-key={r.key}>
  <button class="top" onclick={() => (opened = !open)} aria-expanded={open}>
    <span class="ic"><Icon size={17} /></span>
    <span class="col">
      <b>{LABEL[r.key]}</b>
      <span class="dim ellipsis" title={r.headline}>{r.headline}</span>
    </span>
    <span class="chip">{STATE_LABEL[r.state]}</span>
    <span class="chev" class:up={open}><ChevronDown size={15} /></span>
  </button>

  {#if r.apps.length}
    <ul class="apps">
      {#each r.apps as a (a.id + a.since)}
        <li>
          <span class="ellipsis grow" title={a.exe || a.name}>{a.name}</span>
          <span class="dim small">{a.verified ? (a.since ? `since ${clock(a.since)}` : 'now') : 'running · not verified'}</span>
          {#if a.window !== null}
            <button class="mini" onclick={() => ipc.appActivate(a.window!)} title="Bring {a.name} to the front to stop it there"><AppWindow size={13} /> Show</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if open}
    <div class="body">
      {#if r.reason}<p class="reason">{r.reason}</p>{/if}

      {#if r.key === 'mic' && muted !== null}
        <button class="act" onclick={toggleMute} disabled={busy === 'mute'}>
          {#if muted}<MicOff size={14} />{:else}<Mic size={14} />{/if}
          <span class="col"><b>{muted ? 'Unmute' : 'Mute'} default microphone</b><span class="dim small">Apps keep their access but get silence. Not the same as blocking an app.</span></span>
        </button>
      {/if}

      {#if r.devices.length}
        <h4 class="dim">{r.key === 'mic' ? 'Input devices' : 'Cameras'}</h4>
        <ul class="devs">
          {#each r.devices as d (d.instance)}
            <li class:off={!d.enabled}>
              <span class="ellipsis grow" title={d.instance}>{d.name}</span>
              {#if d.default}<span class="tag">Default</span>{/if}
              {#if !d.enabled}<span class="tag">Off</span>{/if}
              {#if r.key === 'mic' && d.enabled && !d.default}
                <button class="mini" disabled={!!busy} onclick={() => makeDefault(d)} title="Make this the default microphone">Use</button>
              {/if}
              <button class="mini" disabled={!!busy} onclick={() => (d.enabled ? (confirm = d) : setDevice(d, true))}
                      title={d.enabled ? `Turn this ${word} off for every app (needs administrator)` : `Turn this ${word} back on (needs administrator)`}>
                {#if busy === `dev:${d.instance}`}…{:else if d.enabled}<PowerOff size={13} /> Turn off{:else}<Power size={13} /> Turn on{/if}
              </button>
            </li>
            {#if confirm?.instance === d.instance}
              <li><div class="confirm" role="alertdialog" aria-label="Turn off {d.name}?">
                <b>Turn off {d.name}?</b>
                <span class="small">No app can use it until you turn it back on, including any app using it now. Windows will ask for administrator permission. This turns the device off; it doesn't change app permissions{r.key === 'mic' ? ' and isn\'t the same as muting' : ''}.</span>
                <span class="btns">
                  <button class="mini" onclick={() => (confirm = null)}>Cancel</button>
                  <button class="mini danger" onclick={() => setDevice(d, false)}>Turn off</button>
                </span>
              </div></li>
            {/if}
          {/each}
        </ul>
      {/if}

      {#if r.facts.length}
        <dl>
          {#each r.facts as [k, v], i (i)}<dt class="dim">{k}</dt><dd>{v}</dd>{/each}
        </dl>
      {/if}

      {#if settings.history && r.recent.length}
        <h4 class="dim">Recent activity (Windows' record)</h4>
        <ul class="recent">
          {#each r.recent as a (a.id + a.until)}
            <li><span class="ellipsis grow" title={a.exe || a.name}>{a.name}</span><span class="dim small" title="{clock(a.since)} – {clock(a.until)}">{ago(a.until, Date.now())}</span></li>
          {/each}
        </ul>
      {/if}

      <p class="method dim small">{r.method}</p>

      <div class="links">
        {#each LINKS[r.key] as l (l.label)}
          <button class="link" onclick={() => openLink(l)} title={l.hint}><ExternalLink size={12} /> {l.label}</button>
        {/each}
      </div>
    </div>
  {/if}

  {#if msg}<p class="msg" class:bad={!msg.ok} role="status">{msg.text}</p>{/if}
</section>

<style>
  .pc { --lit: var(--accent); --warn: #ff9f0a; --bad: #ff453a;
        border-radius: var(--r-inner); background: var(--pill); border: 1px solid var(--line-soft); box-shadow: inset 0 1px 0 var(--line);
        transition: border-color 0.3s var(--ease), box-shadow 0.3s var(--ease); }
  [data-key='camera'] { --lit: #30d158; }
  [data-key='mic'] { --lit: #ff9f0a; }
  [data-key='location'] { --lit: #0a84ff; }
  [data-key='recording'] { --lit: #ff453a; }
  [data-key='sharing'] { --lit: #bf5af2; }

  .top { display: flex; align-items: center; gap: var(--s-3); width: 100%; padding: 10px var(--s-3); text-align: left; border-radius: var(--r-inner); }
  .top:hover { background: var(--pill-hover); }
  .top:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .ic { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 50%; flex: none; background: var(--bg-2); color: var(--fg);
        opacity: 0.55; transition: opacity 0.3s var(--ease), background 0.3s var(--ease), color 0.3s var(--ease), box-shadow 0.3s var(--ease); }
  .col { display: flex; flex-direction: column; min-width: 0; flex: 1; line-height: 1.3; }
  .col b { font-size: 13px; font-weight: 600; }
  .col .dim { font-size: 12px; }
  .chip { flex: none; padding: 2px 8px; border-radius: var(--r-chip); font-size: 11px; font-weight: 500; background: var(--bg-2); color: var(--fg-dim); }
  .chev { display: grid; color: var(--fg-3); transition: transform 0.2s var(--ease); }
  .chev.up { transform: rotate(180deg); }

  /* States */
  [data-state='active'] { border-color: color-mix(in srgb, var(--lit) 45%, transparent); }
  [data-state='active'] .ic { opacity: 1; background: var(--lit); color: #fff; box-shadow: 0 0 10px color-mix(in srgb, var(--lit) 55%, transparent); }
  [data-state='active'] .chip { background: color-mix(in srgb, var(--lit) 22%, transparent); color: var(--fg); }
  [data-state='blocked'] .chip { background: color-mix(in srgb, var(--bad) 18%, transparent); color: var(--fg); }
  [data-state='blocked'] .ic { opacity: 0.8; box-shadow: inset 0 0 0 1.5px var(--bad); }
  [data-state='unknown'] .chip, [data-state='limited'] .chip { border: 1px dashed var(--line-soft); background: transparent; }
  [data-state='unknown'] .ic { opacity: 0.7; }
  [data-state='disabled'] .ic, [data-state='unavailable'] .ic { opacity: 0.35; }
  [data-state='unavailable'] .chip { color: var(--fg); }
  @media (prefers-reduced-motion: reduce) { .pc, .ic, .chev { transition: none; } }

  .apps, .devs, .recent { list-style: none; margin: 0; padding: 0 var(--s-3) var(--s-2); display: flex; flex-direction: column; gap: 2px; }
  .apps li, .devs li, .recent li { display: flex; align-items: center; gap: var(--s-2); min-height: 30px; font-size: 12.5px; }
  .apps li { padding: 0 var(--s-2); border-radius: calc(8px * var(--rs)); background: var(--bg-2); }
  .devs li.off .grow { color: var(--fg-dim); }
  .grow { flex: 1; min-width: 0; }
  .small { font-size: 11.5px; }
  .tag { padding: 1px 6px; border-radius: var(--r-chip); background: var(--bg-2); font-size: 10.5px; color: var(--fg-dim); flex: none; }

  .body { padding: 0 var(--s-3) var(--s-3); border-top: 1px solid var(--line-soft); }
  .body .devs, .body .recent { padding: 0; }
  .reason { margin: var(--s-2) 0; font-size: 12px; line-height: 1.45; }
  [data-state='blocked'] .reason, [data-state='unavailable'] .reason { padding-left: var(--s-2); box-shadow: inset 2px 0 0 var(--warn); }
  h4 { margin: var(--s-3) 0 var(--s-1); font-size: 11.5px; font-weight: 500; }

  .act { display: flex; align-items: center; gap: var(--s-3); width: 100%; margin-top: var(--s-2); padding: var(--s-2) var(--s-3); border-radius: var(--r-inner); background: var(--bg-2); text-align: left; }
  .act:hover { background: var(--pill-hover); }
  .mini { display: inline-flex; align-items: center; gap: 4px; flex: none; height: 24px; padding: 0 8px; border-radius: var(--r-chip); font-size: 11.5px; background: var(--bg-2); }
  .mini:hover:not(:disabled) { background: var(--pill-hover); }
  .mini:disabled { opacity: 0.45; }
  .mini.danger { background: var(--bad); color: #fff; }
  .confirm { display: flex; flex: 1; flex-direction: column; gap: 6px; padding: var(--s-2) var(--s-3); margin: 2px 0 4px; border-radius: var(--r-inner);
             background: color-mix(in srgb, var(--bad) 10%, var(--bg-2)); font-size: 12px; line-height: 1.4; }
  .btns { display: flex; justify-content: flex-end; gap: var(--s-2); }

  dl { display: grid; grid-template-columns: auto 1fr; gap: 3px var(--s-3); margin: var(--s-3) 0 0; font-size: 12px; }
  dt { white-space: nowrap; }
  dd { margin: 0; text-align: right; overflow-wrap: anywhere; }
  .method { margin: var(--s-3) 0 0; line-height: 1.4; }
  .links { display: flex; flex-wrap: wrap; gap: var(--s-1) var(--s-3); margin-top: var(--s-2); }
  .link { display: inline-flex; align-items: center; gap: 4px; color: var(--accent-ink); font-size: 12px; }
  .link:hover { text-decoration: underline; }
  .msg { margin: 0; padding: 0 var(--s-3) var(--s-2); font-size: 12px; color: var(--fg); }
  .msg.bad { color: var(--bad); }
</style>
