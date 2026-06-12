use super::compression::{TimelineEntry, compress_timeline};
use super::grouping::group_events;
use crate::github::events::GitHubEvent;
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

const MAX_EVENTS: usize = 500;

#[derive(Debug, Clone)]
pub enum AnimationEvent {
    NewEntry(usize),
    UpdatedEntry(usize),
}

pub struct TimelineState {
    pub entries: Vec<TimelineEntry>,
    events: Vec<GitHubEvent>,
    seen_event_ids: HashSet<String>,
    last_update: Option<Instant>,
    animation_queue: VecDeque<AnimationEvent>,
    scroll_offset: f32,
}

impl TimelineState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            events: Vec::new(),
            seen_event_ids: HashSet::new(),
            last_update: None,
            animation_queue: VecDeque::new(),
            scroll_offset: 0.0,
        }
    }

    pub fn update(&mut self, new_events: Vec<GitHubEvent>) {
        let mut added_any = false;
        for event in new_events {
            if self.seen_event_ids.contains(&event.id) {
                continue;
            }
            self.seen_event_ids.insert(event.id.clone());
            self.events.push(event);
            added_any = true;
        }

        if !added_any {
            return;
        }

        if self.events.len() > MAX_EVENTS {
            let excess = self.events.len() - MAX_EVENTS;
            let removed: Vec<_> = self.events.drain(..excess).collect();
            for ev in &removed {
                self.seen_event_ids.remove(&ev.id);
            }
        }

        let groups = group_events(&self.events);
        let old_count = self.entries.len();
        self.entries = compress_timeline(groups);

        if self.entries.len() > old_count {
            for i in old_count..self.entries.len() {
                self.animation_queue.push_back(AnimationEvent::NewEntry(i));
            }
        } else {
            for i in 0..self.entries.len() {
                self.animation_queue
                    .push_back(AnimationEvent::UpdatedEntry(i));
            }
        }

        self.last_update = Some(Instant::now());
    }

    pub fn get_entries(&self) -> &[TimelineEntry] {
        &self.entries
    }

    pub fn pop_animation(&mut self) -> Option<AnimationEvent> {
        self.animation_queue.pop_front()
    }

    pub fn scroll(&mut self, delta: f32) {
        self.scroll_offset += delta;
        if self.scroll_offset > 0.0 {
            self.scroll_offset = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::events::{EventPayloadType, EventType, GitHubEvent};
    use chrono::Utc;

    fn make_push_event(id: &str, repo: &str) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Push,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at: Utc::now(),
            payload: EventPayloadType::Push {
                ref_name: "refs/heads/main".to_string(),
                head: "abc".to_string(),
                before: "def".to_string(),
            },
        }
    }

    fn make_issue_event(id: &str, repo: &str) -> GitHubEvent {
        GitHubEvent {
            id: id.to_string(),
            event_type: EventType::Issues,
            actor: "testuser".to_string(),
            repo_name: repo.to_string(),
            created_at: Utc::now(),
            payload: EventPayloadType::Issues {
                action: "opened".to_string(),
                title: "Bug".to_string(),
                number: 1,
                url: "https://example.com".to_string(),
            },
        }
    }

    #[test]
    fn test_new_state_is_empty() {
        let state = TimelineState::new();
        assert!(state.entries.is_empty());
        assert!(state.events.is_empty());
        assert!(state.seen_event_ids.is_empty());
        assert!(state.last_update.is_none());
    }

    #[test]
    fn test_update_adds_new_events() {
        let mut state = TimelineState::new();
        let events = vec![
            make_push_event("1", "owner/repo"),
            make_push_event("2", "owner/repo"),
        ];
        state.update(events);
        assert_eq!(state.events.len(), 2);
        assert!(!state.entries.is_empty());
        assert!(state.last_update.is_some());
    }

    #[test]
    fn test_update_deduplicates_events() {
        let mut state = TimelineState::new();
        let event = make_push_event("1", "owner/repo");
        state.update(vec![event.clone()]);
        let count_after_first = state.events.len();
        state.update(vec![event]);
        assert_eq!(state.events.len(), count_after_first);
    }

    #[test]
    fn test_update_partial_deduplication() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo")]);
        state.update(vec![
            make_push_event("1", "owner/repo"),
            make_push_event("2", "owner/repo"),
        ]);
        assert_eq!(state.events.len(), 2);
    }

    #[test]
    fn test_update_respects_max_events() {
        let mut state = TimelineState::new();
        let events: Vec<_> = (0..600)
            .map(|i| make_push_event(&i.to_string(), "owner/repo"))
            .collect();
        state.update(events);
        assert!(state.events.len() <= 500);
        assert_eq!(state.events.len(), 500);
    }

    #[test]
    fn test_update_removes_old_ids_when_trimming() {
        let mut state = TimelineState::new();
        let events: Vec<_> = (0..600)
            .map(|i| make_push_event(&i.to_string(), "owner/repo"))
            .collect();
        state.update(events);
        assert!(!state.seen_event_ids.contains("0"));
        assert!(!state.seen_event_ids.contains("99"));
        assert!(state.seen_event_ids.contains("599"));
    }

    #[test]
    fn test_update_with_no_new_events_doesnt_change_entries() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo")]);
        let entries_before = state.entries.len();
        state.update(vec![make_push_event("1", "owner/repo")]);
        assert_eq!(state.entries.len(), entries_before);
    }

    #[test]
    fn test_scroll_does_not_panic() {
        let mut state = TimelineState::new();
        state.scroll(0.0);
        state.scroll(1.0);
        state.scroll(-1.0);
        state.scroll(100.5);
    }

    #[test]
    fn test_pop_animation_empty() {
        let mut state = TimelineState::new();
        assert!(state.pop_animation().is_none());
    }

    #[test]
    fn test_pop_animation_after_update() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo")]);
        let anim = state.pop_animation();
        assert!(anim.is_some());
    }

    #[test]
    fn test_pop_animation_returns_all_events() {
        let mut state = TimelineState::new();
        state.update(vec![
            make_push_event("1", "owner/repo"),
            make_push_event("2", "owner/repo"),
        ]);
        let mut count = 0;
        while state.pop_animation().is_some() {
            count += 1;
        }
        assert!(count > 0);
        assert!(state.pop_animation().is_none());
    }

    #[test]
    fn test_pop_animation_new_entry_on_first_update() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo")]);
        match state.pop_animation() {
            Some(AnimationEvent::NewEntry(_)) => {}
            other => panic!("Expected NewEntry, got {:?}", other),
        }
    }

    #[test]
    fn test_pop_animation_updated_entry_on_subsequent_update() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo")]);
        while state.pop_animation().is_some() {}

        state.update(vec![
            make_push_event("1", "owner/repo"),
            make_push_event("2", "owner/repo"),
        ]);
        match state.pop_animation() {
            Some(AnimationEvent::UpdatedEntry(_)) => {}
            other => panic!("Expected UpdatedEntry, got {:?}", other),
        }
    }

    #[test]
    fn test_get_entries_returns_current_entries() {
        let mut state = TimelineState::new();
        assert!(state.get_entries().is_empty());
        state.update(vec![make_push_event("1", "owner/repo")]);
        assert!(!state.get_entries().is_empty());
    }

    #[test]
    fn test_update_with_different_event_types() {
        let mut state = TimelineState::new();
        state.update(vec![
            make_push_event("1", "owner/repo"),
            make_issue_event("2", "owner/repo"),
        ]);
        assert_eq!(state.events.len(), 2);
        assert!(!state.entries.is_empty());
    }

    #[test]
    fn test_update_across_multiple_calls() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo-a")]);
        assert_eq!(state.events.len(), 1);
        state.update(vec![make_push_event("2", "owner/repo-b")]);
        assert_eq!(state.events.len(), 2);
        state.update(vec![make_push_event("3", "owner/repo-a")]);
        assert_eq!(state.events.len(), 3);
    }

    #[test]
    fn test_animation_queue_drains() {
        let mut state = TimelineState::new();
        state.update(vec![make_push_event("1", "owner/repo")]);
        let first = state.pop_animation();
        assert!(first.is_some());
        let second = state.pop_animation();
        assert!(second.is_none());
    }
}
