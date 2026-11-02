import { windows, sleep, shot, seq, curate, widthAt, lodpi } from './cdp.mjs';
const w = await windows(); const bar = w.bar;
let ids = await curate(bar);
await widthAt(bar, 2300, 60, 2);
await bar.ev(`document.querySelector('.scroller')?.scrollTo(0,0)`);
await bar.ev(`window.__TAURI_INTERNALS__.invoke('set_edit_mode', { on: true })`);
await sleep(900);
await shot(bar, 'edit_on');
const rect = async (id) => JSON.parse(await bar.ev(`JSON.stringify((r => ({ x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width }))(document.querySelectorAll('.scroller > .slot')[${ids.indexOf(id)}].getBoundingClientRect()))`));
const from = await rect('solat'), to = await rect('clock');
console.log(from, to);
const mouse = (type, x, y) => bar.call('Input.dispatchMouseEvent', { type, x, y, button: 'left', buttons: type === 'mouseReleased' ? 0 : 1, clickCount: 1, pointerType: 'mouse' });
const run = seq(bar, 'drag', 3600, 20);
await sleep(300);
await mouse('mousePressed', from.x, from.y);
await sleep(320);
const N = 40;
for (let i = 1; i <= N; i++) { const p = i / N, e = p < .5 ? 2 * p * p : 1 - Math.pow(-2 * p + 2, 2) / 2; await mouse('mouseMoved', from.x + (to.x - from.x - to.w / 2) * e, from.y + Math.sin(p * Math.PI) * 6); await sleep(30); }
await sleep(250);
await mouse('mouseReleased', to.x - to.w / 2, to.y);
console.log('drag frames', await run);
await sleep(800);
// remove: the badge on a slot in edit mode
console.log(await bar.ev(`JSON.stringify([...document.querySelectorAll('.scroller > .slot .badge, .scroller .remove, .scroller [aria-label*=Remove]')].slice(0,5).map(e => e.className + '|' + (e.getAttribute('aria-label')||'')))`));
process.exit(0);
