pub mod media;
pub mod telemetry;
pub mod file_shelf;
pub mod system_controls;
pub mod timer;
pub mod clipboard;
pub mod notes;
pub mod deck_server;
pub mod indicators;

pub use media::{MediaService, MediaTrack, PlaybackStatus};
pub use telemetry::TelemetryService;
pub use file_shelf::{FileShelfService, ShelfItem};
pub use system_controls::SystemControlsService;
pub use timer::{PomodoroTimer, TimerPhase};
pub use clipboard::{ClipboardService, ClipboardSnippet};
pub use notes::NotesService;
pub use deck_server::DeckServerService;
pub use indicators::SystemIndicators;

pub struct ServiceManager {
    pub media: MediaService,
    pub telemetry: TelemetryService,
    pub controls: SystemControlsService,
    pub file_shelf: FileShelfService,
    pub timer: PomodoroTimer,
    pub clipboard: ClipboardService,
    pub notes: NotesService,
    pub indicators: SystemIndicators,
    pub deck_server: DeckServerService,
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
            controls: SystemControlsService::new(),
            file_shelf: FileShelfService::new(),
            timer: PomodoroTimer::new(),
            clipboard: ClipboardService::new(),
            notes: NotesService::new(),
            indicators: SystemIndicators::new(),
            deck_server: DeckServerService::new(),
        }
    }

    /// Primary tick called on every UI frame to advance media progress, refresh hardware telemetry, track frame rate, and sync mobile deck
    pub fn update(&mut self) {
        self.media.update();
        self.telemetry.update();
        self.telemetry.register_frame();
        self.timer.update();
        self.clipboard.update();
        self.indicators.update();

        // Synchronize state with background phone deck server and dispatch remote commands
        self.deck_server.sync_and_poll_commands(
            &mut self.media,
            &self.telemetry,
            &mut self.controls,
        );
    }
}
