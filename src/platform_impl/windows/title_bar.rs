//! Pure geometry for windows that extend their content into the title bar.

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HTCAPTION, HTMAXBUTTON, HTTOP, HTTOPLEFT, HTTOPRIGHT,
};

use crate::dpi::{PhysicalPosition, PhysicalSize};

/// Client-rect top after `DefWindowProc` for `WM_NCCALCSIZE`: the caption is dropped. A maximised
/// window sits `inset` px beyond the monitor (the side inset the system applied), so its client
/// area starts `inset` px lower.
pub(super) fn nccalc_top(saved_top: i32, maximized: bool, inset: i32) -> i32 {
    if maximized {
        saved_top + inset
    } else {
        saved_top
    }
}

/// Hit-test code for the resize band along the top edge of the client area, which no longer has
/// a non-client border to resize from. `x`/`y` are client coordinates. The left corner wins when
/// the window is narrower than two frames.
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

/// Client rect of a caption region given by its origin and size.
pub(super) fn rect(origin: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> RECT {
    let width = i32::try_from(size.width).unwrap_or(i32::MAX);
    let height = i32::try_from(size.height).unwrap_or(i32::MAX);
    RECT {
        left: origin.x,
        top: origin.y,
        right: origin.x.saturating_add(width),
        bottom: origin.y.saturating_add(height),
    }
}

/// Whether the client point lies in one of the regions; right and bottom edges are exclusive.
pub(super) fn contains(regions: &[RECT], x: i32, y: i32) -> bool {
    regions.iter().any(|r| (r.left..r.right).contains(&x) && (r.top..r.bottom).contains(&y))
}

/// Hit-test code for a client point inside a title bar region: `HTMAXBUTTON` over the maximize
/// region, which wins where it overlaps a caption region, then `HTCAPTION`.
pub(super) fn region_hit(
    caption: &[RECT],
    maximize: Option<RECT>,
    x: i32,
    y: i32,
) -> Option<isize> {
    if maximize.is_some_and(|r| contains(&[r], x, y)) {
        Some(HTMAXBUTTON as isize)
    } else if contains(caption, x, y) {
        Some(HTCAPTION as isize)
    } else {
        None
    }
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

    const TOP: isize = HTTOP as isize;
    const LEFT: isize = HTTOPLEFT as isize;
    const RIGHT: isize = HTTOPRIGHT as isize;

    #[test]
    fn nccalc_top_restored_keeps_top() {
        assert_eq!(nccalc_top(100, false, 8), 100);
        assert_eq!(nccalc_top(-8, false, 8), -8);
    }

    #[test]
    fn nccalc_top_maximized_uses_measured_inset() {
        assert_eq!(nccalc_top(-8, true, 8), 0);
        assert_eq!(nccalc_top(-11, true, 11), 0);
        // Non-resizable maximised window: the system only applied a 1 px border.
        assert_eq!(nccalc_top(-1, true, 1), 0);
        assert_eq!(nccalc_top(-1, true, 0), -1);
    }

    #[test]
    fn top_band_edges_at_frame_11_and_14() {
        for frame in [11, 14] {
            assert_eq!(top_band_hit(200, 0, 800, frame), Some(TOP));
            assert_eq!(top_band_hit(200, frame - 1, 800, frame), Some(TOP));
            assert_eq!(top_band_hit(200, frame, 800, frame), None);
            assert_eq!(top_band_hit(200, -1, 800, frame), None);
        }
    }

    #[test]
    fn top_band_corner_boundaries() {
        let (w, f) = (800, 8);
        assert_eq!(top_band_hit(0, 0, w, f), Some(LEFT));
        assert_eq!(top_band_hit(f - 1, 0, w, f), Some(LEFT));
        assert_eq!(top_band_hit(f, 0, w, f), Some(TOP));
        assert_eq!(top_band_hit(w - f - 1, 0, w, f), Some(TOP));
        assert_eq!(top_band_hit(w - f, 0, w, f), Some(RIGHT));
        assert_eq!(top_band_hit(w - 1, 0, w, f), Some(RIGHT));
    }

    #[test]
    fn top_band_narrow_window_prefers_left_corner() {
        // Width 10 < 2 * frame (16): x = 5 is in both corner ranges.
        assert_eq!(top_band_hit(5, 0, 10, 8), Some(LEFT));
        assert_eq!(top_band_hit(8, 0, 10, 8), Some(RIGHT));
    }

    #[test]
    fn caption_rect_from_origin_and_size() {
        let r = rect(PhysicalPosition::new(40, 0), PhysicalSize::new(300, 32));
        assert_eq!((r.left, r.top, r.right, r.bottom), (40, 0, 340, 32));
        // Sizes past i32 saturate instead of wrapping to a negative right edge.
        let r = rect(PhysicalPosition::new(10, 0), PhysicalSize::new(u32::MAX, 1));
        assert_eq!(r.right, i32::MAX);
    }

    #[test]
    fn caption_contains_is_end_exclusive() {
        let regions = [
            rect(PhysicalPosition::new(40, 0), PhysicalSize::new(300, 32)),
            rect(PhysicalPosition::new(500, 0), PhysicalSize::new(100, 32)),
        ];
        assert!(contains(&regions, 40, 0));
        assert!(contains(&regions, 339, 31));
        assert!(!contains(&regions, 340, 10));
        assert!(!contains(&regions, 100, 32));
        assert!(!contains(&regions, 39, 10));
        assert!(contains(&regions, 550, 5));
        assert!(!contains(&[], 0, 0));
    }

    #[test]
    fn region_hit_prefers_maximize_over_caption() {
        let caption = [rect(PhysicalPosition::new(0, 0), PhysicalSize::new(600, 32))];
        let maximize = Some(rect(PhysicalPosition::new(500, 0), PhysicalSize::new(46, 32)));
        let (max, cap) = (HTMAXBUTTON as isize, HTCAPTION as isize);
        assert_eq!(region_hit(&caption, maximize, 500, 0), Some(max));
        assert_eq!(region_hit(&caption, maximize, 545, 31), Some(max));
        assert_eq!(region_hit(&caption, maximize, 546, 10), Some(cap));
        assert_eq!(region_hit(&caption, maximize, 499, 10), Some(cap));
        assert_eq!(region_hit(&caption, maximize, 500, 32), None);
        assert_eq!(region_hit(&caption, None, 520, 10), Some(cap));
        assert_eq!(region_hit(&[], maximize, 520, 10), Some(max));
        assert_eq!(region_hit(&[], None, 520, 10), None);
    }

    #[test]
    fn zero_top_margin_only_touches_top() {
        let original = RECT { left: 0, top: 10, right: 400, bottom: 310 };
        let adjusted = RECT { left: -8, top: -21, right: 408, bottom: 318 };
        let r = zero_top_margin(original, adjusted);
        assert_eq!((r.left, r.top, r.right, r.bottom), (-8, 10, 408, 318));
    }
}
