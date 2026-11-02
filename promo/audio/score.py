# /// script
# requires-python = ">=3.10"
# dependencies = ["librosa>=0.10", "numpy", "scipy", "soundfile"]
# ///
"""GlassBar score: bright marimba + piano pop at 128 BPM, written for the film's own grid.

    MOTION_REEL_SKILL=<skill> uv run promo/audio/score.py      -> promo/audio/music.wav

128 BPM, 32 bars = 60 s, bar 0 starts at t = 0, so beats.json (offset 0, period 0.46875) stays the grid.
Form follows the shot list:
  bars 0-1   intro: piano + marimba, no drums (dock rises, title)
  bars 2-13  A: soft kick, claps, shaker, bass, marimba riff
  bars 14-17 calm: drums out, piano + strings only (the prayer scene)
  bars 18-29 B: everything back, plus a plucked-guitar melody
  bars 30-31 end: last chord rings under the end card
"""
import os
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

sys.path.insert(0, os.path.join(os.environ["MOTION_REEL_SKILL"], "scripts"))
from dsp import *  # noqa: E402,F403

HERE = Path(__file__).resolve().parent
BPM, BARS = 128, 32
DUR = 60.0
seed(7)

# ---------------------------------------------------------------- instruments (new voices)

def marimba(note, length=0.9):
    n = int(length * SR); t = tvec(n); f = hz(note)
    x = (np.sin(2 * np.pi * f * t) * np.exp(-t / 0.45)
         + 0.35 * np.sin(2 * np.pi * f * 3.93 * t) * np.exp(-t / 0.07)
         + 0.12 * np.sin(2 * np.pi * f * 9.6 * t) * np.exp(-t / 0.02))
    return x * (1 - np.exp(-t / 0.001))


def piano(notes, length=2.4, vel=1.0):
    n = int(length * SR); t = tvec(n); x = np.zeros(n)
    for nt in notes:
        f = hz(nt)
        for k in range(1, 9):
            fk = f * k * np.sqrt(1 + 0.0004 * k * k)          # slight inharmonicity
            x += (0.9 ** k / k) * np.sin(2 * np.pi * fk * t + rng.uniform() * 6) * np.exp(-t * (0.8 + 0.5 * k))
    ham = filt(rng.standard_normal(n), "bandpass", [800, 4000]) * np.exp(-t / 0.004) * 0.05
    x = (x / len(notes) + ham) * (1 - np.exp(-t / 0.002))
    return filt(x, "lowpass", 3800) * vel


def guitar(note, length=1.2):
    return filt(ks_pluck(hz(note), length, bright=0.45, buzz=0.0), "highpass", 140)


def shaker():
    n = int(0.09 * SR); t = tvec(n)
    x = filt(rng.standard_normal(n), "bandpass", [5000, 11000])
    return x * np.sin(np.pi * np.clip(t / 0.06, 0, 1)) ** 2


def snap():
    n = int(0.2 * SR); t = tvec(n)
    return filt(rng.standard_normal(n), "bandpass", [1500, 5000]) * np.exp(-t / 0.018) + 0.3 * np.sin(2 * np.pi * 1900 * t) * np.exp(-t / 0.01)


def sub(note, length):
    n = int(length * SR); t = tvec(n); f = hz(note)
    x = np.sin(2 * np.pi * f * t) + 0.25 * np.sin(4 * np.pi * f * t)
    return x * adsr(n, 0.01, 0.2, 0.7, 0.08)


# ---------------------------------------------------------------- harmony: I - V - vi - IV in D major, one chord per bar
PROG = [
    (["D4", "F#4", "A4", "D5"], "D2", ["D5", "F#5", "A5"]),
    (["A3", "C#4", "E4", "A4"], "A1", ["C#5", "E5", "A5"]),
    (["B3", "D4", "F#4", "B4"], "B1", ["B4", "D5", "F#5"]),
    (["G3", "B3", "D4", "G4"], "G1", ["B4", "D5", "G5"]),
]
RIFF = [0, 2, 1, 2, 0, 2, 1, 2]                  # marimba eighths over the chord's top three notes
MEL = {0: ["A5", None, "F#5", "E5", "D5", None, "E5", "F#5"],
       1: ["E5", None, "C#5", None, "A4", "B4", "C#5", None],
       2: ["D5", None, "F#5", "E5", "D5", None, "B4", None],
       3: ["D5", "E5", "D5", None, "B4", None, "A4", None]}

beat = 60 / BPM
drums, keysb, mar, bass, mel, pads = (Bus(DUR, BPM) for _ in range(6))
kk, cl, sh, sn = kick(soft=True), clap(), shaker(), snap()

for bar in range(BARS):
    b0 = bar * 4
    chord, root, top = PROG[bar % 4]
    calm = 14 <= bar <= 17
    main = 2 <= bar <= 29
    end = bar >= 30
    if end:
        if bar == 30:
            keysb.add(piano(["D3"] + PROG[0][0], 6.0, 1.1), b0, 0.9)
            mar.add(marimba("D6", 1.6), b0, 0.5, 0.2)
            pads.add(pad_chord(PROG[0][0], 4.5, 1400, attack=0.05, release=1.5), b0, 0.35)
            bass.add(sub("D2", 3.5), b0, 0.8)
        continue
    # piano: chord on 1, push on the "and" of 2
    keysb.add(piano(chord, 2.0, 0.9), b0, 0.75)
    keysb.add(piano(chord[1:], 1.4, 0.6), b0 + 1.5, 0.45)
    if calm:
        pads.add(pad_chord(chord, 4 * beat + 0.3, (900, 1600), attack=0.6, release=0.6), b0, 0.4)
        keysb.add(piano([top[2]], 2.4, 0.7), b0 + 2, 0.4, 0.3)
        # the music keeps going, just without drums: marimba riff + bass + a light shaker
        for i, k in enumerate(RIFF):
            mar.add(marimba(top[k], 0.7), b0 + i * 0.5, 0.32 if i % 2 == 0 else 0.22, -0.25)
        bass.add(sub(root, 3.6 * beat), b0, 0.6)
        for e in range(8):
            drums.add(sh, b0 + e * 0.5, 0.08 if e % 2 else 0.05, 0.35)
        continue
    # marimba riff (from the intro on)
    for i, k in enumerate(RIFF):
        mar.add(marimba(top[k], 0.7), b0 + i * 0.5, 0.42 if i % 2 == 0 else 0.3, -0.25)
    if not main:
        continue
    # drums: soft kick on 1 and 3, claps on 2 and 4, shaker 8ths, a snap ghost
    for q in (0, 2):
        drums.add(kk, b0 + q, 0.85)
    if bar % 2 == 1:
        drums.add(kk, b0 + 3.5, 0.45)
    for q in (1, 3):
        drums.add(cl, b0 + q, 0.38, 0.05)
    for e in range(8):
        drums.add(sh, b0 + e * 0.5 + 0.0, 0.16 if e % 2 else 0.1, 0.35)
    drums.add(sn, b0 + 2.75, 0.12, -0.3)
    # bass: root on 1, octave pickup
    bass.add(sub(root, 1.6 * beat), b0, 0.75)
    bass.add(sub(root, 0.4 * beat), b0 + 2.5, 0.45)
    bass.add(sub(root, 0.9 * beat), b0 + 3, 0.5)
    # B section: plucked melody + a light pad
    if bar >= 18:
        for i, nt in enumerate(MEL[bar % 4]):
            if nt:
                mel.add(guitar(nt, 1.0), b0 + i * 0.5, 0.33, 0.25)
        pads.add(pad_chord(chord, 4 * beat, 1100, attack=0.2, release=0.4), b0, 0.16)
    if bar in (2, 18):                       # lift into each section
        pads.add(crash(2.2), b0, 0.12)

# the beat returns after the calm part with a small fill
for i, nt in enumerate(["A4", "B4", "C#5", "D5", "E5", "F#5", "G5", "A5"]):
    mar.add(marimba(nt, 0.4), 17 * 4 + 2 + i * 0.25, 0.3, 0.2)

# ---------------------------------------------------------------- mix
n = int(DUR * SR)
mix = (drums.x * 1.0 + reverb(keysb.x, 0.28, length=2.2, decay=0.6) * 0.8 + reverb(mar.x, 0.22) * 0.9
       + bass.x * 0.9 + reverb(mel.x, 0.3) * 0.9 + reverb(pads.x, 0.4, length=2.6, decay=0.8) * 0.8)[:, :n]
# fade in nothing (bar 0 is the hit); short fade at the very end
mix[:, -int(0.6 * SR):] *= np.linspace(1, 0, int(0.6 * SR))
mix = mix / (np.abs(mix).max() + 1e-9) * 0.8
sf.write(HERE / "music.wav", mix.T, SR, subtype="PCM_24")
print("-> music.wav", mix.shape[1] / SR, "s")
