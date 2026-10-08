use std::{collections::HashMap, io::Write, path::PathBuf, sync::OnceLock};

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, SharedString, WindowAppearance};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub sidebar: bool,
    pub inspector: bool,
    pub library: Option<String>,
    pub tray_library: Option<String>,
    pub language: String,
    pub appearance: String,
    pub update_channel: String,
    pub automatic_updates: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            sidebar: true,
            inspector: false,
            library: None,
            tray_library: None,
            language: "system".into(),
            appearance: "system".into(),
            update_channel: "stable".into(),
            automatic_updates: false,
        }
    }
}

impl Preferences {
    fn path() -> Result<PathBuf, String> {
        dirs::config_dir()
            .map(|p| p.join("bibcitex/linux.json"))
            .ok_or_else(|| "Configuration directory unavailable".into())
    }

    pub fn load() -> Result<Self, String> {
        match std::fs::read(Self::path()?) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path()?;
        let parent = path.parent().ok_or("Invalid configuration path")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn chinese(&self) -> bool {
        let locale = if self.language == "system" {
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .iter()
                .filter_map(|key| std::env::var(key).ok())
                .find(|s| !s.is_empty())
                .unwrap_or_default()
        } else {
            self.language.clone()
        };
        let locale = locale.to_lowercase().replace('_', "-");
        locale == "zh"
            || locale.starts_with("zh-hans")
            || locale.starts_with("zh-cn")
            || locale.starts_with("zh-sg")
    }

    pub fn text(&self, key: &str) -> SharedString {
        static EN: OnceLock<HashMap<String, String>> = OnceLock::new();
        static ZH: OnceLock<HashMap<String, String>> = OnceLock::new();
        let en = EN.get_or_init(|| {
            serde_json::from_str(include_str!("../../localization/en.json")).unwrap_or_default()
        });
        let zh = ZH.get_or_init(|| {
            serde_json::from_str(include_str!("../../localization/zh-Hans.json"))
                .unwrap_or_default()
        });
        let catalog = if self.chinese() { zh } else { en };
        catalog
            .get(key)
            .or_else(|| en.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_owned())
            .into()
    }

    pub fn apply_theme(&self, cx: &mut App) {
        let mode = match self.appearance.as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => match cx.window_appearance() {
                WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeMode::Dark,
                _ => ThemeMode::Light,
            },
        };
        Theme::change(mode, None, cx);
    }
}
