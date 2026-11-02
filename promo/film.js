// GlassBar promo: pure function of time. window.seek(t) paints frame t (see lib/motion.js).
// Timing in beats (u) on the measured 128 BPM grid; 4 beats = 1 bar. docs/shotlist.md is the plan.
// Every UI pixel is a real GlassBar capture (assets/film); only the wallpaper and captions are drawn here.
import * as M from './lib/motion.js';
import MAN from './assets/film/manifest.js';
import EDIT from './assets/slots_edit.json' with { type: 'json' };

const { W, H, E, prog, lerp, clamp, springU, springKeys, SPRING, wobble, font, layout, glyph, text, rrect, noise1, bump, kf } = M;

const ACCENT = '#0a84ff';
const DISPLAY = 'Display', UI = 'UI';
const SLOTS = MAN.slots.slots;           // css px in the 2900-wide hero dock
const DCSS = 2900;                        // hero dock width (css px); images are 2x
const K = 1840 / DCSS;                    // world px per css px for the establishing dock
const DX = (W - DCSS * K) / 2, DY = 992;  // dock top-left in world space
const DH = 60 * K;
const cx = (css) => DX + css * K;         // dock css x -> world x
const slotC = (id) => cx(SLOTS[id][0] + SLOTS[id][1] / 2);

// ------------------------------------------------------------------ sound hits
const HITS = [
  // Few and soft: gentle UI sounds only where the app itself would make one.
  [2, 'title', 'pop', { pitch: 'E5', gain: 0.18 }],
  [19, 'folder pops', 'pop', { pitch: 'A5', gain: 0.14 }],
  [36, 'weather', 'tick', { gain: 0.12 }],
  [37, 'quick settings', 'tick', { gain: 0.12 }],
  [38, 'sound', 'tick', { gain: 0.12 }],
  [45, 'shutter', 'click', { gain: 0.1 }],
  [53, 'notification chime', 'bell', { pitch: 'E6', gain: 0.16 }],
  [53.5, 'chime 2', 'bell', { pitch: 'A6', gain: 0.13 }],
  [72, 'pomodoro', 'pop', { pitch: 'C6', gain: 0.12 }],
  [73, 'todo', 'pop', { pitch: 'E6', gain: 0.12 }],
  [74, 'notes', 'pop', { pitch: 'G6', gain: 0.12 }],
  [88, 'plug in', 'blip', { pitch: 'C6', gain: 0.12 }],
  [88.25, 'plug in 2', 'blip', { pitch: 'G6', gain: 0.12 }],
  [91, 'unplug', 'blip', { pitch: 'G5', gain: 0.12 }],
  [91.25, 'unplug 2', 'blip', { pitch: 'C5', gain: 0.12 }],
  [109.5, 'pick up', 'tick', { gain: 0.12 }],
  [112, 'drop', 'tick', { gain: 0.14 }],
  [114, 'remove', 'pop', { pitch: 'C6', to: 'C5', gain: 0.12 }],
  [120.5, 'logo', 'bell', { pitch: 'F5', gain: 0.16 }],
];

// ------------------------------------------------------------------ wallpaper (drawn in code)
// A slow liquid aurora at low resolution (soft by construction) plus a much smaller copy that
// stands in for the glass's backdrop blur.
const WP = document.createElement('canvas'); WP.width = 320; WP.height = 180;
const WB = document.createElement('canvas'); WB.width = 40; WB.height = 23;
const wctx = WP.getContext('2d'), bctx = WB.getContext('2d');
const BLOBS = [
  ['#7b3cff', 0.18, 0.30, 0.55], ['#ff4fa3', 0.70, 0.22, 0.50], ['#19c8ff', 0.85, 0.75, 0.55],
  ['#ff9a3c', 0.30, 0.85, 0.45], ['#2b5cff', 0.50, 0.55, 0.60], ['#00e0b0', 0.05, 0.80, 0.40],
];
function paintWallpaper(t, calm) {
  const w = WP.width, h = WP.height;
  wctx.globalCompositeOperation = 'source-over';
  const g = wctx.createLinearGradient(0, 0, w, h);
  g.addColorStop(0, '#120a35'); g.addColorStop(1, '#04203d');
  wctx.fillStyle = g; wctx.fillRect(0, 0, w, h);
  wctx.globalCompositeOperation = 'screen';
  BLOBS.forEach(([c, x, y, r], i) => {
    const sp = 0.05 * (1 - 0.6 * calm);
    const px = (x + 0.16 * noise1(t * sp + i * 7.3, 11 + i)) * w, py = (y + 0.16 * noise1(t * sp + i * 3.1, 31 + i)) * h;
    const rr = r * w * (1 + 0.12 * Math.sin(t * 0.3 + i));
    const rg = wctx.createRadialGradient(px, py, 0, px, py, rr);
    rg.addColorStop(0, c + 'b0'); rg.addColorStop(1, c + '00');
    wctx.fillStyle = rg; wctx.fillRect(0, 0, w, h);
  });
  wctx.globalCompositeOperation = 'source-over';
  if (calm > 0) { wctx.fillStyle = `rgba(10,14,40,${0.35 * calm})`; wctx.fillRect(0, 0, w, h); }
  bctx.drawImage(WP, 0, 0, WB.width, WB.height);
}

// ------------------------------------------------------------------ glass: a real capture over a blurred backdrop
const G = document.createElement('canvas'); G.width = 2400; G.height = 1400;
const gctx = G.getContext('2d');
// Draw source rect of img into dest rect (in the current transform), with the wallpaper blurred behind it
// through the capture's own alpha shape (that is how the real acrylic looks).
function glass(ctx, img, s, x, y, w, h, alpha = 1) {
  if (!img || alpha <= 0.001 || w <= 1 || h <= 1) return;
  const m = ctx.getTransform();
  let [sx, sy, ssw, ssh] = s || [0, 0, img.width, img.height];
  // screen rect, clipped to the frame (zoomed-in docks are far wider than the screen)
  const X0 = m.a * x + m.e, Y0 = m.d * y + m.f, X1 = X0 + w * m.a, Y1 = Y0 + h * m.d;
  const X = Math.max(X0, -60), Y = Math.max(Y0, -60), XR = Math.min(X1, W + 60), YB = Math.min(Y1, H + 60);
  const sw = Math.min(G.width, Math.ceil(XR - X)), sh = Math.min(G.height, Math.ceil(YB - Y));
  if (sw < 2 || sh < 2) return;
  const fx = ssw / (X1 - X0), fy = ssh / (Y1 - Y0);
  sx += (X - X0) * fx; sy += (Y - Y0) * fy; ssw = (XR - X) * fx; ssh = (YB - Y) * fy;
  gctx.globalCompositeOperation = 'copy';
  gctx.drawImage(img, sx, sy, ssw, ssh, 0, 0, sw, sh);
  gctx.globalCompositeOperation = 'source-in';
  const kx = WB.width / W, ky = WB.height / H;
  gctx.drawImage(WB, X * kx, Y * ky, sw * kx, sh * ky, 0, 0, sw, sh);
  gctx.globalCompositeOperation = 'source-over';
  gctx.drawImage(img, sx, sy, ssw, ssh, 0, 0, sw, sh);
  ctx.save();
  ctx.globalAlpha *= alpha;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.shadowColor = 'rgba(5,8,30,0.35)'; ctx.shadowBlur = 40 * Math.min(1, m.a); ctx.shadowOffsetY = 14 * Math.min(1, m.a);
  ctx.drawImage(G, 0, 0, sw, sh, X, Y, sw, sh);
  ctx.restore();
}
// A popup card: real capture, popped in on a spring from its anchor.
function card(ctx, img, s, x, y, w, h, u, u0, { from = [0, 60], out = null, scale0 = 0.9 } = {}) {
  const k = springU(u, u0, SPRING.snappy);
  if (k <= 0.001) return;
  let a = clamp(k * 1.6), dx = (1 - k) * from[0], dy = (1 - k) * from[1], sc = lerp(scale0, 1, k);
  if (out) { const o = E.inCubic(prog(u, out[0], out[1])); a *= 1 - o; dy += o * 60; sc *= 1 - 0.06 * o; }
  if (a <= 0.001) return;
  ctx.save();
  ctx.translate(x + w / 2 + dx, y + h / 2 + dy); ctx.scale(sc, sc);
  glass(ctx, img, s, -w / 2, -h / 2, w, h, a);
  ctx.restore();
}

// ------------------------------------------------------------------ captions: kinetic, left-aligned
function caption(ctx, u, u0, u1, head, sub, { x = 110, y = 150, size = 68 } = {}) {
  if (u < u0 - 0.1 || u > u1 + 1) return;
  const out = E.inCubic(prog(u, u1, u1 + 0.75));
  const words = head.split(' ');
  const f = font(size, 700, DISPLAY);
  ctx.save();
  ctx.shadowColor = 'rgba(8,10,40,0.45)'; ctx.shadowBlur = 28; ctx.shadowOffsetY = 6;
  let px = x;
  ctx.font = f;
  const space = ctx.measureText(' ').width;
  words.forEach((wd, i) => {
    const L = layout(ctx, wd, f, -0.02 * size);
    const k = springU(u, u0 + i * 0.22, SPRING.snappy);
    if (k > 0.001) {
      ctx.save();
      ctx.beginPath(); ctx.rect(px - 10, y - size * 1.1, L.width + 30, size * 1.45); ctx.clip();
      ctx.globalAlpha = (1 - out);
      text(ctx, wd, px, y + (1 - k) * size * 1.1 - out * size * 0.4, f, '#ffffff', 'left', -0.02 * size);
      ctx.restore();
    }
    px += L.width + space;
  });
  if (sub) {
    const k = E.outCubic(prog(u, u0 + 0.6, u0 + 1.6));
    ctx.globalAlpha = k * (1 - out) * 0.88;
    text(ctx, sub, x + 2, y + size * 0.85 + (1 - k) * 18, font(Math.max(34, size * 0.42), 500, UI), '#ffffff');
  }
  ctx.restore();
}

// ------------------------------------------------------------------ dock
// Camera over world space: [focusX, focusY, logZoom, screenX, screenY] chased on springs.
const CAM = [
  [0, [960, 1011, Math.log(1.6), 960, 720]],
  [3.5, [960, 1011, Math.log(1.0), 960, 700]],
  [8, [960, 1011, 0, 960, 760]],
  [8.6, [960, 1011, Math.log(2.6), 960, 760]],
  [12, [960, 1011, 0, 960, 760]],
  [15.5, [960, 1011, 0, 960, 760]],
  [16, [slotC('apps') + 120, 1011, Math.log(2.3), 760, 930]],
  [24, [slotC('apps') + 120, 1011, Math.log(2.3), 760, 1180]],   // dock leaves for the music close-up
  [36, [cx(1370), 1011, Math.log(1.75), 960, 975]],
  [44, [slotC('ocr'), 1011, Math.log(2.1), 520, 975]],
  [48, [slotC('sysmon'), 1011, Math.log(2.1), 1300, 975]],
  [52, [slotC('notifications') - 40, 1011, Math.log(2.6), 1500, 975]],
  [56, [960, 1011, Math.log(3.2), 960, 780]],
  [60, [960, 1011, Math.log(3.8), 700, 800]],
  [72, [960, 1011, 0, 960, 1011]],
  [84, [slotC('mirror'), 1011, Math.log(4.5), 1150, 700]],
  [88, [960, 1011, Math.log(3.6), 960, 760]],
  [96, [960, 1011, Math.log(3.6), 960, 1300]],                   // dock drops away under Settings
  [108, [cx(2000), 1011, Math.log(3.0), 700, 800]],
  [120, [960, 1011, 0, 960, 830]],
];
function camera(u) {
  const [fx, fy, lz, sx, sy] = springKeys(u, CAM, SPRING.gentle);
  return { fx, fy, z: Math.exp(lz), sx, sy };
}
function applyCam(ctx, c) { ctx.translate(c.sx, c.sy); ctx.scale(c.z, c.z); ctx.translate(-c.fx, -c.fy); }

// Dock image for the resize shot: the real dock at the nearest captured length, revealed to a
// continuous width so the glass edge glides while the layout steps like the real app.
function dockAtWidth(ctx, IMG, wcss, y) {
  const ws = MAN.dockw;
  const wi = ws.find((x) => x >= wcss - 0.5) ?? ws[ws.length - 1];
  const img = IMG['dockw_' + wi];
  const off = (wi - wcss) / 2;                           // css cut from each end
  const s = img.width / wi;
  ctx.save();
  rrect(ctx, 960 - (wcss * K) / 2, y, wcss * K, DH, 16 * K); ctx.clip();
  glass(ctx, img, [off * s, 0, wcss * s, img.height], 960 - (wcss * K) / 2, y, wcss * K, DH);
  ctx.restore();
}

function seqFrame(IMG, name, sec) {
  const S = MAN.seq[name];
  const i = clamp(Math.floor((sec / S.dur) * S.n), 0, S.n - 1);
  return IMG[`${name}_${i}`];
}

// The dock row, drawn in world space under the camera.
function drawDock(ctx, u, IMG) {
  const rise = springU(u, 0.3, SPRING.gentle);
  const y = DY + (1 - rise) * 260;
  if (u >= 8 && u < 16) {
    // resize: full -> short -> long
    const wc = Math.exp(springKeys(u, [[8, Math.log(2900)], [8.6, Math.log(900)], [12, Math.log(2900)]], { stiffness: 60, damping: 14 }));
    dockAtWidth(ctx, IMG, clamp(wc, 900, 2900), y);
    return;
  }
  // which dock image: alerts take over the whole dock (real alert frames)
  let img = IMG.dock_hero, a = 1;
  const alertAt = (name, u0, rate = 1) => { if (u >= u0) img = seqFrame(IMG, name, (u - u0) * M.GRID.period * rate) || img; };
  if (u >= 56 && u < 72) alertAt('al_prayer', 56.4);
  if (u >= 88 && u < 91) alertAt('al_plug', 87.8, 1.1);
  if (u >= 91 && u < 93.5) alertAt('al_unplug', 90.8, 1.1);
  if (u >= 93.5 && u < 95) alertAt('al_volume', 93.3, 1.3);
  if (u >= 95 && u < 97) alertAt('al_bright', 94.8, 1.3);
  if (u >= 108 && u < 120) return drawEditDock(ctx, u, IMG, y);
  // prayer alert fades back to the dock as the beat returns
  glass(ctx, img, null, DX, y, DCSS * K, DH, a);
  if (u >= 70 && u < 73) glass(ctx, IMG.dock_hero, null, DX, y, DCSS * K, DH, E.inOutSine(prog(u, 70.5, 72)));
  // notification bell pulse
  if (u >= 52 && u < 56) {
    const p = wobble(u - 52.5, 3, 4);
    const [bx, bw] = SLOTS.notifications, s = IMG.dock_hero.width / DCSS;
    if (u > 52.5) {
      const z = 1 + 0.25 * Math.abs(p);
      ctx.save(); ctx.translate(cx(bx + bw / 2), y + DH / 2); ctx.scale(z, z);
      glass(ctx, IMG.dock_hero, [bx * s, 0, bw * s, IMG.dock_hero.height], -bw * K / 2, -DH / 2, bw * K, DH);
      ctx.restore();
    }
  }
  // mirror highlight ring
  if (u >= 84 && u < 88) {
    const [mx, mw] = SLOTS.mirror, p = (u - 84.5) % 2 / 2;
    if (u > 84.5) {
      ctx.save(); ctx.strokeStyle = `rgba(255,255,255,${0.7 * (1 - p)})`; ctx.lineWidth = 2 / camera(u).z * 2;
      rrect(ctx, cx(mx) - p * 10, y - p * 10, mw * K + p * 20, DH + p * 20, 12 * K + p * 8); ctx.stroke(); ctx.restore();
    }
  }
}

// Edit mode: the real edit-mode dock cut into pieces; the Waktu Solat widget is lifted and moved
// before the System Monitor, then Mirror is removed with its x. Pieces are real pixels.
function drawEditDock(ctx, u, IMG, y) {
  const img = IMG.dock_edit, s = img.width / DCSS, S = EDIT.slots;
  const enter = E.inOutSine(prog(u, 108, 109));
  if (enter < 1) glass(ctx, IMG.dock_hero, null, DX, y, DCSS * K, DH, 1 - enter);
  const gap = 6.4;
  const sysL = S.sysmon[0], solL = S.solat[0], solW = S.solat[1], prodL = S.productivity[0];
  const mirL = S.mirror[0], mirW = S.mirror[1], notL = S.notifications[0];
  const mv = E.inOutCubic(prog(u, 109.5, 112));                       // carried by the pointer
  const lift = springU(u, 109.2, SPRING.snappy) - springU(u, 112, SPRING.snappy);
  const close = springU(u, 114.2, SPRING.snappy);                     // gap closes after removal
  const gone = E.inBack(prog(u, 114, 114.6), 1.6);
  const piece = (a, b, dx, alpha = enter, sc = 1, dy = 0) => {
    const x = cx(a) + dx * K;
    ctx.save();
    ctx.translate(x + (b - a) * K / 2, y + DH / 2 + dy); ctx.scale(sc, sc);
    glass(ctx, img, [a * s, 0, (b - a) * s, img.height], -(b - a) * K / 2, -DH / 2, (b - a) * K, DH, alpha);
    ctx.restore();
  };
  const shiftSys = (solW + gap) * E.inOutCubic(prog(u, 110.2, 111.6));
  piece(0, sysL - gap / 2, 0);                                          // apps .. ocr
  piece(sysL - gap / 2, solL - gap / 2, shiftSys);                     // sysmon slides right
  piece(prodL - gap / 2, mirL - gap / 2, 0);                            // productivity
  piece(mirL - gap / 2, mirL + mirW + gap / 2, 0, enter * (1 - gone), 1 - 0.7 * gone);
  piece(notL - gap / 2, DCSS, -(mirW + gap) * close);                   // bell + edit buttons
  // the lifted widget, drawn last
  const dx = (sysL - solL) * mv;
  piece(solL - gap / 2, solL + solW + gap / 2, dx, enter, 1 + 0.12 * lift, -10 * lift);
  // pointer
  if (u > 109 && u < 115) {
    let px = cx(solL + solW / 2 + (sysL - solL) * mv), py = y + DH * 0.7 - 10 * lift;
    if (u > 112.5) { const q = E.inOutCubic(prog(u, 112.5, 113.8)); px = lerp(px, cx(mirL + mirW - 2), q); py = lerp(py, y + 4, q); }
    ctx.save(); ctx.translate(px, py); ctx.scale(0.9 / camera(u).z * 1.6, 0.9 / camera(u).z * 1.6);
    ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(0, 22); ctx.lineTo(6, 17); ctx.lineTo(10, 26); ctx.lineTo(14, 24); ctx.lineTo(10, 15); ctx.lineTo(17, 15); ctx.closePath();
    ctx.fillStyle = '#fff'; ctx.strokeStyle = '#111'; ctx.lineWidth = 1.5; ctx.fill(); ctx.stroke();
    ctx.restore();
  }
}

// ------------------------------------------------------------------ scenes (screen space, over the dock)
const P = (IMG, n) => IMG['pop_' + n];

function sceneIntro(ctx, u, IMG) {
  const k = springU(u, 1.6, SPRING.bouncy);
  if (k > 0.001 && u < 9) {
    const out = E.inCubic(prog(u, 7.6, 8.4));
    ctx.save(); ctx.globalAlpha = clamp(k) * (1 - out);
    ctx.translate(196, 380 - out * 40); ctx.scale(k, k);
    ctx.shadowColor = 'rgba(10,20,80,0.4)'; ctx.shadowBlur = 50; ctx.shadowOffsetY = 18;
    ctx.drawImage(IMG.icon, -86, -86, 172, 172);
    ctx.restore();
  }
  caption(ctx, u, 2, 7.6, 'Meet GlassBar.', 'Your desktop dock, made of glass.', { x: 320, y: 400, size: 104 });
}

function sceneResize(ctx, u) { caption(ctx, u, 8.3, 15.3, 'Short or long.', 'Stretch the dock to any length you like.'); }

function sceneApps(ctx, u, IMG) {
  caption(ctx, u, 16.4, 23.3, 'Apps & folders.', 'One click away, right on the dock.');
  const c = camera(u), fxw = (slotC('folders') - c.fx) * c.z + c.sx;
  const img = P(IMG, 'folders'), w = 600, h = w * img.height / img.width;
  card(ctx, img, null, fxw - w / 2, c.sy - 40 - h, w, h, u, 19, { from: [0, 70], out: [23.2, 24] });
}

function sceneMusic(ctx, u, IMG) {
  caption(ctx, u, 24.4, 35.3, 'Music, lyrics & a live equalizer.', 'Every lyric line in time, bars that move with the beat.');
  // close-up strip of the real player + lyric + equalizer, 10 s of captured frames
  const whip = E.inOutCubic(prog(u, 30, 31.2));
  const k = springU(u, 24, SPRING.gentle);
  const S = MAN.seq.eq, sec = clamp((u - 24) * M.GRID.period + 0.156, 0, S.dur - 0.01);   // same beat phase as the clip it was captured from (music 20-30 s)
  const img = IMG['eq_' + Math.floor(sec / S.dur * S.n)];
  if (img && whip < 1) {
    const w = 1500, h = w * img.height / img.width, x = 210 - whip * 1900, y = 520 + (1 - k) * 500;
    ctx.save();
    const g = ctx.createLinearGradient(x, 0, x + w, 0);
    ctx.globalAlpha = clamp(k * 2);
    glass(ctx, img, null, x, y, w, h);
    ctx.restore();
  }
  const pm = P(IMG, 'music'), w = 900, h = w * pm.height / pm.width;
  if (u > 29.5) {
    const x = lerp(1950, 510, whip), y = 250;
    ctx.save(); glass(ctx, pm, null, x, y, w, h, E.outCubic(prog(u, 29.8, 30.6)) * (1 - E.inCubic(prog(u, 35.4, 36)))); ctx.restore();
  }
}

function sceneWeather(ctx, u, IMG) {
  caption(ctx, u, 36.3, 43.3, 'Weather, quick settings, sound.', null, { y: 132 });
  const out = [43.3, 44.2];
  const H0 = 640;
  const place = (n, src, x, u0) => { const img = P(IMG, n), s = src || [0, 0, img.width, img.height]; const h = H0, w = h * s[2] / s[3]; card(ctx, img, s, x - w / 2, 190, w, h, u, u0, { from: [0, 120], out }); };
  place('weather', [0, 0, 830, 1740], 600, 36);
  place('quicksettings', [0, 0, 900, 1300], 980, 37);
  place('sound', null, 1360, 38);
}

function sceneCapture(ctx, u, IMG) {
  caption(ctx, u, 44.3, 47.6, 'Capture anything.', 'Screenshot, record, OCR, clipboard, colour.', { y: 150 });
  caption(ctx, u, 48.4, 51.4, 'Watch your PC.', 'CPU, temperature, memory and network.', { x: 980, y: 150 });
  const c = camera(u);
  const ocr = P(IMG, 'ocr'), h1 = 660, w1 = h1 * ocr.width / ocr.height;
  const wx = (slotC('ocr') - c.fx) * c.z + c.sx;
  card(ctx, ocr, null, wx - w1 / 2, 215, w1, h1, u, 44.5, { from: [0, 90], out: [47.8, 48.6] });
  const sm = P(IMG, 'sysmon'), src = [0, 760, 900, 1078], h2 = 660, w2 = h2 * src[2] / src[3];
  const sx = (slotC('sysmon') - c.fx) * c.z + c.sx;
  card(ctx, sm, src, sx - w2 / 2 - 300, 215, w2, h2, u, 48.4, { from: [500, 0], out: [51.4, 52.2] });
}

function sceneNotify(ctx, u, IMG) {
  caption(ctx, u, 52.3, 55.4, 'Notifications, right on your dock.', null, { y: 150 });
  const img = P(IMG, 'notifications'), src = [36, 366, 780, 148], w = 920, h = w * src[3] / src[2];
  card(ctx, img, src, 1840 - w, 640, w, h, u, 53, { from: [0, 160], out: [55.5, 56.3], scale0: 0.7 });
}

function sceneAdhan(ctx, u, IMG) {
  caption(ctx, u, 57, 70.5, 'Never miss a prayer.', 'Prayer times for your zone, and an alert right on the dock.', { y: 170 });
  const img = P(IMG, 'solat'), src = [0, 0, 800, 520], w = 480, h = w * src[3] / src[2];
  card(ctx, img, src, 1340, 170, w, h, u, 60, { from: [0, 60], out: [70.5, 71.5] });
}

function sceneFocus(ctx, u, IMG) {
  caption(ctx, u, 72.3, 83.3, 'Focus. Plan. Remember.', 'Pomodoro, to-do and quick notes in one tap.', { y: 132 });
  const out = [83.3, 84.2];
  const deal = (n, src, xc, u0, h, rot) => {
    const img = P(IMG, n), s = src || [0, 0, img.width, img.height], w = h * s[2] / s[3];
    ctx.save(); ctx.translate(xc, 520); ctx.rotate(rot * (1 - springU(u, u0, SPRING.gentle)));
    card(ctx, img, s, -w / 2, -h / 2 + 40, w, h, u, u0, { from: [0, 300], out });
    ctx.restore();
  };
  deal('productivity', [0, 0, 850, 1285], 520, 72, 600, -0.08);
  deal('todo', null, 970, 73, 520, 0.05);
  deal('notes', null, 1420, 74, 520, 0.08);
}

function sceneMirror(ctx, u) { caption(ctx, u, 84.3, 87.4, 'A mirror before every call.', 'Check yourself in one click.', { y: 200 }); }

function sceneBattery(ctx, u) {
  caption(ctx, u, 88.2, 95.6, 'Battery, volume & brightness.', 'The whole dock tells you, then gets out of the way.', { y: 200 });
}

function sceneSettings(ctx, u, IMG) {
  caption(ctx, u, 96.3, 107.3, 'Make it look your way.', 'Themes, glass, opacity, every widget has its own settings.', { y: 132 });
  const k = springU(u, 96.2, SPRING.gentle), out = E.inCubic(prog(u, 107.4, 108.3));
  if (k < 0.001 || out >= 1) return;
  // the settings window, its theme changing on the beat (real captures)
  const seq = [['liquidglass', 96], ['glass', 98], ['dark', 100], ['light', 102], ['custom', 104]];
  const shrink = E.inOutCubic(prog(u, 104.6, 105.8));
  const w = lerp(1060, 820, shrink), h = w * 960 / 1320, x = lerp(120, 100, shrink), y = 205 + (1 - k) * 700 + out * 80;
  ctx.save(); ctx.globalAlpha = (1 - out);
  ctx.shadowColor = 'rgba(0,0,30,0.45)'; ctx.shadowBlur = 60; ctx.shadowOffsetY = 20;
  rrect(ctx, x, y, w, h, 14); ctx.fillStyle = '#16171c'; ctx.fill();
  ctx.shadowColor = 'transparent';
  ctx.save(); rrect(ctx, x, y, w, h, 14); ctx.clip();
  seq.forEach(([n, u0], i) => {
    const a = i === 0 ? 1 : E.inOutSine(prog(u, u0 - 0.25, u0 + 0.25));
    if (a > 0) { ctx.globalAlpha = a * (1 - out); ctx.drawImage(IMG['set_theme_' + n], x, y, w, h); }
  });
  ctx.restore(); ctx.restore();
  // example widget settings: equalizer + battery
  const eq = P(IMG, 'equalizer'), bat = P(IMG, 'battery');
  card(ctx, eq, [0, 0, 900, 1250], 990, 230, 430, 430 * 1250 / 900, u, 105.2, { from: [300, 0], out: [107.4, 108.3] });
  card(ctx, bat, [0, 0, 800, 1050], 1450, 300, 400, 400 * 1050 / 800, u, 105.8, { from: [300, 0], out: [107.4, 108.3] });
}

function sceneEdit(ctx, u, IMG) {
  caption(ctx, u, 108.4, 119.2, 'Drag. Drop. Done.', 'Reorder, remove or add widgets, right on the dock.', { y: 150 });
  const img = IMG.set_widgets, w = 860, h = w * 960 / 1320;
  const k = springU(u, 116, SPRING.gentle), out = E.inCubic(prog(u, 119.2, 120));
  if (k > 0.001 && out < 1) {
    ctx.save(); ctx.globalAlpha = clamp(k * 1.5) * (1 - out);
    const y = 250 + (1 - k) * 500 + out * 60;
    ctx.shadowColor = 'rgba(0,0,30,0.45)'; ctx.shadowBlur = 60; ctx.shadowOffsetY = 20;
    rrect(ctx, 1000, y - 80, w, h, 14); ctx.fillStyle = '#16171c'; ctx.fill();
    ctx.shadowColor = 'transparent';
    rrect(ctx, 1000, y - 80, w, h, 14); ctx.clip(); ctx.drawImage(img, 1000, y - 80, w, h);
    ctx.restore();
  }
}

function sceneEnd(ctx, u, IMG) {
  const k = springU(u, 120.3, SPRING.bouncy);
  if (k > 0.001) {
    ctx.save(); ctx.globalAlpha = clamp(k);
    ctx.translate(960, 250); ctx.scale(k, k);
    ctx.shadowColor = 'rgba(10,20,80,0.45)'; ctx.shadowBlur = 60; ctx.shadowOffsetY = 20;
    ctx.drawImage(IMG.icon, -90, -90, 180, 180);
    ctx.restore();
  }
  const f = font(130, 700, DISPLAY), L = layout(ctx, 'GlassBar', f, -2.6);
  ctx.save(); ctx.shadowColor = 'rgba(8,10,40,0.45)'; ctx.shadowBlur = 30; ctx.shadowOffsetY = 8;
  L.glyphs.forEach((g, j) => {
    const p = springU(u, 120.8 + j * 0.06, SPRING.snappy);
    if (p <= 0.001) return;
    ctx.globalAlpha = clamp(p * 1.5);
    glyph(ctx, g.ch, 960 - L.width / 2 + g.cx, 480 + (1 - p) * 60, f, '#fff');
  });
  const tp = E.outCubic(prog(u, 122, 123.5));
  ctx.globalAlpha = tp * 0.9;
  text(ctx, 'Your dock, your way.', 960, 560 + (1 - tp) * 20, font(44, 500, UI), '#fff', 'center');
  ctx.restore();
}

// ------------------------------------------------------------------ images
const images = { icon: 'assets/film/icon.svg' };
for (const n of MAN.stills) images[n] = `assets/film/${n}.png`;
for (const w of MAN.dockw) images['dockw_' + w] = `assets/film/dockw/${w}.png`;
for (const [n, s] of Object.entries(MAN.seq)) for (let i = 0; i < s.n; i++) images[`${n}_${i}`] = `assets/film/${n}/${String(i).padStart(3, '0')}.png`;

M.film({
  fonts: [font(100, 700, DISPLAY), font(40, 500, UI)],
  images,
  hits: HITS,
  samples: 6,
  draw(ctx, u, t, IMG) {
    const calm = bump(u, 64, 9) > 0 ? clamp((u - 56) / 2) * (1 - clamp((u - 70) / 2)) : 0;
    paintWallpaper(t, calm);
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(WP, 0, 0, W, H);
    // dock under the camera
    const c = camera(u);
    ctx.save(); applyCam(ctx, c); drawDock(ctx, u, IMG); ctx.restore();
    // scene layer
    if (u < 8.5) sceneIntro(ctx, u, IMG);
    if (u >= 8 && u < 16.5) sceneResize(ctx, u);
    if (u >= 16 && u < 24.5) sceneApps(ctx, u, IMG);
    if (u >= 24 && u < 36.5) sceneMusic(ctx, u, IMG);
    if (u >= 36 && u < 44.5) sceneWeather(ctx, u, IMG);
    if (u >= 44 && u < 52.5) sceneCapture(ctx, u, IMG);
    if (u >= 52 && u < 56.5) sceneNotify(ctx, u, IMG);
    if (u >= 56 && u < 72.5) sceneAdhan(ctx, u, IMG);
    if (u >= 72 && u < 84.5) sceneFocus(ctx, u, IMG);
    if (u >= 84 && u < 88.5) sceneMirror(ctx, u);
    if (u >= 88 && u < 96.5) sceneBattery(ctx, u);
    if (u >= 96 && u < 108.5) sceneSettings(ctx, u, IMG);
    if (u >= 108 && u < 120.5) sceneEdit(ctx, u, IMG);
    if (u >= 120) sceneEnd(ctx, u, IMG);
  },
});
