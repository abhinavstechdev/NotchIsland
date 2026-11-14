use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ToolsService {
    pub is_recording: bool,
    pub recording_seconds: u32,
    pub last_screenshot_status: Option<String>,
    pub custom_tool_slots: Vec<String>,
    record_timer: Instant,
}

impl Default for ToolsService {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolsService {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            recording_seconds: 0,
            last_screenshot_status: None,
            custom_tool_slots: vec!["Terminal".to_string(), "Calculator".to_string()],
            record_timer: Instant::now(),
        }
    }

    pub fn take_screenshot(&mut self) {
        #[cfg(windows)]
        {
            // Trigger Windows native screen capture via keybd_event PrtScn or snipping tool
            let _ = std::process::Command::new("powershell")
                .args(&["-Command", "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{PRTSC}')"])
                .spawn();
        }
        self.last_screenshot_status = Some("Screenshot captured to clipboard".to_string());
    }

    pub fn launch_snipping_tool(&mut self) {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("explorer.exe")
                .arg("ms-screenclip:")
                .spawn();
        }
    }

    pub fn toggle_recording(&mut self) {
        self.is_recording = !self.is_recording;
        if self.is_recording {
            self.recording_seconds = 0;
            self.record_timer = Instant::now();
        }
    }

    pub fn update(&mut self) {
        if self.is_recording && self.record_timer.elapsed().as_secs() >= 1 {
            self.recording_seconds += self.record_timer.elapsed().as_secs() as u32;
            self.record_timer = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_service() {
        let mut tools = ToolsService::new();
        assert!(!tools.is_recording);

        tools.toggle_recording();
        assert!(tools.is_recording);

        tools.toggle_recording();
        assert!(!tools.is_recording);
    }
}
