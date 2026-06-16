use std::fmt::Write;
use std::sync::{Mutex, OnceLock};

use crate::core::{
    bib::{Reference, parse},
    search::search_references as search_references_impl,
    setting::Setting,
};
use crate::native_helper::ffi;
use chrono::{DateTime, Local};

struct HelperBibliography {
    name: String,
    path: String,
    updated_at: String,
    description: Option<String>,
}

#[derive(Default)]
struct NativeHelperState {
    current_bibliography: Option<(String, String)>,
    bibliographies: Vec<HelperBibliography>,
    current_references: Vec<Reference>,
    last_error: Option<String>,
}

impl NativeHelperState {
    fn set_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
    }
}

static STATE: OnceLock<Mutex<NativeHelperState>> = OnceLock::new();

fn state() -> &'static Mutex<NativeHelperState> {
    STATE.get_or_init(|| Mutex::new(NativeHelperState::default()))
}

fn with_state<T>(f: impl FnOnce(&mut NativeHelperState) -> T) -> T {
    let lock = state().lock().unwrap();
    let mut guard = lock;
    f(&mut guard)
}

fn format_datetime(value: &DateTime<Local>) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "{value}");
    out.trim_end_matches('\n').to_string()
}

fn refresh_bibliographies_from_setting() -> Vec<HelperBibliography> {
    let setting = Setting::load();
    let mut result = setting
        .bibliographies
        .into_iter()
        .map(|(name, info)| HelperBibliography {
            name,
            path: info.path.to_string_lossy().to_string(),
            updated_at: format_datetime(&info.updated_at),
            description: info.description,
        })
        .collect::<Vec<_>>();
    result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    result
}

fn parse_references(path: &str) -> Result<Vec<Reference>, String> {
    let bib = parse(path).map_err(|e| format!("解析文献库失败：{e}"))?;
    let references = bib.iter().map(Reference::from).collect::<Vec<_>>();
    Ok(references)
}

pub fn list_bibliographies() -> Vec<(String, String, String, Option<String>)> {
    with_state(|state| {
        let bibliographies = refresh_bibliographies_from_setting();
        state.bibliographies = bibliographies
            .into_iter()
            .map(|item| HelperBibliography {
                name: item.name.clone(),
                path: item.path,
                updated_at: item.updated_at,
                description: item.description,
            })
            .collect::<Vec<_>>();
        state
            .bibliographies
            .iter()
            .map(|item| {
                (
                    item.name.clone(),
                    item.path.clone(),
                    item.updated_at.clone(),
                    item.description.clone(),
                )
            })
            .collect()
    })
}

pub fn list_bibliographies_ffi() -> ffi::FfiBibliographyArray {
    let values = list_bibliographies();
    ffi::bibliography_to_ffi_array(values)
}

pub fn set_current_bibliography(
    name: String,
    path: String,
) -> Result<ffi::FfiBibliography, String> {
    let references = parse_references(&path)?;
    with_state(|state| {
        state.current_bibliography = Some((name.clone(), path.clone()));
        state.current_references = references;
        state.set_error("");
        state.last_error = None;
    });

    let mut current_updated_at = String::new();
    let mut current_description = None;
    let bibliographies = list_bibliographies();
    for (existing_name, existing_path, updated_at, description) in bibliographies {
        if existing_name == name && existing_path == path {
            current_updated_at = updated_at;
            current_description = description;
            break;
        }
    }

    if current_updated_at.is_empty() {
        current_updated_at = Local::now().to_string();
    }
    Ok(ffi::to_ffi_bibliography(
        name,
        path,
        current_updated_at,
        current_description,
    ))
}

pub fn current_bibliography() -> Option<(String, String)> {
    with_state(|state| state.current_bibliography.clone())
}

pub fn current_references() -> Vec<Reference> {
    with_state(|state| state.current_references.clone())
}

pub fn search_references(query: &str) -> Vec<Reference> {
    let references = current_references();
    if query.trim().is_empty() {
        return references;
    }
    search_references_impl(&references, query)
}

pub fn get_last_error() -> Option<String> {
    with_state(|state| state.last_error.clone())
}

pub fn set_last_error(error: String) {
    with_state(|state| state.set_error(error));
}

pub fn clear_last_error() {
    with_state(|state| state.last_error = None);
}
