use std::time::Instant;

#[derive(Debug, Clone)]
pub struct SystemIndicators {
    pub caps_lock: bool,
    pub num_lock: bool,
    pub mic_active: bool,
    pub camera_active: bool,
    pub caps_changed_time: Option<Instant>,
    last_caps_state: bool,
    last_poll: Instant,
}

impl Default for SystemIndicators {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemIndicators {
    pub fn new() -> Self {
        let (caps, num) = query_lock_keys();
        Self {
            caps_lock: caps,
            num_lock: num,
            mic_active: false,
            camera_active: false,
            caps_changed_time: None,
            last_caps_state: caps,
            last_poll: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        if self.last_poll.elapsed().as_millis() >= 100 {
            let (caps, num) = query_lock_keys();
            if caps != self.last_caps_state {
                self.caps_lock = caps;
                self.last_caps_state = caps;
                self.caps_changed_time = Some(Instant::now());
            }
            self.num_lock = num;
            self.last_poll = Instant::now();
        }
    }

    pub fn should_show_caps_pill(&self) -> bool {
        if let Some(t) = self.caps_changed_time {
            t.elapsed().as_secs_f32() < 1.8
        } else {
            false
        }
    }
}

#[cfg(windows)]
fn query_lock_keys() -> (bool, bool) {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState;

    const VK_CAPITAL: i32 = 0x14;
    const VK_NUMLOCK: i32 = 0x90;

    unsafe {
        let caps = (GetKeyState(VK_CAPITAL) & 0x0001) != 0;
        let num = (GetKeyState(VK_NUMLOCK) & 0x0001) != 0;
        (caps, num)
    }
}

#[cfg(not(windows))]
fn query_lock_keys() -> (bool, bool) {
    (false, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_indicators() {
        let mut ind = SystemIndicators::new();
        ind.update();
        assert!(!ind.should_show_caps_pill());

        ind.caps_changed_time = Some(Instant::now());
        assert!(ind.should_show_caps_pill());
    }
}
