// Live frequency spectrum for the Equalizer widget.
// The page calls `audio_spectrum` every ~2 s while an Equalizer is on screen. A thread captures
// what the speakers play (WASAPI loopback), runs a 2048-point FFT and emits "audio:spectrum":
// BANDS log-spaced levels 0..1 from 20 Hz to 20 kHz, ~45 times a second (the page smooths between frames). It stops
// capturing 3 s after the last call. The page picks its own band count and range from these.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

pub const N: usize = 2048;
pub const BANDS: usize = 96;
const F_LO: f32 = 20.0;
const F_HI: f32 = 20_000.0;
/// dB range mapped to 0..1. Loud music peaks sit around -10 dB per bin.
const DB_FLOOR: f32 = -75.0;
const DB_TOP: f32 = -5.0;

static LEASE: AtomicI64 = AtomicI64::new(0);
static STARTED: std::sync::Once = std::sync::Once::new();

fn live() -> bool { crate::db::now_ms() - LEASE.load(Ordering::Relaxed) < 3000 }

#[tauri::command]
pub fn audio_spectrum(app: tauri::AppHandle) {
    LEASE.store(crate::db::now_ms(), Ordering::Relaxed);
    STARTED.call_once(|| {
        std::thread::spawn(move || {
            use tauri::Emitter;
            loop {
                if !live() { std::thread::sleep(Duration::from_millis(300)); continue; }
                let mut an = Analyzer::default();
                let r = crate::platform::audio().loopback(&mut |s, rate| {
                    if let Some(b) = an.push(s, rate) { let _ = app.emit("audio:spectrum", b); }
                    live()
                });
                // No output device or capture refused: try again shortly.
                if r.is_err() { std::thread::sleep(Duration::from_secs(1)); }
            }
        });
    });
}

/// Keeps the last N samples and turns them into bands at most every 15 ms.
#[derive(Default)]
pub struct Analyzer { buf: Vec<f32>, last: Option<Instant> }

impl Analyzer {
    pub fn push(&mut self, s: &[f32], rate: u32) -> Option<Vec<f32>> {
        if s.is_empty() { self.buf.clear(); return None; } // nothing playing: the page fades to idle
        self.buf.extend_from_slice(s);
        if self.buf.len() > N { self.buf.drain(..self.buf.len() - N); }
        if self.buf.len() < N || self.last.is_some_and(|t| t.elapsed() < Duration::from_millis(15)) { return None; }
        self.last = Some(Instant::now());
        Some(bands(&spectrum(&self.buf), rate))
    }
}

/// In-place iterative radix-2 FFT. `re.len()` must be a power of two.
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j |= bit;
        if i < j { re.swap(i, j); im.swap(i, j); }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f32::consts::PI / len as f32;
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (ang * k as f32).sin_cos();
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * c - im[b] * s, re[b] * s + im[b] * c);
                re[b] = re[a] - tr; im[b] = im[a] - ti;
                re[a] += tr; im[a] += ti;
            }
        }
        len <<= 1;
    }
}

/// Hann-windowed amplitude per bin (N/2 bins); a full-scale sine reads ~1.0.
pub fn spectrum(x: &[f32]) -> Vec<f32> {
    let n = x.len();
    let w = |i: usize| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / n as f32).cos();
    let mut re: Vec<f32> = x.iter().enumerate().map(|(i, v)| v * w(i)).collect();
    let mut im = vec![0.0; n];
    fft(&mut re, &mut im);
    let scale = 4.0 / n as f32; // 2/N for one-sided, x2 for the Hann window's 0.5 gain
    (0..n / 2).map(|k| (re[k] * re[k] + im[k] * im[k]).sqrt() * scale).collect()
}

/// Bins -> BANDS log-spaced levels 0..1 (dB scale). Bands narrower than a bin (the low end)
/// read the bin at their centre so they don't all copy one value.
pub fn bands(amp: &[f32], rate: u32) -> Vec<f32> {
    let hz_per_bin = rate as f32 / (2 * amp.len()) as f32;
    let edge = |i: usize| F_LO * (F_HI / F_LO).powf(i as f32 / BANDS as f32);
    (0..BANDS)
        .map(|i| {
            let (lo, hi) = (edge(i) / hz_per_bin, edge(i + 1) / hz_per_bin);
            let a = if hi.floor() - lo.ceil() < 1.0 {
                let c = ((lo + hi) / 2.0).min(amp.len() as f32 - 1.0);
                let (k, t) = (c.floor() as usize, c.fract());
                amp[k] * (1.0 - t) + amp[(k + 1).min(amp.len() - 1)] * t
            } else {
                // Low sample rates: bands above Nyquist come out empty (0).
                let e = (hi.floor() as usize).min(amp.len());
                amp[(lo.ceil() as usize).min(e)..e].iter().cloned().fold(0.0, f32::max)
            };
            let v = ((20.0 * a.max(1e-9).log10() - DB_FLOOR) / (DB_TOP - DB_FLOOR)).clamp(0.0, 1.0);
            (v * 1000.0).round() / 1000.0
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(hz: f32, rate: f32) -> Vec<f32> { (0..N).map(|i| (2.0 * std::f32::consts::PI * hz * i as f32 / rate).sin()).collect() }

    #[test]
    fn fft_finds_a_sine_at_full_scale() {
        // Exactly on bin 64.
        let amp = spectrum(&sine(64.0 * 48_000.0 / N as f32, 48_000.0));
        let (k, a) = amp.iter().enumerate().fold((0, 0.0f32), |m, (i, &v)| if v > m.1 { (i, v) } else { m });
        assert_eq!(k, 64);
        assert!((a - 1.0).abs() < 0.05, "amplitude {a}");
    }

    #[test]
    fn a_1khz_tone_peaks_in_the_1khz_band() {
        let b = bands(&spectrum(&sine(1000.0, 48_000.0)), 48_000);
        assert_eq!(b.len(), BANDS);
        let top = b.iter().enumerate().fold((0, 0.0f32), |m, (i, &v)| if v > m.1 { (i, v) } else { m }).0;
        let lo = F_LO * (F_HI / F_LO).powf(top as f32 / BANDS as f32);
        let hi = F_LO * (F_HI / F_LO).powf((top + 1) as f32 / BANDS as f32);
        assert!(lo <= 1000.0 && 1000.0 < hi * 1.1, "peak band {top}: {lo}-{hi} Hz");
        assert!(b[5] < 0.2 && b[90] < 0.2, "far bands stay low");
    }

    #[test]
    fn silence_is_all_zero_and_empty_chunks_reset() {
        assert!(bands(&spectrum(&vec![0.0; N]), 48_000).iter().all(|&v| v == 0.0));
        let mut an = Analyzer::default();
        assert!(an.push(&vec![0.1; N], 48_000).is_some());
        assert!(an.push(&[], 48_000).is_none());
        assert!(an.buf.is_empty());
    }
}
