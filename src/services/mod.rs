pub mod media;
pub mod telemetry;
pub mod file_shelf;
pub mod system_controls;
pub mod timer;
pub mod clipboard;
pub mod notes;
pub mod deck_server;
pub mod indicators;
pub mod notifications;
pub mod calls;
pub mod websites;
pub mod tools;
pub mod settings;

pub use media::{MediaService, MediaTrack, PlaybackStatus};
pub use telemetry::TelemetryService;
pub use file_shelf::{FileShelfService, ShelfItem};
pub use system_controls::SystemControlsService;
pub use timer::{PomodoroTimer, TimerPhase};
pub use clipboard::{ClipboardService, ClipboardSnippet};
pub use notes::NotesService;
pub use deck_server::DeckServerService;
pub use indicators::SystemIndicators;
pub use notifications::{NotificationsService, NotificationItem};
pub use calls::{CallsService, AudioDeviceInfo};
pub use websites::{WebsitesService, PinnedSite};
pub use tools::ToolsService;
pub use settings::{SettingsService, SettingsCategory, AppSettings};

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
    pub notifications: NotificationsService,
    pub calls: CallsService,
    pub websites: WebsitesService,
    pub tools: ToolsService,
    pub settings: SettingsService,
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
            notifications: NotificationsService::new(),
            calls: CallsService::new(),
            websites: WebsitesService::new(),
            tools: ToolsService::new(),
            settings: SettingsService::new(),
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
        self.calls.update();
        self.tools.update();

        // Synchronize state with background phone deck server and dispatch remote commands
        self.deck_server.sync_and_poll_commands(
            &mut self.media,
            &self.telemetry,
            &mut self.controls,
        );
    }
}
