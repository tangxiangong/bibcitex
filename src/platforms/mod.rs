pub static MAIN_WINDOW_TITLE: &str = "BibCiTeX";
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::paste::{focus_previous_window, observe_app};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::paste::{focus_previous_window, observe_app};
