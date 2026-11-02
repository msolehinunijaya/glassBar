// Capture scenes in one session (keeps the demo script installed). node scenes.mjs [--fresh] <scene...>
import { installAndReload, windows, shot, seq, sleep, curate, widthAt, hidpi, lodpi } from './cdp.mjs';
import { DEMO } from './demo.mjs';
const args = process.argv.slice(2);
const w = args.includes('--fresh') ? await installAndReload(DEMO) : await windows();
const bar = w.bar, pop = w.popup;
const ids = await curate(bar);
await bar.ev(`window.__demo.emit('media:changed', window.__demo.media()).then(()=>1)`);
const DOCKW = 2900;

const slot = (i) => `document.querySelectorAll('.scroller > .slot')[${i}]`;
async function reveal(id) {
  const i = ids.indexOf(id);
  await bar.ev(`${slot(i)}?.scrollIntoView({ inline: 'center', block: 'nearest' })`);
  await sleep(450);
  return i;
}
async function openPopup(id) {
  const i = await reveal(id);
  const r = await bar.ev(`(() => { const el = ${slot(i)}; if (!el) return 'no el'; (el.querySelector('.pill, button') ?? el).click(); return 'clicked'; })()`);
  await sleep(1800);
  return r;
}
async function popupAt(id, key, width, height) {
  const i = await reveal(id);
  const inst = JSON.parse(await bar.ev(`window.__TAURI_INTERNALS__.invoke('get_config').then(c => JSON.stringify(c.profiles[c.activeProfile].widgets[${i}].instance))`));
  await bar.ev(`(() => { const r = ${slot(i)}.getBoundingClientRect();
    return window.__TAURI_INTERNALS__.invoke('popup_open', { instance: ${JSON.stringify(key)}.replace('$I', ${JSON.stringify(inst)}), x: r.x, y: r.y, w: r.width, h: r.height, width: ${width}, height: ${height}, focus: true }); })()`);
  await sleep(2000);
}
async function clickIn(id, sel, n = 0) {
  const i = await reveal(id);
  const r = await bar.ev(`(() => { const e = ${slot(i)}.querySelectorAll(${JSON.stringify(sel)})[${n}]; if (!e) return 'none'; e.click(); return 'ok'; })()`);
  await sleep(1900);
  return r;
}
const closePopup = async () => { await bar.ev(`window.__TAURI_INTERNALS__.invoke('popup_hide')`); await sleep(700); };
const snapPop = async (n) => { await hidpi(pop, 2.5); await shot(pop, 'pop_' + n); await lodpi(pop); await closePopup(); console.log(n); };
const emitTo = (event, payload) => bar.ev(`window.__TAURI_INTERNALS__.invoke('plugin:event|emit_to', { target: { kind: 'AnyLabel', label: 'bar' }, event: ${JSON.stringify(event)}, payload: ${JSON.stringify(payload)} }).then(()=>'ok', e=>'ERR '+e)`);
async function alertSeq(name, ms, fire) {
  await widthAt(bar, DOCKW, 60, 2);
  await bar.ev(`document.querySelector('.scroller')?.scrollTo(0, 0)`); await sleep(200);
  const run = seq(bar, name, ms, 20);
  await sleep(150); console.log(' fire', await fire());
  console.log(name, await run, 'frames');
}

import fs from 'node:fs';
const S = {
  async hero() {
    await widthAt(bar, DOCKW, 60, 2);
    await bar.ev(`document.querySelector('.scroller')?.scrollTo(0, 0)`);
    await bar.ev(`(window.__demo.restart(51000), window.__demo.emit('media:changed', window.__demo.media())).then(() => 1)`);
    await sleep(1500);
    await shot(bar, 'dock_hero');
    const r = JSON.parse(await bar.ev(`JSON.stringify([...document.querySelectorAll('.scroller > .slot')].map(e => { const r = e.getBoundingClientRect(); return [Math.round(r.x*10)/10, Math.round(r.width*10)/10]; }))`));
    const out = {}; ids.forEach((id, i) => { if (r[i] && r[i][1] > 0) out[id] = r[i]; });
    const surf = JSON.parse(await bar.ev(`JSON.stringify((r => [r.x, r.y, r.width, r.height])(document.querySelector('.surface').getBoundingClientRect()))`));
    fs.writeFileSync(new URL('../assets/slots.json', import.meta.url), JSON.stringify({ width: DOCKW, surface: surf, slots: out }, null, 1));
    console.log(out, surf);
    await bar.ev(`window.__TAURI_INTERNALS__.invoke('set_edit_mode', { on: true })`); await sleep(1000);
    await shot(bar, 'dock_edit');
    await bar.ev(`window.__TAURI_INTERNALS__.invoke('set_edit_mode', { on: false })`); await sleep(800);
    await lodpi(bar);
  },
  async eq() {
    const b64 = fs.readFileSync(new URL('./eqclip.mp3', import.meta.url)).toString('base64');
    await widthAt(bar, DOCKW, 60, 2);
    await bar.ev(`document.querySelector('.scroller')?.scrollTo(0, 0)`);
    await bar.ev(`(window.__demo.restart(38500), window.__demo.emit('media:changed', window.__demo.media())).then(() => 1)`);
    await bar.ev(`window.__demo.spectrum('${b64}').then(stop => { window.__eqStop = stop; return 'spectrum on'; })`).then(console.log);
    await sleep(600);
    const [a, b] = [ids.indexOf('music'), ids.indexOf('equalizer')];
    const clip = JSON.parse(await bar.ev(`(() => { const r1 = ${slot(a)}.getBoundingClientRect(), r2 = ${slot(b)}.getBoundingClientRect(); return JSON.stringify({ x: Math.floor(r1.left) - 6, y: 0, width: Math.ceil(r2.right - r1.left) + 12, height: 60 }); })()`));
    console.log(clip);
    console.log('eq', await seq(bar, 'eq', 10000, 20, clip));
    await bar.ev(`window.__eqStop(); 1`);
    await lodpi(bar);
  },
  async popups() {
    for (const id of ['music', 'equalizer', 'quicksettings', 'sysmon', 'productivity', 'notifications']) { await openPopup(id); await snapPop(id); }
  },
  async popups2() {
    for (const [id, key, W, H, name] of [['folders', '__folder:$I|fmvz0q9yfz27', 360, 300, 'folders'], ['ocr', '$I', 380, 540, 'ocr'], ['clock', '$I', 340, 700, 'clock'], ['solat', '$I', 320, 735, 'solat']]) {
      await popupAt(id, key, W, H); await snapPop(name);
    }
  },
  async popups3() {
    await clickIn('clock', 'button.wx'); await snapPop('weather');
    await clickIn('quicksettings', 'div.pill', 2); await snapPop('battery');
    await popupAt('productivity', '__part:todo:$I', 340, 460); await snapPop('todo');
    await popupAt('productivity', '__part:notes:$I', 360, 480); await snapPop('notes');
  },
  async sound() { await clickIn('quicksettings', 'span.more'); await snapPop('sound'); },
  async dockw() {
    for (let W = 900; W <= DOCKW; W += 125) { await widthAt(bar, W, 60, 2); await shot(bar, `dockw/${W}`); }
    await lodpi(bar);
  },
  async alerts() {
    await alertSeq('al_plug', 3200, () => emitTo('dock:alert', { kind: 'charger', plugged: true, percent: 76, secsLeft: null, test: true }));
    await sleep(1500);
    await alertSeq('al_unplug', 3200, () => emitTo('dock:alert', { kind: 'charger', plugged: false, percent: 76, secsLeft: 15600, test: true }));
    await sleep(1500);
    await alertSeq('al_volume', 2600, async () => { for (let v = 40; v <= 72; v += 4) { await emitTo('dock:alert', { kind: 'volume', percent: v, muted: false, test: true }); await sleep(110); } return 'ramp'; });
    await sleep(1800);
    await alertSeq('al_bright', 2600, async () => { for (let v = 50; v <= 85; v += 5) { await emitTo('dock:alert', { kind: 'brightness', percent: v, test: true }); await sleep(110); } return 'ramp'; });
    await sleep(1800);
    await alertSeq('al_prayer', 4500, () => emitTo('dock:alert', { kind: 'prayer', name: 'Maghrib', at: new Date().setHours(19, 0, 0, 0), test: true }));
    await sleep(9000);
    await lodpi(bar);
  },
  async banner() {
    const b = w.banner;
    await bar.ev(`window.__demo.emit('banner:push', { id: 901, source: 'WhatsApp', title: 'Aisyah', body: 'The new dock looks amazing!', at: Date.now(), data: null }).then(()=>1)`);
    await sleep(80);
    await hidpi(b, 2);
    console.log('banner', await seq(b, 'banner', 2500, 20));
    await lodpi(b);
  },
};
for (const a of args.filter((x) => !x.startsWith('--'))) { console.log('==', a); await S[a](); }
process.exit(0);
