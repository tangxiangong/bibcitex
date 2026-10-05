//! Shared desktop operations, independent of either platform binding runtime.
use crate::records::{LibraryRecord, ReferenceRecord};
use crate::{
    core::{self, bib::Reference},
    registry,
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
    time::SystemTime,
};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("{message}")]
    Operation { message: String },
}
impl From<String> for CoreError {
    fn from(message: String) -> Self {
        Self::Operation { message }
    }
}
#[derive(Clone, PartialEq)]
struct Fingerprint {
    modified: Option<SystemTime>,
    length: u64,
}
struct Cached {
    fingerprint: Fingerprint,
    references: Arc<Vec<Reference>>,
}
static CACHE: LazyLock<Mutex<HashMap<PathBuf, Cached>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
fn fingerprint(path: &PathBuf) -> Result<Fingerprint, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(Fingerprint {
        modified: metadata.modified().ok(),
        length: metadata.len(),
    })
}
fn references(path: &str) -> Result<Arc<Vec<Reference>>, String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let before = fingerprint(&path)?;
    {
        let cache = CACHE.lock().map_err(|_| "Cache lock poisoned")?;
        if let Some(cached) = cache.get(&path)
            && cached.fingerprint == before
        {
            return Ok(Arc::clone(&cached.references));
        }
    }
    // Parse outside the shared lock: a large main-window library must not block the helper.
    let parsed = Arc::new(core::utils::read_bibliography(
        core::bib::parse(&path).map_err(|e| e.to_string())?,
    ));
    if fingerprint(&path)? != before {
        return Err("Bibliography changed while reading; retry search".into());
    }
    let mut cache = CACHE.lock().map_err(|_| "Cache lock poisoned")?;
    if cache.len() >= 8 {
        cache.clear();
    }
    cache.insert(
        path,
        Cached {
            fingerprint: before,
            references: Arc::clone(&parsed),
        },
    );
    Ok(parsed)
}
fn search_references(
    path: String,
    query: String,
    field: String,
    type_filter: String,
) -> Result<Vec<Reference>, String> {
    let source = references(&path)?;
    let query = query.trim();
    if !matches!(
        field.as_str(),
        "all" | "author" | "title" | "journal" | "year"
    ) {
        return Err("Unknown search field".into());
    }
    let results = if query.is_empty() {
        source.as_ref().clone()
    } else {
        match field.as_str() {
            "all" => core::search::search_references(&source, query),
            "author" => core::search::search_references_by_author(&source, query),
            "title" => core::search::search_references_by_title(&source, query),
            "journal" => core::search::search_references_by_journal(&source, query),
            "year" => core::search::search_references_by_year(&source, query),
            _ => return Err("Unknown search field".into()),
        }
    };
    let results = match type_filter.as_str() {
        "all" => results,
        "Article" => core::filter_article(results),
        "Book" => core::filter_book(results),
        "Thesis" => core::filter_thesis(results),
        "Booklet" => core::filter_booklet(results),
        "InBook" => core::filter_inbook(results),
        "InCollection" => core::filter_incollection(results),
        "InProceedings" => core::filter_inproceedings(results),
        "Misc" => core::filter_misc(results),
        "TechReport" => core::filter_techreport(results),
        _ => return Err("Unknown bibliography type filter".into()),
    };
    Ok(results)
}

/// Call during app startup before activating the main window.
pub fn initialize() {
    xpaste::observe_app();
}

pub fn libraries() -> Result<Vec<LibraryRecord>, CoreError> {
    initialize();
    serde_json::from_value(registry::libraries()?).map_err(|e| CoreError::from(e.to_string()))
}
pub fn add_library(
    name: String,
    path: String,
    description: Option<String>,
) -> Result<LibraryRecord, CoreError> {
    serde_json::from_value(registry::add(&name, &path, description)?)
        .map_err(|e| CoreError::from(e.to_string()))
}
pub fn update_library(
    name: String,
    new_name: String,
    path: Option<String>,
    description: Option<String>,
) -> Result<LibraryRecord, CoreError> {
    serde_json::from_value(registry::update(
        &name,
        &new_name,
        path.as_deref(),
        description,
    )?)
    .map_err(|e| CoreError::from(e.to_string()))
}
pub fn set_library_pinned(name: String, pinned: bool) -> Result<(), CoreError> {
    registry::set_pinned(&name, pinned)?;
    Ok(())
}
pub fn remove_library(name: String) -> Result<(), CoreError> {
    registry::remove(&name)?;
    Ok(())
}
pub fn search(
    path: String,
    query: String,
    field: String,
    type_filter: String,
) -> Result<Vec<ReferenceRecord>, CoreError> {
    search_references(path.clone(), query, field, type_filter)?
        .iter()
        .map(|reference| ReferenceRecord::from_reference(reference, &path).map_err(CoreError::from))
        .collect()
}
pub fn helper_current() -> Result<Option<LibraryRecord>, CoreError> {
    serde_json::from_value(registry::current()?).map_err(|e| CoreError::from(e.to_string()))
}
pub fn helper_select(name: String, path: String) -> Result<LibraryRecord, CoreError> {
    references(&path)?;
    serde_json::from_value(registry::select(&name, &path)?)
        .map_err(|e| CoreError::from(e.to_string()))
}
pub fn capture_paste_target() -> Result<(), CoreError> {
    xpaste::capture_paste_target().map_err(Into::into)
}
pub fn copy(text: String) -> Result<(), CoreError> {
    xpaste::copy(&text).map_err(Into::into)
}
pub fn paste(text: String) -> Result<(), CoreError> {
    xpaste::paste(&text).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clearing_field_search_keeps_records_without_that_field() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        use std::io::Write;
        file.write_all(b"@misc{noYear,title={Example}}").unwrap();
        let result = search(
            file.path().to_str().unwrap().into(),
            "  ".into(),
            "year".into(),
            "all".into(),
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entry_type, "Misc");
        assert_eq!(result[0].year, None);
    }
    fn search_file(source: &str) -> tempfile::NamedTempFile {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(source.as_bytes()).unwrap();
        file
    }
    #[test]
    fn search_fields_types_and_invalid_inputs_keep_expected_semantics() {
        let file = search_file(
            "@article{alpha,title={Alpha},author={Doe, Jane},journal={Venue},year={2024}}\n@book{beta,title={Beta},year={2025}}",
        );
        let path = file.path().to_str().unwrap().to_owned();
        for (field, query) in [
            ("all", "Alpha"),
            ("title", " Alpha "),
            ("author", "Doe"),
            ("journal", "Venue"),
            ("year", "2024"),
        ] {
            let found = search(path.clone(), query.into(), field.into(), "all".into()).unwrap();
            assert_eq!(found.len(), 1, "{field}");
            assert_eq!(found[0].cite_key, "alpha");
        }
        let books = search(path.clone(), " ".into(), "author".into(), "Book".into()).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].cite_key, "beta");
        assert!(search(path.clone(), "".into(), "invalid".into(), "all".into()).is_err());
        assert!(search(path, "".into(), "all".into(), "invalid".into()).is_err());
    }
    #[test]
    fn changed_bibliography_invalidates_cached_records() {
        let file = search_file("@misc{before,title={Before}}");
        let path = file.path().to_str().unwrap().to_owned();
        assert_eq!(
            search(path.clone(), "".into(), "all".into(), "all".into()).unwrap()[0].cite_key,
            "before"
        );
        std::fs::write(file.path(), "@misc{after,title={After the source changed}}").unwrap();
        assert_eq!(
            search(path, "".into(), "all".into(), "all".into()).unwrap()[0].cite_key,
            "after"
        );
    }
}
