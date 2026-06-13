use crate::render::shapes::ShapeRenderer;
use crate::render::text::TextSegment;

const MENU_WIDTH: f32 = 180.0;
const MENU_ITEM_HEIGHT: f32 = 32.0;
const MENU_CORNER_RADIUS: f32 = 6.0;
const MENU_PADDING: f32 = 4.0;
const MENU_ITEM_PADDING_H: f32 = 12.0;
const MENU_FONT_SIZE: f32 = 13.0;

#[derive(Clone, Copy)]
pub enum ContextAction {
    OpenSettings,
    ToggleDemo,
    ToggleNotifications,
    RefreshNow,
    Exit,
}

pub struct ContextMenu {
    visible: bool,
    x: f32,
    y: f32,
    items: Vec<ContextMenuItem>,
    width: f32,
    item_height: f32,
}

struct ContextMenuItem {
    label: String,
    action: ContextAction,
}

impl ContextMenu {
    pub fn new() -> Self {
        Self::with_notifications_enabled(false)
    }

    pub fn with_notifications_enabled(notifications_enabled: bool) -> Self {
        let notif_label = if notifications_enabled {
            "Disable Notifications"
        } else {
            "Enable Notifications"
        };
        let items = vec![
            ContextMenuItem {
                label: "Settings".to_string(),
                action: ContextAction::OpenSettings,
            },
            ContextMenuItem {
                label: "Demo Mode".to_string(),
                action: ContextAction::ToggleDemo,
            },
            ContextMenuItem {
                label: notif_label.to_string(),
                action: ContextAction::ToggleNotifications,
            },
            ContextMenuItem {
                label: "Refresh Now".to_string(),
                action: ContextAction::RefreshNow,
            },
            ContextMenuItem {
                label: "Exit".to_string(),
                action: ContextAction::Exit,
            },
        ];

        Self {
            visible: false,
            x: 0.0,
            y: 0.0,
            items,
            width: MENU_WIDTH,
            item_height: MENU_ITEM_HEIGHT,
        }
    }

    pub fn set_notifications_enabled(&mut self, enabled: bool) {
        let label = if enabled {
            "Disable Notifications"
        } else {
            "Enable Notifications"
        };
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|i| matches!(i.action, ContextAction::ToggleNotifications))
        {
            item.label = label.to_string();
        }
    }

    pub fn show(&mut self, x: f32, y: f32) {
        self.visible = true;
        self.x = x;
        self.y = y;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<ContextAction> {
        if !self.visible {
            return None;
        }

        let items = self.layout();
        for (i, (ix, iy, iw, ih, _)) in items.iter().enumerate() {
            if x >= *ix && x <= *ix + *iw && y >= *iy && y <= *iy + *ih {
                return self.items.get(i).map(|item| item.action);
            }
        }
        None
    }

    pub fn layout(&self) -> Vec<(f32, f32, f32, f32, &str)> {
        let mut result = Vec::with_capacity(self.items.len());
        let mut y = self.y + MENU_PADDING;

        for item in &self.items {
            result.push((
                self.x + MENU_PADDING,
                y,
                self.width - MENU_PADDING * 2.0,
                self.item_height,
                item.label.as_str(),
            ));
            y += self.item_height;
        }

        result
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

        let total_height = self.items.len() as f32 * self.item_height + MENU_PADDING * 2.0;

        let mut bg_color = [0.12, 0.12, 0.15, 0.95];
        bg_color[3] *= opacity;

        shape_renderer.push_rounded_rect(
            self.x,
            self.y,
            self.width,
            total_height,
            MENU_CORNER_RADIUS,
            bg_color,
            8,
            screen_w,
            screen_h,
        );

        let mut border_color = [0.3, 0.3, 0.35, 0.6];
        border_color[3] *= opacity;

        shape_renderer.push_rounded_rect(
            self.x + 0.5,
            self.y + 0.5,
            self.width - 1.0,
            total_height - 1.0,
            MENU_CORNER_RADIUS,
            border_color,
            8,
            screen_w,
            screen_h,
        );

        let mut inner_bg = [0.12, 0.12, 0.15, 0.95];
        inner_bg[3] *= opacity;

        shape_renderer.push_rounded_rect(
            self.x + 1.0,
            self.y + 1.0,
            self.width - 2.0,
            total_height - 2.0,
            MENU_CORNER_RADIUS - 1.0,
            inner_bg,
            8,
            screen_w,
            screen_h,
        );

        let items = self.layout();
        for (ix, iy, iw, ih, label) in items {
            let mut item_bg = [0.2, 0.2, 0.25, 0.4];
            item_bg[3] *= opacity;

            shape_renderer.push_rounded_rect(ix, iy, iw, ih, 4.0, item_bg, 4, screen_w, screen_h);

            let mut text_color = [0.92, 0.92, 0.94, 1.0];
            text_color[3] *= opacity;

            text_segments.push(TextSegment {
                text: label.to_string(),
                x: ix + MENU_ITEM_PADDING_H,
                y: iy + (ih - MENU_FONT_SIZE) / 2.0,
                font_size: MENU_FONT_SIZE,
                color: text_color,
                max_width: Some(iw - MENU_ITEM_PADDING_H * 2.0),
            });
        }
    }
}
