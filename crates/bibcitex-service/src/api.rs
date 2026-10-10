//! Shared desktop operations, independent of either platform binding runtime.
use crate::records::{LibraryRecord, ReferenceRecord};
use crate::{
    core::{
        self,
        bib::Reference,
        search::{SearchField, SearchIndex},
    },
    registry,
};
use biblatex::EntryType;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
    time::{Instant, SystemTime},
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
    data: Arc<LibraryData>,
    last_used: Instant,
}
struct LibraryData {
    references: Vec<Reference>,
    search: SearchIndex,
    search_bytes: usize,
}
impl LibraryData {
    fn new(references: Vec<Reference>) -> Self {
        let search = SearchIndex::new(&references);
        let search_bytes = search.allocated_bytes();
        Self {
            references,
            search,
            search_bytes,
        }
    }
}
#[derive(Default)]
struct SearchCache {
    entries: HashMap<PathBuf, Cached>,
    search_bytes: usize,
}
const MAX_CACHED_LIBRARIES: usize = 8;
const MAX_SEARCH_BYTES: usize = 64 * 1024 * 1024;
impl SearchCache {
    fn get(&mut self, path: &PathBuf, fingerprint: &Fingerprint) -> Option<Arc<LibraryData>> {
        let cached = self.entries.get_mut(path)?;
        if &cached.fingerprint != fingerprint {
            return None;
        }
        cached.last_used = Instant::now();
        Some(Arc::clone(&cached.data))
    }
    fn remove(&mut self, path: &PathBuf) {
        if let Some(cached) = self.entries.remove(path) {
            self.search_bytes -= cached.data.search_bytes;
        }
    }
    fn insert(&mut self, path: PathBuf, fingerprint: Fingerprint, data: Arc<LibraryData>) {
        self.remove(&path);
        let bytes = data.search_bytes;
        if bytes > MAX_SEARCH_BYTES {
            return;
        }
        while self.entries.len() >= MAX_CACHED_LIBRARIES
            || self.search_bytes + bytes > MAX_SEARCH_BYTES
        {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, cached)| cached.last_used)
                .map(|(path, _)| path.clone());
            let Some(oldest) = oldest else {
                break;
            };
            self.remove(&oldest);
        }
        self.search_bytes += bytes;
        self.entries.insert(
            path,
            Cached {
                fingerprint,
                data,
                last_used: Instant::now(),
            },
        );
    }
}
static CACHE: LazyLock<Mutex<SearchCache>> = LazyLock::new(|| Mutex::new(SearchCache::default()));
fn fingerprint(path: &PathBuf) -> Result<Fingerprint, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(Fingerprint {
        modified: metadata.modified().ok(),
        length: metadata.len(),
    })
}
fn references(path: &str) -> Result<Arc<LibraryData>, String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let before = fingerprint(&path)?;
    {
        let mut cache = CACHE.lock().map_err(|_| "Cache lock poisoned")?;
        if let Some(data) = cache.get(&path, &before) {
            return Ok(data);
        }
    }
    // Parse outside the shared lock: a large main-window library must not block the helper.
    let parsed =
        core::utils::read_bibliography(core::bib::parse(&path).map_err(|e| e.to_string())?);
    let data = Arc::new(LibraryData::new(parsed));
    if fingerprint(&path)? != before {
        return Err("Bibliography changed while reading; retry search".into());
    }
    let mut cache = CACHE.lock().map_err(|_| "Cache lock poisoned")?;
    // Another request may have loaded the same file while parsing outside the lock.
    if let Some(existing) = cache.get(&path, &before) {
        return Ok(existing);
    }
    cache.insert(path, before, Arc::clone(&data));
    Ok(data)
}
fn search_type(type_filter: &str) -> Result<Option<EntryType>, String> {
    match type_filter {
        "all" => Ok(None),
        "Article" => Ok(Some(EntryType::Article)),
        "Book" => Ok(Some(EntryType::Book)),
        "Thesis" => Ok(Some(EntryType::Thesis)),
        "Booklet" => Ok(Some(EntryType::Booklet)),
        "InBook" => Ok(Some(EntryType::InBook)),
        "InCollection" => Ok(Some(EntryType::InCollection)),
        "InProceedings" => Ok(Some(EntryType::InProceedings)),
        "Misc" => Ok(Some(EntryType::Misc)),
        "TechReport" => Ok(Some(EntryType::TechReport)),
        _ => Err("Unknown bibliography type filter".into()),
    }
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
    let field = match field.as_str() {
        "all" => SearchField::All,
        "author" => SearchField::Author,
        "title" => SearchField::Title,
        "journal" => SearchField::Journal,
        "year" => SearchField::Year,
        _ => return Err("Unknown search field".to_owned().into()),
    };
    let type_filter = search_type(&type_filter)?;
    let source = references(&path)?;
    source
        .search
        .search_filtered(&query, field, |i| {
            let kind = &source.references[i].type_;
            type_filter.as_ref().is_none_or(|filter| {
                kind == filter
                    || (*filter == EntryType::Thesis
                        && matches!(kind, EntryType::MastersThesis | EntryType::PhdThesis))
            })
        })
        .into_iter()
        .map(|i| {
            ReferenceRecord::from_reference(&source.references[i], &path).map_err(CoreError::from)
        })
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
    #[test]
    fn ordinary_search_combines_fields_types_and_ranked_results_without_altering_records() {
        let file = search_file(
            "@article{paper,title={Neural methods for graph analysis},author={Smith, Alice},year={2026}}\n@book{book,title={Graph neural models},author={Smith, Bob}}\n@article{missing,title={Graph analysis},author={Smith, Alice}}",
        );
        let path = file.path().to_str().unwrap().to_owned();
        let found = search(
            path.clone(),
            "Smith graph neural".into(),
            "all".into(),
            "Article".into(),
        )
        .unwrap();
        assert_eq!(
            found
                .iter()
                .map(|r| r.cite_key.as_str())
                .collect::<Vec<_>>(),
            ["paper"]
        );
        assert_eq!(found[0].id, format!("{path}\u{1f}paper"));
        assert!(found[0].source.contains("Neural methods"));
        assert!(
            search(
                path.clone(),
                "Smith graph".into(),
                "title".into(),
                "all".into()
            )
            .unwrap()
            .is_empty()
        );
        let ranked = search(path, "graph neural".into(), "all".into(), "all".into()).unwrap();
        assert_eq!(
            ranked
                .iter()
                .map(|r| r.cite_key.as_str())
                .collect::<Vec<_>>(),
            ["book", "paper"]
        );
    }
    #[test]
    fn changed_bibliography_replaces_preprocessed_search_texts_together_with_records() {
        let file = search_file("@misc{paper,title={Neural networks},author={Smith, Alice}}");
        let path = file.path().to_str().unwrap().to_owned();
        assert_eq!(
            search(
                path.clone(),
                "Smith neural".into(),
                "all".into(),
                "all".into()
            )
            .unwrap()
            .len(),
            1
        );
        std::fs::write(
            file.path(),
            "@misc{paper,title={Graph algorithms changed},author={Doe, Jane}}",
        )
        .unwrap();
        assert!(
            search(
                path.clone(),
                "Smith neural".into(),
                "all".into(),
                "all".into()
            )
            .unwrap()
            .is_empty()
        );
        let found = search(path, "Doe graph".into(), "all".into(), "all".into()).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].source.contains("Graph algorithms changed"));
    }
    #[test]
    fn cache_reuses_snapshots_and_evicts_only_the_least_recently_used_library() {
        let mut cache = SearchCache::default();
        let fingerprint = Fingerprint {
            modified: None,
            length: 1,
        };
        let data = Arc::new(LibraryData::new(Vec::new()));
        for i in 0..MAX_CACHED_LIBRARIES {
            cache.insert(
                PathBuf::from(i.to_string()),
                fingerprint.clone(),
                Arc::clone(&data),
            );
        }
        assert!(Arc::ptr_eq(
            &cache.get(&PathBuf::from("0"), &fingerprint).unwrap(),
            &data
        ));
        cache.insert(PathBuf::from("new"), fingerprint.clone(), Arc::clone(&data));
        assert_eq!(cache.entries.len(), MAX_CACHED_LIBRARIES);
        assert!(cache.get(&PathBuf::from("1"), &fingerprint).is_none());
        assert!(cache.get(&PathBuf::from("0"), &fingerprint).is_some());
    }
    #[test]
    fn thesis_search_keeps_all_existing_thesis_variants() {
        let file = search_file(
            "@thesis{general,title={Network}}\n@phdthesis{phd,title={Network}}\n@mastersthesis{masters,title={Network}}\n@book{book,title={Network}}",
        );
        let found = search(
            file.path().to_str().unwrap().into(),
            "network".into(),
            "title".into(),
            "Thesis".into(),
        )
        .unwrap();
        assert_eq!(
            found
                .iter()
                .map(|r| r.cite_key.as_str())
                .collect::<Vec<_>>(),
            ["general", "masters", "phd"]
        );
    }
    #[test]
    fn parsed_formulas_preserve_operators_in_title_and_note_search() {
        let file = search_file(
            "@misc{equal,title={Gödel $x=1$ methods},note={$y=2$}}\n@misc{minus,title={Gödel $x-1$ methods},note={$y-2$}}\n@misc{divide,title={Gödel $x/1$ methods},note={$y/2$}}",
        );
        let path = file.path().to_str().unwrap().to_owned();
        for (key, formula, note) in [
            ("equal", "x=1", "y=2"),
            ("minus", "x-1", "y-2"),
            ("divide", "x/1", "y/2"),
        ] {
            let title_matches = search(
                path.clone(),
                format!("godel {formula} methods"),
                "title".into(),
                "all".into(),
            )
            .unwrap();
            assert_eq!(
                title_matches
                    .iter()
                    .map(|r| r.cite_key.as_str())
                    .collect::<Vec<_>>(),
                [key]
            );
            let note_matches =
                search(path.clone(), note.into(), "all".into(), "all".into()).unwrap();
            assert_eq!(
                note_matches
                    .iter()
                    .map(|r| r.cite_key.as_str())
                    .collect::<Vec<_>>(),
                [key]
            );
        }
    }
}
