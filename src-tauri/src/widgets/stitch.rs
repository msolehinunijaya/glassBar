//! Scrolling screenshots: join frames taken while a window scrolls. Pure functions on
//! per-row hashes, so they're cheap and fully tested. A frame = rows of BGRA pixels.
use std::hash::{Hash, Hasher};

/// One hash per pixel row, ignoring a strip at each side: the scrollbar thumb moves with
/// every scroll and would make otherwise identical rows look different.
pub fn row_hashes(bgra: &[u8], w: u32, h: u32) -> Vec<u64> {
    let w = w as usize;
    let (x0, x1) = if w > 120 { (8, w - 36) } else { (0, w) };
    (0..h as usize).map(|r| {
        let mut s = std::collections::hash_map::DefaultHasher::new();
        bgra[(r * w + x0) * 4..(r * w + x1) * 4].hash(&mut s);
        s.finish()
    }).collect()
}

/// Rows that stayed put between two frames at the top (toolbars, tabs) and bottom
/// (status bars): they belong to the window, not to the scrolling page.
pub fn fixed_edges(a: &[u64], b: &[u64]) -> (usize, usize) {
    let n = a.len().min(b.len());
    let top = (0..n).take_while(|&i| a[i] == b[i]).count();
    if top == n { return (n, 0); }
    let bottom = (0..n - top).take_while(|&i| a[a.len() - 1 - i] == b[b.len() - 1 - i]).count();
    (top, bottom)
}

/// A row to compare: one hash (u64) or one hash per vertical strip (Vec<u64>). Strips let a
/// row match even when one part of it changes on its own (a parallax picture, a video).
pub trait Cells {
    fn same(&self, o: &Self) -> usize;
    fn cells(&self) -> usize;
}
impl Cells for u64 {
    fn same(&self, o: &Self) -> usize { (self == o) as usize }
    fn cells(&self) -> usize { 1 }
}
impl Cells for Vec<u64> {
    fn same(&self, o: &Self) -> usize { self.iter().zip(o).filter(|(a, b)| a == b).count() }
    fn cells(&self) -> usize { self.len() }
}

/// Share of matching cells when `prev[d..]` is laid over `next[..]`.
fn score<R: Cells>(prev: &[R], next: &[R], d: usize) -> f64 {
    let rows = prev.len().min(next.len()) - d;
    let (same, all) = (0..rows).fold((0, 0), |(s, a), i| (s + prev[i + d].same(&next[i]), a + next[i].cells()));
    if all == 0 { 0.0 } else { same as f64 / all as f64 }
}

/// The two frames are (nearly) the same: nothing moved, only e.g. a cursor blinked.
pub fn unchanged<R: Cells>(prev: &[R], next: &[R]) -> bool {
    prev.len().min(next.len()) > 0 && score(prev, next, 0) >= 0.90
}

/// How far the page moved (in rows) from `prev` to `next` (both the scrolling part only):
/// the shift d where prev[d..] best lines up with next[..]. None when nothing moved or no
/// reliable overlap (≥ 85 % of the cells of at least 16 rows must match: web pages have
/// animated ads, videos and images fading in, so an exact match is too strict).
///
/// `expect` = how far this wheel step should have moved the page (learned from earlier
/// steps, or a browser's usual distances before any step lined up). Photos are redrawn a
/// little differently after a scroll, pictures load in, and fixed backgrounds stay put, so
/// on such pages nothing reaches 85 %. Then the best shift is taken when it is one of the
/// expected ones and beats every shift more than 2 rows away: repeating content (several
/// equal peaks) don't pass. A shorter shift (the page ran out) must win by a margin.
pub fn scroll_offset<R: Cells>(prev: &[R], next: &[R], expect: &[usize]) -> Option<usize> {
    let n = prev.len().min(next.len());
    // Already lined up without a shift (only a cursor blinked): the page didn't move. Without
    // this, repetitive content (lists, code) could "match" at some period and repeat forever.
    if unchanged(prev, next) { return None; }
    let scores: Vec<(usize, f64)> = (1..n).take_while(|d| n - d >= 16).map(|d| (d, score(prev, next, d))).collect();
    let (d, best) = scores.iter().copied().fold((0, -1.0), |b, s| if s.1 > b.1 { s } else { b });
    if best >= 0.85 { return Some(d); }
    let rival = scores.iter().filter(|(x, _)| x.abs_diff(d) > 2).map(|s| s.1).fold(0.0, f64::max);
    if best < 0.4 { return None; }
    // Where the wheel step should land; or shorter, when the page ran out (the last scroll),
    // which then has to win by a margin.
    let at_expected = expect.iter().any(|e| e.abs_diff(d) <= 1);
    let ran_out = expect.iter().max().is_some_and(|m| d < *m);
    (at_expected && best > rival || ran_out && best - rival >= 0.1).then_some(d)
}

/// Columns at the left and right edge that are (almost all) identical in both frames over
/// `rows`: side panels and vertical tab strips that don't scroll with the page. A few rows
/// may differ (a see-through bar or a tooltip over them).
pub fn fixed_sides(a: &[u8], b: &[u8], w: usize, rows: std::ops::Range<usize>) -> (usize, usize) {
    let still = |x: usize| rows.clone().filter(|&r| { let i = (r * w + x) * 4; a[i..i + 4] == b[i..i + 4] }).count() * 100 >= rows.len() * 97;
    let left = (0..w).take_while(|&x| still(x)).count();
    if left == w { return (0, 0); }
    let right = (0..w - left).take_while(|&i| still(w - 1 - i)).count();
    (left, right)
}

/// How many columns left of `x_end` (searching the last `span`) are a scrollbar: they line
/// up better unshifted than shifted by the scroll `d`, the opposite of page content. Their
/// thumb would otherwise appear in pieces down the joined image.
pub fn scrollbar_cols(a: &[u8], b: &[u8], w: usize, rows: std::ops::Range<usize>, d: usize, x_end: usize, span: usize) -> usize {
    let px = |f: &[u8], r: usize, x: usize| { let i = (r * w + x) * 4; [f[i], f[i + 1], f[i + 2]] };
    let frac = |x: usize, shift: usize| {
        let n = rows.len().saturating_sub(shift).max(1);
        rows.clone().take(n).filter(|&r| px(a, r + shift, x) == px(b, r, x)).count() as f64 / n as f64
    };
    (x_end.saturating_sub(span)..x_end).find(|&x| frac(x, 0) > frac(x, d) + 0.02).map_or(0, |x| x_end - x)
}

pub const STRIPS: usize = 8;

/// Per row, one hash per vertical strip of columns `x0..x1` (the scrolling page only).
pub fn cell_hashes(bgra: &[u8], w: u32, h: u32, x0: usize, x1: usize) -> Vec<Vec<u64>> {
    let w = w as usize;
    let k = if x1 - x0 >= STRIPS * 16 { STRIPS } else { 1 };
    let step = (x1 - x0) / k;
    (0..h as usize).map(|r| (0..k).map(|j| {
        let (c0, c1) = (x0 + j * step, if j + 1 == k { x1 } else { x0 + (j + 1) * step });
        let mut s = std::collections::hash_map::DefaultHasher::new();
        bgra[(r * w + c0) * 4..(r * w + c1) * 4].hash(&mut s);
        s.finish()
    }).collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(rows: std::ops::Range<u64>) -> Vec<u64> { rows.collect() }

    #[test]
    fn finds_how_far_the_page_scrolled() {
        let a = page(100..200); // 100 visible rows of a long page
        let b = page(130..230); // scrolled by 30
        assert_eq!(scroll_offset(&a, &b, &[]), Some(30));
        assert_eq!(scroll_offset(&a, &a, &[]), None, "nothing moved: end of page");
        assert_eq!(scroll_offset(&a, &page(500..600), &[]), None, "no overlap: don't guess");
    }

    #[test]
    fn image_heavy_views_line_up_at_the_expected_shift() {
        // Scrolled by 30 over a gallery: only the rows of text (every 3rd) are pixel-equal;
        // the photos between them were redrawn a little differently. ~33 % match.
        let a: Vec<u64> = (100..200).map(|r| if r % 3 == 0 { r } else { 1_000_000 + r }).collect();
        let b: Vec<u64> = (130..230).map(|r| if r % 3 == 0 { r } else { 2_000_000 + r }).collect();
        assert_eq!(scroll_offset(&a, &b, &[]), None, "too little matches to trust on its own");
        // With every 3rd row a match, shifts 27 and 33 also line text up; the guard is that 30
        // must be the single best shift. Make the text rows unique so it is.
        let a: Vec<u64> = (100..200).map(|r| if r % 2 == 0 { r } else { 1_000_000 + r }).collect();
        let b: Vec<u64> = (130..230).map(|r| if r % 2 == 0 { r } else { 2_000_000 + r }).collect();
        assert_eq!(scroll_offset(&a, &b, &[30]), Some(30), "the expected shift is the clear best");
        assert_eq!(scroll_offset(&a, &b, &[20]), None, "further than the wheel step can go: refuse");
    }

    #[test]
    fn the_last_short_scroll_at_the_end_of_the_page_lines_up() {
        // Wheel step should move 60 rows, but only 25 were left; text rows (every 2nd) match.
        let a: Vec<u64> = (100..200).map(|r| if r % 2 == 0 { r } else { 1_000_000 + r }).collect();
        let b: Vec<u64> = (125..225).map(|r| if r % 2 == 0 { r } else { 2_000_000 + r }).collect();
        assert_eq!(scroll_offset(&a, &b, &[60]), Some(25));
    }

    #[test]
    fn repeating_content_is_not_joined_on_expectation_alone() {
        // A list repeating every 10 rows, scrolled by 3, every 5th row changed: shifts 3, 13,
        // 23… score alike, so even the expected shift isn't trusted.
        let a: Vec<u64> = (0..120).map(|i| i % 10).collect();
        let b: Vec<u64> = (3..123).map(|i| if i % 5 == 0 { 999 } else { i % 10 }).collect();
        assert_eq!(scroll_offset(&a, &b, &[3]), None);
    }

    #[test]
    fn end_of_page_with_a_blinking_cursor_is_not_a_scroll() {
        // Repetitive text (period 4) that did not move, except one changed row.
        let a: Vec<u64> = (0..120).map(|i| i % 4).collect();
        let mut b = a.clone();
        b[50] = 777;
        assert_eq!(scroll_offset(&a, &b, &[]), None);
    }

    #[test]
    fn tolerates_a_few_changed_rows() {
        let a = page(0..100);
        let mut b = page(40..140);
        b[10] = 9999; // e.g. a blinking cursor
        assert_eq!(scroll_offset(&a, &b, &[]), Some(40));
    }

    #[test]
    fn tolerates_animated_content_on_web_pages() {
        let a = page(0..200);
        let mut b = page(60..260);
        for r in 20..40 { b[r] = 10_000 + r as u64; } // a playing video / carousel: 10 % of rows differ
        assert_eq!(scroll_offset(&a, &b, &[]), Some(60));
        assert!(!unchanged(&a, &b));
    }

    #[test]
    fn strips_line_up_a_page_with_a_parallax_picture() {
        // 8 strips; strip 7 is a picture that moves at half speed in the top third of the view.
        let frame = |off: u64| -> Vec<Vec<u64>> { (0..120u64).map(|r| (0..8u64).map(|k|
            if k == 7 && r < 40 { 900_000 + r + off / 2 } else { k * 10_000 + r + off }).collect()).collect() };
        assert_eq!(scroll_offset(&frame(0), &frame(30), &[]), Some(30));
        // As whole rows (one hash each) those top rows never match, and it would stop.
        let whole = |f: Vec<Vec<u64>>| -> Vec<u64> { f.iter().map(|r| r.iter().fold(0u64, |h, c| h.wrapping_mul(31).wrapping_add(*c))).collect() };
        assert_eq!(scroll_offset(&whole(frame(0)), &whole(frame(30)), &[]), None);
    }

    #[test]
    fn finds_side_panels_that_do_not_scroll() {
        let (w, h) = (10usize, 4usize);
        let mut a = vec![0u8; w * h * 4];
        let mut b = a.clone();
        for r in 0..h { for x in 3..8 { a[(r * w + x) * 4] = r as u8; b[(r * w + x) * 4] = r as u8 + 1; } }
        assert_eq!(fixed_sides(&a, &b, w, 0..h), (3, 2));
        assert_eq!(fixed_sides(&a, &a, w, 0..h), (0, 0), "nothing moved: no sides");
    }

    #[test]
    fn finds_the_scrollbar_at_the_right() {
        // 20 columns: page content (moves with the scroll) and a 4 px scrollbar whose thumb
        // only creeps down a little.
        let (w, h, d) = (20usize, 60usize, 10usize);
        let frame = |off: usize, thumb: usize| -> Vec<u8> {
            let mut f = vec![0u8; w * h * 4];
            for r in 0..h { for x in 0..w {
                f[(r * w + x) * 4] = if x < 16 { ((r + off) * 7 + x) as u8 } else if (thumb..thumb + 20).contains(&r) { 200 } else { 90 };
            } }
            f
        };
        assert_eq!(scrollbar_cols(&frame(0, 10), &frame(d, 12), w, 0..h, d, w, 8), 4);
        let no_bar = |off: usize| -> Vec<u8> { let mut f = frame(off, 0); for r in 0..h { for x in 16..w { f[(r * w + x) * 4] = ((r + off) * 3) as u8; } } f };
        assert_eq!(scrollbar_cols(&no_bar(0), &no_bar(d), w, 0..h, d, w, 8), 0);
    }

    #[test]
    fn detects_fixed_toolbar_and_status_bar() {
        let mut a = vec![1, 2, 3]; a.extend(page(100..150)); a.extend([8, 9]);
        let mut b = vec![1, 2, 3]; b.extend(page(120..170)); b.extend([8, 9]);
        assert_eq!(fixed_edges(&a, &b), (3, 2));
        assert_eq!(fixed_edges(&a, &a), (a.len(), 0));
    }

    #[test]
    fn a_moving_scrollbar_does_not_change_row_hashes() {
        let (w, h) = (200u32, 2u32);
        let mut a = vec![50u8; (w * h * 4) as usize];
        let b = a.clone();
        for r in 0..h as usize { for x in 185..200 { a[(r * w as usize + x) * 4] = 255; } } // thumb on the right
        assert_eq!(row_hashes(&a, w, h), row_hashes(&b, w, h));
    }

    #[test]
    fn hashes_rows_independently() {
        let img = [1u8, 2, 3, 4, 1, 2, 3, 4, 9, 9, 9, 9, 9, 9, 9, 9]; // 2x2, two different rows
        let h = row_hashes(&img, 2, 2);
        assert_eq!(h.len(), 2);
        assert_ne!(h[0], h[1]);
    }
}
