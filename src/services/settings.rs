use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Accessibility,
    Appearance,
    Island,
    Taskbar,
    Panels,
    Widgets,
    Calls,
    Streaming,
    Gaming,
    AiAgents,
    Tools,
    Keys,
    About,
}

impl SettingsCategory {
    pub fn all() -> &'static [SettingsCategory] {
        &[
            SettingsCategory::General,
            SettingsCategory::Accessibility,
            SettingsCategory::Appearance,
            SettingsCategory::Island,
            SettingsCategory::Taskbar,
            SettingsCategory::Panels,
            SettingsCategory::Widgets,
            SettingsCategory::Calls,
            SettingsCategory::Streaming,
            SettingsCategory::Gaming,
            SettingsCategory::AiAgents,
            SettingsCategory::Tools,
            SettingsCategory::Keys,
            SettingsCategory::About,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsCategory::General => "General",
            SettingsCategory::Accessibility => "Accessibility",
            SettingsCategory::Appearance => "Appearance",
            SettingsCategory::Island => "Island",
            SettingsCategory::Taskbar => "Taskbar",
            SettingsCategory::Panels => "Panels",
            SettingsCategory::Widgets => "Widgets",
            SettingsCategory::Calls => "Calls",
            SettingsCategory::Streaming => "Streaming",
            SettingsCategory::Gaming => "Gaming",
            SettingsCategory::AiAgents => "AI agents",
            SettingsCategory::Tools => "Tools",
            SettingsCategory::Keys => "Keys",
            SettingsCategory::About => "About",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingsCategory::General => "⚙",
            SettingsCategory::Accessibility => "♿",
            SettingsCategory::Appearance => "🎨",
            SettingsCategory::Island => "🏝",
            SettingsCategory::Taskbar => "▬",
            SettingsCategory::Panels => "▦",
            SettingsCategory::Widgets => "❖",
            SettingsCategory::Calls => "🎧",
            SettingsCategory::Streaming => "📡",
            SettingsCategory::Gaming => "🎮",
            SettingsCategory::AiAgents => "✨",
            SettingsCategory::Tools => "🛠",
            SettingsCategory::Keys => "⌨",
            SettingsCategory::About => "ℹ",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub run_at_startup: bool,
    pub slim_under_maximized: bool,
    pub open_behavior: String, // "Hover" or "Click"
    pub theme: String, // "Pure Black", "Acrylic", "Mica"
    pub accent_cyan: bool,
    pub sound_effects: bool,
    pub pet_enabled: bool,
    pub pet_species: String, // "Cat" or "Bunny"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "English (US)".to_string(),
            run_at_startup: true,
            slim_under_maximized: true,
            open_behavior: "Hover".to_string(),
            theme: "Pure Black".to_string(),
            accent_cyan: true,
            sound_effects: true,
            pet_enabled: true,
            pet_species: "Cat".to_string(),
        }
    }
}

pub struct SettingsService {
    pub active_category: SettingsCategory,
    pub settings: AppSettings,
    pub is_window_open: bool,
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsService {
    pub fn new() -> Self {
        let loaded = Self::load_from_disk().unwrap_or_default();
        Self {
            active_category: SettingsCategory::General,
            settings: loaded,
            is_window_open: false,
        }
    }

    pub fn toggle_window(&mut self) {
        self.is_window_open = !self.is_window_open;
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
            let _ = fs::write("notch_settings.json", json);
        }
    }

    fn load_from_disk() -> Option<AppSettings> {
        let content = fs::read_to_string("notch_settings.json").ok()?;
        serde_json::from_str(&content).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_service() {
        let mut s = SettingsService::new();
        assert_eq!(s.active_category, SettingsCategory::General);
        assert!(!s.is_window_open);

        s.toggle_window();
        assert!(s.is_window_open);

        s.settings.language = "Español".to_string();
        assert_eq!(s.settings.language, "Español");
    }
}
