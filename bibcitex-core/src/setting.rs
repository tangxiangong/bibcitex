use crate::{Error, Result, bib::parse};
use biblatex::Bibliography;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Write, path::PathBuf};

/// Setting for BibCiTeX
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Setting {
    /// List of bibliographies
    pub bibliographies: BTreeMap<String, PathBuf>,
}

impl Setting {
    /// Load the setting from the config file
    ///
    /// # NOTE
    ///
    /// - The config file is located at the default config directory, see [`Self::config_file_path`]
    /// - If the config file does not exist or fails to load and/or parse, the default setting will be used and saved to the config file.
    pub fn load() -> Self {
        let path = Self::config_file_path();
        let use_and_save_default = || {
            let default_setting = Self::default();
            let _ = default_setting.update_file();
            // TODO: handle error
            default_setting
        };
        if path.exists() {
            if let Ok(cfg) = config::Config::builder()
                .add_source(config::File::with_name(path.as_os_str().to_str().unwrap()))
                .build()
            {
                cfg.try_deserialize::<Setting>()
                    .unwrap_or_else(|_| use_and_save_default())
            } else {
                use_and_save_default()
            }
        } else {
            use_and_save_default()
        }
    }

    /// Override the config file or create file if not exists.
    pub fn update_file(&self) -> Result<()> {
        let path = Self::config_file_path();
        let mut file = if path.exists() {
            fs::File::options().write(true).truncate(true).open(path)?
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::File::create(path)?
        };
        let setting = serde_json::to_string_pretty(self)?;
        file.write_all(setting.as_bytes())?;
        Ok(())
    }

    /// The config file is located at the default config directory, e.g., `$XDG_CONFIG_HOME` or `$HOME/.config` for Linux, `$HOME/Library/Application Support` for macOS, and `{FOLDERID_RoamingAppData}` for Windows
    pub fn config_file_path() -> PathBuf {
        dirs::config_dir()
            .unwrap()
            .join("BibCiTeX")
            .join("setting.json")
    }

    pub fn delete() -> Result<()> {
        let path = Self::config_file_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Add or update a bibliography
    ///
    /// If the name already exists, the old value will be returned, otherwise `None` will be returned.
    pub fn add_update_bibliography(
        &mut self,
        name: &str,
        path: PathBuf,
    ) -> Result<Option<PathBuf>> {
        if !path.exists() {
            return Err(Error::BibNotFound(
                path.as_os_str().to_str().unwrap().to_string(),
            ));
        }
        Ok(self.bibliographies.insert(name.to_string(), path))
    }

    /// Remove a bibliography
    ///
    /// If the name does not exist, `None` will be returned.
    pub fn remove_bibliography(&mut self, name: &str) -> Option<PathBuf> {
        self.bibliographies.remove(name)
    }

    pub fn parse(&self, name: &str) -> Result<Bibliography> {
        let path = self
            .bibliographies
            .get(name)
            .ok_or(Error::BibNotFound(name.to_string()))?;
        parse(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let setting = Setting::load();
        assert!(setting.bibliographies.is_empty());
        Setting::delete().unwrap();
    }

    #[test]
    fn test_add_update_bibliography() {
        let mut setting = Setting::default();
        setting
            .add_update_bibliography("test", "../database.bib".into())
            .unwrap();
        assert!(setting.bibliographies.contains_key("test"));
        assert_eq!(
            setting.bibliographies.get("test"),
            Some(&"../database.bib".into())
        );
    }

    #[test]
    fn test_update() {
        let mut setting = Setting::load();
        setting
            .add_update_bibliography(
                "test",
                "/Users/xiaoyu/Projects/bibcitex/bibcitex-core/database.bib".into(),
            )
            .unwrap();
        setting.update_file().unwrap();
        let reload_setting = Setting::load();
        assert_eq!(setting, reload_setting);
    }
}
