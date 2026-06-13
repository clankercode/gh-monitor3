use crate::config::Config;
use crate::render::shapes::ShapeRenderer;
use crate::render::text::TextSegment;

const PANEL_WIDTH: f32 = 300.0;
const PANEL_PADDING: f32 = 12.0;
const FIELD_HEIGHT: f32 = 28.0;
const FIELD_GAP: f32 = 6.0;
const CORNER_RADIUS: f32 = 8.0;
const FONT_SIZE: f32 = 13.0;
const LABEL_FONT_SIZE: f32 = 11.0;
const BUTTON_WIDTH: f32 = 80.0;
const REMOVE_BUTTON_WIDTH: f32 = 24.0;

#[derive(Clone)]
pub enum SettingsFieldType {
    Token,
    RepoOwner(usize),
    RepoName(usize),
    Org(usize),
    PollInterval,
    CloseButton,
    AddRepoButton,
    AddOrgButton,
    RemoveRepoButton(usize),
    RemoveOrgButton(usize),
}

struct SettingsField {
    label: String,
    value: String,
    #[allow(dead_code)]
    editable: bool,
    field_type: SettingsFieldType,
}

pub struct SettingsPanel {
    visible: bool,
    #[allow(dead_code)]
    scroll_offset: f32,
    fields: Vec<SettingsField>,
    #[allow(dead_code)]
    selected_field: Option<usize>,
    whoami_name: Option<String>,
}

impl SettingsPanel {
    pub fn new(config: &Config) -> Self {
        let mut panel = Self {
            visible: false,
            scroll_offset: 0.0,
            fields: Vec::new(),
            selected_field: None,
            whoami_name: None,
        };
        panel.rebuild_fields(config);
        panel
    }

    fn rebuild_fields(&mut self, config: &Config) {
        self.fields.clear();

        let masked_token = match &config.github_token {
            Some(t) if t.len() > 8 => format!("{}***", &t[..8]),
            Some(t) => format!("{}***", t),
            None => "(none)".to_string(),
        };
        self.fields.push(SettingsField {
            label: "Token".to_string(),
            value: masked_token,
            editable: false,
            field_type: SettingsFieldType::Token,
        });

        self.fields.push(SettingsField {
            label: "Poll Interval (s)".to_string(),
            value: config.poll_interval_secs.to_string(),
            editable: false,
            field_type: SettingsFieldType::PollInterval,
        });

        for (i, repo) in config.repos.iter().enumerate() {
            self.fields.push(SettingsField {
                label: format!("Repo Owner [{}]", i),
                value: repo.owner.clone(),
                editable: false,
                field_type: SettingsFieldType::RepoOwner(i),
            });
            self.fields.push(SettingsField {
                label: format!("Repo Name [{}]", i),
                value: repo.name.clone(),
                editable: false,
                field_type: SettingsFieldType::RepoName(i),
            });
        }

        for (i, org) in config.orgs.iter().enumerate() {
            self.fields.push(SettingsField {
                label: format!("Org [{}]", i),
                value: org.clone(),
                editable: false,
                field_type: SettingsFieldType::Org(i),
            });
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_whoami(&mut self, name: Option<String>) {
        self.whoami_name = name;
    }

    fn panel_x(&self, screen_w: f32) -> f32 {
        (screen_w - PANEL_WIDTH) / 2.0
    }

    fn panel_y(&self) -> f32 {
        40.0
    }

    fn panel_height(&self) -> f32 {
        let field_count = self.fields.len() as f32;
        let button_rows = 3.0;
        let whoami_height = if self.whoami_name.is_some() {
            FIELD_HEIGHT + FIELD_GAP
        } else {
            0.0
        };
        whoami_height
            + field_count * (FIELD_HEIGHT + FIELD_GAP)
            + button_rows * (FIELD_HEIGHT + FIELD_GAP)
            + PANEL_PADDING * 2.0
    }

    pub fn hit_test(&self, x: f32, y: f32, screen_w: f32) -> Option<SettingsFieldType> {
        if !self.visible {
            return None;
        }

        let px = self.panel_x(screen_w);
        let py = self.panel_y();
        let pw = PANEL_WIDTH;
        let ph = self.panel_height();

        if x < px || x > px + pw || y < py || y > py + ph {
            return None;
        }

        let mut cy = py + PANEL_PADDING;

        if self.whoami_name.is_some() {
            cy += FIELD_HEIGHT + FIELD_GAP;
        }

        for field in &self.fields {
            if y >= cy
                && y <= cy + FIELD_HEIGHT
                && matches!(
                    field.field_type,
                    SettingsFieldType::RepoOwner(_)
                        | SettingsFieldType::RepoName(_)
                        | SettingsFieldType::Org(_)
                )
            {
                let remove_x = px + pw - PANEL_PADDING - REMOVE_BUTTON_WIDTH;
                if x >= remove_x && x <= remove_x + REMOVE_BUTTON_WIDTH {
                    let idx = match &field.field_type {
                        SettingsFieldType::RepoOwner(i) => *i,
                        SettingsFieldType::RepoName(i) => *i,
                        SettingsFieldType::Org(i) => *i,
                        _ => unreachable!(),
                    };
                    if matches!(field.field_type, SettingsFieldType::Org(_)) {
                        return Some(SettingsFieldType::RemoveOrgButton(idx));
                    } else {
                        return Some(SettingsFieldType::RemoveRepoButton(idx));
                    }
                }
            }
            cy += FIELD_HEIGHT + FIELD_GAP;
        }

        cy += FIELD_GAP;

        let btn_w = BUTTON_WIDTH;
        let btn_x = px + PANEL_PADDING;

        if y >= cy && y <= cy + FIELD_HEIGHT && x >= btn_x && x <= btn_x + btn_w {
            return Some(SettingsFieldType::AddRepoButton);
        }
        cy += FIELD_HEIGHT + FIELD_GAP;

        if y >= cy && y <= cy + FIELD_HEIGHT && x >= btn_x && x <= btn_x + btn_w {
            return Some(SettingsFieldType::AddOrgButton);
        }
        cy += FIELD_HEIGHT + FIELD_GAP;

        if y >= cy && y <= cy + FIELD_HEIGHT && x >= btn_x && x <= btn_x + btn_w {
            return Some(SettingsFieldType::CloseButton);
        }

        None
    }

    pub fn apply_field_click(&mut self, field_type: &SettingsFieldType, config: &mut Config) {
        match field_type {
            SettingsFieldType::CloseButton => {
                self.visible = false;
            }
            SettingsFieldType::AddRepoButton => {
                config.repos.push(crate::config::RepoConfig {
                    owner: "owner".to_string(),
                    name: "repo".to_string(),
                });
                let _ = config.save(None);
                self.rebuild_fields(config);
            }
            SettingsFieldType::AddOrgButton => {
                config.orgs.push("org".to_string());
                let _ = config.save(None);
                self.rebuild_fields(config);
            }
            SettingsFieldType::RemoveRepoButton(i) => {
                if *i < config.repos.len() {
                    config.repos.remove(*i);
                    let _ = config.save(None);
                    self.rebuild_fields(config);
                }
            }
            SettingsFieldType::RemoveOrgButton(i) => {
                if *i < config.orgs.len() {
                    config.orgs.remove(*i);
                    let _ = config.save(None);
                    self.rebuild_fields(config);
                }
            }
            _ => {}
        }
    }

    pub fn render(
        &self,
        shape_renderer: &mut ShapeRenderer,
        text_segments: &mut Vec<TextSegment>,
        opacity: f32,
        screen_w: f32,
        screen_h: f32,
    ) {
        if !self.visible {
            return;
        }

        let px = self.panel_x(screen_w);
        let py = self.panel_y();
        let pw = PANEL_WIDTH;
        let ph = self.panel_height();

        let mut bg = [0.10, 0.10, 0.13, 0.96];
        bg[3] *= opacity;

        shape_renderer.push_rounded_rect(px, py, pw, ph, CORNER_RADIUS, bg, 8, screen_w, screen_h);

        let mut border = [0.3, 0.3, 0.35, 0.5];
        border[3] *= opacity;
        shape_renderer.push_rounded_rect(
            px + 0.5,
            py + 0.5,
            pw - 1.0,
            ph - 1.0,
            CORNER_RADIUS,
            border,
            8,
            screen_w,
            screen_h,
        );

        let mut inner_bg = [0.10, 0.10, 0.13, 0.96];
        inner_bg[3] *= opacity;
        shape_renderer.push_rounded_rect(
            px + 1.0,
            py + 1.0,
            pw - 2.0,
            ph - 2.0,
            CORNER_RADIUS - 1.0,
            inner_bg,
            8,
            screen_w,
            screen_h,
        );

        let mut cy = py + PANEL_PADDING;

        let mut title_color = [0.85, 0.85, 0.90, 1.0];
        title_color[3] *= opacity;
        text_segments.push(TextSegment {
            text: "Settings".to_string(),
            x: px + PANEL_PADDING,
            y: cy,
            font_size: FONT_SIZE + 2.0,
            color: title_color,
            max_width: Some(pw - PANEL_PADDING * 2.0),
        });
        cy += FIELD_HEIGHT + FIELD_GAP;

        if let Some(ref name) = self.whoami_name {
            let mut whoami_color = [0.6, 0.85, 0.7, 1.0];
            whoami_color[3] *= opacity;
            text_segments.push(TextSegment {
                text: format!("User: {}", name),
                x: px + PANEL_PADDING,
                y: cy,
                font_size: FONT_SIZE,
                color: whoami_color,
                max_width: Some(pw - PANEL_PADDING * 2.0),
            });
            cy += FIELD_HEIGHT + FIELD_GAP;
        }

        for field in &self.fields {
            let mut label_color = [0.6, 0.6, 0.65, 1.0];
            label_color[3] *= opacity;
            text_segments.push(TextSegment {
                text: field.label.clone(),
                x: px + PANEL_PADDING,
                y: cy + 2.0,
                font_size: LABEL_FONT_SIZE,
                color: label_color,
                max_width: Some(pw * 0.4),
            });

            let mut value_color = [0.9, 0.9, 0.92, 1.0];
            value_color[3] *= opacity;
            let label_w = field.label.len() as f32 * (LABEL_FONT_SIZE * 0.55) + 8.0;
            text_segments.push(TextSegment {
                text: field.value.clone(),
                x: px + PANEL_PADDING + label_w,
                y: cy + 2.0,
                font_size: FONT_SIZE,
                color: value_color,
                max_width: Some(pw - PANEL_PADDING * 2.0 - label_w - REMOVE_BUTTON_WIDTH - 4.0),
            });

            if matches!(
                field.field_type,
                SettingsFieldType::RepoOwner(_)
                    | SettingsFieldType::RepoName(_)
                    | SettingsFieldType::Org(_)
            ) {
                let remove_x = px + pw - PANEL_PADDING - REMOVE_BUTTON_WIDTH;
                let mut remove_bg = [0.6, 0.2, 0.2, 0.6];
                remove_bg[3] *= opacity;
                shape_renderer.push_rounded_rect(
                    remove_x,
                    cy + 2.0,
                    REMOVE_BUTTON_WIDTH,
                    FIELD_HEIGHT - 4.0,
                    4.0,
                    remove_bg,
                    4,
                    screen_w,
                    screen_h,
                );
                let mut remove_text_color = [0.95, 0.9, 0.9, 1.0];
                remove_text_color[3] *= opacity;
                text_segments.push(TextSegment {
                    text: "\u{00d7}".to_string(),
                    x: remove_x + 6.0,
                    y: cy + 4.0,
                    font_size: FONT_SIZE,
                    color: remove_text_color,
                    max_width: Some(REMOVE_BUTTON_WIDTH),
                });
            }

            cy += FIELD_HEIGHT + FIELD_GAP;
        }

        cy += FIELD_GAP;

        let btn_x = px + PANEL_PADDING;
        self.render_button(
            shape_renderer,
            text_segments,
            btn_x,
            cy,
            "Add Repo",
            opacity,
            screen_w,
            screen_h,
        );
        cy += FIELD_HEIGHT + FIELD_GAP;

        self.render_button(
            shape_renderer,
            text_segments,
            btn_x,
            cy,
            "Add Org",
            opacity,
            screen_w,
            screen_h,
        );
        cy += FIELD_HEIGHT + FIELD_GAP;

        self.render_button(
            shape_renderer,
            text_segments,
            btn_x,
            cy,
            "Close",
            opacity,
            screen_w,
            screen_h,
        );
    }

    fn render_button(
        &self,
        shape_renderer: &mut ShapeRenderer,
        text_segments: &mut Vec<TextSegment>,
        x: f32,
        y: f32,
        label: &str,
        opacity: f32,
        screen_w: f32,
        screen_h: f32,
    ) {
        let mut btn_bg = [0.22, 0.22, 0.28, 0.7];
        btn_bg[3] *= opacity;
        shape_renderer.push_rounded_rect(
            x,
            y,
            BUTTON_WIDTH,
            FIELD_HEIGHT,
            4.0,
            btn_bg,
            4,
            screen_w,
            screen_h,
        );

        let mut btn_text = [0.92, 0.92, 0.94, 1.0];
        btn_text[3] *= opacity;
        text_segments.push(TextSegment {
            text: label.to_string(),
            x: x + 10.0,
            y: y + (FIELD_HEIGHT - FONT_SIZE) / 2.0,
            font_size: FONT_SIZE,
            color: btn_text,
            max_width: Some(BUTTON_WIDTH - 20.0),
        });
    }
}
