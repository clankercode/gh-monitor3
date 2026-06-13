use crate::github::events::EventType;

#[derive(Clone)]
pub struct Theme {
    pub bg_color: [f32; 4],
    pub text_color: [f32; 4],
    pub badge_pr: [f32; 4],
    pub badge_issue: [f32; 4],
    pub badge_push: [f32; 4],
    pub badge_release: [f32; 4],
    pub badge_fork: [f32; 4],
    pub badge_create: [f32; 4],
    pub badge_other: [f32; 4],
    pub font_size: f32,
    pub padding: f32,
    #[allow(dead_code)]
    pub corner_radius: f32,
    pub line_height: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg_color: [0.1, 0.1, 0.12, 0.85],
            text_color: [0.92, 0.92, 0.94, 1.0],
            badge_pr: [0.36, 0.53, 0.85, 1.0],
            badge_issue: [0.45, 0.78, 0.42, 1.0],
            badge_push: [0.75, 0.65, 0.35, 1.0],
            badge_release: [0.82, 0.45, 0.72, 1.0],
            badge_fork: [0.55, 0.55, 0.62, 1.0],
            badge_create: [0.42, 0.72, 0.72, 1.0],
            badge_other: [0.58, 0.58, 0.62, 1.0],
            font_size: 14.0,
            padding: 12.0,
            corner_radius: 8.0,
            line_height: 20.0,
        }
    }
}

impl Theme {
    pub fn from_config(config: &crate::config::ThemeConfig) -> Self {
        Self {
            bg_color: config.background_color,
            text_color: config.text_color,
            badge_pr: config.badge_colors.pr,
            badge_issue: config.badge_colors.issue,
            badge_push: config.badge_colors.push,
            badge_release: config.badge_colors.release,
            badge_fork: config.badge_colors.fork,
            badge_create: config.badge_colors.create,
            badge_other: config.badge_colors.other,
            font_size: config.font_size,
            padding: 12.0,
            corner_radius: 8.0,
            line_height: config.font_size * 1.4,
        }
    }

    pub fn badge_color_for_event_type(&self, event_type: &EventType) -> [f32; 4] {
        match event_type {
            EventType::PullRequest
            | EventType::PullRequestReview
            | EventType::PullRequestReviewComment => self.badge_pr,
            EventType::Issues | EventType::IssueComment => self.badge_issue,
            EventType::Push | EventType::CommitComment => self.badge_push,
            EventType::Release => self.badge_release,
            EventType::Fork => self.badge_fork,
            EventType::Create | EventType::Delete | EventType::Public => self.badge_create,
            _ => self.badge_other,
        }
    }

}
