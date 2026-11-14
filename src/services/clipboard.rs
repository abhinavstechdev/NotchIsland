use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ClipboardSnippet {
    pub id: usize,
    pub text: String,
    pub is_pinned: bool,
    pub time_label: String,
}

impl ClipboardSnippet {
    pub fn preview(&self, max_len: usize) -> String {
        let clean = self.text.replace('\n', " ").replace('\r', "");
        if clean.chars().count() <= max_len {
            clean
        } else {
            let truncated: String = clean.chars().take(max_len).collect();
            format!("{}...", truncated)
        }
    }
}

pub struct ClipboardService {
    pub snippets: Vec<ClipboardSnippet>,
    next_id: usize,
    last_captured: String,
    last_poll: Instant,
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardService {
    pub fn new() -> Self {
        let mut initial_snippets = Vec::new();
        initial_snippets.push(ClipboardSnippet {
            id: 1,
            text: "git clone https://github.com/notchpixel/dynamic-island.git".to_string(),
            is_pinned: true,
            time_label: "Pinned".to_string(),
        });
        initial_snippets.push(ClipboardSnippet {
            id: 2,
            text: "npm run build --prefix assets/deck".to_string(),
            is_pinned: false,
            time_label: "Just now".to_string(),
        });
        initial_snippets.push(ClipboardSnippet {
            id: 3,
            text: "#5ee9ff - NotchPixel Signature Cyan Glow Accent".to_string(),
            is_pinned: false,
            time_label: "5m ago".to_string(),
        });

        Self {
            snippets: initial_snippets,
            next_id: 4,
            last_captured: String::new(),
            last_poll: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        if self.last_poll.elapsed().as_millis() >= 500 {
            if let Some(text) = self.read_system_clipboard() {
                if !text.is_empty() && text != self.last_captured {
                    self.last_captured = text.clone();
                    self.add_snippet(&text);
                }
            }
            self.last_poll = Instant::now();
        }
    }

    pub fn add_snippet(&mut self, text: &str) {
        if let Some(first) = self.snippets.first() {
            if first.text == text {
                return;
            }
        }

        let snippet = ClipboardSnippet {
            id: self.next_id,
            text: text.to_string(),
            is_pinned: false,
            time_label: "Just now".to_string(),
        };
        self.next_id += 1;

        let insert_idx = self.snippets.iter().position(|s| !s.is_pinned).unwrap_or(0);
        self.snippets.insert(insert_idx, snippet);

        if self.snippets.len() > 25 {
            if let Some(last_unpinned) = self.snippets.iter().rposition(|s| !s.is_pinned) {
                self.snippets.remove(last_unpinned);
            }
        }
    }

    pub fn toggle_pin(&mut self, id: usize) {
        if let Some(s) = self.snippets.iter_mut().find(|s| s.id == id) {
            s.is_pinned = !s.is_pinned;
        }
        self.sort_snippets();
    }

    pub fn remove(&mut self, id: usize) {
        self.snippets.retain(|s| s.id != id);
    }

    pub fn clear_unpinned(&mut self) {
        self.snippets.retain(|s| s.is_pinned);
    }

    pub fn copy_to_clipboard(&self, id: usize) {
        if let Some(s) = self.snippets.iter().find(|s| s.id == id) {
            self.write_system_clipboard(&s.text);
        }
    }

    fn sort_snippets(&mut self) {
        self.snippets.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.id.cmp(&a.id),
        });
    }

    #[cfg(windows)]
    fn read_system_clipboard(&self) -> Option<String> {
        use win32::*;

        unsafe {
            if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
                return None;
            }
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return None;
            }

            let handle = GetClipboardData(CF_UNICODETEXT);
            if handle.is_null() {
                CloseClipboard();
                return None;
            }

            let ptr = GlobalLock(handle) as *const u16;
            if ptr.is_null() {
                CloseClipboard();
                return None;
            }

            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let s = String::from_utf16_lossy(slice);

            GlobalUnlock(handle);
            CloseClipboard();

            Some(s)
        }
    }

    #[cfg(not(windows))]
    fn read_system_clipboard(&self) -> Option<String> {
        None
    }

    #[cfg(windows)]
    fn write_system_clipboard(&self, text: &str) {
        use win32::*;

        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return;
            }
            EmptyClipboard();

            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide.len() * std::mem::size_of::<u16>();

            let h_mem = GlobalAlloc(GMEM_MOVEABLE, size);
            if !h_mem.is_null() {
                let p_mem = GlobalLock(h_mem) as *mut u16;
                if !p_mem.is_null() {
                    std::ptr::copy_nonoverlapping(wide.as_ptr(), p_mem, wide.len());
                    GlobalUnlock(h_mem);
                    SetClipboardData(CF_UNICODETEXT, h_mem);
                }
            }

            CloseClipboard();
        }
    }

    #[cfg(not(windows))]
    fn write_system_clipboard(&self, _text: &str) {}
}

#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;

    pub const CF_UNICODETEXT: u32 = 13;
    pub const GMEM_MOVEABLE: u32 = 0x0002;

    #[link(name = "user32")]
    extern "system" {
        pub fn OpenClipboard(hwnd: *mut c_void) -> i32;
        pub fn CloseClipboard() -> i32;
        pub fn EmptyClipboard() -> i32;
        pub fn IsClipboardFormatAvailable(format: u32) -> i32;
        pub fn GetClipboardData(format: u32) -> *mut c_void;
        pub fn SetClipboardData(format: u32, hmem: *mut c_void) -> *mut c_void;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GlobalAlloc(uflags: u32, dwbytes: usize) -> *mut c_void;
        pub fn GlobalLock(hmem: *mut c_void) -> *mut c_void;
        pub fn GlobalUnlock(hmem: *mut c_void) -> i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_service() {
        let mut service = ClipboardService::new();
        assert!(!service.snippets.is_empty());

        let initial_count = service.snippets.len();
        service.add_snippet("Cargo test snippet");
        assert_eq!(service.snippets.len(), initial_count + 1);

        let added_item = service.snippets.iter().find(|s| s.text == "Cargo test snippet").unwrap().clone();
        assert!(!added_item.is_pinned);

        service.toggle_pin(added_item.id);
        let pinned_item = service.snippets.iter().find(|s| s.id == added_item.id).unwrap();
        assert!(pinned_item.is_pinned);

        service.remove(added_item.id);
        assert_eq!(service.snippets.len(), initial_count);
    }
}
