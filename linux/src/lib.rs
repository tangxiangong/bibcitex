//! Linux desktop integration, installation and update policy.
#[cfg(target_os = "linux")]
pub mod desktop;
pub mod files;
pub mod instance;
pub mod updates;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Command {
    Main,
    Helper,
    HelperToken(String),
    MainToken(String),
    Tray,
    #[serde(skip)]
    TrayAt {
        x: i32,
        y: i32,
    },
    #[serde(skip)]
    CheckUpdates,
    Settings,
    #[serde(skip)]
    About,
    Quit,
    #[serde(skip)]
    RefreshLibraries,
    #[serde(skip)]
    Error(String),
    #[serde(skip)]
    TrayAvailable(bool),
    #[serde(skip)]
    PasteFailed {
        error: String,
        key: String,
    },
}
