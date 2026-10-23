<script lang="ts">
  import { availableMonitors, type Monitor } from '@tauri-apps/api/window';
  import { ArrowDown, ArrowUp, ChevronDown, Copy, Trash2 } from '@lucide/svelte';
  import { profile, store, update, updateWidgetSettings } from '../lib/config.svelte';
  import { ipc, type Config } from '../lib/ipc';
  import { reorder } from '../lib/geometry';
  import { addWidget, settingsOf, widgets } from '../widgets/registry';

  const sections = ['Bar', 'Appearance', 'Widgets', 'Profiles', 'Startup', 'Backup'] as const;
  let section = $state<(typeof sections)[number]>('Bar');
  let monitors = $state<Monitor[]>([]);
  availableMonitors().then((m) => (monitors = m));

  const c = $derived(store.config!);
  const bar = $derived(profile().bar);
  const set = (f: (c: Config) => void) => update(f);
  const setBar = (f: (b: Config['profiles'][string]['bar']) => void) => set((c) => f(c.profiles[c.activeProfile].bar));

  let open = $state<string | null>(null);
  let newName = $state('');
  let exported = $state('');
  let importText = $state('');
  let message = $state('');

  function moveWidget(i: number, d: number) {
    set((c) => { const p = c.profiles[c.activeProfile]; p.widgets = reorder(p.widgets, i, i + d); });
  }

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
          <select value={bar.monitor} onchange={(e) => setBar((b) => { b.monitor = +e.currentTarget.value; b.floating = null; })}>
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
        <div class="field"><span>Length {Math.round(bar.length * 100)}%</span>
          <input type="range" min="0.2" max="1" step="0.01" value={bar.length} onchange={(e) => setBar((b) => (b.length = +e.currentTarget.value))} />
        </div>
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
        <div class="field"><span>Tint {Math.round(c.theme.opacity * 100)}%</span>
          <input type="range" min="0" max="1" step="0.01" value={c.theme.opacity} onchange={(e) => set((c) => (c.theme.opacity = +e.currentTarget.value))} />
        </div>
        <div class="field" class:off={c.theme.nativeBlur}><span>CSS blur {c.theme.blur}px</span>
          <input type="range" min="0" max="60" value={c.theme.blur} disabled={c.theme.nativeBlur} onchange={(e) => set((c) => (c.theme.blur = +e.currentTarget.value))} />
        </div>
      </div>

    {:else if section === 'Widgets'}
      <div class="group">
        {#each profile().widgets as w, i (w.instance)}
          {@const m = widgets[w.id]}
          {#if m}
            <div class="wrow">
              <span class="wico"><m.icon size={16} /></span>
              <b>{m.name}</b>
              <select value={w.size} onchange={(e) => set((c) => (c.profiles[c.activeProfile].widgets[i].size = e.currentTarget.value as typeof w.size))}>
                {#each m.sizes as s (s)}<option value={s}>{s}</option>{/each}
              </select>
              <button class="icon-btn" disabled={i === 0} onclick={() => moveWidget(i, -1)} aria-label="Move up"><ArrowUp size={14} /></button>
              <button class="icon-btn" disabled={i === profile().widgets.length - 1} onclick={() => moveWidget(i, 1)} aria-label="Move down"><ArrowDown size={14} /></button>
              {#if m.SettingsForm}
                <button class="icon-btn" class:rot={open === w.instance} onclick={() => (open = open === w.instance ? null : w.instance)} aria-label="Widget settings"><ChevronDown size={14} /></button>
              {/if}
              <button class="icon-btn" onclick={() => set((c) => c.profiles[c.activeProfile].widgets.splice(i, 1))} aria-label="Remove"><Trash2 size={14} /></button>
            </div>
            {#if open === w.instance && m.SettingsForm}
              <div class="wform"><m.SettingsForm instance={w.instance} settings={settingsOf(m, w)} save={(p) => updateWidgetSettings(w.instance, p)} /></div>
            {/if}
          {/if}
        {/each}
      </div>
      <h2>Add a widget</h2>
      <div class="gallery">
        {#each Object.values(widgets) as m (m.id)}
          <button class="gcard" onclick={() => addWidget(m.id)}>
            <span class="wico"><m.icon size={18} /></span>
            <b>{m.name}</b>
            <span class="hint">{m.description}</span>
          </button>
        {/each}
      </div>

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

    {:else if section === 'Startup'}
      <div class="group">
        <label class="field"><span>Launch GlassBar when I sign in</span>
          <input type="checkbox" checked={c.startup.launchAtLogin} onchange={(e) => set((c) => (c.startup.launchAtLogin = e.currentTarget.checked))} />
        </label>
      </div>

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
  h1 { font-size: 22px; font-weight: 600; margin: 0 0 16px; }
  h2 { font-size: 15px; font-weight: 600; margin: 22px 0 10px; }
  .group { background: var(--panel); border-radius: 14px; padding: 6px 16px; margin-bottom: 14px; display: flex; flex-direction: column; gap: 8px; border: 1px solid var(--line-soft); }
  .field { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 8px 0; }
  .field + .field { border-top: 1px solid var(--line-soft); }
  .field.off { opacity: 0.5; }
  .field input[type='range'] { width: 220px; }
  .hint { font-size: 12px; color: var(--fg-dim); margin: -4px 0 6px; }
  .seg { display: flex; gap: 2px; padding: 3px; border-radius: 10px; background: var(--pill); }
  .seg button { padding: 5px 12px; border-radius: 8px; text-transform: capitalize; }
  .seg .sel { background: var(--accent); color: #fff; }
  select, input:not([type]), textarea, .grow { padding: 6px 10px; border-radius: 8px; border: 1px solid var(--line-soft); background: var(--settings-bg); }
  textarea { width: 100%; font-family: Consolas, monospace; font-size: 12px; resize: vertical; }
  :global(.btn) { display: inline-flex; align-items: center; gap: 6px; padding: 7px 14px; border-radius: 9px; background: var(--accent); color: #fff; }
  :global(.btn:disabled) { opacity: 0.5; }
  .wrow { display: flex; align-items: center; gap: 8px; padding: 6px 0; }
  .wrow b { flex: 1; font-weight: 500; }
  .grow { flex: 1; }
  .wico { display: grid; place-items: center; width: 28px; height: 28px; border-radius: 8px; background: var(--accent); color: #fff; flex: none; }
  .wform { padding: 4px 0 12px 36px; display: flex; flex-direction: column; gap: 8px; }
  .wform :global(label) { display: flex; align-items: center; gap: 8px; }
  .rot { transform: rotate(180deg); }
  .gallery { display: grid; grid-template-columns: repeat(auto-fill, minmax(190px, 1fr)); gap: 10px; }
  .gcard { display: flex; flex-direction: column; align-items: flex-start; gap: 6px; padding: 14px; text-align: left; border-radius: 14px; background: var(--panel); border: 1px solid var(--line-soft); transition: transform 0.3s var(--spring); }
  .gcard:hover { transform: translateY(-2px); }
  .gcard .hint { margin: 0; }
</style>
