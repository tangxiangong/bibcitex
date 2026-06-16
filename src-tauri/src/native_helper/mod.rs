pub mod bridge;
pub mod ffi;
pub mod state;

use chrono::Local;

#[derive(Copy, Clone)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub fn current_timestamp() -> String {
    Local::now().to_string()
}
