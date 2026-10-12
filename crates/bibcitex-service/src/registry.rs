//! Preserve the existing settings document, including fields unknown to this version.
use crate::core::{BibliographyInfo, Setting};
use chrono::Local;
use serde_json::{Value, json};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

static REGISTRY: Mutex<()> = Mutex::new(());
type Result<T> = std::result::Result<T, String>;

#[cfg(test)]
thread_local! {
    static TEST_DIRECTORY: tempfile::TempDir = tempfile::tempdir().expect("temporary settings directory");
}
pub(crate) fn settings_path() -> PathBuf {
    if let Some(directory) = std::env::var_os("BIBCITEX_CONFIG_DIRECTORY") {
        return PathBuf::from(directory).join("setting.json");
    }
    #[cfg(test)]
    {
        if let Some(directory) = std::env::var_os("BIBCITEX_TEST_REGISTRY_DIRECTORY") {
            PathBuf::from(directory).join("setting.json")
        } else {
            TEST_DIRECTORY.with(|directory| directory.path().join("setting.json"))
        }
    }
    #[cfg(not(test))]
    {
        Setting::config_file_path()
    }
}

/// Lock a stable sidecar, never the setting.json inode replaced by atomic writes.
/// Closing the returned file releases the OS lock, including on an early error.
fn mutation_lock() -> Result<fs::File> {
    let path = settings_path();
    let directory = path.parent().ok_or("Settings directory missing")?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("setting.lock"))
        .map_err(|e| e.to_string())?;
    file.lock().map_err(|e| e.to_string())?;
    Ok(file)
}

fn read() -> Result<Value> {
    let document = match fs::read(settings_path()) {
        Ok(bytes) => {
            serde_json::from_slice::<Value>(&bytes).map_err(|e| format!("Invalid settings: {e}"))?
        }
        Err(e) if e.kind() == ErrorKind::NotFound => json!({"bibliographies": {}}),
        Err(e) => return Err(e.to_string()),
    };
    if !document.is_object() || !document.get("bibliographies").is_some_and(Value::is_object) {
        return Err("Invalid settings: bibliographies must be an object".into());
    }
    // Validate before writing so damaged user data is never silently replaced.
    serde_json::from_value::<Setting>(document.clone())
        .map_err(|e| format!("Invalid settings: {e}"))?;
    Ok(document)
}
fn write(document: &Value) -> Result<()> {
    let path = settings_path();
    let directory = path.parent().ok_or("Settings directory missing")?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(directory).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file.as_file_mut(), document).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(&path).map_err(|e| e.to_string())?;
    Ok(())
}
fn info(name: &str, value: &Value) -> Result<Value> {
    let parsed: BibliographyInfo =
        serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    let available = parsed.path.is_file();
    Ok(
        json!({"uid": value["uid"], "name": name, "path": parsed.path, "created_at": parsed.created_at,
        "updated_at": parsed.updated_at, "description": parsed.description,
        "pinned": value["pinned"].as_bool().unwrap_or(false),
        "available": available}),
    )
}
pub fn libraries() -> Result<Value> {
    let _guard = search_lock()?;
    let document = read()?;
    let mut rows = document["bibliographies"]
        .as_object()
        .ok_or("Invalid settings")?
        .iter()
        .map(|(name, value)| info(name, value))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by_key(|row| !row["pinned"].as_bool().unwrap_or(false));
    Ok(Value::Array(rows))
}
pub fn add(name: &str, path: &str, description: Option<String>) -> Result<Value> {
    if name.trim().is_empty() {
        return Err("文献库名称不能为空".into());
    }
    let canonical = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if !canonical.is_file() {
        return Err("Bibliography path is not a file".into());
    }
    let _guard = search_lock()?;
    let mut document = read()?;
    if document["bibliographies"].get(name).is_some() {
        return Err("该名称已存在，请换一个".into());
    }
    let now = Local::now();
    let path_text = canonical.to_string_lossy().into_owned();
    let uid = document["library_locations"]
        .as_object()
        .and_then(|locations| {
            locations
                .iter()
                .find(|(_, p)| p.as_str() == Some(&path_text))
                .map(|(uid, _)| uid.clone())
        })
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    document["library_locations"][&uid] = json!(canonical);
    document["bibliographies"][name] = json!({"uid": uid, "path": canonical, "created_at": now,
        "updated_at": now, "description": description});
    write(&document)?;
    info(name, &document["bibliographies"][name])
}
pub fn update(
    name: &str,
    new_name: &str,
    path: Option<&str>,
    description: Option<String>,
) -> Result<Value> {
    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err("文献库名称不能为空".into());
    }
    let canonical = path
        .map(|path| {
            let canonical = fs::canonicalize(path).map_err(|e| e.to_string())?;
            if !canonical.is_file() {
                return Err("Bibliography path is not a file".to_string());
            }
            Ok(canonical)
        })
        .transpose()?;
    let _guard = search_lock()?;
    let mut document = read()?;
    let before_registry = document.clone();
    let entries = document["bibliographies"]
        .as_object_mut()
        .ok_or("Invalid settings")?;
    if new_name != name && entries.contains_key(new_name) {
        return Err("该名称已存在，请换一个".into());
    }
    let mut value = entries.remove(name).ok_or("Bibliography not found")?;
    let old_path = value["path"].as_str().unwrap_or_default().to_owned();
    let target_path = canonical.as_ref().map(|p| p.to_string_lossy().into_owned());
    if let Some(path) = canonical {
        value["path"] = json!(path);
    }
    if let Some(description) = description {
        value["description"] = json!(description);
    }
    value["updated_at"] = json!(Local::now());
    let library = info(new_name, &value)?;
    entries.insert(new_name.to_string(), value);
    if document["helper_current"]["name"].as_str() == Some(name) {
        document["helper_current"] =
            json!({"uid": library["uid"], "name": new_name, "path": library["path"]});
    }
    if let Some(target) = target_path.filter(|target| target != &old_path) {
        let uid = library["uid"].as_str().ok_or("MissingLibraryUid")?;
        if document["library_locations"]
            .as_object()
            .is_some_and(|locations| {
                locations
                    .iter()
                    .any(|(id, path)| id != uid && path.as_str() == Some(&target))
            })
        {
            return Err("SourceAlreadyExists".into());
        }
        document["library_locations"][uid] = json!(target);
        for entry in document["bibliographies"]
            .as_object_mut()
            .ok_or("Invalid settings")?
            .values_mut()
        {
            if entry["uid"].as_str() == Some(uid) {
                entry["path"] = json!(target);
            }
        }
        if document["helper_current"]["name"].as_str() == Some(new_name)
            || document["helper_current"]["path"].as_str() == Some(&old_path)
        {
            document["helper_current"]["uid"] = json!(uid);
            document["helper_current"]["path"] = json!(target);
        }
        let before_metadata = crate::tags::read()?;
        let mut after_metadata = before_metadata.clone();
        if let Some(source) = after_metadata.sources.get_mut(uid) {
            source.version.clear();
        }
        after_metadata.revision = after_metadata
            .revision
            .checked_add(1)
            .ok_or("RevisionOverflow")?;
        commit_search_transaction(
            &before_registry,
            &document,
            &before_metadata,
            &after_metadata,
        )?;
        return Ok(library);
    }
    write(&document)?;
    Ok(library)
}
pub fn set_pinned(name: &str, pinned: bool) -> Result<()> {
    let _guard = search_lock()?;
    let mut document = read()?;
    let value = document["bibliographies"]
        .get_mut(name)
        .ok_or("Bibliography not found")?;
    value["pinned"] = json!(pinned);
    write(&document)
}
pub fn remove(name: &str) -> Result<Value> {
    let _guard = search_lock()?;
    let mut document = read()?;
    document["bibliographies"]
        .as_object_mut()
        .ok_or("Invalid settings")?
        .remove(name);
    if document["helper_current"]["name"].as_str() == Some(name) {
        document["helper_current"] = Value::Null;
    }
    write(&document)?;
    Ok(Value::Null)
}
pub fn current() -> Result<Value> {
    let _guard = search_lock()?;
    let document = read()?;
    let Some(name) = document["helper_current"]["name"].as_str() else {
        return Ok(Value::Null);
    };
    let Some(value) = document["bibliographies"].get(name) else {
        return Ok(Value::Null);
    };
    if value["path"] != document["helper_current"]["path"] {
        return Ok(Value::Null);
    }
    info(name, value)
}
pub fn select(name: &str, path: &str) -> Result<Value> {
    let _guard = search_lock()?;
    let mut document = read()?;
    let value = document["bibliographies"]
        .get(name)
        .ok_or("Bibliography not found")?;
    let registered = value["path"].as_str().ok_or("Invalid bibliography path")?;
    if Path::new(path) != Path::new(registered) {
        return Err("Bibliography path does not match registry".into());
    }
    let library = info(name, value)?;
    document["helper_current"] =
        json!({"uid": document["bibliographies"][name]["uid"], "name": name, "path": registered});
    write(&document)?;
    Ok(library)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bibliography() -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"@misc{example,title={Example}}").unwrap();
        file
    }
    #[test]
    fn helper_selection_persists_and_removal_never_deletes_the_file() {
        let file = bibliography();
        let path = file.path().to_str().unwrap();
        let library = add("Work", path, Some("description".into())).unwrap();
        assert_eq!(library["available"], true);
        assert!(current().unwrap().is_null());
        select("Work", library["path"].as_str().unwrap()).unwrap();
        assert_eq!(current().unwrap(), library);
        assert_eq!(read().unwrap()["helper_current"]["name"], "Work");
        remove("Work").unwrap();
        assert!(current().unwrap().is_null());
        assert!(libraries().unwrap().as_array().unwrap().is_empty());
        assert!(file.path().exists());
    }
    #[test]
    fn editing_preserves_metadata_and_tracks_helper_selection() {
        let file = bibliography();
        let replacement = bibliography();
        let old = add(
            "Original",
            file.path().to_str().unwrap(),
            Some("old".into()),
        )
        .unwrap();
        select("Original", old["path"].as_str().unwrap()).unwrap();
        set_pinned("Original", true).unwrap();
        let mut document = read().unwrap();
        document["bibliographies"]["Original"]["future"] = json!({"keep": true});
        write(&document).unwrap();
        let edited = update(
            "Original",
            "Renamed",
            replacement.path().to_str(),
            Some("new".into()),
        )
        .unwrap();
        assert_eq!(edited["created_at"], old["created_at"]);
        assert_eq!(edited["pinned"], true);
        assert_eq!(edited["description"], "new");
        assert_eq!(current().unwrap(), edited);
        let document = read().unwrap();
        assert!(document["bibliographies"].get("Original").is_none());
        assert_eq!(
            document["bibliographies"]["Renamed"]["future"]["keep"],
            true
        );
        assert!(file.path().exists());
        remove("Renamed").unwrap();
        assert!(replacement.path().exists());
    }
    #[test]
    fn invalid_edits_are_atomic_and_rename_does_not_require_a_present_file() {
        let file = bibliography();
        let path = file.path().to_str().unwrap();
        add("Alpha", path, None).unwrap();
        add("Beta", path, None).unwrap();
        let before = fs::read(settings_path()).unwrap();
        assert!(update("Alpha", "Beta", None, None).is_err());
        assert!(update("Alpha", "  ", None, None).is_err());
        assert!(update("Missing", "New", None, None).is_err());
        assert!(update("Alpha", "New", Some("/missing/library.bib"), None).is_err());
        assert!(set_pinned("Missing", true).is_err());
        assert_eq!(fs::read(settings_path()).unwrap(), before);
        drop(file);
        let renamed = update("Alpha", "Offline", None, None).unwrap();
        assert_eq!(renamed["name"], "Offline");
        assert_eq!(renamed["available"], false);
    }
    #[test]
    fn pinning_persists_and_unpinning_restores_name_order() {
        let file = bibliography();
        let path = file.path().to_str().unwrap();
        add("Alpha", path, None).unwrap();
        add("Zulu", path, None).unwrap();
        assert_eq!(libraries().unwrap()[0]["name"], "Alpha");
        set_pinned("Zulu", true).unwrap();
        assert_eq!(read().unwrap()["bibliographies"]["Zulu"]["pinned"], true);
        assert_eq!(libraries().unwrap()[0]["name"], "Zulu");
        set_pinned("Zulu", false).unwrap();
        assert_eq!(libraries().unwrap()[0]["name"], "Alpha");
    }
    #[test]
    fn invalid_json_and_invalid_schema_are_not_overwritten() {
        let file = bibliography();
        for content in [
            b"{not json".as_slice(),
            b"{\"bibliographies\":[]}",
            b"{\"bibliographies\":{\"Bad\":{\"path\":42}}}",
        ] {
            fs::write(settings_path(), content).unwrap();
            assert!(libraries().is_err());
            assert!(add("New", file.path().to_str().unwrap(), None).is_err());
            assert!(remove("Bad").is_err());
            assert_eq!(fs::read(settings_path()).unwrap(), content);
        }
    }
    #[test]
    fn duplicate_name_and_mismatched_selection_leave_registry_unchanged() {
        let file = bibliography();
        add("Work", file.path().to_str().unwrap(), None).unwrap();
        let before = fs::read(settings_path()).unwrap();
        assert!(add("Work", file.path().to_str().unwrap(), None).is_err());
        assert!(select("Work", "/different.bib").is_err());
        assert!(add(" ", file.path().to_str().unwrap(), None).is_err());
        assert_eq!(fs::read(settings_path()).unwrap(), before);
    }
    #[test]
    fn atomic_updates_preserve_unrecognized_settings_fields() {
        fs::write(
            settings_path(),
            br#"{"bibliographies":{},"theme":"system","future":{"enabled":true}}"#,
        )
        .unwrap();
        let file = bibliography();
        add("Work", file.path().to_str().unwrap(), None).unwrap();
        let saved = read().unwrap();
        assert_eq!(saved["theme"], "system");
        assert_eq!(saved["future"]["enabled"], true);
        assert!(settings_path().exists());
        assert!(settings_path().with_file_name("setting.lock").exists());
    }
    #[test]
    fn sidecar_lock_serializes_independent_handles_across_atomic_replacement() {
        let guard = mutation_lock().unwrap();
        let contender = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(settings_path().with_file_name("setting.lock"))
            .unwrap();
        assert!(matches!(
            contender.try_lock(),
            Err(fs::TryLockError::WouldBlock)
        ));
        write(&json!({"bibliographies":{}})).unwrap();
        write(&json!({"bibliographies":{},"revision":2})).unwrap();
        assert!(matches!(
            contender.try_lock(),
            Err(fs::TryLockError::WouldBlock)
        ));
        // Parallel process tests can briefly inherit this descriptor on Unix.
        // Explicitly unlock instead of depending on when those copies close.
        guard.unlock().unwrap();
        contender.try_lock().unwrap();
    }

    #[test]
    fn registry_process_worker() {
        let Some(directory) = std::env::var_os("BIBCITEX_TEST_REGISTRY_DIRECTORY") else {
            return;
        };
        let directory = PathBuf::from(directory);
        let worker = std::env::var("BIBCITEX_TEST_REGISTRY_WORKER").unwrap();
        fs::write(directory.join(format!("ready-{worker}")), b"ready").unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !directory.join("start").exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "parent never released worker barrier"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let file = directory.join("fixture.bib");
        for item in 0..16 {
            let name = format!("worker-{worker}-item-{item}");
            let library = add(&name, file.to_str().unwrap(), None).unwrap();
            if item % 2 == 0 {
                remove(&name).unwrap();
            } else {
                select(&name, library["path"].as_str().unwrap()).unwrap();
            }
        }
    }

    #[test]
    fn concurrent_process_mutations_do_not_lose_registry_entries() {
        use std::process::{Command, Stdio};
        let path = settings_path();
        let directory = path.parent().unwrap();
        fs::write(
            directory.join("fixture.bib"),
            b"@misc{example,title={Example}}",
        )
        .unwrap();
        fs::write(&path, br#"{"bibliographies":{},"theme":"system"}"#).unwrap();
        let children = (0..4)
            .map(|worker| {
                Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "registry::tests::registry_process_worker",
                        "--nocapture",
                    ])
                    .env("BIBCITEX_TEST_REGISTRY_DIRECTORY", directory)
                    .env("BIBCITEX_TEST_REGISTRY_WORKER", worker.to_string())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !(0..4).all(|worker| directory.join(format!("ready-{worker}")).exists()) {
            assert!(
                std::time::Instant::now() < deadline,
                "registry workers did not become ready"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        fs::write(directory.join("start"), b"start").unwrap();
        for child in children {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "registry worker failed: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let document = read().unwrap();
        let libraries = document["bibliographies"].as_object().unwrap();
        assert_eq!(libraries.len(), 32);
        for worker in 0..4 {
            for item in 0..16 {
                assert_eq!(
                    libraries.contains_key(&format!("worker-{worker}-item-{item}")),
                    item % 2 != 0
                );
            }
        }
        assert_eq!(document["theme"], "system");
        assert!(!current().unwrap().is_null());
    }
}

/// File locations live only in the registry. Tag records refer to its stable UIDs.
pub(crate) fn source_paths() -> Result<std::collections::BTreeMap<String, String>> {
    let document = read()?;
    if document
        .get("library_locations")
        .is_some_and(|v| !v.is_object())
    {
        return Err("InvalidLibraryLocations".into());
    }
    document
        .get("library_locations")
        .and_then(Value::as_object)
        .map(|locations| {
            locations
                .iter()
                .map(|(id, value)| {
                    value
                        .as_str()
                        .map(|path| (id.clone(), path.to_owned()))
                        .ok_or("InvalidLibraryLocation".into())
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(std::collections::BTreeMap::new()))
}
pub(crate) fn path_for_uid(uid: &str) -> Result<String> {
    source_paths()?.remove(uid).ok_or("SourceNotFound".into())
}
pub(crate) fn uid_for_path(path: &str) -> Result<String> {
    let _guard = search_lock()?;
    let mut document = read()?;
    if let Some((uid, _)) = document["library_locations"]
        .as_object()
        .and_then(|m| m.iter().find(|(_, p)| p.as_str() == Some(path)))
    {
        return Ok(uid.clone());
    }
    let uid = uuid::Uuid::new_v4().to_string();
    document["library_locations"][&uid] = json!(path);
    write(&document)?;
    Ok(uid)
}
// Existing installations have library registrations but no tag identity schema.
// Initialize UUIDs atomically in the registry; subsequent reads only validate them.
fn initialize_identities_locked() -> Result<()> {
    let before = read()?;
    let mut registry = before.clone();
    if registry.get("library_locations").is_none() {
        registry["library_locations"] = json!({});
    }
    let mut locations = source_paths()?;
    let mut by_path = std::collections::BTreeMap::new();
    for (uid, path) in &locations {
        if uuid::Uuid::parse_str(uid).is_err() || !Path::new(path).is_absolute() {
            return Err("InvalidLibraryLocation".into());
        }
        if by_path.insert(path.clone(), uid.clone()).is_some() {
            return Err("LibraryUidConflict".into());
        }
    }
    // Seed existing UUIDs before filling missing aliases, regardless of name order.
    for value in registry["bibliographies"]
        .as_object()
        .ok_or("Invalid settings")?
        .values()
    {
        if let Some(uid) = value.get("uid") {
            let uid = uid
                .as_str()
                .filter(|id| uuid::Uuid::parse_str(id).is_ok())
                .ok_or("InvalidLibraryUid")?;
            let path = value["path"].as_str().ok_or("InvalidLibraryPath")?;
            if by_path.get(path).is_some_and(|id| id != uid)
                || locations.get(uid).is_some_and(|p| p != path)
            {
                return Err("LibraryUidConflict".into());
            }
            by_path.insert(path.to_owned(), uid.to_owned());
            locations.insert(uid.to_owned(), path.to_owned());
        }
    }
    for value in registry["bibliographies"]
        .as_object_mut()
        .ok_or("Invalid settings")?
        .values_mut()
    {
        let path = value["path"]
            .as_str()
            .ok_or("InvalidLibraryPath")?
            .to_owned();
        let uid = match value.get("uid") {
            Some(Value::String(uid)) if uuid::Uuid::parse_str(uid).is_ok() => uid.clone(),
            None => by_path
                .get(&path)
                .cloned()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            _ => return Err("InvalidLibraryUid".into()),
        };
        if by_path.get(&path).is_some_and(|id| id != &uid)
            || locations.get(&uid).is_some_and(|p| p != &path)
        {
            return Err("LibraryUidConflict".into());
        }
        by_path.insert(path.clone(), uid.clone());
        locations.insert(uid.clone(), path);
        value["uid"] = json!(uid);
    }
    registry["library_locations"] = json!(locations);
    if let Some(name) = registry["helper_current"]["name"]
        .as_str()
        .map(str::to_owned)
    {
        registry["helper_current"]["uid"] = registry["bibliographies"][&name]["uid"].clone();
    }
    if registry != before {
        write(&registry)?;
    }
    Ok(())
}

/// Keep recovery and the following operation in one critical section, including
/// across processes. No writer may slip into an interrupted relocation journal.
pub(crate) struct SearchGuard {
    _metadata: fs::File,
    _registry: fs::File,
    _process: std::sync::MutexGuard<'static, ()>,
}
pub(crate) fn search_lock() -> Result<SearchGuard> {
    let process = REGISTRY.lock().map_err(|_| "Settings lock poisoned")?;
    let registry = mutation_lock()?;
    let metadata = crate::tags::lock()?;
    let guard = SearchGuard {
        _metadata: metadata,
        _registry: registry,
        _process: process,
    };
    recover_transaction_locked()?;
    initialize_identities_locked()?;
    Ok(guard)
}
pub(crate) fn recover_search_transaction() -> Result<()> {
    let _guard = search_lock()?;
    Ok(())
}
fn transaction_path() -> PathBuf {
    settings_path().with_file_name("search-transaction.json")
}
fn recover_transaction_locked() -> Result<()> {
    let bytes = match fs::read(transaction_path()) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    let transaction: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("InvalidSearchTransaction: {e}"))?;
    if transaction["schema"].as_u64() != Some(1) {
        return Err("UnsupportedSearchTransactionSchema".into());
    }
    let current_registry = read()?;
    let current_metadata = crate::tags::storage_document(&crate::tags::read()?)?;
    if (current_registry != transaction["before_registry"]
        && current_registry != transaction["after_registry"])
        || (current_metadata != transaction["before_metadata"]
            && current_metadata != transaction["after_metadata"])
    {
        return Err("SearchTransactionConflict".into());
    }
    // Validate both target states before replaying either write.
    let after_metadata: crate::tags::Metadata =
        serde_json::from_value(transaction["after_metadata"].clone()).map_err(|e| e.to_string())?;
    serde_json::from_value::<Setting>(transaction["after_registry"].clone())
        .map_err(|e| e.to_string())?;
    write(&transaction["after_registry"])?;
    crate::tags::write(&after_metadata)?;
    fs::remove_file(transaction_path()).map_err(|e| e.to_string())?;
    Ok(())
}
fn commit_search_transaction(
    before_registry: &Value,
    after_registry: &Value,
    before_metadata: &crate::tags::Metadata,
    after_metadata: &crate::tags::Metadata,
) -> Result<()> {
    let transaction = json!({"schema":1,"before_registry":before_registry,"after_registry":after_registry,"before_metadata":crate::tags::storage_document(before_metadata)?,"after_metadata":crate::tags::storage_document(after_metadata)?});
    crate::tags::atomic_write(&transaction_path(), &transaction)?;
    transaction_checkpoint(1)?;
    write(after_registry)?;
    transaction_checkpoint(2)?;
    crate::tags::write(after_metadata)?;
    transaction_checkpoint(3)?;
    fs::remove_file(transaction_path()).map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
thread_local! { static TRANSACTION_FAILURE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }
fn transaction_checkpoint(stage: u8) -> Result<()> {
    #[cfg(test)]
    if TRANSACTION_FAILURE.with(|s| s.get() == stage) {
        return Err("InjectedTransactionFailure".into());
    }
    let _ = stage;
    Ok(())
}
/// Explicit source relocation updates registered aliases and metadata in one recoverable transaction.
pub(crate) fn relocate_search_source(id: &str, path: &str, revision: u64) -> Result<()> {
    let canonical = fs::canonicalize(path)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .into_owned();
    if !Path::new(&canonical).is_file() {
        return Err("Bibliography path is not a file".into());
    }
    let _guard = search_lock()?;
    let before_registry = read()?;
    let before_metadata = crate::tags::read()?;
    if before_metadata.revision != revision {
        return Err("StaleTagRevision".into());
    }
    if before_registry["library_locations"]
        .as_object()
        .is_some_and(|locations| {
            locations
                .values()
                .any(|path| path.as_str() == Some(&canonical))
        })
    {
        return Err("SourceAlreadyExists".into());
    }
    let mut after_metadata = before_metadata.clone();
    let source = after_metadata.sources.get_mut(id).ok_or("SourceNotFound")?;
    let old = source.path.clone();
    source.version = String::new();
    after_metadata.revision = after_metadata
        .revision
        .checked_add(1)
        .ok_or("RevisionOverflow")?;
    let mut after_registry = before_registry.clone();
    after_registry["library_locations"][id] = json!(canonical);
    for entry in after_registry["bibliographies"]
        .as_object_mut()
        .ok_or("Invalid settings")?
        .values_mut()
    {
        if entry["uid"].as_str() == Some(id) {
            entry["path"] = json!(canonical);
        }
    }
    if after_registry["helper_current"]["path"].as_str() == Some(&old) {
        after_registry["helper_current"]["path"] = json!(canonical);
    }
    commit_search_transaction(
        &before_registry,
        &after_registry,
        &before_metadata,
        &after_metadata,
    )
}

#[cfg(test)]
mod search_transaction_tests {
    use super::*;
    #[test]
    fn relocation_replays_each_interrupted_stage_without_partial_identity_changes() {
        for stage in 1..=3 {
            let original = tempfile::NamedTempFile::new().unwrap();
            fs::write(original.path(), "@article{A,title={old}}").unwrap();
            let target = tempfile::NamedTempFile::new().unwrap();
            fs::write(target.path(), "@article{A,title={old}}").unwrap();
            let name = format!("stage{stage}");
            add(&name, original.path().to_str().unwrap(), None).unwrap();
            let references = crate::core::utils::read_bibliography(
                crate::core::bib::parse(original.path()).unwrap(),
            );
            let metadata = crate::tags::load_snapshot(
                fs::canonicalize(original.path()).unwrap().to_str().unwrap(),
                "version",
                &references,
            )
            .unwrap();
            let id = metadata
                .sources
                .iter()
                .find(|(_, s)| {
                    s.path == fs::canonicalize(original.path()).unwrap().to_str().unwrap()
                })
                .unwrap()
                .0
                .clone();
            TRANSACTION_FAILURE.with(|s| s.set(stage));
            assert!(
                relocate_search_source(&id, target.path().to_str().unwrap(), metadata.revision)
                    .is_err()
            );
            TRANSACTION_FAILURE.with(|s| s.set(0));
            recover_search_transaction().unwrap();
            assert_eq!(
                read().unwrap()["bibliographies"][&name]["path"],
                fs::canonicalize(target.path()).unwrap().to_str().unwrap()
            );
            assert_eq!(
                crate::tags::read().unwrap().sources[&id].path,
                fs::canonicalize(target.path()).unwrap().to_str().unwrap()
            );
            assert!(!transaction_path().exists());
        }
    }
    #[test]
    fn recovery_rejects_unrelated_external_edits() {
        let before = serde_json::json!({"bibliographies":{}});
        write(&before).unwrap();
        let mut metadata = crate::tags::Metadata::default();
        metadata.schema = 2;
        crate::tags::write(&metadata).unwrap();
        TRANSACTION_FAILURE.with(|s| s.set(1));
        assert!(commit_search_transaction(&before, &before, &metadata, &metadata).is_err());
        TRANSACTION_FAILURE.with(|s| s.set(0));
        write(&serde_json::json!({"bibliographies":{},"external":true})).unwrap();
        assert!(
            recover_search_transaction()
                .unwrap_err()
                .contains("SearchTransactionConflict")
        );
        assert_eq!(read().unwrap()["external"], true);
    }
}

#[cfg(test)]
mod identity_review_tests {
    use super::*;

    #[test]
    fn existing_registrations_receive_one_stable_uuid_per_location() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = fs::canonicalize(file.path()).unwrap();
        let now = Local::now();
        let entry = json!({"path":path,"created_at":now,"updated_at":now});
        write(&json!({"bibliographies":{"Original":entry,"Alias":entry},"future":true})).unwrap();
        let first = libraries().unwrap();
        let uid = first[0]["uid"].as_str().unwrap();
        assert!(uuid::Uuid::parse_str(uid).is_ok());
        assert_eq!(first[1]["uid"], uid);
        let bytes = fs::read(settings_path()).unwrap();
        assert_eq!(libraries().unwrap(), first);
        assert_eq!(fs::read(settings_path()).unwrap(), bytes);
        assert_eq!(read().unwrap()["future"], true);
        assert!(!crate::tags::path().exists());
        remove("Original").unwrap();
        assert_eq!(
            add("Restored", file.path().to_str().unwrap(), None).unwrap()["uid"],
            uid
        );
    }

    #[test]
    fn uid_initialization_prefers_existing_uuid_over_alias_name_order() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = fs::canonicalize(file.path()).unwrap();
        let uid = uuid::Uuid::new_v4().to_string();
        let now = Local::now();
        let bare = json!({"path":path,"created_at":now,"updated_at":now});
        let mut identified = bare.clone();
        identified["uid"] = json!(uid);
        write(&json!({"bibliographies":{"A alias":bare,"Z original":identified}})).unwrap();
        let rows = libraries().unwrap();
        assert!(rows.as_array().unwrap().iter().all(|row| row["uid"] == uid));
        assert_eq!(source_paths().unwrap().len(), 1);
        assert_eq!(libraries().unwrap(), rows);
    }

    #[test]
    fn moving_an_alias_keeps_helper_selection_and_shared_uuid() {
        let original = tempfile::NamedTempFile::new().unwrap();
        let moved = tempfile::NamedTempFile::new().unwrap();
        let first = add("First", original.path().to_str().unwrap(), None).unwrap();
        let alias = add("Alias", original.path().to_str().unwrap(), None).unwrap();
        select("First", first["path"].as_str().unwrap()).unwrap();
        let updated = update(
            "Alias",
            "Renamed",
            Some(moved.path().to_str().unwrap()),
            None,
        )
        .unwrap();
        assert_eq!(updated["uid"], first["uid"]);
        assert_eq!(updated["uid"], alias["uid"]);
        let helper = current().unwrap();
        assert_eq!(helper["name"], "First");
        assert_eq!(helper["path"], updated["path"]);
        assert_eq!(helper["uid"], first["uid"]);
    }

    #[test]
    fn conflicting_locations_never_reassign_an_existing_uuid() {
        let first = tempfile::NamedTempFile::new().unwrap();
        let second = tempfile::NamedTempFile::new().unwrap();
        let one = add("One", first.path().to_str().unwrap(), None).unwrap();
        add("Two", second.path().to_str().unwrap(), None).unwrap();
        let mut document = read().unwrap();
        document["bibliographies"]["Two"]["uid"] = one["uid"].clone();
        write(&document).unwrap();
        let before = fs::read(settings_path()).unwrap();
        assert_eq!(libraries().unwrap_err(), "LibraryUidConflict");
        assert_eq!(fs::read(settings_path()).unwrap(), before);
    }

    #[test]
    fn relocation_rejects_registered_but_unsearched_destination() {
        let first = tempfile::NamedTempFile::new().unwrap();
        let second = tempfile::NamedTempFile::new().unwrap();
        fs::write(first.path(), "@article{A,title={one}}").unwrap();
        let one = add("One", first.path().to_str().unwrap(), None).unwrap();
        add("Two", second.path().to_str().unwrap(), None).unwrap();
        let snapshot =
            crate::tag_snapshot(first.path().to_string_lossy().into(), "A".into()).unwrap();
        let before = fs::read(settings_path()).unwrap();
        assert_eq!(
            relocate_search_source(
                one["uid"].as_str().unwrap(),
                second.path().to_str().unwrap(),
                snapshot.revision
            )
            .unwrap_err(),
            "SourceAlreadyExists"
        );
        assert_eq!(fs::read(settings_path()).unwrap(), before);
        assert_eq!(crate::tags::read().unwrap().revision, snapshot.revision);
    }

    #[test]
    fn next_writer_recovers_interrupted_relocation_before_mutating_registry() {
        for stage in 1..=3 {
            let original = tempfile::NamedTempFile::new().unwrap();
            let moved = tempfile::NamedTempFile::new().unwrap();
            let extra = tempfile::NamedTempFile::new().unwrap();
            fs::write(original.path(), "@article{A,title={one}}").unwrap();
            let name = format!("original{stage}");
            let library = add(&name, original.path().to_str().unwrap(), None).unwrap();
            let snapshot =
                crate::tag_snapshot(original.path().to_string_lossy().into(), "A".into()).unwrap();
            TRANSACTION_FAILURE.with(|s| s.set(stage));
            assert!(
                relocate_search_source(
                    library["uid"].as_str().unwrap(),
                    moved.path().to_str().unwrap(),
                    snapshot.revision
                )
                .is_err()
            );
            TRANSACTION_FAILURE.with(|s| s.set(0));
            add(
                &format!("extra{stage}"),
                extra.path().to_str().unwrap(),
                None,
            )
            .unwrap();
            let state = read().unwrap();
            assert_eq!(state["bibliographies"][&name]["uid"], library["uid"]);
            assert_eq!(
                state["bibliographies"][&name]["path"],
                json!(fs::canonicalize(moved.path()).unwrap())
            );
            assert!(!transaction_path().exists());
        }
    }
}
