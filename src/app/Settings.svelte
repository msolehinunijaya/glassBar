<script lang="ts">
  import { chargerTone, deviceTone } from '../lib/sound';
  import { availableMonitors, type Monitor } from '@tauri-apps/api/window';
  import { Copy, Trash2 } from '@lucide/svelte';
  import { profile, store, update } from '../lib/config.svelte';
  import { events, ipc, type Config } from '../lib/ipc';
  import Updates from './Updates.svelte';
  import LocationSettings from './LocationSettings.svelte';
  import WidgetManager from './widgets/WidgetManager.svelte';
  import About from './About.svelte';

  const sections = ['Bar', 'Appearance', 'Widgets', 'Location', 'Notifications', 'Sounds', 'Profiles', 'Startup', 'Updates', 'Backup', 'About'] as const;
  import { SOURCE_WIDGET } from '../lib/notify';
  const BANNER_SOURCES = [...Object.entries(SOURCE_WIDGET).map(([k, v]) => [k, v.label] as const), ['update', 'Updates'] as const];
  type Section = (typeof sections)[number];
  const asSection = (s: string | null) => sections.find((x) => x === s);
  // Opened on a section (e.g. from a widget's location label): ?section=… or, when already open, an event.
  let section = $state<Section>(asSection(new URLSearchParams(location.search).get('section')) ?? 'Bar');
  events.settingsSection((s) => { const x = asSection(s); if (x) section = x; });
  let monitors = $state<Monitor[]>([]);
  availableMonitors().then((m) => (monitors = m));

  const c = $derived(store.config!);
  const bar = $derived(profile().bar);
  const set = (f: (c: Config) => void) => update(f);
  const setBar = (f: (b: Config['profiles'][string]['bar']) => void) => set((c) => f(c.profiles[c.activeProfile].bar));

  let newName = $state('');
  let exported = $state('');
  let importText = $state('');
  let message = $state('');

  function renameProfile(from: string, to: string) {
    to = to.trim();
    if (!to || to === from || c.profiles[to]) return;
    set((c) => {
      c.profiles = Object.fromEntries(Object.entries(c.profiles).map(([k, v]) => [k === from ? to : k, v]));
      if (c.activeProfile === from) c.activeProfile = to;
    });
  }

  function newProfile() {
    const name = newName.trim();
    if (!name || c.profiles[name]) return;
    set((c) => { c.profiles[name] = JSON.parse(JSON.stringify(c.profiles[c.activeProfile])); c.activeProfile = name; });
    newName = '';
  }

  async function doImport() {
    try { await ipc.importConfig(importText); message = 'Imported.'; importText = ''; }
    catch (e) { message = `Import failed: ${e}`; }
  }
</script>

<div class="settings">
  <nav>
    <div class="brand">GlassBar</div>
    {#each sections as s (s)}
      <button class:sel={section === s} onclick={() => (section = s)}>{s}</button>
    {/each}
  </nav>

  <main>
    <h1>{section}</h1>

    {#if section === 'Bar'}
      <div class="group">
        <div class="field"><span>Position</span>
          <div class="seg">
            {#each ['top', 'bottom', 'left', 'right'] as const as e (e)}
              <button class:sel={!bar.floating && bar.edge === e} onclick={() => setBar((b) => { b.edge = e; b.floating = null; })}>{e}</button>
            {/each}
          </div>
        </div>
        <p class="hint">Or drag the bar anywhere: it snaps to the nearest edge, or floats if dropped mid-screen.</p>
        <div class="field"><span>Monitor</span>
          <select value={bar.monitor ?? ''} onchange={(e) => setBar((b) => { b.monitor = e.currentTarget.value === '' ? null : +e.currentTarget.value; b.floating = null; })}>
            <option value="">Main display</option>
            {#each monitors as m, i (i)}<option value={i}>{i + 1}: {m.name ?? 'Display'} ({m.size.width}×{m.size.height})</option>{/each}
          </select>
        </div>
        <div class="field"><span>Height</span>
          <div class="seg">
            {#each ['compact', 'normal', 'large'] as const as s (s)}
              <button class:sel={bar.size === s} onclick={() => setBar((b) => (b.size = s))}>{s}</button>
            {/each}
          </div>
        </div>
        <div class="field"><span>Side width</span>
          <div class="seg">
            {#each [['slim', 'Slim'], ['regular', 'Regular'], ['wide', 'Wide']] as const as [w, label] (w)}
              <button class:sel={(bar.sideWidth ?? 'regular') === w} onclick={() => setBar((b) => (b.sideWidth = w))}>{label}</button>
            {/each}
          </div>
        </div>
        <p class="hint">How wide the bar is on the left or right edge ({bar.edge === 'left' || bar.edge === 'right' ? 'in use now' : 'used when you move it there'}). Wider shows labels next to the values.</p>
        <div class="field"><span>Length {Math.round(bar.length * 100)}%</span>
          <input type="range" min="0.2" max="1" step="0.01" value={bar.length} onchange={(e) => setBar((b) => (b.length = +e.currentTarget.value))} />
        </div>
        <label class="field"><span>Auto-hide</span>
          <input type="checkbox" checked={bar.autoHide} onchange={(e) => setBar((b) => { b.autoHide = e.currentTarget.checked; b.floating = null; })} />
        </label>
        <p class="hint">The bar slides away and comes back when you touch that screen edge. Tip: drag the bar past the screen edge to turn this on, and drag it back in to turn it off.</p>
        <label class="field"><span>Reserve screen space</span>
          <input type="checkbox" checked={bar.reserveSpace} onchange={(e) => setBar((b) => (b.reserveSpace = e.currentTarget.checked))} />
        </label>
        <p class="hint">Maximized windows stop at the bar instead of covering it.</p>
      </div>
      <div class="group">
        <div class="field"><span>Windows taskbar</span>
          <select value={bar.nativeTaskbar} onchange={(e) => setBar((b) => (b.nativeTaskbar = e.currentTarget.value as typeof b.nativeTaskbar))}>
            <option value="show">Leave as is</option>
            <option value="autohide">Auto-hide</option>
            <option value="hidden">Hide completely</option>
          </select>
        </div>
        <p class="hint">Restored when GlassBar quits. Also restored when you sign out. If GlassBar is force-killed, start it again and choose tray → "Restore Windows taskbar".</p>
      </div>

    {:else if section === 'Appearance'}
      <div class="group">
        <div class="field"><span>Theme</span>
          <div class="seg">
            {#each ['auto', 'light', 'dark'] as const as m (m)}
              <button class:sel={c.theme.mode === m} onclick={() => set((c) => (c.theme.mode = m))}>{m}</button>
            {/each}
          </div>
        </div>
        <label class="field"><span>Accent color</span>
          <input type="color" value={c.theme.accent} onchange={(e) => set((c) => (c.theme.accent = e.currentTarget.value))} />
        </label>
        <label class="field"><span>Native blur (Windows acrylic)</span>
          <input type="checkbox" checked={c.theme.nativeBlur} onchange={(e) => set((c) => (c.theme.nativeBlur = e.currentTarget.checked))} />
        </label>
        <p class="hint">On: real frosted glass over your wallpaper. Off: lighter CSS glass with larger rounded corners.</p>
        {#await ipc.transparencyEffects() then on}{#if !on && c.theme.nativeBlur}
          <p class="hint warn">Transparency effects are off in Windows (Settings › Personalization › Colors), so Windows can't draw the glass. GlassBar uses its own tint until you turn them on.</p>
        {/if}{/await}
        <div class="field"><span>Tint {Math.round(c.theme.opacity * 100)}%</span>
          <input type="range" min="0" max="1" step="0.01" value={c.theme.opacity} onchange={(e) => set((c) => (c.theme.opacity = +e.currentTarget.value))} />
        </div>
        <div class="field" class:off={c.theme.nativeBlur}><span>CSS blur {c.theme.blur}px</span>
          <input type="range" min="0" max="60" value={c.theme.blur} disabled={c.theme.nativeBlur} onchange={(e) => set((c) => (c.theme.blur = +e.currentTarget.value))} />
        </div>
      </div>

    {:else if section === 'Widgets'}
      <WidgetManager />

    {:else if section === 'Location'}
      <LocationSettings />

    {:else if section === 'Profiles'}
      <div class="group">
        {#each Object.keys(c.profiles) as name (name)}
          <div class="wrow">
            <input type="radio" name="active" checked={c.activeProfile === name} onchange={() => set((c) => (c.activeProfile = name))} aria-label="Use {name}" />
            <input class="grow" value={name} onchange={(e) => renameProfile(name, e.currentTarget.value)} aria-label="Profile name" />
            <span class="hint">{c.profiles[name].widgets.length} widgets</span>
            <button class="icon-btn" disabled={Object.keys(c.profiles).length < 2}
                    onclick={() => set((c) => { delete c.profiles[name]; if (c.activeProfile === name) c.activeProfile = Object.keys(c.profiles)[0]; })}
                    aria-label="Delete {name}"><Trash2 size={14} /></button>
          </div>
        {/each}
      </div>
      <div class="wrow">
        <input class="grow" placeholder="New profile name, e.g. Gaming" bind:value={newName} onkeydown={(e) => e.key === 'Enter' && newProfile()} />
        <button class="btn" onclick={newProfile}><Copy size={14} /> Duplicate current</button>
      </div>
      <p class="hint">Switch profiles from here or from the tray icon menu.</p>

    {:else if section === 'Notifications'}
      <div class="group">
        <label class="field"><span>Do Not Disturb</span>
          <input type="checkbox" checked={c.dnd} onchange={(e) => set((c) => (c.dnd = e.currentTarget.checked))} />
        </label>
        <p class="hint">On: no banners, toasts or sounds. Everything still goes into the Notification Center list.</p>
      </div>
      <div class="group">
        <label class="field"><span>Show pop-up banners</span>
          <input type="checkbox" checked={c.banners.show} onchange={(e) => set((c) => (c.banners.show = e.currentTarget.checked))} />
        </label>
        <p class="hint">A small banner next to the dock (timer done, prayer time, download finished…). Click it to jump to the widget. Off: Windows shows its own notification instead.</p>
        {#if c.banners.show}
          <div class="field"><span>Display time</span>
            <span class="segs">{#each [3, 5, 8, 10] as s (s)}<button class="btn" class:on={c.banners.seconds === s} onclick={() => set((c) => (c.banners.seconds = s as 3 | 5 | 8 | 10))}>{s} s</button>{/each}</span>
          </div>
          <div class="field"><span>Position</span>
            <span class="segs"><button class="btn" class:on={c.banners.position === 'auto'} onclick={() => set((c) => (c.banners.position = 'auto'))}>Next to the dock</button>
            <button class="btn" class:on={c.banners.position === 'corner'} onclick={() => set((c) => (c.banners.position = 'corner'))}>Screen corner</button></span>
          </div>
          <div class="field"><span>On click</span>
            <span class="segs"><button class="btn" class:on={c.banners.onClick === 'highlight'} onclick={() => set((c) => (c.banners.onClick = 'highlight'))}>Show the widget</button>
            <button class="btn" class:on={c.banners.onClick === 'open'} onclick={() => set((c) => (c.banners.onClick = 'open'))}>…and open its panel</button></span>
          </div>
          <label class="field"><span>Play the alert sound with the banner</span>
            <input type="checkbox" checked={c.banners.sound} onchange={(e) => set((c) => (c.banners.sound = e.currentTarget.checked))} />
          </label>
          <h2>Banners per widget</h2>
          {#each BANNER_SOURCES as [k, label] (k)}
            <label class="field"><span>{label}</span>
              <input type="checkbox" checked={!c.banners.muted.includes(k)}
                     onchange={(e) => set((c) => (c.banners.muted = e.currentTarget.checked ? c.banners.muted.filter((m) => m !== k) : [...c.banners.muted, k]))} />
            </label>
          {/each}
          <p class="hint">Off: no banner for that widget (it still goes into the list).</p>
        {/if}
      </div>

    {:else if section === 'Sounds'}
      <div class="group">
        <label class="field"><span>Charger sounds (plugged in / unplugged)</span>
          <input type="checkbox" checked={c.deviceSounds.charger} onchange={(e) => set((c) => (c.deviceSounds.charger = e.currentTarget.checked))} />
        </label>
        <div class="field"><span>Preview</span>
          <span><button class="btn" onclick={() => chargerTone(true, c.deviceSounds.volume)}>Plugged in</button>
          <button class="btn" onclick={() => chargerTone(false, c.deviceSounds.volume)}>Unplugged</button></span>
        </div>
        <p class="hint">Plays only when the charger state really changes (Windows power events), never repeatedly while it stays plugged in.</p>
      </div>
      <div class="group">
        <label class="field"><span>Sound when a USB device is connected</span>
          <input type="checkbox" checked={c.deviceSounds.connect} onchange={(e) => set((c) => (c.deviceSounds.connect = e.currentTarget.checked))} />
        </label>
        <label class="field"><span>Sound when a USB device is removed</span>
          <input type="checkbox" checked={c.deviceSounds.disconnect} onchange={(e) => set((c) => (c.deviceSounds.disconnect = e.currentTarget.checked))} />
        </label>
        <div class="field"><span>Volume {Math.round(c.deviceSounds.volume * 100)}%</span>
          <input type="range" min="0.05" max="1" step="0.05" value={c.deviceSounds.volume} onchange={(e) => set((c) => (c.deviceSounds.volume = +e.currentTarget.value))} />
        </div>
        <div class="field"><span>Preview</span>
          <span><button class="btn" onclick={() => deviceTone(true, c.deviceSounds.volume)}>Connect</button>
          <button class="btn" onclick={() => deviceTone(false, c.deviceSounds.volume)}>Disconnect</button></span>
        </div>
        <p class="hint">Uses Windows' USB device notifications (no polling); quick bursts play once. Windows also plays its own device sounds unless you turn them off in Control Panel › Sound › Sounds. Silent while Do Not Disturb is on. Independent of timer, Pomodoro and prayer sounds.</p>
      </div>

    {:else if section === 'Startup'}
      <div class="group">
        <label class="field"><span>Launch GlassBar when I sign in</span>
          <input type="checkbox" checked={c.startup.launchAtLogin} onchange={(e) => set((c) => (c.startup.launchAtLogin = e.currentTarget.checked))} />
        </label>
      </div>

    {:else if section === 'Updates'}
      <Updates />

    {:else if section === 'Backup'}
      <div class="group">
        <div class="wrow">
          <button class="btn" onclick={async () => (exported = await ipc.exportConfig())}>Export configuration</button>
          {#if exported}<button class="btn" onclick={() => navigator.clipboard.writeText(exported).then(() => (message = 'Copied to clipboard.'))}>Copy</button>{/if}
        </div>
        {#if exported}<textarea readonly rows="10">{exported}</textarea>{/if}
      </div>
      <div class="group">
        <b>Import</b>
        <textarea rows="8" placeholder="Paste exported JSON here" bind:value={importText}></textarea>
        <button class="btn" disabled={!importText.trim()} onclick={doImport}>Import and apply</button>
      </div>
      {#if message}<p class="hint">{message}</p>{/if}

    {:else if section === 'About'}
      <About onupdates={() => (section = 'Updates')} />
    {/if}
  </main>
</div>

<style>
  :global(html[data-window='settings'] body) { background: var(--settings-bg); }
  :global(:root[data-theme='light']) { --settings-bg: #f2f2f7; --panel: #fff; }
  :global(:root[data-theme='dark']) { --settings-bg: #1c1c1e; --panel: #2c2c2e; }
  :global(html[data-window='settings']) { user-select: text; }

  .settings { display: flex; height: 100%; }
  nav { width: 190px; padding: 18px 10px; display: flex; flex-direction: column; gap: 2px; border-right: 1px solid var(--line-soft); }
  .brand { font-weight: 600; font-size: 16px; padding: 0 10px 14px; }
  nav button { text-align: left; padding: 8px 12px; border-radius: 10px; }
  nav button:hover { background: var(--pill); }
  nav .sel { background: var(--accent); color: #fff; }
  main { flex: 1; overflow: auto; padding: 18px 28px 40px; }
  /* Used by widget panels in every window too. */
  :global(.btn) { display: inline-flex; align-items: center; gap: 6px; padding: 7px 14px; border-radius: 9px; background: var(--accent); color: #fff; }
  :global(.btn:disabled) { opacity: 0.5; }
  /* Shared by every section component (Location, About, widget manager…). */
  .settings :global {
    h1 { font-size: 22px; font-weight: 600; margin: 0 0 16px; }
    h2 { font-size: 15px; font-weight: 600; margin: 22px 0 10px; }
    .group { background: var(--panel); border-radius: 14px; padding: 6px 16px; margin-bottom: 14px; display: flex; flex-direction: column; gap: 8px; border: 1px solid var(--line-soft); }
    .field { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 8px 0; }
    .field + .field { border-top: 1px solid var(--line-soft); }
    .field.off { opacity: 0.5; }
    .field input[type='range'] { width: 220px; }
    .hint.warn { color: #ff9f0a; }
    .hint { font-size: 12px; color: var(--fg-dim); margin: -4px 0 6px; }
    .seg { display: flex; gap: 2px; padding: 3px; border-radius: 10px; background: var(--pill); }
    .seg button { padding: 5px 12px; border-radius: 8px; text-transform: capitalize; }
    .seg .sel { background: var(--accent); color: #fff; }
    select, input:not([type]), textarea, .grow { padding: 6px 10px; border-radius: 8px; border: 1px solid var(--line-soft); background: var(--settings-bg); }
    textarea { width: 100%; font-family: Consolas, monospace; font-size: 12px; resize: vertical; }
    /* Choice buttons (Notifications): plain, the chosen one in the accent colour. */
    .segs { display: inline-flex; gap: 4px; flex-wrap: wrap; justify-content: flex-end; }
    .segs .btn { background: var(--pill); color: var(--fg); padding: 6px 10px; font-size: 12px; }
    .segs .btn.on { background: var(--accent); color: #fff; }
    .wrow { display: flex; align-items: center; gap: 8px; padding: 6px 0; }
    .wrow b { flex: 1; font-weight: 500; }
    .grow { flex: 1; }
    .wico { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 8px; background: var(--accent); color: #fff; flex: none; }
    .wform { padding: 4px 0 12px 36px; display: flex; flex-direction: column; gap: 8px; }
    .wform label { display: flex; align-items: center; gap: 8px; }
    .gallery { display: grid; grid-template-columns: repeat(auto-fill, minmax(190px, 1fr)); gap: 10px; }
    .gcard { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; padding: 14px; text-align: left; border-radius: 14px; background: var(--panel); border: 1px solid var(--line-soft); transition: transform 0.3s var(--spring); }
    .gcard:hover { transform: translateY(-2px); }
    .gcard .hint { margin: 0; }
  }
</style>
