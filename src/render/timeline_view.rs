use super::shapes::ShapeRenderer;
use super::text::{TextRenderer, TextSegment};
use super::theme::Theme;
use crate::github::events::EventType;
use crate::timeline::compression::TimelineEntry;
use crate::timeline::grouping::TimelineGroup;

const CARD_HEIGHT: f32 = 72.0;
const CARD_PADDING: f32 = 10.0;
const BADGE_HEIGHT: f32 = 20.0;
const BADGE_PADDING_H: f32 = 8.0;
const BADGE_GAP: f32 = 4.0;
const CARD_CORNER_RADIUS: f32 = 8.0;
const BADGE_CORNER_RADIUS: f32 = 4.0;

struct RenderItem {
    x: f32,
    y: f32,
    width: f32,
    repo_name: String,
    badges: Vec<(EventType, u32, [f32; 4])>,
    time_text: String,
}

pub struct TimelineView {
    shape_renderer: ShapeRenderer,
    text_renderer: TextRenderer,
    theme: Theme,
    scroll_offset: f32,
    content_height: f32,
}

impl TimelineView {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        theme: Theme,
    ) -> Self {
        let shape_renderer = ShapeRenderer::new(device, format);
        let text_renderer = TextRenderer::new(device, queue, format);

        Self {
            shape_renderer,
            text_renderer,
            theme,
            scroll_offset: 0.0,
            content_height: 0.0,
        }
    }

    fn badge_label(event_type: &EventType, count: u32) -> String {
        let label = match event_type {
            EventType::Push => "push",
            EventType::PullRequest => "PR",
            EventType::Issues => "issue",
            EventType::Create => "create",
            EventType::Delete => "delete",
            EventType::Release => "release",
            EventType::Fork => "fork",
            EventType::Watch => "watch",
            EventType::IssueComment => "comment",
            EventType::PullRequestReview => "review",
            EventType::PullRequestReviewComment => "review",
            EventType::CommitComment => "comment",
            EventType::Public => "public",
            EventType::Member => "member",
            EventType::Gollum => "wiki",
            EventType::Discussion => "discussion",
            EventType::Other(s) => s.as_str(),
        };
        if count > 1 {
            format!("{} \u{00d7}{}", label, count)
        } else {
            label.to_string()
        }
    }

    fn layout(&mut self, entries: &[TimelineEntry], width: f32, _height: f32) -> Vec<RenderItem> {
        let card_margin = 6.0;
        let card_width = width - self.theme.padding * 2.0;
        let mut items = Vec::new();
        let mut y = self.theme.padding + self.scroll_offset;

        for entry in entries {
            let (repo_name, badges, time_text) = match entry {
                TimelineEntry::Single(group) => {
                    let color = self.theme.badge_color_for_event_type(&group.event_type);
                    let badges = vec![(group.event_type.clone(), group.count, color)];
                    let time_str = Self::group_time_text(group);
                    (group.repo_name.clone(), badges, time_str)
                }
                TimelineEntry::Compressed(comp) => {
                    let badges: Vec<_> = comp
                        .items
                        .iter()
                        .map(|(et, count, _)| {
                            let color = self.theme.badge_color_for_event_type(et);
                            (et.clone(), *count, color)
                        })
                        .collect();
                    (comp.repo_name.clone(), badges, comp.time_range_str.clone())
                }
            };

            items.push(RenderItem {
                x: self.theme.padding,
                y,
                width: card_width,
                repo_name,
                badges,
                time_text,
            });

            y += CARD_HEIGHT + card_margin;
        }

        self.content_height = y - self.scroll_offset;
        items
    }

    fn group_time_text(group: &TimelineGroup) -> String {
        crate::timeline::humanize::humanize_time_range(group.earliest, group.latest)
    }

    pub fn prepare_render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        entries: &[TimelineEntry],
        _opacity: f32,
        width: f32,
        height: f32,
    ) {
        let items = self.layout(entries, width, height);

        self.shape_renderer.begin_frame();

        let mut text_segments = Vec::new();

        for item in &items {
            self.shape_renderer.push_rounded_rect(
                item.x,
                item.y,
                item.width,
                CARD_HEIGHT,
                CARD_CORNER_RADIUS,
                self.theme.bg_color,
                8,
                width,
                height,
            );

            text_segments.push(TextSegment {
                text: item.repo_name.clone(),
                x: item.x + CARD_PADDING,
                y: item.y + CARD_PADDING,
                font_size: self.theme.font_size,
                color: self.theme.text_color,
                max_width: Some(item.width - CARD_PADDING * 2.0),
            });

            let mut badge_x = item.x + CARD_PADDING;
            let badge_y = item.y + CARD_PADDING + self.theme.line_height + 4.0;

            for (event_type, count, color) in &item.badges {
                let label = Self::badge_label(event_type, *count);
                let label_width =
                    label.len() as f32 * (self.theme.font_size * 0.55) + BADGE_PADDING_H * 2.0;

                self.shape_renderer.push_rounded_rect(
                    badge_x,
                    badge_y,
                    label_width,
                    BADGE_HEIGHT,
                    BADGE_CORNER_RADIUS,
                    *color,
                    4,
                    width,
                    height,
                );

                text_segments.push(TextSegment {
                    text: label,
                    x: badge_x + BADGE_PADDING_H,
                    y: badge_y + 2.0,
                    font_size: self.theme.font_size * 0.8,
                    color: [1.0, 1.0, 1.0, 1.0],
                    max_width: Some(label_width),
                });

                badge_x += label_width + BADGE_GAP;
            }

            text_segments.push(TextSegment {
                text: item.time_text.clone(),
                x: item.x + item.width
                    - CARD_PADDING
                    - item.time_text.len() as f32 * (self.theme.font_size * 0.4),
                y: item.y + CARD_PADDING,
                font_size: self.theme.font_size * 0.85,
                color: [0.7, 0.7, 0.75, 1.0],
                max_width: Some(item.width * 0.4),
            });
        }

        self.shape_renderer.upload(device);

        self.text_renderer
            .prepare_text(device, queue, &text_segments, width as u32, height as u32);
    }

    pub fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.shape_renderer.render(render_pass);
        self.text_renderer.render(render_pass);
    }

    pub fn scroll(&mut self, delta: f32) {
        self.scroll_offset += delta;
        if self.scroll_offset > 0.0 {
            self.scroll_offset = 0.0;
        }
    }
}
