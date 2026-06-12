use super::grouping::TimelineGroup;
use super::humanize::humanize_time_range;
use crate::github::events::EventType;

#[derive(Debug, Clone)]
pub enum TimelineEntry {
    Single(TimelineGroup),
    Compressed(CompressedEntry),
}

#[derive(Debug, Clone)]
pub struct CompressedEntry {
    pub repo_name: String,
    pub items: Vec<(EventType, u32, String)>,
    pub time_range_str: String,
    pub count: u32,
}

const COMPRESS_WINDOW_SECS: i64 = 6 * 3600;

pub fn compress_timeline(groups: Vec<TimelineGroup>) -> Vec<TimelineEntry> {
    let mut entries: Vec<TimelineEntry> = Vec::new();
    let mut buffer: Vec<TimelineGroup> = Vec::new();

    fn flush_buffer(buffer: &mut Vec<TimelineGroup>, entries: &mut Vec<TimelineEntry>) {
        if buffer.is_empty() {
            return;
        }
        if buffer.len() == 1 {
            entries.push(TimelineEntry::Single(buffer.remove(0)));
            return;
        }
        let repo_name = buffer[0].repo_name.clone();
        let total_count: u32 = buffer.iter().map(|g| g.count).sum();
        let earliest = buffer.iter().map(|g| g.earliest).min().unwrap();
        let latest = buffer.iter().map(|g| g.latest).max().unwrap();
        let time_range_str = humanize_time_range(earliest, latest);
        let items = buffer
            .iter()
            .map(|g| {
                let range = humanize_time_range(g.earliest, g.latest);
                (g.event_type.clone(), g.count, range)
            })
            .collect();
        entries.push(TimelineEntry::Compressed(CompressedEntry {
            repo_name,
            items,
            time_range_str,
            count: total_count,
        }));
        buffer.clear();
    }

    let mut remaining: Vec<TimelineGroup> = groups;

    while let Some(group) = remaining.pop() {
        if group.is_rare {
            flush_buffer(&mut buffer, &mut entries);
            entries.push(TimelineEntry::Single(group));
            continue;
        }

        let can_merge = buffer.last().is_none_or(|prev| {
            if prev.repo_name != group.repo_name {
                false
            } else {
                let diff = group.latest.signed_duration_since(prev.earliest);
                diff.num_seconds().unsigned_abs() <= COMPRESS_WINDOW_SECS as u64
                    || diff.num_seconds() <= COMPRESS_WINDOW_SECS
            }
        });

        if can_merge {
            buffer.push(group);
        } else {
            flush_buffer(&mut buffer, &mut entries);
            buffer.push(group);
        }
    }

    flush_buffer(&mut buffer, &mut entries);
    entries.reverse();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::events::EventType;
    use crate::timeline::grouping::TimelineGroup;
    use chrono::Utc;

    fn make_non_rare_group(
        repo: &str,
        event_type: EventType,
        count: u32,
        earliest: chrono::DateTime<Utc>,
        latest: chrono::DateTime<Utc>,
    ) -> TimelineGroup {
        TimelineGroup {
            repo_name: repo.to_string(),
            event_type,
            count,
            earliest,
            latest,
            events: Vec::new(),
            is_rare: false,
        }
    }

    fn make_rare_group(
        repo: &str,
        event_type: EventType,
        created_at: chrono::DateTime<Utc>,
    ) -> TimelineGroup {
        TimelineGroup {
            repo_name: repo.to_string(),
            event_type,
            count: 1,
            earliest: created_at,
            latest: created_at,
            events: Vec::new(),
            is_rare: true,
        }
    }

    #[test]
    fn test_empty_input_returns_empty() {
        let entries = compress_timeline(Vec::new());
        assert!(entries.is_empty());
    }

    #[test]
    fn test_rare_group_becomes_single() {
        let now = Utc::now();
        let group = make_rare_group("owner/repo", EventType::Release, now);
        let entries = compress_timeline(vec![group]);
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TimelineEntry::Single(g) => {
                assert_eq!(g.repo_name, "owner/repo");
                assert!(g.is_rare);
            }
            _ => panic!("Expected Single entry"),
        }
    }

    #[test]
    fn test_single_non_rare_group_becomes_single() {
        let now = Utc::now();
        let group = make_non_rare_group("owner/repo", EventType::Push, 1, now, now);
        let entries = compress_timeline(vec![group]);
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TimelineEntry::Single(g) => {
                assert_eq!(g.repo_name, "owner/repo");
                assert!(!g.is_rare);
            }
            _ => panic!("Expected Single entry"),
        }
    }

    #[test]
    fn test_non_rare_groups_same_repo_within_6_hours_compressed() {
        let now = Utc::now();
        let groups = vec![
            make_non_rare_group(
                "owner/repo",
                EventType::Push,
                3,
                now - chrono::Duration::hours(2),
                now - chrono::Duration::hours(2),
            ),
            make_non_rare_group(
                "owner/repo",
                EventType::PullRequest,
                2,
                now - chrono::Duration::hours(1),
                now - chrono::Duration::hours(1),
            ),
        ];
        let entries = compress_timeline(groups);
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TimelineEntry::Compressed(c) => {
                assert_eq!(c.repo_name, "owner/repo");
                assert_eq!(c.count, 5);
                assert_eq!(c.items.len(), 2);
            }
            _ => panic!("Expected Compressed entry"),
        }
    }

    #[test]
    fn test_groups_from_different_repos_stay_separate() {
        let now = Utc::now();
        let groups = vec![
            make_non_rare_group(
                "owner/repo-a",
                EventType::Push,
                2,
                now - chrono::Duration::hours(1),
                now,
            ),
            make_non_rare_group(
                "owner/repo-b",
                EventType::Push,
                3,
                now - chrono::Duration::hours(1),
                now,
            ),
        ];
        let entries = compress_timeline(groups);
        assert_eq!(entries.len(), 2);
        for entry in &entries {
            match entry {
                TimelineEntry::Single(g) => {
                    assert!(!g.is_rare);
                }
                _ => panic!("Expected Single entries for different repos"),
            }
        }
    }

    #[test]
    fn test_rare_group_flushes_buffer_before_adding() {
        let now = Utc::now();
        let groups = vec![
            make_non_rare_group(
                "owner/repo",
                EventType::Push,
                2,
                now - chrono::Duration::hours(5),
                now - chrono::Duration::hours(5),
            ),
            make_non_rare_group(
                "owner/repo",
                EventType::PullRequest,
                1,
                now - chrono::Duration::hours(4),
                now - chrono::Duration::hours(4),
            ),
            make_rare_group(
                "owner/repo",
                EventType::Release,
                now - chrono::Duration::hours(3),
            ),
            make_non_rare_group(
                "owner/repo",
                EventType::Issues,
                1,
                now - chrono::Duration::hours(2),
                now - chrono::Duration::hours(2),
            ),
        ];
        let entries = compress_timeline(groups);
        assert_eq!(entries.len(), 3);
        match &entries[0] {
            TimelineEntry::Compressed(c) => {
                assert_eq!(c.count, 3);
            }
            _ => panic!("Expected Compressed entry"),
        }
        match &entries[1] {
            TimelineEntry::Single(g) => {
                assert!(g.is_rare);
                assert_eq!(g.event_type, EventType::Release);
            }
            _ => panic!("Expected Single rare entry"),
        }
        match &entries[2] {
            TimelineEntry::Single(g) => {
                assert!(!g.is_rare);
                assert_eq!(g.event_type, EventType::Issues);
            }
            _ => panic!("Expected Single non-rare entry"),
        }
    }

    #[test]
    fn test_compressed_entry_has_correct_time_range() {
        let now = Utc::now();
        let groups = vec![
            make_non_rare_group(
                "owner/repo",
                EventType::Push,
                1,
                now - chrono::Duration::minutes(5),
                now - chrono::Duration::minutes(5),
            ),
            make_non_rare_group("owner/repo", EventType::Issues, 1, now, now),
        ];
        let entries = compress_timeline(groups);
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            TimelineEntry::Compressed(c) => {
                assert!(!c.time_range_str.is_empty());
            }
            _ => panic!("Expected Compressed entry"),
        }
    }
}
