# GlassBar promo video

A 60 s, 16:9 promo built with the motion-reel skill. `film.js` is a canvas film that is a pure function of time. Every piece of UI is a real GlassBar capture with demo data, from `assets/film`.

## Re-render after a tweak (run from the repo root; `SK` = the motion-reel skill folder)

    sh -c 'cd promo && sh capture/audio.sh'                  # music.wav (bed + adhan pause)
    node $SK/scripts/render.mjs --film promo --hits
    node $SK/scripts/sfx.mjs promo && node $SK/scripts/mix.mjs promo --no-mux
    node $SK/scripts/render.mjs --film promo --contact --every 2   # quick review sheets
    node $SK/scripts/render.mjs --film promo --video-only --workers 10
    node $SK/scripts/mix.mjs promo                            # -> out/promo/final.mp4

## Files

- `docs/shotlist.md`: the approved shot list.
- `docs/critique.md`: the review rounds.
- `capture/`: re-capture from the running app.
  - Start GlassBar with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333`.
  - Run `node capture/scenes.mjs --fresh <scene...>` and `themes.mjs`, then `python capture/prep.py` (from `promo/`).
  - Demo data is injected in memory only (`demo.mjs`).
  - Back up `%APPDATA%\GlassBar\glassbar.db*` first, and run `restore.mjs` afterwards to put the config back.
