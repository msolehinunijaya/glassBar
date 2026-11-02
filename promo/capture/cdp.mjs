// Tiny CDP client for the running GlassBar webviews (port 9333).
import fs from 'node:fs';
export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
export const OUT = new URL('../assets/cap/', import.meta.url);

async function open(x) {
  const ws = new WebSocket(x.webSocketDebuggerUrl);
  await new Promise((o) => ws.addEventListener('open', o));
  let id = 0; const ls = new Set();
  ws.addEventListener('message', (m) => { const d = JSON.parse(m.data); for (const f of ls) f(d); });
  const call = (method, params = {}) => new Promise((ok, no) => {
    const my = ++id;
    const f = (d) => { if (d.id === my) { ls.delete(f); d.error ? no(new Error(method + ': ' + d.error.message)) : ok(d.result); } };
    ls.add(f); ws.send(JSON.stringify({ id: my, method, params }));
  });
  const ev = async (e) => {
    const r = await call('Runtime.evaluate', { expression: e, awaitPromise: true, returnByValue: true });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
    return r.result?.value;
  };
  return { ws, call, ev, on: (f) => ls.add(f), off: (f) => ls.delete(f), target: x };
}

export async function windows() {
  const t = (await (await fetch('http://127.0.0.1:9333/json')).json()).filter((x) => x.type === 'page');
  const out = {};
  for (const x of t) { const p = await open(x); const lbl = await p.ev('document.documentElement.dataset.window'); out[lbl] = p; }
  return out;
}

// Transparent PNG of the whole webview (or a clip in CSS px).
export async function shot(p, name, clip) {
  await p.call('Emulation.setDefaultBackgroundColorOverride', { color: { r: 0, g: 0, b: 0, a: 0 } });
  const params = { format: 'png', captureBeyondViewport: false };
  if (clip) params.clip = { ...clip, scale: 1 };
  const r = await p.call('Page.captureScreenshot', params);
  fs.writeFileSync(new URL(name + '.png', OUT), Buffer.from(r.data, 'base64'));
  return name;
}

// PNG sequence at ~fps for ms milliseconds, by repeated captures.
export async function seq(p, name, ms, fps = 20, clip) {
  fs.mkdirSync(new URL(name + '/', OUT), { recursive: true });
  const t0 = Date.now(); let i = 0;
  while (Date.now() - t0 < ms) {
    const due = t0 + i * (1000 / fps);
    if (Date.now() < due) await sleep(due - Date.now());
    await shot(p, `${name}/${String(i).padStart(4, '0')}`, clip); i++;
  }
  return i;
}

// Install a script in every webview for every future load, then reload them all.
export async function installAndReload(src) {
  let w = await windows();
  for (const k of Object.keys(w)) {
    await w[k].call('Page.enable');
    await w[k].call('Page.addScriptToEvaluateOnNewDocument', { source: src });
    await w[k].call('Page.reload');
  }
  await sleep(5000);
  return w; // same targets survive reload
}

// Re-render a webview at a higher pixel ratio (same CSS size) for crisp close-ups.
export async function hidpi(p, dsf = 3) {
  const s = JSON.parse(await p.ev('JSON.stringify([innerWidth, innerHeight])'));
  await p.call('Emulation.setDeviceMetricsOverride', { width: s[0], height: s[1], deviceScaleFactor: dsf, mobile: false });
  await sleep(400);
  return s;
}
export const lodpi = (p) => p.call('Emulation.clearDeviceMetricsOverride');

// Show only these widget ids on the dock (CSS only, in this page; the config is untouched).
export const KEEP = ['apps', 'folders', 'music', 'equalizer', 'clock', 'quicksettings', 'ocr', 'sysmon', 'solat', 'productivity', 'mirror', 'notifications'];
export async function curate(bar, keep = KEEP) {
  const cfg = JSON.parse(await bar.ev(`window.__TAURI_INTERNALS__.invoke('get_config').then(JSON.stringify)`));
  const ids = cfg.profiles[cfg.activeProfile].widgets.map((x) => x.id);
  const hide = ids.map((id, i) => (keep.includes(id) ? null : i + 1)).filter(Boolean);
  const css = hide.map((n) => `.scroller > .slot:nth-child(${n})`).join(',') + '{display:none !important}';
  await bar.ev(`(()=>{let s=document.getElementById('demo-css');if(!s){s=document.createElement('style');s.id='demo-css';document.head.append(s)}s.textContent=${JSON.stringify(css)};return 1})()`);
  return ids;
}
// Lay the dock out at a given CSS width (as if it were that long) and render at dsf.
export async function widthAt(p, w, h = 60, dsf = 2) {
  await p.call('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: dsf, mobile: false });
  await sleep(350);
}
