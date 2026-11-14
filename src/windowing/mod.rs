//! Win32 Window Management for NotchIsland
//! Handles top-center screen positioning, per-monitor DPI detection,
//! topmost Z-order enforcement, and dynamic WS_EX_TRANSPARENT click-through toggling.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub width: i32,
    pub height: i32,
    pub dpi_scale: f32,
}

impl Default for MonitorBounds {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
            width: 1920,
            height: 1080,
            dpi_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorPos {
    pub x: i32,
    pub y: i32,
}

pub struct WindowManager {
    #[cfg(windows)]
    hwnd: Option<windows_sys::Win32::Foundation::HWND>,
    is_click_through: bool,
    monitor_bounds: MonitorBounds,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    pub fn new() -> Self {
        let monitor_bounds = Self::query_primary_monitor();
        Self {
            #[cfg(windows)]
            hwnd: None,
            is_click_through: false,
            monitor_bounds,
        }
    }

    /// Set or update the cached HWND using an isize handle
    pub fn set_hwnd(&mut self, hwnd: isize) {
        #[cfg(windows)]
        {
            if hwnd != 0 {
                self.hwnd = Some(hwnd as windows_sys::Win32::Foundation::HWND);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = hwnd;
        }
    }

    /// Set or update the cached HWND using native HWND type
    #[cfg(windows)]
    pub fn set_raw_hwnd(&mut self, hwnd: windows_sys::Win32::Foundation::HWND) {
        if !hwnd.is_null() {
            self.hwnd = Some(hwnd);
        }
    }

    /// Get current monitor bounds
    pub fn monitor_bounds(&self) -> MonitorBounds {
        self.monitor_bounds
    }

    /// Refresh and return primary monitor bounds
    pub fn refresh_monitor_bounds(&mut self) -> MonitorBounds {
        self.monitor_bounds = Self::query_primary_monitor();
        self.monitor_bounds
    }

    /// Query the primary monitor dimensions and DPI scale
    pub fn query_primary_monitor() -> MonitorBounds {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::Foundation::POINT;
            use windows_sys::Win32::Graphics::Gdi::{
                GetDC, GetDeviceCaps, GetMonitorInfoW, MonitorFromPoint, ReleaseDC,
                LOGPIXELSX, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
            };
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

            // Query DPI scale via DC
            let hdc = GetDC(std::ptr::null_mut());
            let dpi_scale = if !hdc.is_null() {
                let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX as i32);
                ReleaseDC(std::ptr::null_mut(), hdc);
                if dpi_x > 0 {
                    dpi_x as f32 / 96.0
                } else {
                    1.0
                }
            } else {
                1.0
            };

            let pt = POINT { x: 0, y: 0 };
            let hmonitor = MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY);
            if !hmonitor.is_null() {
                let mut mi: MONITORINFO = std::mem::zeroed();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(hmonitor, &mut mi) != 0 {
                    let w = mi.rcMonitor.right - mi.rcMonitor.left;
                    let h = mi.rcMonitor.bottom - mi.rcMonitor.top;
                    return MonitorBounds {
                        left: mi.rcMonitor.left,
                        top: mi.rcMonitor.top,
                        right: mi.rcMonitor.right,
                        bottom: mi.rcMonitor.bottom,
                        width: w,
                        height: h,
                        dpi_scale,
                    };
                }
            }

            let w = GetSystemMetrics(SM_CXSCREEN);
            let h = GetSystemMetrics(SM_CYSCREEN);
            MonitorBounds {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
                width: w,
                height: h,
                dpi_scale,
            }
        }

        #[cfg(not(windows))]
        MonitorBounds::default()
    }

    /// Calculate the top-center screen coordinates for a given window size
    pub fn calculate_top_center(&self, window_width: f32, _window_height: f32) -> (f32, f32) {
        let x = self.monitor_bounds.left as f32 + (self.monitor_bounds.width as f32 - window_width) / 2.0;
        let y = self.monitor_bounds.top as f32;
        (x, y)
    }

    /// Get current global cursor position in screen coordinates
    pub fn get_cursor_pos() -> Option<CursorPos> {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::Foundation::POINT;
            use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

            let mut pt = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut pt) != 0 {
                Some(CursorPos { x: pt.x, y: pt.y })
            } else {
                None
            }
        }

        #[cfg(not(windows))]
        None
    }

    /// Apply base Win32 styles: WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST
    pub fn apply_initial_styles(&self) {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
                    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
                    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
                };

                let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
                let mut target = current | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST;
                if self.is_click_through {
                    target |= windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT;
                }
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, target as isize);

                // Frame change notification to DWM
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            }
        }
    }

    /// Enable or disable transparent click-through (WS_EX_TRANSPARENT = 0x20)
    pub fn set_click_through(&mut self, transparent: bool) {
        if self.is_click_through == transparent {
            return;
        }
        self.is_click_through = transparent;

        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TRANSPARENT,
                };

                let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
                let new_style = if transparent {
                    current | WS_EX_TRANSPARENT
                } else {
                    current & !WS_EX_TRANSPARENT
                };
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style as isize);
            }
        }
    }

    pub fn is_click_through(&self) -> bool {
        self.is_click_through
    }

    /// Check if cursor is inside the interactive notch boundary
    /// If cursor is inside: disable click-through (allow mouse interaction).
    /// If cursor is outside: enable click-through (clicks pass through to apps underneath).
    pub fn update_proximity(&mut self, active_screen_rect: (f32, f32, f32, f32)) -> bool {
        let (rx, ry, rw, rh) = active_screen_rect;
        if let Some(cursor) = Self::get_cursor_pos() {
            let cx = cursor.x as f32;
            let cy = cursor.y as f32;

            // Extra proximity margin (8px padding) to prevent edge flickers
            let margin = 8.0;
            let inside = cx >= (rx - margin)
                && cx <= (rx + rw + margin)
                && cy >= (ry - margin)
                && cy <= (ry + rh + margin);

            // Inside -> click-through = false (we want clicks)
            // Outside -> click-through = true (pass through)
            self.set_click_through(!inside);
            inside
        } else {
            false
        }
    }

    /// Re-assert topmost Z-order over other desktop windows
    pub fn reassert_topmost(&self) {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
                };
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
            }
        }
    }

    /// Try to locate window handle by window title
    pub fn find_window_by_title(title: &str) -> Option<isize> {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;
            let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            let hwnd = FindWindowW(std::ptr::null(), wide.as_ptr());
            if !hwnd.is_null() {
                Some(hwnd as isize)
            } else {
                None
            }
        }

        #[cfg(not(windows))]
        {
            let _ = title;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_bounds_default() {
        let bounds = MonitorBounds::default();
        assert_eq!(bounds.width, 1920);
        assert_eq!(bounds.height, 1080);
        assert_eq!(bounds.dpi_scale, 1.0);
    }

    #[test]
    fn test_top_center_calculation() {
        let wm = WindowManager::new();
        let (x, y) = wm.calculate_top_center(400.0, 50.0);
        let expected_x = (wm.monitor_bounds().width as f32 - 400.0) / 2.0;
        assert!((x - expected_x).abs() < 1.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_click_through_toggle_logic() {
        let mut wm = WindowManager::new();
        assert!(!wm.is_click_through());

        wm.set_click_through(true);
        assert!(wm.is_click_through());

        wm.set_click_through(false);
        assert!(!wm.is_click_through());
    }
}
