#[derive(Debug, Clone)]
pub struct NotesService {
    pub content: String,
    pub is_dirty: bool,
}

impl Default for NotesService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesService {
    pub fn new() -> Self {
        Self {
            content: "Welcome to NotchIsland Scratchpad 📝\n\n- Quick reminders\n- Code snippets & terminal commands\n- Ideas saved automatically on your notch island".to_string(),
            is_dirty: false,
        }
    }

    pub fn set_content(&mut self, text: String) {
        if self.content != text {
            self.content = text;
            self.is_dirty = true;
        }
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.is_dirty = true;
    }

    pub fn character_count(&self) -> usize {
        self.content.chars().count()
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn line_count(&self) -> usize {
        if self.content.is_empty() {
            0
        } else {
            self.content.lines().count()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_service() {
        let mut notes = NotesService::new();
        assert!(!notes.content.is_empty());

        notes.set_content("Hello world\nSecond line".to_string());
        assert_eq!(notes.word_count(), 4);
        assert_eq!(notes.line_count(), 2);

        notes.clear();
        assert_eq!(notes.character_count(), 0);
        assert_eq!(notes.word_count(), 0);
    }
}
