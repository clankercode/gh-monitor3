use std::time::{Duration, Instant};

use chrono::Utc;

use crate::github::events::{EventPayloadType, EventType, GitHubEvent};

const DEMO_DURATION: Duration = Duration::from_secs(120);
const EVENT_COUNT: usize = 50;

pub struct DemoMode {
    active: bool,
    start_time: Option<Instant>,
    events_to_send: Vec<(Duration, GitHubEvent)>,
    next_event_index: usize,
}

impl DemoMode {
    pub fn new() -> Self {
        Self {
            active: false,
            start_time: None,
            events_to_send: Vec::new(),
            next_event_index: 0,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.start_time = Some(Instant::now());
        self.next_event_index = 0;
        self.events_to_send = generate_demo_events();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn tick(&mut self) -> Vec<GitHubEvent> {
        if !self.active {
            return Vec::new();
        }

        let elapsed = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        if elapsed >= DEMO_DURATION {
            self.active = false;
            return Vec::new();
        }

        let mut result = Vec::new();
        while self.next_event_index < self.events_to_send.len() {
            let (scheduled_at, _) = &self.events_to_send[self.next_event_index];
            if elapsed >= *scheduled_at {
                let (_, event) = self.events_to_send[self.next_event_index].clone();
                result.push(event);
                self.next_event_index += 1;
            } else {
                break;
            }
        }

        result
    }
}

fn generate_demo_events() -> Vec<(Duration, GitHubEvent)> {
    let repos = [
        "rust-lang/rust",
        "tokio-rs/tokio",
        "facebook/react",
        "denoland/deno",
        "vercel/next.js",
    ];
    let actors = [
        "alice", "bob", "charlie", "diana", "eve", "frank", "grace", "hank",
    ];

    let mut events = Vec::new();

    let offsets = generate_time_offsets(EVENT_COUNT, DEMO_DURATION);

    for (i, offset) in offsets.iter().enumerate() {
        let repo = repos[i % repos.len()];
        let actor = actors[i % actors.len()];
        let event = make_event(i, repo, actor);
        events.push((*offset, event));
    }

    events
}

fn generate_time_offsets(count: usize, total: Duration) -> Vec<Duration> {
    let total_ms = total.as_millis() as f64;
    let mut offsets = Vec::with_capacity(count);

    for i in 0..count {
        let base = (i as f64 / count as f64) * total_ms;
        let burst_offset = match i % 7 {
            0 => -800.0,
            1 => -400.0,
            2 => 0.0,
            3 => 200.0,
            4 => 1200.0,
            5 => 2000.0,
            _ => 3500.0,
        };
        let jitter = ((i * 37 + 13) % 500) as f64;
        let ms = (base + burst_offset + jitter)
            .max(0.0)
            .min(total_ms - 1000.0);
        offsets.push(Duration::from_millis(ms as u64));
    }

    offsets.sort();
    offsets
}

fn make_event(index: usize, repo: &str, actor: &str) -> GitHubEvent {
    let id = format!("demo-{}-{}", index, (index * 2654435761) % 1_000_000);
    let event_type = pick_event_type(index);
    let payload = make_payload(index, &event_type, repo);

    GitHubEvent {
        id,
        event_type,
        actor: actor.to_string(),
        repo_name: repo.to_string(),
        created_at: Utc::now(),
        payload,
    }
}

fn pick_event_type(index: usize) -> EventType {
    match index % 15 {
        0..=3 => EventType::Push,
        4 | 5 => EventType::PullRequest,
        6 | 7 => EventType::Issues,
        8 => EventType::Create,
        9 => EventType::Delete,
        10 => EventType::Release,
        11 => EventType::Fork,
        12 => EventType::Watch,
        13 => EventType::IssueComment,
        _ => EventType::PullRequestReview,
    }
}

fn make_payload(index: usize, event_type: &EventType, repo: &str) -> EventPayloadType {
    match event_type {
        EventType::Push => {
            let refs = [
                "main",
                "develop",
                "feature/auth",
                "fix/memory-leak",
                "release/v2",
            ];
            let ref_name = refs[index % refs.len()];
            let head = format!(
                "{:04x}{:04x}",
                (index * 7919) % 0xffff,
                (index * 104729) % 0xffff
            );
            let before = format!(
                "{:04x}{:04x}",
                (index * 6271) % 0xffff,
                (index * 47591) % 0xffff
            );
            EventPayloadType::Push {
                ref_name: ref_name.to_string(),
                head,
                before,
            }
        }
        EventType::PullRequest => {
            let titles = [
                "feat: add new feature",
                "fix: resolve memory leak",
                "refactor: simplify error handling",
                "docs: update API reference",
                "perf: optimize hot path",
                "chore: bump dependencies",
                "feat: add dark mode support",
                "fix: race condition in worker",
            ];
            let actions = ["opened", "closed", "merged", "reopened"];
            let num = (index % 100) + 1;
            EventPayloadType::PullRequest {
                action: actions[index % actions.len()].to_string(),
                title: titles[index % titles.len()].to_string(),
                number: num as u64,
                url: format!("https://github.com/{}/pull/{}", repo, num),
            }
        }
        EventType::Issues => {
            let titles = [
                "Bug: crash on startup",
                "Feature request: dark mode",
                "Bug: incorrect timestamp display",
                "Enhancement: improve search",
                "Bug: memory usage grows over time",
                "Feature request: export to CSV",
                "Bug: race condition in sync",
                "Docs: missing examples",
            ];
            let actions = ["opened", "closed", "reopened"];
            let num = (index % 80) + 10;
            EventPayloadType::Issues {
                action: actions[index % actions.len()].to_string(),
                title: titles[index % titles.len()].to_string(),
                number: num as u64,
                url: format!("https://github.com/{}/issues/{}", repo, num),
            }
        }
        EventType::Create => {
            let ref_type = if index.is_multiple_of(2) {
                "branch"
            } else {
                "tag"
            };
            let ref_name = if ref_type == "branch" {
                [
                    "feature/new-ui",
                    "fix/edge-case",
                    "experiment/wasm",
                    "release/v3",
                ][index % 4]
            } else {
                ["v1.0.0", "v2.1.3", "v3.0.0-beta.1", "v0.9.8"][index % 4]
            };
            EventPayloadType::Create {
                ref_type: ref_type.to_string(),
                ref_name: ref_name.to_string(),
            }
        }
        EventType::Delete => {
            let ref_type = if index.is_multiple_of(3) {
                "branch"
            } else {
                "tag"
            };
            let ref_name = if ref_type == "branch" {
                [
                    "feature/old-branch",
                    "hotfix/temp-fix",
                    "experiment/abandoned",
                ][index % 3]
            } else {
                ["v0.1.0-rc.1", "v2.0.0-beta.3", "v1.5.0-alpha.2"][index % 3]
            };
            EventPayloadType::Delete {
                ref_type: ref_type.to_string(),
                ref_name: ref_name.to_string(),
            }
        }
        EventType::Release => {
            let tags = ["v1.0.0", "v2.1.3", "v3.0.0", "v0.9.8", "v4.2.0-rc.1"];
            let names = [
                "Release 1.0.0",
                "Patch 2.1.3",
                "Major 3.0.0",
                "Hotfix 0.9.8",
                "Release Candidate 4.2.0",
            ];
            let tag = tags[index % tags.len()];
            EventPayloadType::Release {
                action: "published".to_string(),
                tag_name: tag.to_string(),
                name: names[index % names.len()].to_string(),
                url: format!("https://github.com/{}/releases/tag/{}", repo, tag),
            }
        }
        EventType::Fork => {
            let fork_owners = ["user1", "devbot", "contributor99", "forker42"];
            let fork_owner = fork_owners[index % fork_owners.len()];
            let repo_short = repo.split('/').nth(1).unwrap_or("repo");
            EventPayloadType::Fork {
                full_name: format!("{}/{}", fork_owner, repo_short),
            }
        }
        EventType::Watch => {
            let actions = ["started", "started", "started"];
            EventPayloadType::Watch {
                action: actions[index % actions.len()].to_string(),
            }
        }
        EventType::IssueComment => {
            let issue_titles = [
                "Bug: crash on startup",
                "Feature request: dark mode",
                "Enhancement: improve search",
                "Docs: missing examples",
            ];
            let num = (index % 50) + 5;
            EventPayloadType::IssueComment {
                action: "created".to_string(),
                issue_title: issue_titles[index % issue_titles.len()].to_string(),
                issue_number: num as u64,
                comment_url: format!(
                    "https://github.com/{}/issues/{}#issuecomment-{}",
                    repo,
                    num,
                    index * 1000
                ),
            }
        }
        EventType::PullRequestReview => {
            let pr_titles = [
                "feat: add new feature",
                "fix: resolve memory leak",
                "refactor: simplify error handling",
                "perf: optimize hot path",
            ];
            let actions = ["approved", "changes_requested", "commented"];
            let num = (index % 60) + 1;
            EventPayloadType::PullRequestReview {
                action: actions[index % actions.len()].to_string(),
                pr_title: pr_titles[index % pr_titles.len()].to_string(),
                pr_number: num as u64,
                review_url: format!(
                    "https://github.com/{}/pull/{}#pullrequestreview-{}",
                    repo,
                    num,
                    index * 500
                ),
            }
        }
        _ => EventPayloadType::Unknown("demo".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_mode_starts_inactive() {
        let mut demo = DemoMode::new();
        assert!(!demo.is_active());
        assert!(demo.tick().is_empty());
    }

    #[test]
    fn demo_mode_start_activates() {
        let mut demo = DemoMode::new();
        demo.start();
        assert!(demo.is_active());
    }

    #[test]
    fn demo_mode_tick_returns_events() {
        let mut demo = DemoMode::new();
        demo.start();
        let events = demo.tick();
        assert!(!events.is_empty());
    }

    #[test]
    fn demo_events_have_valid_ids() {
        let mut demo = DemoMode::new();
        demo.start();
        let events = demo.tick();
        for event in &events {
            assert!(event.id.starts_with("demo-"));
        }
    }

    #[test]
    fn demo_events_use_known_repos() {
        let known = [
            "rust-lang/rust",
            "tokio-rs/tokio",
            "facebook/react",
            "denoland/deno",
            "vercel/next.js",
        ];
        let mut demo = DemoMode::new();
        demo.start();
        let events = demo.tick();
        for event in &events {
            assert!(known.contains(&event.repo_name.as_str()));
        }
    }

    #[test]
    fn demo_events_cover_multiple_types() {
        let events = generate_demo_events();
        let mut types: Vec<String> = events
            .iter()
            .map(|(_, e)| format!("{:?}", e.event_type))
            .collect();
        types.sort();
        types.dedup();
        assert!(types.len() >= 5, "expected >=5 types, got {}", types.len());
    }
}
