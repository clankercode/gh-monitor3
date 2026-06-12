use winit::event::{ElementState, MouseButton};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseState {
    pub position: (f64, f64),
    pub left_pressed: bool,
    pub is_hovered: bool,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            left_pressed: false,
            is_hovered: false,
        }
    }
}

impl MouseState {
    pub fn update_position(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }

    pub fn update_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            self.left_pressed = state == ElementState::Pressed;
        }
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn is_in_rect(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let (mx, my) = self.position;
        mx >= x as f64 && mx <= (x + w) as f64 && my >= y as f64 && my <= (y + h) as f64
    }
}
