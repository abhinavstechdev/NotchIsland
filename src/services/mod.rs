pub mod media;
pub mod telemetry;
pub mod file_shelf;
pub mod system_controls;

pub use media::{MediaService, MediaTrack, PlaybackStatus};
pub use telemetry::TelemetryService;
pub use file_shelf::{FileShelfService, ShelfItem};
pub use system_controls::SystemControlsService;

pub struct ServiceManager {
    pub media: MediaService,
    pub telemetry: TelemetryService,
    pub file_shelf: FileShelfService,
    pub controls: SystemControlsService,
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            media: MediaService::new(),
            telemetry: TelemetryService::new(),
            file_shelf: FileShelfService::new(),
            controls: SystemControlsService::new(),
        }
    }

    /// Primary tick called on every UI frame to advance media progress, refresh hardware telemetry, and track frame rate
    pub fn update(&mut self) {
        self.media.update();
        self.telemetry.update();
        self.telemetry.register_frame();
    }
}
