use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct MediaTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f32,
    pub current_pos_secs: f32,
    pub accent_color: [u8; 3], // RGB tint for UI cover
}

impl MediaTrack {
    pub fn new(title: &str, artist: &str, album: &str, duration_secs: f32, accent: [u8; 3]) -> Self {
        Self {
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            duration_secs,
            current_pos_secs: 0.0,
            accent_color: accent,
        }
    }

    pub fn formatted_current_time(&self) -> String {
        format_time(self.current_pos_secs)
    }

    pub fn formatted_duration(&self) -> String {
        format_time(self.duration_secs)
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.duration_secs <= 0.0 {
            0.0
        } else {
            (self.current_pos_secs / self.duration_secs).clamp(0.0, 1.0)
        }
    }
}

fn format_time(secs: f32) -> String {
    let s = secs.max(0.0) as u32;
    let m = s / 60;
    let rem_s = s % 60;
    format!("{:02}:{:02}", m, rem_s)
}

#[derive(Debug, Clone)]
pub struct MediaService {
    pub playlist: Vec<MediaTrack>,
    pub current_index: usize,
    pub status: PlaybackStatus,
    last_tick: Instant,
}

impl Default for MediaService {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaService {
    pub fn new() -> Self {
        let tracks = vec![
            MediaTrack {
                title: "Resonance".to_string(),
                artist: "HOME".to_string(),
                album: "Odyssey".to_string(),
                duration_secs: 212.0,
                current_pos_secs: 42.0,
                accent_color: [255, 107, 107], // Coral
            },
            MediaTrack {
                title: "Midnight City".to_string(),
                artist: "M83".to_string(),
                album: "Hurry Up, We're Dreaming".to_string(),
                duration_secs: 243.0,
                current_pos_secs: 15.0,
                accent_color: [78, 205, 196], // Turquoise
            },
            MediaTrack {
                title: "After Dark".to_string(),
                artist: "Mr.Kitty".to_string(),
                album: "Time".to_string(),
                duration_secs: 259.0,
                current_pos_secs: 88.0,
                accent_color: [168, 85, 247], // Purple
            },
            MediaTrack {
                title: "Starboy".to_string(),
                artist: "The Weeknd, Daft Punk".to_string(),
                album: "Starboy".to_string(),
                duration_secs: 230.0,
                current_pos_secs: 110.0,
                accent_color: [239, 68, 68], // Red
            },
        ];

        Self {
            playlist: tracks,
            current_index: 0,
            status: PlaybackStatus::Playing,
            last_tick: Instant::now(),
        }
    }

    pub fn current_track(&self) -> &MediaTrack {
        &self.playlist[self.current_index]
    }

    pub fn current_track_mut(&mut self) -> &mut MediaTrack {
        &mut self.playlist[self.current_index]
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;

        if self.status == PlaybackStatus::Playing && !self.playlist.is_empty() {
            let track = &mut self.playlist[self.current_index];
            track.current_pos_secs += dt;
            if track.current_pos_secs >= track.duration_secs {
                track.current_pos_secs = 0.0;
                self.next();
            }
        }
    }

    pub fn toggle_play_pause(&mut self) {
        self.status = match self.status {
            PlaybackStatus::Playing => PlaybackStatus::Paused,
            PlaybackStatus::Paused | PlaybackStatus::Stopped => PlaybackStatus::Playing,
        };
        self.last_tick = Instant::now();
    }

    pub fn next(&mut self) {
        if !self.playlist.is_empty() {
            self.current_index = (self.current_index + 1) % self.playlist.len();
            self.playlist[self.current_index].current_pos_secs = 0.0;
            self.last_tick = Instant::now();
        }
    }

    pub fn previous(&mut self) {
        if !self.playlist.is_empty() {
            if self.playlist[self.current_index].current_pos_secs > 3.0 {
                // Restart current track if played more than 3 seconds
                self.playlist[self.current_index].current_pos_secs = 0.0;
            } else if self.current_index == 0 {
                self.current_index = self.playlist.len() - 1;
                self.playlist[self.current_index].current_pos_secs = 0.0;
            } else {
                self.current_index -= 1;
                self.playlist[self.current_index].current_pos_secs = 0.0;
            }
            self.last_tick = Instant::now();
        }
    }

    pub fn seek_ratio(&mut self, ratio: f32) {
        if !self.playlist.is_empty() {
            let track = &mut self.playlist[self.current_index];
            track.current_pos_secs = ratio.clamp(0.0, 1.0) * track.duration_secs;
            self.last_tick = Instant::now();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_playback_lifecycle() {
        let mut service = MediaService::new();
        assert_eq!(service.status, PlaybackStatus::Playing);
        assert!(!service.playlist.is_empty());

        let initial_track = service.current_track().title.clone();
        assert_eq!(initial_track, "Resonance");

        service.toggle_play_pause();
        assert_eq!(service.status, PlaybackStatus::Paused);

        service.toggle_play_pause();
        assert_eq!(service.status, PlaybackStatus::Playing);

        service.next();
        assert_eq!(service.current_track().title, "Midnight City");

        service.previous();
        assert_eq!(service.current_track().title, "Resonance");

        service.seek_ratio(0.5);
        let track = service.current_track();
        assert!((track.current_pos_secs - track.duration_secs * 0.5).abs() < 1.0);
    }
}
