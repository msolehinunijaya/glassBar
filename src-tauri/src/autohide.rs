//! Auto-hide decisions, made from the real mouse position (polled 10×/s only while auto-hide
//! is on) rather than the page's enter/leave events, which go missing when the bar moves
//! under a still pointer, a menu is open, or the pointer jumps to another window.
use crate::platform::Rect;

pub const TICK_MS: u32 = 100;
/// Pointer away from the bar this long (and nothing open) → slide away.
pub const HIDE_AFTER_MS: u32 = 700;
/// Pointer resting on the screen edge this long → slide in (avoids reveals on fly-bys).
pub const SHOW_AFTER_MS: u32 = 100;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Act { Show, Hide, Stay }

#[derive(Default)]
pub struct Timers { away: u32, dwell: u32 }

pub struct Inputs {
    pub revealed: bool,
    /// Pointer over the shown bar (a few px of slack).
    pub over_bar: bool,
    /// Pointer touching the screen edge within the bar's span.
    pub at_edge: bool,
    /// A panel, menu, drag, edit mode or held mouse button: never hide during these.
    pub busy: bool,
}

pub fn step(i: &Inputs, t: &mut Timers) -> Act {
    if i.revealed {
        t.dwell = 0;
        // The edge counts as "on the bar": it is what revealed it, so resting there (below a
        // bottom dock, in the gap) must not hide it only for the edge to show it again.
        if i.over_bar || i.at_edge || i.busy { t.away = 0; return Act::Stay; }
        t.away += TICK_MS;
        if t.away >= HIDE_AFTER_MS { t.away = 0; return Act::Hide; }
    } else {
        t.away = 0;
        if i.at_edge && !i.busy { t.dwell += TICK_MS; if t.dwell >= SHOW_AFTER_MS { t.dwell = 0; return Act::Show; } } else { t.dwell = 0; }
    }
    Act::Stay
}

pub fn contains(r: Rect, x: i32, y: i32, slack: i32) -> bool {
    x >= r.x - slack && x < r.x + r.w + slack && y >= r.y - slack && y < r.y + r.h + slack
}

/// The strip along the screen edge (2 px deep) that the bar slides out from, limited to the
/// bar's own length.
pub fn edge_zone(shown: Rect, edge: crate::config::Edge, mon: Rect) -> Rect {
    use crate::config::Edge::*;
    match edge {
        Bottom => Rect { x: shown.x, y: mon.y + mon.h - 2, w: shown.w, h: 2 },
        Top => Rect { x: shown.x, y: mon.y, w: shown.w, h: 2 },
        Left => Rect { x: mon.x, y: shown.y, w: 2, h: shown.h },
        Right => Rect { x: mon.x + mon.w - 2, y: shown.y, w: 2, h: shown.h },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(i: &Inputs, t: &mut Timers, ticks: u32) -> Vec<Act> { (0..ticks).map(|_| step(i, t)).collect() }

    #[test]
    fn hides_after_the_pointer_has_been_away_for_a_while() {
        let mut t = Timers::default();
        let away = Inputs { revealed: true, over_bar: false, at_edge: false, busy: false };
        let acts = run(&away, &mut t, 7);
        assert_eq!(acts[..6], [Act::Stay; 6]);
        assert_eq!(acts[6], Act::Hide);
    }

    #[test]
    fn never_hides_while_a_panel_or_menu_is_open_or_the_pointer_is_back() {
        let mut t = Timers::default();
        let busy = Inputs { revealed: true, over_bar: false, at_edge: false, busy: true };
        assert!(run(&busy, &mut t, 50).iter().all(|a| *a == Act::Stay));
        let away = Inputs { busy: false, ..busy };
        run(&away, &mut t, 5);
        let back = Inputs { over_bar: true, ..away };
        step(&back, &mut t); // pointer returns: the countdown restarts
        assert!(run(&away, &mut t, 6).iter().all(|a| *a == Act::Stay));
    }

    #[test]
    fn shows_when_the_pointer_rests_on_the_edge() {
        let mut t = Timers::default();
        let edge = Inputs { revealed: false, over_bar: false, at_edge: true, busy: false };
        assert_eq!(step(&edge, &mut t), Act::Show);
        let elsewhere = Inputs { at_edge: false, ..edge };
        assert!(run(&elsewhere, &mut t, 20).iter().all(|a| *a == Act::Stay));
        let dragging_past = Inputs { busy: true, ..edge }; // e.g. a window being dragged across the edge
        assert_eq!(step(&dragging_past, &mut t), Act::Stay);
    }

    #[test]
    fn resting_on_the_edge_keeps_it_shown_instead_of_looping() {
        let mut t = Timers::default();
        let on_edge = Inputs { revealed: true, over_bar: false, at_edge: true, busy: false };
        assert!(run(&on_edge, &mut t, 50).iter().all(|a| *a == Act::Stay));
    }

    #[test]
    fn edge_zone_follows_the_dock_edge() {
        use crate::config::Edge;
        let mon = Rect { x: 0, y: 0, w: 1920, h: 1080 };
        let bar = Rect { x: 94, y: 1015, w: 1747, h: 60 };
        assert_eq!(edge_zone(bar, Edge::Bottom, mon), Rect { x: 94, y: 1078, w: 1747, h: 2 });
        let side = Rect { x: 1862, y: 49, w: 60, h: 983 };
        assert_eq!(edge_zone(side, Edge::Right, mon), Rect { x: 1918, y: 49, w: 2, h: 983 });
        assert!(contains(bar, 100, 1040, 0) && !contains(bar, 100, 900, 4));
    }
}
