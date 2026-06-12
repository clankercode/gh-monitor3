use chrono::{DateTime, Utc};
use octocrab::models::events::payload::EventPayload;
use octocrab::models::events::Event;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubEvent {
    pub id: String,
    pub event_type: EventType,
    pub actor: String,
    pub repo_name: String,
    pub created_at: DateTime<Utc>,
    pub payload: EventPayloadType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    Push,
    PullRequest,
    Issues,
    Create,
    Delete,
    Release,
    Fork,
    Watch,
    IssueComment,
    PullRequestReview,
    PullRequestReviewComment,
    CommitComment,
    Public,
    Member,
    Gollum,
    Discussion,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayloadType {
    Push {
        ref_name: String,
        head: String,
        before: String,
    },
    PullRequest {
        action: String,
        title: String,
        number: u64,
        url: String,
    },
    Issues {
        action: String,
        title: String,
        number: u64,
        url: String,
    },
    Create {
        ref_type: String,
        ref_name: String,
    },
    Delete {
        ref_type: String,
        ref_name: String,
    },
    Release {
        action: String,
        tag_name: String,
        name: String,
        url: String,
    },
    Fork {
        full_name: String,
    },
    Watch {
        action: String,
    },
    IssueComment {
        action: String,
        issue_title: String,
        issue_number: u64,
        comment_url: String,
    },
    PullRequestReview {
        action: String,
        pr_title: String,
        pr_number: u64,
        review_url: String,
    },
    PullRequestReviewComment {
        action: String,
        pr_title: String,
        pr_number: u64,
        comment_url: String,
    },
    CommitComment {
        comment_url: String,
    },
    Public,
    Member {
        action: String,
        member_login: String,
    },
    Gollum {
        page_count: usize,
    },
    Unknown(String),
}

impl GitHubEvent {
    pub fn event_type_icon(&self) -> &'static str {
        match &self.event_type {
            EventType::Push => "\u{1f4e6}",
            EventType::PullRequest => "\u{1f500}",
            EventType::Issues => "\u{1f41b}",
            EventType::Create => "\u{1f331}",
            EventType::Delete => "\u{1f5d1}",
            EventType::Release => "\u{1f389}",
            EventType::Fork => "\u{1f517}",
            EventType::Watch => "\u{2b50}",
            EventType::IssueComment => "\u{1f4ac}",
            EventType::PullRequestReview => "\u{1f4dd}",
            EventType::PullRequestReviewComment => "\u{1f4dd}",
            EventType::CommitComment => "\u{1f4ac}",
            EventType::Public => "\u{1f310}",
            EventType::Member => "\u{1f464}",
            EventType::Gollum => "\u{1f4d6}",
            EventType::Discussion => "\u{1f4e2}",
            EventType::Other(_) => "\u{2753}",
        }
    }

    pub fn event_type_label(&self) -> &'static str {
        match &self.event_type {
            EventType::Push => "Push",
            EventType::PullRequest => "Pull Request",
            EventType::Issues => "Issue",
            EventType::Create => "Create",
            EventType::Delete => "Delete",
            EventType::Release => "Release",
            EventType::Fork => "Fork",
            EventType::Watch => "Watch",
            EventType::IssueComment => "Issue Comment",
            EventType::PullRequestReview => "PR Review",
            EventType::PullRequestReviewComment => "PR Review Comment",
            EventType::CommitComment => "Commit Comment",
            EventType::Public => "Public",
            EventType::Member => "Member",
            EventType::Gollum => "Wiki",
            EventType::Discussion => "Discussion",
            EventType::Other(_) => "Other",
        }
    }
}

impl From<Event> for GitHubEvent {
    fn from(event: Event) -> Self {
        let event_type = match &event.r#type {
            octocrab::models::events::EventType::PushEvent => EventType::Push,
            octocrab::models::events::EventType::PullRequestEvent => EventType::PullRequest,
            octocrab::models::events::EventType::IssuesEvent => EventType::Issues,
            octocrab::models::events::EventType::CreateEvent => EventType::Create,
            octocrab::models::events::EventType::DeleteEvent => EventType::Delete,
            octocrab::models::events::EventType::ReleaseEvent => EventType::Release,
            octocrab::models::events::EventType::ForkEvent => EventType::Fork,
            octocrab::models::events::EventType::WatchEvent => EventType::Watch,
            octocrab::models::events::EventType::IssueCommentEvent => EventType::IssueComment,
            octocrab::models::events::EventType::PullRequestReviewEvent => {
                EventType::PullRequestReview
            }
            octocrab::models::events::EventType::PullRequestReviewCommentEvent => {
                EventType::PullRequestReviewComment
            }
            octocrab::models::events::EventType::CommitCommentEvent => EventType::CommitComment,
            octocrab::models::events::EventType::PublicEvent => EventType::Public,
            octocrab::models::events::EventType::MemberEvent => EventType::Member,
            octocrab::models::events::EventType::GollumEvent => EventType::Gollum,
            octocrab::models::events::EventType::WorkflowRunEvent => {
                EventType::Other("WorkflowRun".to_string())
            }
            octocrab::models::events::EventType::UnknownEvent(s) => EventType::Other(s.clone()),
            _ => EventType::Other("Unknown".to_string()),
        };

        let payload = extract_payload(&event);

        let repo_name = event.repo.name.clone();

        GitHubEvent {
            id: event.id,
            event_type,
            actor: event.actor.login,
            repo_name,
            created_at: event.created_at,
            payload,
        }
    }
}

fn extract_payload(event: &Event) -> EventPayloadType {
    let wrapped = match &event.payload {
        Some(w) => w,
        None => return EventPayloadType::Unknown("no payload".to_string()),
    };
    let specific = match &wrapped.specific {
        Some(s) => s,
        None => return EventPayloadType::Unknown("no specific payload".to_string()),
    };
    match specific {
        EventPayload::PushEvent(p) => EventPayloadType::Push {
            ref_name: p.r#ref.clone(),
            head: p.head.clone(),
            before: p.before.clone(),
        },
        EventPayload::CreateEvent(p) => EventPayloadType::Create {
            ref_type: p.ref_type.clone(),
            ref_name: p.r#ref.clone().unwrap_or_default(),
        },
        EventPayload::DeleteEvent(p) => EventPayloadType::Delete {
            ref_type: p.ref_type.clone(),
            ref_name: p.r#ref.clone(),
        },
        EventPayload::PullRequestEvent(p) => EventPayloadType::PullRequest {
            action: format!("{:?}", p.action),
            title: p.pull_request.title.clone().unwrap_or_default(),
            number: p.number,
            url: p
                .pull_request
                .html_url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
        },
        EventPayload::IssuesEvent(p) => EventPayloadType::Issues {
            action: format!("{:?}", p.action),
            title: p.issue.title.clone(),
            number: p.issue.number,
            url: p.issue.html_url.to_string(),
        },
        EventPayload::ReleaseEvent(p) => EventPayloadType::Release {
            action: format!("{:?}", p.action),
            tag_name: p.release.tag_name.clone(),
            name: p.release.name.clone().unwrap_or_default(),
            url: p.release.html_url.to_string(),
        },
        EventPayload::ForkEvent(p) => EventPayloadType::Fork {
            full_name: p
                .forkee
                .full_name
                .clone()
                .unwrap_or_else(|| p.forkee.name.clone()),
        },
        EventPayload::WatchEvent(p) => EventPayloadType::Watch {
            action: format!("{:?}", p.action),
        },
        EventPayload::IssueCommentEvent(p) => EventPayloadType::IssueComment {
            action: format!("{:?}", p.action),
            issue_title: p.issue.title.clone(),
            issue_number: p.issue.number,
            comment_url: p.comment.html_url.to_string(),
        },
        EventPayload::PullRequestReviewEvent(p) => EventPayloadType::PullRequestReview {
            action: format!("{:?}", p.action),
            pr_title: p.pull_request.title.clone().unwrap_or_default(),
            pr_number: p.pull_request.number.unwrap_or(0),
            review_url: p.review.html_url.to_string(),
        },
        EventPayload::PullRequestReviewCommentEvent(p) => {
            EventPayloadType::PullRequestReviewComment {
                action: format!("{:?}", p.action),
                pr_title: p.pull_request.title.clone().unwrap_or_default(),
                pr_number: p.pull_request.number.unwrap_or(0),
                comment_url: p.comment.html_url.to_string(),
            }
        }
        EventPayload::CommitCommentEvent(p) => EventPayloadType::CommitComment {
            comment_url: p.comment.html_url.to_string(),
        },
        EventPayload::PublicEvent(_) => EventPayloadType::Public,
        EventPayload::MemberEvent(p) => EventPayloadType::Member {
            action: format!("{:?}", p.action),
            member_login: p.member.login.clone(),
        },
        EventPayload::GollumEvent(p) => EventPayloadType::Gollum {
            page_count: p.pages.len(),
        },
        _ => EventPayloadType::Unknown(format!("{:?}", specific)),
    }
}
