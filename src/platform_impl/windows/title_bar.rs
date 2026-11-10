//! Pure geometry for windows that extend their content into the title bar.

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{HTTOP, HTTOPLEFT, HTTOPRIGHT};

/// Client-rect top after `DefWindowProc` for `WM_NCCALCSIZE`: the caption is dropped. A maximised
/// window sits `frame` px above the monitor, so its client area starts `frame` px lower.
pub(super) fn nccalc_top(saved_top: i32, maximized: bool, frame: i32) -> i32 {
    if maximized {
        saved_top + frame
    } else {
        saved_top
    }
}

/// Hit-test code for the resize band along the top edge of the client area, which no longer has
/// a non-client border to resize from. `x`/`y` are client coordinates.
pub(super) fn top_band_hit(x: i32, y: i32, width: i32, frame: i32) -> Option<isize> {
    if !(0..frame).contains(&y) {
        return None;
    }
    Some(if x < frame {
        HTTOPLEFT
    } else if x >= width - frame {
        HTTOPRIGHT
    } else {
        HTTOP
    } as isize)
}

/// Keeps the top edge of the requested client rect, discarding the caption margin that
/// `AdjustWindowRectEx` added.
pub(super) fn zero_top_margin(original: RECT, mut adjusted: RECT) -> RECT {
    adjusted.top = original.top;
    adjusted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nccalc_top_restored_keeps_top() {
        assert_eq!(nccalc_top(100, false, 8), 100);
    }

    #[test]
    fn nccalc_top_maximized_adds_frame() {
        assert_eq!(nccalc_top(-8, true, 8), 0);
    }

    #[test]
    fn top_band_classifies_edge_and_corners() {
        assert_eq!(top_band_hit(100, 3, 800, 8), Some(HTTOP as isize));
        assert_eq!(top_band_hit(2, 0, 800, 8), Some(HTTOPLEFT as isize));
        assert_eq!(top_band_hit(799, 7, 800, 8), Some(HTTOPRIGHT as isize));
        assert_eq!(top_band_hit(792, 7, 800, 8), Some(HTTOPRIGHT as isize));
        assert_eq!(top_band_hit(791, 7, 800, 8), Some(HTTOP as isize));
    }

    #[test]
    fn top_band_ignores_outside_band() {
        assert_eq!(top_band_hit(100, 8, 800, 8), None);
        assert_eq!(top_band_hit(100, -1, 800, 8), None);
    }

    #[test]
    fn zero_top_margin_only_touches_top() {
        let original = RECT { left: 0, top: 10, right: 400, bottom: 310 };
        let adjusted = RECT { left: -8, top: -21, right: 408, bottom: 318 };
        let r = zero_top_margin(original, adjusted);
        assert_eq!((r.left, r.top, r.right, r.bottom), (-8, 10, 408, 318));
    }
}
