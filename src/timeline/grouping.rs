use crate::github::events::{EventPayloadType, EventType, GitHubEvent};
use chrono::{DateTime, Utc};
use std::time::Duration;

const TIME_WINDOW: Duration = Duration::from_secs(3 * 3600);

#[derive(Debug, Clone)]
pub struct TimelineGroup {
    pub repo_name: String,
    pub event_type: EventType,
    pub count: u32,
    pub earliest: DateTime<Utc>,
    pub latest: DateTime<Utc>,
    pub events: Vec<GitHubEvent>,
    pub is_rare: bool,
}

fn is_rare(event: &GitHubEvent) -> bool {
    matches!(&event.event_type, EventType::Public | EventType::Release)
        || matches!(
            (&event.event_type, &event.payload),
            (EventType::Create, EventPayloadType::Create { ref_type, .. }) if ref_type == "repo"
        )
}

pub fn group_events(events: &[GitHubEvent]) -> Vec<TimelineGroup> {
    let mut rare_groups: Vec<TimelineGroup> = Vec::new();
    let mut non_rare: Vec<&GitHubEvent> = Vec::new();

    for event in events {
        if is_rare(event) {
            rare_groups.push(TimelineGroup {
                repo_name: event.repo_name.clone(),
                event_type: event.event_type.clone(),
                count: 1,
                earliest: event.created_at,
                latest: event.created_at,
                events: vec![event.clone()],
                is_rare: true,
            });
        } else {
            non_rare.push(event);
        }
    }

    non_rare.sort_by(|a, b| {
        a.repo_name
            .cmp(&b.repo_name)
            .then_with(|| format!("{:?}", a.event_type).cmp(&format!("{:?}", b.event_type)))
            .then_with(|| a.created_at.cmp(&b.created_at))
    });

    let mut non_rare_groups: Vec<TimelineGroup> = Vec::new();

    for event in non_rare {
        let bucket = non_rare_groups.last_mut();
        let matches_bucket = bucket.is_some_and(|g: &mut TimelineGroup| {
            g.repo_name == event.repo_name
                && g.event_type == event.event_type
                && event.created_at.signed_duration_since(g.earliest)
                    <= chrono::Duration::from_std(TIME_WINDOW).unwrap_or(chrono::Duration::hours(3))
        });

        if matches_bucket {
            let g = non_rare_groups.last_mut().unwrap();
            g.count += 1;
            g.latest = event.created_at;
            g.events.push(event.clone());
        } else {
            non_rare_groups.push(TimelineGroup {
                repo_name: event.repo_name.clone(),
                event_type: event.event_type.clone(),
                count: 1,
                earliest: event.created_at,
                latest: event.created_at,
                events: vec![event.clone()],
                is_rare: false,
            });
        }
    }

    let mut all_groups: Vec<TimelineGroup> = rare_groups;
    all_groups.extend(non_rare_groups);
    all_groups.sort_by(|a, b| b.latest.cmp(&a.latest));
    all_groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::events::{EventPayloadType, GitHubEvent};

    fn make_push_event(id: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Push,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at,
            payload: EventPayloadType::Push {
                ref_name: "refs/heads/main".to_string(),
                head: "abc123".to_string(),
                before: "def456".to_string(),
            },
        }
    }

    fn make_release_event(id: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Release,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at,
            payload: EventPayloadType::Release {
                action: "published".to_string(),
                tag_name: "v1.0.0".to_string(),
                name: "Release 1.0".to_string(),
                url: "https://example.com".to_string(),
            },
        }
    }

    fn make_create_repo_event(id: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Create,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at,
            payload: EventPayloadType::Create {
                ref_type: "repo".to_string(),
                ref_name: "".to_string(),
            },
        }
    }

    fn make_create_branch_event(id: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Create,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at,
            payload: EventPayloadType::Create {
                ref_type: "branch".to_string(),
                ref_name: "feature".to_string(),
            },
        }
    }

    fn make_pr_event(id: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::PullRequest,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at,
            payload: EventPayloadType::PullRequest {
                action: "opened".to_string(),
                title: "Test PR".to_string(),
                number: 1,
                url: "https://example.com".to_string(),
            },
        }
    }

    fn make_issue_event(id: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Issues,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at,
            payload: EventPayloadType::Issues {
                action: "opened".to_string(),
                title: "Test Issue".to_string(),
                number: 1,
                url: "https://example.com".to_string(),
            },
        }
    }

    #[test]
    fn test_empty_events_returns_empty_groups() {
        let groups = group_events(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_single_event_creates_one_group() {
        let now = Utc::now();
        let events = vec![make_push_event("1", "owner/repo", now)];
        let groups = group_events(&events);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].repo_name, "owner/repo");
        assert_eq!(groups[0].count, 1);
        assert_eq!(groups[0].event_type, EventType::Push);
    }

    #[test]
    fn test_multiple_same_repo_type_get_grouped() {
        let now = Utc::now();
        let events = vec![
            make_push_event("1", "owner/repo", now - chrono::Duration::minutes(10)),
            make_push_event("2", "owner/repo", now - chrono::Duration::minutes(5)),
            make_push_event("3", "owner/repo", now),
        ];
        let groups = group_events(&events);
        let non_rare: Vec<_> = groups.iter().filter(|g| !g.is_rare).collect();
        assert_eq!(non_rare.len(), 1);
        assert_eq!(non_rare[0].count, 3);
    }

    #[test]
    fn test_same_repo_outside_time_window_creates_separate_groups() {
        let now = Utc::now();
        let events = vec![
            make_push_event("1", "owner/repo", now - chrono::Duration::hours(5)),
            make_push_event("2", "owner/repo", now),
        ];
        let groups = group_events(&events);
        let non_rare: Vec<_> = groups.iter().filter(|g| !g.is_rare).collect();
        assert_eq!(non_rare.len(), 2);
        assert_eq!(non_rare[0].count, 1);
        assert_eq!(non_rare[1].count, 1);
    }

    #[test]
    fn test_rare_events_get_own_groups() {
        let now = Utc::now();
        let events = vec![
            make_release_event("1", "owner/repo", now - chrono::Duration::minutes(5)),
            make_release_event("2", "owner/repo", now),
        ];
        let groups = group_events(&events);
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| g.is_rare));
    }

    #[test]
    fn test_create_repo_is_rare() {
        let now = Utc::now();
        let events = vec![
            make_create_repo_event("1", "owner/repo", now),
            make_create_repo_event("2", "owner/other", now),
        ];
        let groups = group_events(&events);
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| g.is_rare));
    }

    #[test]
    fn test_create_branch_is_not_rare() {
        let now = Utc::now();
        let events = vec![
            make_create_branch_event("1", "owner/repo", now - chrono::Duration::minutes(5)),
            make_create_branch_event("2", "owner/repo", now),
        ];
        let groups = group_events(&events);
        let non_rare: Vec<_> = groups.iter().filter(|g| !g.is_rare).collect();
        assert_eq!(non_rare.len(), 1);
        assert_eq!(non_rare[0].count, 2);
    }

    #[test]
    fn test_different_repos_get_separate_groups() {
        let now = Utc::now();
        let events = vec![
            make_push_event("1", "owner/repo-a", now),
            make_push_event("2", "owner/repo-b", now),
        ];
        let groups = group_events(&events);
        assert_eq!(groups.len(), 2);
        let repos: Vec<_> = groups.iter().map(|g| g.repo_name.as_str()).collect();
        assert!(repos.contains(&"owner/repo-a"));
        assert!(repos.contains(&"owner/repo-b"));
    }

    #[test]
    fn test_same_repo_different_types_get_separate_groups() {
        let now = Utc::now();
        let events = vec![
            make_push_event("1", "owner/repo", now),
            make_pr_event("2", "owner/repo", now),
            make_issue_event("3", "owner/repo", now),
        ];
        let groups = group_events(&events);
        let non_rare: Vec<_> = groups.iter().filter(|g| !g.is_rare).collect();
        assert_eq!(non_rare.len(), 3);
    }

    #[test]
    fn test_groups_sorted_by_latest_desc() {
        let now = Utc::now();
        let events = vec![
            make_push_event("1", "owner/repo", now - chrono::Duration::hours(2)),
            make_pr_event("2", "owner/repo", now),
        ];
        let groups = group_events(&events);
        for i in 0..groups.len() - 1 {
            assert!(groups[i].latest >= groups[i + 1].latest);
        }
    }

    #[test]
    fn test_mixed_rare_and_non_rare() {
        let now = Utc::now();
        let events = vec![
            make_push_event("1", "owner/repo", now - chrono::Duration::minutes(10)),
            make_push_event("2", "owner/repo", now - chrono::Duration::minutes(5)),
            make_release_event("3", "owner/repo", now),
        ];
        let groups = group_events(&events);
        let rare: Vec<_> = groups.iter().filter(|g| g.is_rare).collect();
        let non_rare: Vec<_> = groups.iter().filter(|g| !g.is_rare).collect();
        assert_eq!(rare.len(), 1);
        assert_eq!(non_rare.len(), 1);
        assert_eq!(non_rare[0].count, 2);
    }
}
