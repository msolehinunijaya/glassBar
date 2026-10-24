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

/// How far the page moved (in rows) from `prev` to `next` (both the scrolling part only):
/// the shift d where prev[d..] best lines up with next[..]. None when nothing moved or no
/// reliable overlap (≥ 95 % of at least 16 rows must match).
pub fn scroll_offset(prev: &[u64], next: &[u64]) -> Option<usize> {
    let n = prev.len().min(next.len());
    // Already lined up without a shift (only a cursor blinked): the page didn't move. Without
    // this, repetitive content (lists, code) could "match" at some period and repeat forever.
    if n > 0 && (0..n).filter(|&i| prev[i] == next[i]).count() * 100 >= n * 90 { return None; }
    let mut best: Option<(usize, f64)> = None;
    for d in 1..n {
        let overlap = n - d;
        if overlap < 16 { break; }
        let same = (0..overlap).filter(|&i| prev[i + d] == next[i]).count();
        let score = same as f64 / overlap as f64;
        if score >= 0.95 && best.is_none_or(|(_, s)| score > s) { best = Some((d, score)); if score == 1.0 { break; } }
    }
    best.map(|(d, _)| d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(rows: std::ops::Range<u64>) -> Vec<u64> { rows.collect() }

    #[test]
    fn finds_how_far_the_page_scrolled() {
        let a = page(100..200); // 100 visible rows of a long page
        let b = page(130..230); // scrolled by 30
        assert_eq!(scroll_offset(&a, &b), Some(30));
        assert_eq!(scroll_offset(&a, &a), None, "nothing moved: end of page");
        assert_eq!(scroll_offset(&a, &page(500..600)), None, "no overlap: don't guess");
    }

    #[test]
    fn end_of_page_with_a_blinking_cursor_is_not_a_scroll() {
        // Repetitive text (period 4) that did not move, except one changed row.
        let a: Vec<u64> = (0..120).map(|i| i % 4).collect();
        let mut b = a.clone();
        b[50] = 777;
        assert_eq!(scroll_offset(&a, &b), None);
    }

    #[test]
    fn tolerates_a_few_changed_rows() {
        let a = page(0..100);
        let mut b = page(40..140);
        b[10] = 9999; // e.g. a blinking cursor
        assert_eq!(scroll_offset(&a, &b), Some(40));
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
