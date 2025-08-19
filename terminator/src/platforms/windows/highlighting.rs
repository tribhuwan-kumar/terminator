//! Element highlighting functionality for Windows

use super::types::{FontStyle, HighlightHandle, TextPosition};
use crate::platforms::windows::utils::convert_uiautomation_element_to_terminator;
use crate::AutomationError;
use crate::UIElement as TerminatorElement;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error};

use uiautomation::UIElement;
// Windows GDI imports
use windows::Win32::Foundation::{COLORREF, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, GetDC, Rectangle,
    ReleaseDC, SelectObject, SetBkMode, SetTextColor, DT_SINGLELINE, HBRUSH, HGDIOBJ, PS_SOLID,
    TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

// Additional imports for overlay window approach
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, LoadCursorW, RegisterClassExW,
    SetLayeredWindowAttributes, ShowWindow, HICON, IDC_ARROW, LWA_COLORKEY, SW_SHOWNOACTIVATE,
    WM_DESTROY, WM_PAINT, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

const OVERLAY_CLASS_NAME: PCWSTR = w!("TerminatorHighlightOverlay");

fn rects_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    let ar = ax + aw;
    let ab = ay + ah;
    let br = bx + bw;
    let bb = by + bh;
    ax < br && ar > bx && ay < bb && ab > by
}

/// Implementation of element highlighting for Windows UI elements
pub fn highlight(
    element: Arc<UIElement>,
    color: Option<u32>,
    duration: Option<Duration>,
    text: Option<&str>,
    text_position: Option<TextPosition>,
    font_style: Option<FontStyle>,
) -> Result<HighlightHandle, AutomationError> {
    // Best-effort: ensure element is in view before computing bounds
    // Wrap UIA element into our cross-platform UIElement and use the helper
    let wrapped: TerminatorElement =
        convert_uiautomation_element_to_terminator(element.as_ref().clone());
    // Determine if element intersects window viewport; if not, try to scroll it into view
    let mut need_scroll = false;
    if let Ok((ex, ey, ew, eh)) = wrapped.bounds() {
        // info!("highlight: element bounds: x={ex}, y={ey}, w={ew}, h={eh}");

        // Try to get window bounds, but if that fails, use heuristics
        if let Ok(Some(win)) = wrapped.window() {
            if let Ok((wx, wy, ww, wh)) = win.bounds() {
                // info!("highlight: window bounds: x={wx}, y={wy}, w={ww}, h={wh}");
                let e_box = (ex as i32, ey as i32, ew as i32, eh as i32);
                let w_box = (wx as i32, wy as i32, ww as i32, wh as i32);
                if !rects_intersect(e_box, w_box) {
                    // info!("highlight: element NOT in viewport, need scroll");
                    need_scroll = true;
                } else {
                    // info!("highlight: element IS in viewport, no scroll needed");
                }
            } else {
                // info!("highlight: could not get window bounds, using heuristic");
                // Heuristic: if element Y > 1080 (typical viewport height), probably needs scroll
                if ey > 1080.0 {
                    // info!("highlight: element Y={ey} > 1080, assuming need scroll");
                    need_scroll = true;
                }
            }
        } else {
            // info!("highlight: could not get window, using heuristic");
            // Heuristic: if element Y > 1080 (typical viewport height), probably needs scroll
            if ey > 1080.0 {
                // info!("highlight: element Y={ey} > 1080, assuming need scroll");
                need_scroll = true;
            }
        }
    } else if !wrapped.is_visible().unwrap_or(true) {
        // info!("highlight: element not visible, need scroll");
        need_scroll = true;
    }
    if need_scroll {
        // First try focusing the element to allow the application to auto-scroll it into view.
        // info!("highlight: element outside viewport; attempting focus() to auto-scroll into view");
        match wrapped.focus() {
            Ok(()) => {
                // Re-check visibility/intersection after focus
                let mut still_offscreen = false;
                if let Ok((_ex2, ey2, _ew2, _eh2)) = wrapped.bounds() {
                    // info!("highlight: after focus(), element bounds: x={ex2}, y={ey2}, w={ew2}, h={eh2}");
                    // Use same heuristic as before
                    if ey2 > 1080.0 {
                        // info!("highlight: after focus(), element Y={ey2} still > 1080");
                        still_offscreen = true;
                    } else {
                        // info!("highlight: after focus(), element Y={ey2} now <= 1080, in view!");
                    }
                } else if !wrapped.is_visible().unwrap_or(true) {
                    still_offscreen = true;
                }
                if !still_offscreen {
                    // info!(
                    //     "highlight: focus() brought element into view; skipping scroll_into_view"
                    // );
                    need_scroll = false;
                } else {
                    // info!("highlight: focus() did not bring element into view; will attempt scroll_into_view()");
                }
            }
            Err(_e) => {
                // info!("highlight: focus() failed: {e}; will attempt scroll_into_view()");
            }
        }

        if need_scroll {
            // info!("highlight: element outside viewport; attempting scroll_into_view()");
            if let Err(_e) = wrapped.scroll_into_view() {
                // info!("highlight: scroll_into_view failed: {e}");
            } else {
                // info!("highlight: scroll_into_view succeeded");

                // After initial scroll, verify element position and adjust if needed
                std::thread::sleep(Duration::from_millis(50)); // Let initial scroll settle

                if let Ok((_ex, ey, _ew, eh)) = wrapped.bounds() {
                    // info!("highlight: after scroll_into_view, element at y={ey}");

                    // Define optimal viewport zones (assuming typical 1080p screen)
                    const VIEWPORT_TOP_EDGE: f64 = 100.0; // Too close to top
                    const VIEWPORT_OPTIMAL_BOTTOM: f64 = 700.0; // Good zone ends here
                    const VIEWPORT_BOTTOM_EDGE: f64 = 900.0; // Too close to bottom

                    // Check if we have window bounds for more accurate positioning
                    let mut needs_adjustment = false;
                    let mut adjustment_direction: Option<&str> = None;

                    if let Ok(Some(window)) = wrapped.window() {
                        if let Ok((_wx, wy, _ww, wh)) = window.bounds() {
                            // We have window bounds - use precise positioning
                            let element_relative_y = ey - wy;
                            let element_bottom = element_relative_y + eh;

                            // info!("highlight: element relative_y={element_relative_y}, window_height={wh}");

                            // Check if element is poorly positioned
                            if element_relative_y < 50.0 {
                                // Too close to top - scroll up a bit
                                // info!("highlight: element too close to top ({element_relative_y}px)");
                                needs_adjustment = true;
                                adjustment_direction = Some("up");
                            } else if element_bottom > wh - 50.0 {
                                // Too close to bottom or cut off - scroll down a bit
                                // info!("highlight: element too close to bottom or cut off");
                                needs_adjustment = true;
                                adjustment_direction = Some("down");
                            } else if element_relative_y > wh * 0.7 {
                                // Element is in lower 30% of viewport - not ideal
                                // info!("highlight: element in lower portion of viewport");
                                needs_adjustment = true;
                                adjustment_direction = Some("down");
                            }
                        } else {
                            // No window bounds - use heuristic based on absolute Y position
                            if ey < VIEWPORT_TOP_EDGE {
                                // info!("highlight: element at y={ey} < {VIEWPORT_TOP_EDGE}, too high");
                                needs_adjustment = true;
                                adjustment_direction = Some("up");
                            } else if ey > VIEWPORT_BOTTOM_EDGE {
                                // info!("highlight: element at y={ey} > {VIEWPORT_BOTTOM_EDGE}, too low");
                                needs_adjustment = true;
                                adjustment_direction = Some("down");
                            } else if ey > VIEWPORT_OPTIMAL_BOTTOM {
                                // Element is lower than optimal but not at edge
                                // info!("highlight: element at y={ey} lower than optimal");
                                needs_adjustment = true;
                                adjustment_direction = Some("down");
                            }
                        }
                    } else {
                        // No window available - use simple heuristics
                        if !(VIEWPORT_TOP_EDGE..=VIEWPORT_BOTTOM_EDGE).contains(&ey) {
                            needs_adjustment = true;
                            adjustment_direction =
                                Some(if ey < VIEWPORT_TOP_EDGE { "up" } else { "down" });
                        }
                    }

                    // Perform fine adjustment if needed
                    if needs_adjustment {
                        if let Some(direction) = adjustment_direction {
                            // info!("highlight: performing fine adjustment scroll {direction}");
                            // Use smaller scroll amount for fine adjustment (0.3 = ~3 lines)
                            let _ = wrapped.scroll(direction, 0.3);
                            std::thread::sleep(Duration::from_millis(50));

                            // Check final position
                            if let Ok((_, _final_y, _, _)) = wrapped.bounds() {
                                // info!("highlight: final position after adjustment: y={_final_y}");
                            }
                        }
                    }
                }
            }
        }
    }

    // Get the (possibly updated) element bounding rectangle
    // First check what the wrapped element thinks its bounds are
    if let Ok((_wx, _wy, _ww, _wh)) = wrapped.bounds() {
        // info!("highlight: wrapped element final bounds: x={wx}, y={wy}, w={ww}, h={wh}");
    }

    // Small delay to let any scrolling animation settle
    std::thread::sleep(Duration::from_millis(100));

    let rect = element.get_bounding_rectangle().map_err(|e| {
        AutomationError::PlatformError(format!("Failed to get element bounds: {e}"))
    })?;

    // Log the rectangle bounds - these are what we'll use for the highlight
    // info!(
    //     "highlight: UIAutomation element final bounds for overlay: left={}, top={}, width={}, height={}",
    //     rect.get_left(),
    //     rect.get_top(),
    //     rect.get_width(),
    //     rect.get_height()
    // );

    // Try to get scale factor from focused window first, fall back to cursor position,
    // but allow disabling via env for debugging
    let scale_factor = if std::env::var("TERMINATOR_NO_DPI").is_ok() {
        1.0
    } else {
        get_scale_factor_from_focused_window().unwrap_or_else(get_scale_factor_from_cursor)
    };

    // Constants for border appearance
    const DEFAULT_RED_COLOR: u32 = 0x0000FF; // Pure red in BGR format

    // Use provided color or default to red
    let highlight_color = color.unwrap_or(DEFAULT_RED_COLOR);

    // Scale the coordinates and dimensions
    // info!("highlight: applying scale_factor={scale_factor} to coordinates");
    let mut x = (rect.get_left() as f64 * scale_factor) as i32;
    let mut y = (rect.get_top() as f64 * scale_factor) as i32;
    let mut width = (rect.get_width() as f64 * scale_factor) as i32;
    let mut height = (rect.get_height() as f64 * scale_factor) as i32;

    // Validate coordinates
    if width <= 0 || height <= 0 {
        return Err(AutomationError::PlatformError(format!(
            "Invalid element dimensions: width={width}, height={height}"
        )));
    }

    // info!(
    //     "highlight: scaled coordinates for overlay: x={}, y={}, width={}, height={}",
    //     x, y, width, height
    // );

    // Validate coordinates against virtual screen bounds; if out-of-bounds, fallback to no-DPI scaling
    let vs_x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let vs_y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let vs_w = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let vs_h = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    let out_of_bounds = x < vs_x - 100
        || y < vs_y - 100
        || x + width > vs_x + vs_w + 100
        || y + height > vs_y + vs_h + 100;
    if out_of_bounds && (scale_factor - 1.0).abs() > f64::EPSILON {
        // info!(
        //     "DPI fallback: coords out of virtual screen (vs: {},{} {}x{}). Using unscaled bounds.",
        //     vs_x, vs_y, vs_w, vs_h
        // );
        x = rect.get_left();
        y = rect.get_top();
        width = rect.get_width();
        height = rect.get_height();
        debug!(
            "Unscaled highlight coordinates: x={}, y={}, width={}, height={}",
            x, y, width, height
        );
    }

    // Prepare text overlay data (no truncation for better readability)
    let text_data = text.map(|t| {
        let display_text = if t.len() > 30 {
            format!("{}...", &t[..27]) // Allow longer text (30 chars max)
        } else {
            t.to_string()
        };
        let font_style = font_style.unwrap_or_default();
        let position = text_position.unwrap_or(TextPosition::Top);
        (display_text, font_style, position)
    });

    // Create atomic bool for controlling the highlight thread
    let should_close = Arc::new(AtomicBool::new(false));
    let should_close_clone = should_close.clone();

    // Spawn a thread to handle the highlighting
    let handle = thread::spawn(move || {
        let start_time = Instant::now();
        let duration = duration.unwrap_or(Duration::from_millis(3000)); // Default 3 seconds

        // info!("OVERLAY_THREAD_START duration_ms={}", duration.as_millis());

        // Compute overlay extents and draw overlay window content
        let mut overlay_x = x;
        let mut overlay_y = y;
        let mut overlay_w = width;
        let mut overlay_h = height;
        let mut border_offset_x = 0;
        let mut border_offset_y = 0;
        let mut text_rect: Option<(i32, i32, i32, i32)> = None; // (x, y, w, h) relative to overlay

        if let Some((_, ref fs, pos)) = text_data {
            // Approximate text box size and position
            let (tw, th) = if fs.size > 0 {
                (width.clamp(200, 600), (fs.size as i32 + 22).max(40))
            } else {
                (width.max(200), 50)
            };
            let (tx_abs, ty_abs) = match pos {
                TextPosition::Top => (x, y - th - 10),
                TextPosition::TopRight => (x + width + 15, y - th - 10),
                TextPosition::Right => (x + width + 15, y + height / 2 - th / 2),
                TextPosition::BottomRight => (x + width + 15, y + height + 15),
                TextPosition::Bottom => (x, y + height + 15),
                TextPosition::BottomLeft => (x - tw - 15, y + height + 15),
                TextPosition::Left => (x - tw - 15, y + height / 2 - th / 2),
                TextPosition::TopLeft => (x - tw - 15, y - th - 10),
                TextPosition::Inside => (x + 15, y + 15),
            };
            let right = (x + width).max(tx_abs + tw);
            let bottom = (y + height).max(ty_abs + th);
            overlay_x = overlay_x.min(tx_abs);
            overlay_y = overlay_y.min(ty_abs);
            overlay_w = right - overlay_x;
            overlay_h = bottom - overlay_y;
            border_offset_x = x - overlay_x;
            border_offset_y = y - overlay_y;
            text_rect = Some((tx_abs - overlay_x, ty_abs - overlay_y, tw, th));
            debug!(
                "Overlay with text: overlay_x={}, overlay_y={}, overlay_w={}, overlay_h={}, border_offset=({}, {}), text_rect={:?}",
                overlay_x, overlay_y, overlay_w, overlay_h, border_offset_x, border_offset_y, text_rect
            );
        } else {
            debug!(
                "Overlay without text: overlay_x={}, overlay_y={}, overlay_w={}, overlay_h={}",
                overlay_x, overlay_y, overlay_w, overlay_h
            );
        }

        if let Err(e) = create_and_show_overlay(
            overlay_x,
            overlay_y,
            overlay_w,
            overlay_h,
            border_offset_x,
            border_offset_y,
            width,
            height,
            highlight_color,
            text_data
                .as_ref()
                .map(|(t, fs, _)| (t.as_str(), fs.clone())),
            text_rect,
        ) {
            error!("Failed to create overlay highlight: {}", e);
        }

        debug!("Waiting for highlight duration: {:?}", duration);
        while start_time.elapsed() < duration && !should_close_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(50));
        }

        // Let the overlay window be destroyed by the OS when the process exits
        // or when a subsequent highlight replaces it. Avoid explicit DestroyWindow
        // here to reduce flakiness if the caller drops the handle early.

        // info!(
        //     "OVERLAY_THREAD_DONE elapsed_ms={}",
        //     start_time.elapsed().as_millis()
        // );
    });

    Ok(HighlightHandle {
        should_close,
        handle: Some(handle),
    })
}

/// Helper function to get scale factor from cursor position
fn get_scale_factor_from_cursor() -> f64 {
    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        let _ = GetCursorPos(&mut point);
    }
    match xcap::Monitor::from_point(point.x, point.y) {
        Ok(monitor) => match monitor.scale_factor() {
            Ok(factor) => factor as f64,
            Err(e) => {
                error!("Failed to get scale factor from cursor position: {}", e);
                1.0 // Fallback to default scale factor
            }
        },
        Err(e) => {
            error!("Failed to get monitor from cursor position: {}", e);
            1.0 // Fallback to default scale factor
        }
    }
}

/// Helper function to get scale factor from focused window
fn get_scale_factor_from_focused_window() -> Option<f64> {
    match xcap::Window::all() {
        Ok(windows) => windows
            .iter()
            .find(|w| w.is_focused().unwrap_or(false))
            .and_then(|focused_window| focused_window.current_monitor().ok())
            .and_then(|monitor| monitor.scale_factor().ok().map(|factor| factor as f64)),
        Err(e) => {
            error!("Failed to get windows: {}", e);
            None
        }
    }
}

// Thread-local storage for the last created overlay window to destroy later
thread_local! {
    static LAST_CREATED_OVERLAY: std::cell::RefCell<Option<HWND>> = const { std::cell::RefCell::new(None) };
}

/// Creates and shows a transparent overlay window and draws the highlight and optional text
#[allow(clippy::too_many_arguments)]
fn create_and_show_overlay(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    border_offset_x: i32,
    border_offset_y: i32,
    element_w: i32,
    element_h: i32,
    border_color_bgr: u32,
    text: Option<(&str, FontStyle)>,
    text_rect: Option<(i32, i32, i32, i32)>,
) -> Result<(), AutomationError> {
    unsafe {
        let instance = GetModuleHandleW(None)
            .map_err(|e| AutomationError::PlatformError(format!("GetModuleHandleW failed: {e}")))?;

        // Register window class (ignore already registered)
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
            lpfnWndProc: Some(overlay_window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: HICON::default(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: OVERLAY_CLASS_NAME,
            hIconSm: HICON::default(),
        };
        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            debug!("RegisterClassExW returned 0 (class may already exist)");
        }

        // Create overlay window
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            OVERLAY_CLASS_NAME,
            w!("Highlight Overlay"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .map_err(|e| AutomationError::PlatformError(format!("CreateWindowExW failed: {e}")))?;

        if hwnd.is_invalid() {
            return Err(AutomationError::PlatformError(
                "CreateWindowExW returned invalid HWND".to_string(),
            ));
        }

        // Make black transparent and allow drawing colored border
        SetLayeredWindowAttributes(hwnd, COLORREF(0x000000), 255, LWA_COLORKEY).map_err(|e| {
            AutomationError::PlatformError(format!("SetLayeredWindowAttributes failed: {e}"))
        })?;

        // Show without activating
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        // Draw contents once
        draw_highlight_on_window(
            hwnd,
            border_offset_x,
            border_offset_y,
            element_w,
            element_h,
            border_color_bgr,
            text,
            text_rect,
        );

        // Save HWND for later destruction
        LAST_CREATED_OVERLAY.with(|cell| {
            *cell.borrow_mut() = Some(hwnd);
        });
    }
    Ok(())
}

/// Draw the highlight border and optional text on the overlay window
#[allow(clippy::too_many_arguments)]
fn draw_highlight_on_window(
    hwnd: HWND,
    border_offset_x: i32,
    border_offset_y: i32,
    element_w: i32,
    element_h: i32,
    border_color_bgr: u32,
    text: Option<(&str, FontStyle)>,
    text_rect: Option<(i32, i32, i32, i32)>,
) {
    unsafe {
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            return;
        }

        // Fill entire window with transparent color (black)
        let black_brush = CreateSolidBrush(COLORREF(0x000000));
        let mut window_rect = RECT::default();
        let _ = GetClientRect(hwnd, &mut window_rect);
        let _ = FillRect(hdc, &window_rect, black_brush);
        let _ = DeleteObject(black_brush.into());

        // Draw the highlight border inside the overlay
        let hpen = CreatePen(PS_SOLID, 6, COLORREF(border_color_bgr));
        let old_pen = SelectObject(hdc, HGDIOBJ(hpen.0));
        // Use black brush for interior so it remains transparent due to color key
        let black_brush = CreateSolidBrush(COLORREF(0x000000));
        let old_brush = SelectObject(hdc, HGDIOBJ(black_brush.0));

        // Rectangle around the element region inside overlay (2px inset for aesthetics)
        let left = border_offset_x + 2;
        let top = border_offset_y + 2;
        let right = border_offset_x + element_w - 2;
        let bottom = border_offset_y + element_h - 2;
        let _ = Rectangle(hdc, left, top, right, bottom);

        // Optional text inside the overlay
        if let (Some((txt, fs)), Some((tx, ty, tw, th))) = (text, text_rect) {
            let font_size = if fs.size > 0 { fs.size as i32 } else { 18 };
            let font = CreateFontW(
                font_size,
                0,
                0,
                0,
                700,
                0,
                0,
                0,
                windows::Win32::Graphics::Gdi::FONT_CHARSET(1),
                windows::Win32::Graphics::Gdi::FONT_OUTPUT_PRECISION(0),
                windows::Win32::Graphics::Gdi::FONT_CLIP_PRECISION(0),
                windows::Win32::Graphics::Gdi::FONT_QUALITY(0),
                0,
                PCWSTR::null(),
            );
            let old_font = SelectObject(hdc, HGDIOBJ(font.0));

            let text_color = if fs.color == 0 { 0x00FF00 } else { fs.color };
            SetTextColor(hdc, COLORREF(text_color));
            SetBkMode(hdc, TRANSPARENT);

            let mut wide_text: Vec<u16> = txt.encode_utf16().collect();
            wide_text.push(0);

            let mut rect = RECT {
                left: tx + 5,
                top: ty + 5,
                right: tx + tw - 5,
                bottom: ty + th - 5,
            };
            let _ = DrawTextW(hdc, &mut wide_text, &mut rect, DT_SINGLELINE);

            // Restore font
            SelectObject(hdc, old_font);
            let _ = DeleteObject(HGDIOBJ(font.0));
        }

        // Cleanup
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(black_brush.into());
        let _ = DeleteObject(hpen.into());
        let _ = ReleaseDC(Some(hwnd), hdc);
    }
}

/// Minimal window procedure; repaints draw content again if needed
unsafe extern "system" fn overlay_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => LRESULT(0),
        WM_PAINT => {
            // On paint, redraw a basic border without text as a best-effort fallback
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            draw_highlight_on_window(hwnd, 0, 0, rect.right, rect.bottom, 0x0000FF, None, None);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
