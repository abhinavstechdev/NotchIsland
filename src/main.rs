#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notch_island::app::{NotchApp, WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use notch_island::windowing::WindowManager;

fn main() -> eframe::Result<()> {
    // Query primary monitor to calculate top-center screen position
    let win_mgr = WindowManager::new();
    let monitor = win_mgr.monitor_bounds();
    let (window_x, window_y) = win_mgr.calculate_top_center(WINDOW_WIDTH, WINDOW_HEIGHT);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_position([window_x as f32, window_y as f32])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_active(false)
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        native_options,
        Box::new(|_cc| Ok(Box::new(NotchApp::new()))),
    )
}
