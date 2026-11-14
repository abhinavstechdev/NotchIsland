use eframe::App;
use egui::{CentralPanel, Color32, Frame};
use crate::services::ServiceManager;
use crate::ui::{IslandMode, IslandUi};
use crate::windowing::WindowManager;

pub const WINDOW_TITLE: &str = "NotchIsland";
pub const WINDOW_WIDTH: f32 = 640.0;
pub const WINDOW_HEIGHT: f32 = 280.0;

pub struct NotchApp {
    pub ui: IslandUi,
    pub services: ServiceManager,
    pub window_manager: WindowManager,
    pub initialized: bool,
    pub frame_counter: u64,
}

impl Default for NotchApp {
    fn default() -> Self {
        Self::new()
    }
}

impl NotchApp {
    pub fn new() -> Self {
        let services = ServiceManager::new();

        Self {
            ui: IslandUi::new(),
            services,
            window_manager: WindowManager::new(),
            initialized: false,
            frame_counter: 0,
        }
    }

    /// Finds the OS window handle on Windows and applies transparent extended styles
    fn try_init_window(&mut self) {
        #[cfg(windows)]
        {
            if let Some(hwnd) = WindowManager::find_window_by_title(WINDOW_TITLE) {
                self.window_manager.set_hwnd(hwnd as isize);
                self.window_manager.apply_initial_styles();
                self.window_manager.reassert_topmost();
                self.initialized = true;
            }
        }
        #[cfg(not(windows))]
        {
            self.initialized = true;
        }
    }
}

impl App for NotchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_counter += 1;

        // One-time initialization of Win32 window styles
        if !self.initialized || self.frame_counter < 10 {
            self.try_init_window();
        }

        // Periodically re-assert topmost Z-order every ~300 frames (~5s at 60Hz)
        if self.frame_counter % 300 == 0 {
            self.window_manager.reassert_topmost();
        }

        // Update all background services (media player, telemetry, controls)
        self.services.update();

        // Automatic collapse into Slim HUD when full-screen DirectX/D3D game is detected
        if self.services.telemetry.is_gaming_fullscreen
            && self.ui.mode == IslandMode::Expanded
        {
            self.ui.set_mode(IslandMode::Slim);
        }

        // Render fully transparent root panel
        let panel_frame = Frame::none().fill(Color32::TRANSPARENT);
        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // Render the Dynamic Island squircle, views, and animations
            self.ui.render(ui, &mut self.services);

            // Compute active bounding box in screen coordinates for hit-testing
            let bounds = self.window_manager.monitor_bounds();
            let current_size = self.ui.morph.current_size();
            let screen_width = bounds.width as f32;
            let window_x = bounds.left as f32 + (screen_width - WINDOW_WIDTH) * 0.5;
            let window_y = bounds.top as f32;

            let island_center_x = window_x + WINDOW_WIDTH * 0.5;
            let island_top = if self.ui.notch_style == crate::ui::NotchStyle::BezelNotch {
                window_y
            } else {
                window_y + 8.0
            };

            let active_screen_rect = (
                island_center_x - current_size.x * 0.5,
                island_top,
                current_size.x,
                current_size.y,
            );

            // Dynamically toggle WS_EX_TRANSPARENT:
            // When cursor is inside: window captures input.
            // When cursor is outside: clicks pass through to underlying windows.
            self.window_manager.update_proximity(active_screen_rect);
        });

        // Request continuous repaint if springs are in motion or media is playing
        if !self.ui.morph.is_settled() || self.services.media.status == crate::services::PlaybackStatus::Playing {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notch_app_initialization() {
        let app = NotchApp::new();
        assert_eq!(app.ui.mode, IslandMode::Compact);
        assert!(!app.initialized);
        assert_eq!(app.frame_counter, 0);
        assert_eq!(WINDOW_WIDTH, 640.0);
        assert_eq!(WINDOW_HEIGHT, 280.0);
    }

    #[test]
    fn test_notch_app_mode_switch() {
        let mut app = NotchApp::new();
        app.ui.set_mode(IslandMode::Expanded);
        assert_eq!(app.ui.mode, IslandMode::Expanded);
        assert_eq!(app.ui.morph.width.target, 480.0);
        assert_eq!(app.ui.morph.height.target, 230.0);

        app.ui.set_mode(IslandMode::Slim);
        assert_eq!(app.ui.mode, IslandMode::Slim);
        assert_eq!(app.ui.morph.width.target, 380.0);
        assert_eq!(app.ui.morph.height.target, 28.0);
    }
}
