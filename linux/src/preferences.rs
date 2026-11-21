use nsh_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub revision: u64,
    pub sidebar: bool,
    pub inspector: bool,
    pub library: Option<String>,
    pub tray_library: Option<String>,
    pub language: String,
    pub appearance: String,
    pub update_channel: String,
    pub automatic_updates: bool,
    pub tray_icon_enabled: bool,
    pub helper_shortcut: String,
    #[serde(rename = "citationFormat")]
    pub citation_format: String,
    #[serde(rename = "citationMode")]
    pub citation_mode: String,
    #[serde(
        rename = "mainAdvancedSearchEnabled",
        deserialize_with = "boolean_preference"
    )]
    /// Main search mode only; per-mode query drafts stay in memory.
    pub main_advanced_search_enabled: bool,
    #[serde(
        rename = "trayAdvancedSearchEnabled",
        deserialize_with = "boolean_preference"
    )]
    /// Tray search mode, persisted independently from the main window.
    pub tray_advanced_search_enabled: bool,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

fn boolean_preference<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(serde_json::Value::deserialize(deserializer)?
        .as_bool()
        .unwrap_or(false))
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            revision: 0,
            sidebar: true,
            inspector: false,
            library: None,
            tray_library: None,
            language: "system".into(),
            appearance: "system".into(),
            update_channel: "stable".into(),
            automatic_updates: false,
            tray_icon_enabled: true,
            helper_shortcut: crate::shortcut::DEFAULT_SHORTCUT.into(),
            citation_format: "raw".into(),
            citation_mode: "merged".into(),
            main_advanced_search_enabled: false,
            tray_advanced_search_enabled: false,
            extra: HashMap::new(),
        }
    }
}

impl Preferences {
    fn path() -> Result<PathBuf, String> {
        dirs::config_dir()
            .map(|p| {
                p.join(if cfg!(target_os = "linux") {
                    "bibcitex/linux.json"
                } else {
                    "bibcitex/nsh.json"
                })
            })
            .ok_or_else(|| "Configuration directory unavailable".into())
    }

    pub fn load() -> Result<Self, String> {
        Self::load_at(&Self::path()?)
    }

    fn load_at(path: &Path) -> Result<Self, String> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn save_next(&mut self, expected: u64) -> Result<(), String> {
        self.save_next_at(&Self::path()?, expected)
    }

    fn save_next_at(&mut self, path: &Path, expected: u64) -> Result<(), String> {
        let parent = path.parent().ok_or("Invalid configuration path")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        // Lock a stable sibling: replacing the JSON changes its inode.
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.with_extension("lock"))
            .map_err(|e| e.to_string())?;
        File::lock(&lock).map_err(|e| e.to_string())?;
        let current = Self::load_at(path)?;
        if current.revision != expected {
            return Err("Preferences changed; reload before saving".into());
        }
        self.revision = expected
            .checked_add(1)
            .ok_or("Preferences revision overflow")?;
        self.extra = current.extra;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Persisted workbench preferences shared by every window.
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct PreferencesDto {
    /// Decimal revision required for the next save.
    pub revision: String,
    /// Show the library sidebar.
    pub sidebar: bool,
    /// Show the reference inspector.
    pub inspector: bool,
    /// Selected main workbench library.
    pub library: Option<String>,
    /// Selected tray workbench library.
    pub tray_library: Option<String>,
    /// Preferred language: system, zh-Hans or en.
    pub language: String,
    /// Preferred theme: system, light or dark.
    pub appearance: String,
    /// Selected stable, beta or alpha update channel.
    pub update_channel: String,
    /// Enable automatic installation of eligible updates.
    pub automatic_updates: bool,
    /// Show the tray icon.
    pub tray_icon_enabled: bool,
    /// Canonical global helper accelerator.
    pub helper_shortcut: String,
    /// Shared citation format identifier.
    pub citation_format: String,
    /// Citation combination mode: merged or individual.
    pub citation_mode: String,
    /// Use advanced search in the main workbench.
    pub main_advanced_search_enabled: bool,
    /// Use advanced search in the tray workbench.
    pub tray_advanced_search_enabled: bool,
}

/// A partial change; absent fields retain their saved values.
#[derive(Clone, Debug, Default, PartialEq, JsObject)]
pub struct PreferencePatch {
    /// Show the library sidebar.
    pub sidebar: Option<bool>,
    /// Show the reference inspector.
    pub inspector: Option<bool>,
    /// Selected main workbench library.
    pub library: Option<String>,
    /// Selected tray workbench library.
    pub tray_library: Option<String>,
    /// Preferred language: system, zh-Hans or en.
    pub language: Option<String>,
    /// Preferred theme: system, light or dark.
    pub appearance: Option<String>,
    /// Selected stable, beta or alpha update channel.
    pub update_channel: Option<String>,
    /// Enable automatic installation of eligible updates.
    pub automatic_updates: Option<bool>,
    /// Show the tray icon.
    pub tray_icon_enabled: Option<bool>,
    /// Canonical global helper accelerator.
    pub helper_shortcut: Option<String>,
    /// Shared citation format identifier.
    pub citation_format: Option<String>,
    /// Citation combination mode: merged or individual.
    pub citation_mode: Option<String>,
    /// Use advanced search in the main workbench.
    pub main_advanced_search_enabled: Option<bool>,
    /// Use advanced search in the tray workbench.
    pub tray_advanced_search_enabled: Option<bool>,
    /// Clear the saved main library.
    pub clear_library: Option<bool>,
    /// Clear the saved tray library.
    pub clear_tray_library: Option<bool>,
}

impl From<Preferences> for PreferencesDto {
    fn from(p: Preferences) -> Self {
        Self {
            revision: p.revision.to_string(),
            sidebar: p.sidebar,
            inspector: p.inspector,
            library: p.library,
            tray_library: p.tray_library,
            language: p.language,
            appearance: p.appearance,
            update_channel: p.update_channel,
            automatic_updates: p.automatic_updates,
            tray_icon_enabled: p.tray_icon_enabled,
            helper_shortcut: p.helper_shortcut,
            citation_format: p.citation_format,
            citation_mode: p.citation_mode,
            main_advanced_search_enabled: p.main_advanced_search_enabled,
            tray_advanced_search_enabled: p.tray_advanced_search_enabled,
        }
    }
}

impl PreferencePatch {
    pub(crate) fn apply(self, p: &mut Preferences) -> Result<(), String> {
        if self.clear_library == Some(true) && self.library.is_some()
            || self.clear_tray_library == Some(true) && self.tray_library.is_some()
        {
            return Err("Cannot select and clear the same library".into());
        }
        if self.clear_library == Some(true) {
            p.library = None;
        }
        if self.clear_tray_library == Some(true) {
            p.tray_library = None;
        }
        if let Some(value) = self.sidebar {
            p.sidebar = value;
        }
        if let Some(value) = self.inspector {
            p.inspector = value;
        }
        if let Some(value) = self.library {
            p.library = Some(value);
        }
        if let Some(value) = self.tray_library {
            p.tray_library = Some(value);
        }
        if let Some(value) = self.language {
            if !matches!(value.as_str(), "system" | "zh-Hans" | "en") {
                return Err("Invalid language".into());
            }
            p.language = value;
        }
        if let Some(value) = self.appearance {
            if !matches!(value.as_str(), "system" | "light" | "dark") {
                return Err("Invalid appearance".into());
            }
            p.appearance = value;
        }
        if let Some(value) = self.update_channel {
            if !matches!(value.as_str(), "stable" | "beta" | "alpha") {
                return Err("Invalid update_channel".into());
            }
            p.update_channel = value;
        }
        if let Some(value) = self.automatic_updates {
            p.automatic_updates = value;
        }
        if let Some(value) = self.tray_icon_enabled {
            p.tray_icon_enabled = value;
        }
        if let Some(value) = self.helper_shortcut {
            p.helper_shortcut = crate::shortcut::Shortcut::parse(&value)?.value;
        }
        if let Some(value) = self.citation_format {
            if !bibcitex_service::CITATION_FORMATS
                .iter()
                .any(|(id, _)| *id == value)
            {
                return Err("Invalid citation format".into());
            }
            p.citation_format = value;
        }
        if let Some(value) = self.citation_mode {
            if !matches!(value.as_str(), "merged" | "individual") {
                return Err("Invalid citation_mode".into());
            }
            p.citation_mode = value;
        }
        if let Some(value) = self.main_advanced_search_enabled {
            p.main_advanced_search_enabled = value;
        }
        if let Some(value) = self.tray_advanced_search_enabled {
            p.tray_advanced_search_enabled = value;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_preserve_unknown_preferences_and_check_revision_under_lock() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("linux.json");
        std::fs::write(&path, r#"{"custom":42,"mainAdvancedSearchEnabled":"old"}"#).unwrap();
        let mut original = Preferences::load_at(&path).unwrap();
        let mut stale = original.clone();
        original.appearance = "dark".into();
        original.save_next_at(&path, 0).unwrap();
        assert!(stale.save_next_at(&path, 0).is_err());
        let saved = Preferences::load_at(&path).unwrap();
        assert_eq!(saved.revision, 1);
        assert_eq!(saved.extra["custom"], 42);
        assert_eq!(saved.appearance, "dark");
        assert!(!saved.main_advanced_search_enabled);
    }

    #[test]
    fn independent_writers_cannot_both_commit_the_same_revision() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("linux.json");
        let ready = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let workers = ["light", "dark"].map(|appearance| {
                let path = &path;
                let ready = &ready;
                scope.spawn(move || {
                    let mut p = Preferences::load_at(path).unwrap();
                    ready.wait();
                    p.appearance = appearance.into();
                    p.save_next_at(path, 0)
                })
            });
            let committed = workers.map(|worker| worker.join().unwrap().is_ok());
            assert_ne!(committed[0], committed[1]);
            assert_eq!(Preferences::load_at(&path).unwrap().revision, 1);
        });
    }

    #[test]
    fn invalid_disk_preferences_are_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("linux.json");
        std::fs::write(&path, b"{").unwrap();
        assert!(Preferences::default().save_next_at(&path, 0).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{");
    }
    #[test]
    fn patch_only_changes_supplied_values_and_handles_clear() {
        let mut prefs = Preferences {
            library: Some("A".into()),
            tray_library: Some("B".into()),
            ..Default::default()
        };
        PreferencePatch {
            clear_library: Some(true),
            appearance: Some("dark".into()),
            ..Default::default()
        }
        .apply(&mut prefs)
        .unwrap();
        assert!(prefs.library.is_none());
        assert_eq!(prefs.tray_library.as_deref(), Some("B"));
        assert_eq!(prefs.appearance, "dark");
    }
}
