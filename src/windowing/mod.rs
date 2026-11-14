//! Win32 Window Management for NotchIsland
//! Handles top-center screen positioning, per-monitor DPI detection,
//! topmost Z-order enforcement, dynamic WS_EX_TRANSPARENT click-through toggling,
//! Notch vs Floating Island geometry modes, and CapsLock / NumLock popup alerts.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IslandGeometryMode {
    /// Top-anchored bezel notch: upper boundary attached flush to physical bezel (top = 0)
    BezelNotch,
    /// Floating dynamic island detached with a top screen margin (default 12.0)
    FloatingIsland { top_margin: f32 },
}

impl Default for IslandGeometryMode {
    fn default() -> Self {
        Self::FloatingIsland { top_margin: 12.0 }
    }
}

impl IslandGeometryMode {
    pub fn is_bezel(&self) -> bool {
        matches!(self, Self::BezelNotch)
    }

    pub fn top_margin(&self) -> f32 {
        match self {
            Self::BezelNotch => 0.0,
            Self::FloatingIsland { top_margin } => *top_margin,
        }
    }
}

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

/// Win32 Lock Key Status (CapsLock, NumLock, ScrollLock)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockKeyState {
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
}

impl Default for LockKeyState {
    fn default() -> Self {
        Self {
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockKeyChange {
    CapsLock(bool),
    NumLock(bool),
    ScrollLock(bool),
}

/// Monitors keyboard lock keys (CapsLock, NumLock) and provides timed popup notifications
pub struct LockKeyMonitor {
    last_state: LockKeyState,
    active_popup: Option<(LockKeyChange, Instant)>,
    popup_duration: Duration,
}

impl Default for LockKeyMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl LockKeyMonitor {
    pub fn new() -> Self {
        let current = Self::query_system_lock_keys();
        Self {
            last_state: current,
            active_popup: None,
            popup_duration: Duration::from_millis(2200),
        }
    }

    /// Query current hardware lock key states
    pub fn query_system_lock_keys() -> LockKeyState {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL, VK_NUMLOCK, VK_SCROLL};
            let caps = (GetKeyState(VK_CAPITAL as i32) & 1) != 0;
            let num = (GetKeyState(VK_NUMLOCK as i32) & 1) != 0;
            let scroll = (GetKeyState(VK_SCROLL as i32) & 1) != 0;
            LockKeyState {
                caps_lock: caps,
                num_lock: num,
                scroll_lock: scroll,
            }
        }

        #[cfg(not(windows))]
        LockKeyState::default()
    }

    /// Poll for changes in key state. Returns change event if a key was toggled.
    pub fn update(&mut self) -> Option<LockKeyChange> {
        let current = Self::query_system_lock_keys();
        let mut change = None;

        if current.caps_lock != self.last_state.caps_lock {
            change = Some(LockKeyChange::CapsLock(current.caps_lock));
        } else if current.num_lock != self.last_state.num_lock {
            change = Some(LockKeyChange::NumLock(current.num_lock));
        } else if current.scroll_lock != self.last_state.scroll_lock {
            change = Some(LockKeyChange::ScrollLock(current.scroll_lock));
        }

        if let Some(chg) = change {
            self.last_state = current;
            self.active_popup = Some((chg, Instant::now()));
        }

        // Clean up expired popup
        if let Some((_, start)) = self.active_popup {
            if start.elapsed() >= self.popup_duration {
                self.active_popup = None;
            }
        }

        change
    }

    /// Get current active popup banner text and state, if any is visible
    pub fn active_popup_banner(&self) -> Option<(&'static str, bool)> {
        self.active_popup.map(|(chg, _)| match chg {
            LockKeyChange::CapsLock(on) => ("CAPS LOCK", on),
            LockKeyChange::NumLock(on) => ("NUM LOCK", on),
            LockKeyChange::ScrollLock(on) => ("SCROLL LOCK", on),
        })
    }

    pub fn last_state(&self) -> LockKeyState {
        self.last_state
    }
}

#[repr(C)]
struct MARGINS {
    cx_left_width: i32,
    cx_right_width: i32,
    cy_top_height: i32,
    cy_bottom_height: i32,
}

#[cfg(windows)]
#[link(name = "dwmapi")]
extern "system" {
    fn DwmExtendFrameIntoClientArea(
        hwnd: windows_sys::Win32::Foundation::HWND,
        p_mar_inset: *const MARGINS,
    ) -> i32;
}

pub struct WindowManager {
    #[cfg(windows)]
    hwnd: Option<windows_sys::Win32::Foundation::HWND>,
    is_click_through: bool,
    geometry_mode: IslandGeometryMode,
    monitor_bounds: MonitorBounds,
    last_cursor_inside: Instant,
    hover_exit_cooldown: Duration,
    lock_keys: LockKeyMonitor,
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
            geometry_mode: IslandGeometryMode::default(),
            monitor_bounds,
            last_cursor_inside: Instant::now(),
            hover_exit_cooldown: Duration::from_millis(180),
            lock_keys: LockKeyMonitor::new(),
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

    /// Geometry mode getter & setter
    pub fn geometry_mode(&self) -> IslandGeometryMode {
        self.geometry_mode
    }

    pub fn set_geometry_mode(&mut self, mode: IslandGeometryMode) {
        self.geometry_mode = mode;
    }

    /// Access the lock key monitor
    pub fn lock_keys(&self) -> &LockKeyMonitor {
        &self.lock_keys
    }

    pub fn lock_keys_mut(&mut self) -> &mut LockKeyMonitor {
        &mut self.lock_keys
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

    /// Calculate the top-center screen coordinates for a given window size based on active GeometryMode
    pub fn calculate_position(&self, window_width: f32, _window_height: f32) -> (f32, f32) {
        let x = self.monitor_bounds.left as f32 + (self.monitor_bounds.width as f32 - window_width) / 2.0;
        let y = self.monitor_bounds.top as f32 + self.geometry_mode.top_margin();
        (x, y)
    }

    /// Legacy helper for top center
    pub fn calculate_top_center(&self, window_width: f32, window_height: f32) -> (f32, f32) {
        self.calculate_position(window_width, window_height)
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

                // Extend frame into client area for 100% glass transparency
                let margins = MARGINS {
                    cx_left_width: -1,
                    cx_right_width: -1,
                    cy_top_height: -1,
                    cy_bottom_height: -1,
                };
                DwmExtendFrameIntoClientArea(hwnd, &margins);
            }
        }
    }

    /// Make the entire window client area 100% transparent using DWM frame extension
    pub fn make_window_transparent(&self) {
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd {
            unsafe {
                let margins = MARGINS {
                    cx_left_width: -1,
                    cx_right_width: -1,
                    cy_top_height: -1,
                    cy_bottom_height: -1,
                };
                DwmExtendFrameIntoClientArea(hwnd, &margins);
            }
        }
    }

    /// Resize and reposition the window to match the current mode dimensions
    pub fn resize_and_reposition(&self, width: f32, height: f32) {
        let (x, y) = self.calculate_position(width, height);
        #[cfg(windows)]
        if let Some(hwnd) = self.hwnd {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE};
                let w = (width * self.monitor_bounds.dpi_scale).round() as i32;
                let h = (height * self.monitor_bounds.dpi_scale).round() as i32;
                let px = (x * self.monitor_bounds.dpi_scale).round() as i32;
                let py = (y * self.monitor_bounds.dpi_scale).round() as i32;
                SetWindowPos(hwnd, HWND_TOPMOST, px, py, w, h, SWP_NOACTIVATE);
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
    pub fn update_proximity(&mut self, active_screen_rect: (f32, f32, f32, f32)) -> bool {
        self.update_proximity_multi(&[active_screen_rect])
    }

    /// Check if cursor is inside ANY of multiple interactive screen regions (notch body, pet, popup banners)
    /// Incorporates a subtle exit hysteresis cooldown to avoid input flickering near edges.
    pub fn update_proximity_multi(&mut self, active_screen_rects: &[(f32, f32, f32, f32)]) -> bool {
        let margin = 10.0;
        let mut physically_inside = false;

        if let Some(cursor) = Self::get_cursor_pos() {
            let cx = cursor.x as f32;
            let cy = cursor.y as f32;

            for &(rx, ry, rw, rh) in active_screen_rects {
                if cx >= (rx - margin)
                    && cx <= (rx + rw + margin)
                    && cy >= (ry - margin)
                    && cy <= (ry + rh + margin)
                {
                    physically_inside = true;
                    break;
                }
            }
        }

        if physically_inside {
            self.last_cursor_inside = Instant::now();
            self.set_click_through(false); // Enable mouse interaction
            true
        } else {
            // Check debounce cooldown before locking window back into click-through mode
            let within_cooldown = self.last_cursor_inside.elapsed() < self.hover_exit_cooldown;
            if !within_cooldown {
                self.set_click_through(true); // Pass through
            }
            within_cooldown
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
    fn test_geometry_mode_positioning() {
        let mut wm = WindowManager::new();

        // Floating island with 12px margin
        wm.set_geometry_mode(IslandGeometryMode::FloatingIsland { top_margin: 12.0 });
        let (x, y) = wm.calculate_position(400.0, 50.0);
        let expected_x = (wm.monitor_bounds().width as f32 - 400.0) / 2.0;
        assert!((x - expected_x).abs() < 1.0);
        assert_eq!(y, 12.0);

        // Bezel notch (top = 0)
        wm.set_geometry_mode(IslandGeometryMode::BezelNotch);
        let (_, y_notch) = wm.calculate_position(400.0, 50.0);
        assert_eq!(y_notch, 0.0);
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

    #[test]
    fn test_lock_key_state_init() {
        let monitor = LockKeyMonitor::new();
        let state = monitor.last_state();
        // Just verify it queries without panicking
        let _ = state.caps_lock;
        let _ = state.num_lock;
    }
}
