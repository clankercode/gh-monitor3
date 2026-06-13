use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

use crate::github::events::GitHubEvent;

const RATE_LIMIT_DURATION: Duration = Duration::minutes(5);

pub struct NotificationManager {
    enabled: bool,
    last_notified: HashMap<String, DateTime<Utc>>,
    last_notified_id: HashSet<String>,
}

impl NotificationManager {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_notified: HashMap::new(),
            last_notified_id: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn process_events(&mut self, events: &[GitHubEvent]) {
        if !self.enabled {
            return;
        }

        let mut by_repo: HashMap<String, Vec<&GitHubEvent>> = HashMap::new();
        for event in events {
            if self.last_notified_id.contains(&event.id) {
                continue;
            }
            by_repo
                .entry(event.repo_name.clone())
                .or_default()
                .push(event);
        }

        let now = Utc::now();

        for (repo_name, repo_events) in &by_repo {
            if repo_events.is_empty() {
                continue;
            }

            if let Some(last) = self.last_notified.get(repo_name)
                && now - *last < RATE_LIMIT_DURATION
            {
                continue;
            }

            let count = repo_events.len();
            let body = format!(
                "{}: {} new event{}",
                repo_name,
                count,
                if count == 1 { "" } else { "s" }
            );

            notify_rust::Notification::new()
                .summary("gh-monitor3")
                .body(&body)
                .appname("gh-monitor3")
                .timeout(5000)
                .show()
                .ok();

            self.last_notified.insert(repo_name.clone(), now);
            for event in repo_events {
                self.last_notified_id.insert(event.id.clone());
            }
        }
    }
}
