use winit::window::Window;

pub fn set_click_through(window: &Window, click_through: bool) {
    let _ = window.set_cursor_hittest(!click_through);
}

pub fn set_always_on_top(window: &Window, always_on_top: bool) {
    window.set_window_level(if always_on_top {
        winit::window::WindowLevel::AlwaysOnTop
    } else {
        winit::window::WindowLevel::Normal
    });
}
