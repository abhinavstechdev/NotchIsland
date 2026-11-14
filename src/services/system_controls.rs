#[derive(Debug, Clone)]
pub struct SystemControlsService {
    pub volume: f32, // 0.0 to 1.0
    pub is_muted: bool,
    pub is_mic_muted: bool,
    pub brightness: f32, // 0.0 to 1.0
}

impl Default for SystemControlsService {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemControlsService {
    pub fn new() -> Self {
        Self {
            volume: 0.65,
            is_muted: false,
            is_mic_muted: false,
            brightness: 0.80,
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if self.volume > 0.0 && self.is_muted {
            self.is_muted = false;
        }
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    pub fn toggle_mic_mute(&mut self) {
        self.is_mic_muted = !self.is_mic_muted;
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness.clamp(0.0, 1.0);
    }

    pub fn volume_percent(&self) -> u32 {
        if self.is_muted {
            0
        } else {
            (self.volume * 100.0).round() as u32
        }
    }

    pub fn brightness_percent(&self) -> u32 {
        (self.brightness * 100.0).round() as u32
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_controls_clamping_and_toggles() {
        let mut ctrl = SystemControlsService::new();
        assert_eq!(ctrl.volume_percent(), 65);
        assert!(!ctrl.is_muted);

        ctrl.set_volume(1.5);
        assert_eq!(ctrl.volume, 1.0);
        assert_eq!(ctrl.volume_percent(), 100);

        ctrl.toggle_mute();
        assert!(ctrl.is_muted);
        assert_eq!(ctrl.volume_percent(), 0);

        ctrl.toggle_mic_mute();
        assert!(ctrl.is_mic_muted);

        ctrl.set_brightness(0.42);
        assert_eq!(ctrl.brightness_percent(), 42);
    }
}
