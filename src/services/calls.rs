use std::time::Instant;

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct CallsService {
    pub is_mic_muted: bool,
    pub mic_level: f32, // 0.0 .. 1.0 for test visualizer
    pub is_testing_mic: bool,
    pub is_testing_speaker: bool,
    pub camera_active: bool,
    pub speakers: Vec<AudioDeviceInfo>,
    pub selected_speaker: usize,
    pub microphones: Vec<AudioDeviceInfo>,
    pub selected_microphone: usize,
    last_update: Instant,
}

impl Default for CallsService {
    fn default() -> Self {
        Self::new()
    }
}

impl CallsService {
    pub fn new() -> Self {
        let speakers = vec![
            AudioDeviceInfo {
                id: "spk_1".to_string(),
                name: "Realtek High Definition Audio (Speakers)".to_string(),
                is_default: true,
            },
            AudioDeviceInfo {
                id: "spk_2".to_string(),
                name: "Headphones (Wireless Bluetooth)".to_string(),
                is_default: false,
            },
        ];

        let microphones = vec![
            AudioDeviceInfo {
                id: "mic_1".to_string(),
                name: "Microphone Array (Intel Smart Sound Technology)".to_string(),
                is_default: true,
            },
            AudioDeviceInfo {
                id: "mic_2".to_string(),
                name: "External USB Condenser Mic".to_string(),
                is_default: false,
            },
        ];

        Self {
            is_mic_muted: false,
            mic_level: 0.15,
            is_testing_mic: false,
            is_testing_speaker: false,
            camera_active: false,
            speakers,
            selected_speaker: 0,
            microphones,
            selected_microphone: 0,
            last_update: Instant::now(),
        }
    }

    pub fn toggle_mic(&mut self) {
        self.is_mic_muted = !self.is_mic_muted;
    }

    pub fn toggle_test_mic(&mut self) {
        self.is_testing_mic = !self.is_testing_mic;
    }

    pub fn test_speaker(&mut self) {
        self.is_testing_speaker = true;
        // On Windows, play system sound asterisk/beep
        #[cfg(windows)]
        {
            extern "system" {
                fn MessageBeep(utype: u32) -> i32;
            }
            unsafe {
                MessageBeep(0x00000040); // MB_ICONASTERISK
            }
        }
    }

    pub fn check_camera(&mut self) {
        self.camera_active = !self.camera_active;
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let _dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        if self.is_testing_mic && !self.is_mic_muted {
            // Simulated live mic input animation
            let t = now.elapsed().as_secs_f32() * 6.0;
            self.mic_level = ((t.sin() * 0.4 + 0.5) + (t * 2.3).cos() * 0.2).clamp(0.05, 0.95);
        } else if self.is_mic_muted {
            self.mic_level = 0.0;
        } else {
            self.mic_level = 0.05;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calls_service() {
        let mut calls = CallsService::new();
        assert!(!calls.is_mic_muted);

        calls.toggle_mic();
        assert!(calls.is_mic_muted);

        calls.update();
        assert_eq!(calls.mic_level, 0.0);

        calls.toggle_mic();
        calls.toggle_test_mic();
        calls.update();
        assert!(calls.mic_level > 0.0);
    }
}
