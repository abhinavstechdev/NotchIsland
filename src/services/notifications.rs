#[derive(Debug, Clone)]
pub struct NotificationItem {
    pub id: usize,
    pub title: String,
    pub body: String,
    pub time_label: String,
    pub app_name: String,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub struct NotificationsService {
    pub items: Vec<NotificationItem>,
    next_id: usize,
}

impl Default for NotificationsService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationsService {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, app_name: &str, title: &str, body: &str, icon: &str) {
        let item = NotificationItem {
            id: self.next_id,
            title: title.to_string(),
            body: body.to_string(),
            time_label: "Just now".to_string(),
            app_name: app_name.to_string(),
            icon: icon.to_string(),
        };
        self.next_id += 1;
        self.items.insert(0, item);
    }

    pub fn remove(&mut self, id: usize) {
        self.items.retain(|i| i.id != id);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifications_service() {
        let mut notifs = NotificationsService::new();
        assert!(notifs.is_empty());

        notifs.add("Discord", "New Message", "Hey there!", "💬");
        assert_eq!(notifs.items.len(), 1);

        let id = notifs.items[0].id;
        notifs.remove(id);
        assert!(notifs.is_empty());
    }
}
