use winit::window::Window;

pub struct OverlayConfig {
    pub opacity: f32,
    pub hover_opacity: f32,
    pub click_through: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            opacity: 0.15,
            hover_opacity: 0.95,
            click_through: true,
        }
    }
}

pub fn apply_overlay_settings(window: &Window, config: &OverlayConfig) {
    window.set_transparent(true);
    super::platform::set_click_through(window, config.click_through);
    super::platform::set_always_on_top(window, true);
}
