// Demo data, injected into a webview by patching window.__TAURI_INTERNALS__.invoke in memory.
// Nothing is written to GlassBar's DB: data calls answer from here, everything else passes through.
const SRC = String.raw`(function install() {
  if (window.__demo) return 'already';
  const T = window.__TAURI_INTERNALS__; if (!T || !T.invoke || !document.documentElement) { setTimeout(install, 0); return 'wait'; }
  const HIDE = __HIDE__;   // your real place name (env HIDE_PLACE), shown as Kuala Lumpur
  const now = Date.now();
  const art = (() => { const c = document.createElement('canvas'); c.width = c.height = 128; const g = c.getContext('2d');
    const l = g.createLinearGradient(0, 0, 128, 128); l.addColorStop(0, '#ff7ab6'); l.addColorStop(.5, '#7a5cff'); l.addColorStop(1, '#19d3ff');
    g.fillStyle = l; g.fillRect(0, 0, 128, 128); g.fillStyle = 'rgba(255,255,255,.85)'; g.beginPath(); g.arc(64, 64, 26, 0, 7); g.fill();
    g.fillStyle = 'rgba(122,92,255,.9)'; g.beginPath(); g.arc(64, 64, 8, 0, 7); g.fill(); return c.toDataURL(); })();
  let start = now - 48000;
  const media = () => ({ app: 'Spotify', title: 'Glass Hours', artist: 'Nova Lane', album: 'Clear Skies', art, playing: !window.__demoPaused,
    positionMs: Date.now() - start, durationMs: 197000, updatedAtMs: Date.now(), canNext: true, canPrev: true });
  const L = ['Light comes through the glass', 'Every hour feels brand new', 'Hold the moment, let it last', 'All the colours shining through',
    'Clear skies over everything', 'Glass hours, golden hours', 'Turn it up and let it ring', 'Glass hours, all of ours'];
  const lyrics = { lines: L.map((text, i) => ({ atMs: 40000 + i * 4500, text })), source: 'lrclib', fetchedAt: now };
  const todos = [
    { id: 1, text: 'Finish the launch deck', done: true, dueAt: null, createdAt: now - 9e6 },
    { id: 2, text: 'Call Aisyah about the design', done: false, dueAt: now + 3.6e6, createdAt: now - 8e6 },
    { id: 3, text: 'Gym at 6 pm', done: false, dueAt: null, createdAt: now - 7e6 },
    { id: 4, text: 'Buy groceries', done: false, dueAt: null, createdAt: now - 6e6 } ];
  const notes = [
    { id: 1, text: 'Ideas for the weekend\n- Hike Bukit Gasing\n- Try the new cafe', color: 'yellow', pinned: true, images: [], updatedAt: now - 3e5, deletedAt: null },
    { id: 2, text: 'Wi-Fi for guests: GlassHome', color: 'blue', pinned: false, images: [], updatedAt: now - 6e6, deletedAt: null } ];
  const alerts = [
    { id: 1, source: 'WhatsApp', title: 'Aisyah', body: 'The new dock looks amazing!', at: now - 6e4, missed: false, read: false, data: null },
    { id: 2, source: 'Calendar', title: 'Design review', body: 'Today at 3:00 PM', at: now - 9e5, missed: false, read: false, data: null } ];
  const clips = ['Meeting moved to 3:00 PM', 'https://glassbar.app', 'Order #4821 shipped', 'Jalan Ampang, Kuala Lumpur'].map((text, i) =>
    ({ id: i + 1, at: now - (i + 1) * 4e5, kind: 'text', text, thumb: null, w: null, h: null, pinned: i === 0 }));
  const audio = { volume: 0.65, muted: false, devices: [{ id: 'spk', name: 'Speakers (Realtek(R) Audio)', isDefault: true }, { id: 'bt', name: 'Galaxy Buds', isDefault: false }] };
  const sessions = [{ pid: 11, name: 'Spotify', exe: 'Spotify.exe', volume: 0.85, muted: false }, { pid: 12, name: 'chrome', exe: 'chrome.exe', volume: 0.6, muted: false }];
  const map = { audio_state: () => audio, audio_sessions: () => sessions, shots_list: () => [], clip_list: () => clips, media_info: media, lyrics_get: () => lyrics, todo_list: () => todos, notes_list: (a) => (a && a.trash ? [] : notes), alerts_list: () => alerts,
    media_source: () => ({ name: 'Spotify', id: 'Spotify', exe: 'Spotify.exe', window: null, aumid: null }),
    media_control: (a) => { if (a.action === 'toggle') window.__demoPaused = !window.__demoPaused; return null; } };

  // Tauri's own invoke is locked, but its IPC goes through fetch('http://ipc.localhost/<cmd>'):
  // answer the demo commands there, and filter the real media events at listen time.
  const block = { 'media:changed': (p) => p && p.title === 'Glass Hours' };
  const f0 = window.fetch.bind(window);
  const json = (v) => new Response(JSON.stringify(v ?? null), { headers: { 'Content-Type': 'application/json', 'Tauri-Response': 'ok' } });
  window.fetch = async (url, init) => {
    const u = typeof url === 'string' ? url : url && url.url;
    const m = u && /^https?:\/\/ipc\.localhost\/(.+)$/.exec(u);
    if (m) {
      const cmd = decodeURIComponent(m[1]);
      let args = {}; try { args = init && typeof init.body === 'string' ? JSON.parse(init.body) : {}; } catch {}
      if (map[cmd]) return json(map[cmd](args));
      if (cmd === 'plugin:event|listen' && block[args.event]) {
        const orig = T.callbacks.get(args.handler), f = block[args.event];
        if (orig) { args.handler = T.transformCallback((ev) => { if (f(ev && ev.payload)) orig(ev); }); init = { ...init, body: JSON.stringify(args) }; }
      }
    }
    const r = await f0(url, init);
    if (!m || !(r.headers.get('content-type') || '').startsWith('application/json')) return r;
    // Hide the real place name everywhere (weather, prayer zone, location).
    const t = await r.text();
    const clean = HIDE ? t.split(HIDE).join('Kuala Lumpur') : t;
    return new Response(clean, { status: r.status, headers: r.headers });
  };
  const real = (cmd, args) => T.invoke(cmd, args);
  // Feed the real equalizer from our own track (WebAudio analyser; plays silently, so a muted PC is fine).
  const spectrum = async (b64, n = 32) => {
    const ac = new AudioContext(); const buf = await ac.decodeAudioData(Uint8Array.from(atob(b64), (c) => c.charCodeAt(0)).buffer);
    const src = ac.createBufferSource(); src.buffer = buf; const an = ac.createAnalyser(); an.fftSize = 2048; an.smoothingTimeConstant = 0.6;
    const g = ac.createGain(); g.gain.value = 0; src.connect(an); an.connect(g); g.connect(ac.destination); src.start();
    const bins = new Uint8Array(an.frequencyBinCount); const hz = ac.sampleRate / an.fftSize;
    const edges = Array.from({ length: n + 1 }, (_, i) => 40 * Math.pow(16000 / 40, i / n));
    const id = setInterval(() => { an.getByteFrequencyData(bins);
      const out = edges.slice(0, n).map((lo, i) => { const a = Math.floor(lo / hz), b = Math.max(a + 1, Math.floor(edges[i + 1] / hz)); let m = 0; for (let k = a; k < b; k++) m = Math.max(m, bins[k]); return m; });
      real('plugin:event|emit', { event: 'audio:spectrum', payload: out }); }, 33);
    return () => { clearInterval(id); src.stop(); ac.close(); };
  };
  window.__demo = { spectrum, restart: (pos) => { start = Date.now() - pos; }, media, emit: (event, payload) => real('plugin:event|emit', { event, payload }) };
  return 'ok';
})()`;
export const DEMO = SRC.replace('__HIDE__', JSON.stringify(process.env.HIDE_PLACE || ''));
