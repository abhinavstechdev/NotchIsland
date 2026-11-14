#[derive(Debug, Clone)]
pub struct PinnedSite {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub icon_color: [u8; 3],
    pub icon_letter: &'static str,
}

#[derive(Debug, Clone)]
pub struct WebsitesService {
    pub sites: Vec<PinnedSite>,
    next_id: usize,
}

impl Default for WebsitesService {
    fn default() -> Self {
        Self::new()
    }
}

impl WebsitesService {
    pub fn new() -> Self {
        let sites = vec![
            PinnedSite {
                id: 1,
                name: "YT Music".to_string(),
                url: "https://music.youtube.com".to_string(),
                icon_color: [255, 0, 0],
                icon_letter: "▶",
            },
            PinnedSite {
                id: 2,
                name: "YouTube".to_string(),
                url: "https://www.youtube.com".to_string(),
                icon_color: [255, 68, 68],
                icon_letter: "YT",
            },
            PinnedSite {
                id: 3,
                name: "Spotify".to_string(),
                url: "https://open.spotify.com".to_string(),
                icon_color: [30, 215, 96],
                icon_letter: "≋",
            },
        ];

        Self {
            sites,
            next_id: 4,
        }
    }

    pub fn add_site(&mut self, name: &str, url: &str) {
        let id = self.next_id;
        self.next_id += 1;
        self.sites.push(PinnedSite {
            id,
            name: name.to_string(),
            url: url.to_string(),
            icon_color: [94, 233, 255],
            icon_letter: "🌐",
        });
    }

    pub fn remove_site(&mut self, id: usize) {
        self.sites.retain(|s| s.id != id);
    }

    pub fn launch(&self, url: &str) {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("cmd")
                .args(&["/c", "start", "", url])
                .spawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websites_service() {
        let mut web = WebsitesService::new();
        assert_eq!(web.sites.len(), 3);

        web.add_site("GitHub", "https://github.com");
        assert_eq!(web.sites.len(), 4);

        let added_id = web.sites[3].id;
        web.remove_site(added_id);
        assert_eq!(web.sites.len(), 3);
    }
}
