use std::time::Instant;
use sysinfo::System;

#[derive(Debug)]
pub struct TelemetryService {
    system: System,
    pub cpu_usage: f32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub ram_usage_pct: f32,
    pub fps: f32,
    pub is_gaming_fullscreen: bool,
    last_poll: Instant,
    fps_accum_frames: u32,
    fps_last_calc: Instant,
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryService {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let total_ram = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        let used_ram = sys.used_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        let ram_pct = if total_ram > 0.0 { (used_ram / total_ram) * 100.0 } else { 0.0 };

        Self {
            system: sys,
            cpu_usage: 0.0,
            ram_used_gb: used_ram,
            ram_total_gb: total_ram,
            ram_usage_pct: ram_pct,
            fps: 60.0,
            is_gaming_fullscreen: false,
            last_poll: Instant::now(),
            fps_accum_frames: 0,
            fps_last_calc: Instant::now(),
        }
    }

    /// Record a rendered frame for FPS computation
    pub fn register_frame(&mut self) {
        self.fps_accum_frames += 1;
        let elapsed = self.fps_last_calc.elapsed().as_secs_f32();
        if elapsed >= 0.5 {
            self.fps = (self.fps_accum_frames as f32 / elapsed).round();
            self.fps_accum_frames = 0;
            self.fps_last_calc = Instant::now();
        }
    }

    /// Periodic refresh of system metrics (every ~1s to conserve CPU)
    pub fn update(&mut self) {
        if self.last_poll.elapsed().as_millis() >= 800 {
            self.system.refresh_cpu_usage();
            self.system.refresh_memory();

            self.cpu_usage = self.system.global_cpu_usage();

            let total = self.system.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
            let used = self.system.used_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
            self.ram_total_gb = total;
            self.ram_used_gb = used;
            self.ram_usage_pct = if total > 0.0 { (used / total) * 100.0 } else { 0.0 };

            // Check if high GPU/CPU or foreground fullscreen game is active
            #[cfg(windows)]
            {
                self.check_fullscreen_gaming();
            }

            self.last_poll = Instant::now();
        }
    }

    #[cfg(windows)]
    fn check_fullscreen_gaming(&mut self) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowRect,
        };
        use windows_sys::Win32::Graphics::Gdi::{
            MonitorFromWindow, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        };
        use windows_sys::Win32::Foundation::RECT;

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                self.is_gaming_fullscreen = false;
                return;
            }

            let mut wnd_rect: RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut wnd_rect) == 0 {
                return;
            }

            let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            if hmon.is_null() {
                return;
            }

            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(hmon, &mut mi) != 0 {
                let mon_rect = mi.rcMonitor;
                // If foreground window covers the entire primary monitor and is not taskbar/desktop
                let matches_monitor = wnd_rect.left <= mon_rect.left
                    && wnd_rect.top <= mon_rect.top
                    && wnd_rect.right >= mon_rect.right
                    && wnd_rect.bottom >= mon_rect.bottom;

                self.is_gaming_fullscreen = matches_monitor && self.cpu_usage > 15.0;
            }
        }
    }

    #[cfg(not(windows))]
    fn check_fullscreen_gaming(&mut self) {
        self.is_gaming_fullscreen = false;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_service_initialization() {
        let mut telem = TelemetryService::new();
        assert!(telem.ram_total_gb > 0.0);
        assert!(telem.ram_used_gb > 0.0);
        assert!(telem.ram_usage_pct >= 0.0 && telem.ram_usage_pct <= 100.0);

        for _ in 0..10 {
            telem.register_frame();
        }
        telem.update();
    }
}
