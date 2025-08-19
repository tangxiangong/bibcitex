pub mod config;
mod error;
pub use error::*;
mod updater;
pub use updater::*;
pub mod github;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
pub use github::*;
pub mod utils;
pub use utils::*;
