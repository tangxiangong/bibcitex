//! Desktop path services, shared string catalogs and shortcut descriptions.
use super::catalog::Catalog;
use nsh_core::{
    element::{open_path, pick_paths, reveal_path},
    file_dialog::{FileFilter, OpenDialog},
    prelude::*,
};
use std::path::{Path, PathBuf};

/// File selection, bibliography opening, and BibTeX attachment resolution.
#[module]
pub struct Files;

#[module]
impl Files {
    /// Opens the native .bib picker. Returns an absolute path, or null on cancellation.
    pub async fn pick_bibliography() -> Result<Option<String>, String> {
        let selected =
            pick_paths(OpenDialog::files().filter(FileFilter::new("BibTeX", ["bib"]))).await?;
        selected
            .into_iter()
            .flatten()
            .next()
            .map(bibliography_path)
            .transpose()
    }
    /// Opens the original bibliography belonging to a stable library identifier.
    pub async fn open_library(uid: String) -> Result<(), String> {
        let library = Catalog::libraries()
            .await?
            .into_iter()
            .find(|entry| entry.uid == uid)
            .ok_or_else(|| "Library not found".to_owned())?;
        open_path(PathBuf::from(library.path)).await
    }
    /// Reveals a local path in the file manager, or opens it if reveal is unavailable.
    pub async fn reveal_or_open_path(path: String) -> Result<(), String> {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err("Invalid bibliography path".into());
        }
        if reveal_path(path.clone()).await.is_err() {
            open_path(path).await?;
        }
        Ok(())
    }
    /// Resolves an attachment field, file URI or relative path against the bibliography file.
    pub fn resolve_attachment(library_path: String, raw_file: String) -> Result<String, String> {
        let path = crate::files::attachment_path(
            &raw_file,
            Path::new(&library_path),
            dirs::home_dir().as_deref(),
        )?;
        path.into_os_string()
            .into_string()
            .map_err(|_| "Invalid bibliography path".into())
    }
}

fn bibliography_path(path: PathBuf) -> Result<String, String> {
    if !path.is_absolute()
        || !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bib"))
    {
        return Err("Invalid bibliography path".into());
    }
    path.into_os_string()
        .into_string()
        .map_err(|_| "Invalid bibliography path".into())
}

#[cfg(any(target_os = "linux", test))]
fn supported_locale(locale: &str) -> &'static str {
    let locale = locale
        .split(['.', '@'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .replace('_', "-");
    if locale == "zh"
        || locale.starts_with("zh-hans")
        || matches!(locale.as_str(), "zh-cn" | "zh-sg")
    {
        "zh-Hans"
    } else {
        "en"
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn resolved_language(pref: &str) -> String {
    match pref {
        "en" | "zh-Hans" => pref.to_owned(),
        _ => supported_locale(&sys_locale::get_locale().unwrap_or_default()).to_owned(),
    }
}

/// Normalized global shortcut and its platform-specific display label.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct ShortcutDto {
    /// Canonical key combination suitable for persistence and registration.
    pub value: String,
    /// Human-readable label for the native platform.
    pub display_value: String,
}

/// Validates helper shortcuts and provides the native default binding.
#[module]
pub struct Shortcut;

#[module]
impl Shortcut {
    /// Validates a recorded binding and normalizes its key and modifier names.
    pub fn parse(spec: String) -> Result<ShortcutDto, String> {
        let shortcut = crate::shortcut::Shortcut::parse(&spec)?;
        Ok(ShortcutDto {
            display_value: shortcut.display_value(),
            value: shortcut.value,
        })
    }
    /// The platform's initial global shortcut before customization.
    pub fn default_value() -> String {
        crate::shortcut::DEFAULT_SHORTCUT.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_locale_rules_match_supported_languages() {
        assert_eq!(supported_locale("zh_CN.UTF-8"), "zh-Hans");
        assert_eq!(supported_locale("zh-Hant-TW"), "en");
        assert_eq!(supported_locale("zh-Hans-SG"), "zh-Hans");
    }
    #[test]
    fn selected_bibliography_must_be_absolute_and_match_extension() {
        let path = std::env::temp_dir().join("引用.BIB");
        assert_eq!(
            bibliography_path(path.clone()).unwrap(),
            path.to_str().unwrap()
        );
        assert!(bibliography_path("relative.bib".into()).is_err());
        assert!(bibliography_path(path.with_extension("pdf")).is_err());
    }
}
