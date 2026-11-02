import { windows, sleep, shot, hidpi, lodpi, curate, widthAt } from './cdp.mjs';
const w = await windows(); const s = w.settings, bar = w.bar;
await curate(bar);
await s.ev(`[...document.querySelectorAll('.nitem')].find(b => b.dataset.s === 'Appearance')?.click(); 1`);
await sleep(1000);
for (const name of ['Glass', 'Dark', 'Light', 'Custom', 'Liquid Glass']) {
  const r = await s.ev(`(() => { const b = [...document.querySelectorAll('button, [role=button]')].find(e => e.innerText.trim().split(String.fromCharCode(10))[0] === ${JSON.stringify(name)}); if (!b) return 'none'; b.click(); return 'ok'; })()`);
  await sleep(1500);
  const tag = name.replace(' ', '').toLowerCase();
  await hidpi(s, 1.5); await shot(s, 'set_theme_' + tag); await lodpi(s);
  await widthAt(bar, 2300, 60, 2); await bar.ev(`document.querySelector('.scroller')?.scrollTo(0,0)`); await sleep(300);
  await shot(bar, 'dock_theme_' + tag); await lodpi(bar);
  console.log(name, r);
}
process.exit(0);
