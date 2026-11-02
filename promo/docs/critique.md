# Critique log

## Round 1 (contact sheet, every 2 beats)
Scores (out of 10): hook 5 · readability 4 · motion 6 · variety 7 · brand 7 · sound sync n/a
1. The wallpaper was washed-out pastel, so the glass barely read and white text lacked contrast. **Fix:** darker base, `screen` blend at a lower alpha.
2. The dock alerts (battery, adhan, mirror) were tiny at the bottom of the frame. **Fix:** the camera zooms each alert to 3.2–4.5x at mid-frame.
3. The end title sat on the dock. **Fix:** moved the title up and the dock down; subtitles raised to a 34 px minimum.

## Round 2 (contact + phone sheets, sync.py)
Scores (out of 10): hook 6 · readability 7 · motion 7 · variety 8 · brand 8 · sound sync 9 (44/45 hits within 20 ms, median 1.8 ms)
- Bug found and fixed before scoring: a zoomed dock was clipped at the 2400 px glass buffer. The glass helper now draws only the on-screen part.
1. The opening was empty for about 1.5 s. **Fix:** the dock rises at once and the title lands at beat 2.
2. The dock was too small in the resize and edit shots. **Fix:** the short dock zooms to 2.6x; edit mode zooms to 3x at mid-frame.
3. The notification card was small. **Fix:** 920 px wide, higher up.

## Round 3 (range video 0–14 s, stills)
Scores (out of 10): hook 8 · readability 8 · motion 8 · variety 8 · brand 9 · sound sync 9
1. The opening showed a tiny dock under a big title. **Fix:** the camera starts at 1.6x on the dock and pulls back to the whole dock under the title.

## Known remaining issues
- The equalizer close-up frames were captured at about 9 fps, so the bars step rather than glide.
- The resize shot steps between 17 captured lengths; the content jumps a little at each step (this is how the real app re-lays out).
- The notification banner window couldn't be captured, so shot 7 uses the Notifications panel card instead.
