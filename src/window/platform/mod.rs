pub mod linux;
pub mod macos;
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::{set_always_on_top, set_click_through};

#[cfg(target_os = "macos")]
pub use macos::{set_always_on_top, set_click_through};

#[cfg(target_os = "windows")]
pub use windows::{set_always_on_top, set_click_through};
