//! Locked, atomic metadata with stable identities independent of paths and cite keys.
use crate::core::bib::Reference;
use crate::{PendingAssociation, TagAction, TagMutation, TagRecord, TagSnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct Metadata {
    pub schema: u32,
    pub revision: u64,
    pub tags: BTreeMap<String, StoredTag>,
    pub redirects: BTreeMap<String, String>,
    pub sources: BTreeMap<String, Source>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredTag {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Source {
    pub path: String,
    pub version: String,
    pub records: BTreeMap<String, Record>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Record {
    pub key: String,
    fingerprint: String,
    pub present: bool,
    pub tags: BTreeSet<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
pub(crate) fn path() -> PathBuf {
    crate::registry::settings_path().with_file_name("search-metadata.json")
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
pub(crate) fn comparison(name: &str) -> String {
    crate::core::search::advanced::normalize_tag_name(name)
}
fn display(name: &str) -> Result<String> {
    let name: String = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .nfc()
        .collect();
    if name.is_empty() || name.len() > 65536 {
        return Err("InvalidTagName".into());
    }
    Ok(name)
}
pub(crate) fn lock() -> Result<fs::File> {
    let path = path();
    let dir = path.parent().ok_or("MetadataDirectoryMissing")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))
        .map_err(|e| e.to_string())?;
    file.lock().map_err(|e| e.to_string())?;
    Ok(file)
}
pub(crate) fn read() -> Result<Metadata> {
    let data = match fs::read(path()) {
        Ok(bytes) => serde_json::from_slice::<Metadata>(&bytes)
            .map_err(|e| format!("InvalidSearchMetadata: {e}"))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Metadata {
            schema: 1,
            ..Metadata::default()
        },
        Err(e) => return Err(e.to_string()),
    };
    if data.schema != 1 {
        return Err("UnsupportedSearchMetadataSchema".into());
    }
    let mut names = BTreeSet::new();
    for (id, tag) in &data.tags {
        if id != &tag.id || display(&tag.name)? != tag.name || !names.insert(comparison(&tag.name))
        {
            return Err("InvalidSearchMetadata".into());
        }
    }
    let mut paths = BTreeSet::new();
    let mut records = BTreeSet::new();
    for (id, source) in &data.sources {
        if id.is_empty() || !Path::new(&source.path).is_absolute() || !paths.insert(&source.path) {
            return Err("InvalidSearchMetadata".into());
        }
        let mut keys = BTreeSet::new();
        for (id, record) in &source.records {
            if id.is_empty()
                || !records.insert(id)
                || (record.present && !keys.insert(&record.key))
                || record.tags.iter().any(|id| !data.tags.contains_key(id))
            {
                return Err("InvalidSearchMetadata".into());
            }
        }
    }
    for id in data.redirects.keys() {
        let mut seen = BTreeSet::new();
        let mut current = id;
        while let Some(next) = data.redirects.get(current) {
            if !seen.insert(current) {
                return Err("InvalidSearchMetadata".into());
            }
            current = next;
        }
    }
    Ok(data)
}
pub(crate) fn atomic_write<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let directory = path.parent().ok_or("MetadataDirectoryMissing")?;
    let mut tmp = tempfile::NamedTempFile::new_in(directory).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(tmp.as_file_mut(), value).map_err(|e| e.to_string())?;
    tmp.write_all(b"\n").map_err(|e| e.to_string())?;
    tmp.as_file().sync_all().map_err(|e| e.to_string())?;
    tmp.persist(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    fs::File::open(directory)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub(crate) fn write(data: &Metadata) -> Result<()> {
    atomic_write(&path(), data)
}
fn fingerprint(reference: &Reference) -> Result<String> {
    let bibliography =
        biblatex::Bibliography::parse(&reference.source).map_err(|e| e.to_string())?;
    let entry = bibliography.iter().next().ok_or("EmptyReference")?;
    let fields: BTreeMap<_, _> = entry
        .fields
        .iter()
        .map(|(name, chunks)| {
            (
                name,
                chunks.iter().map(|chunk| &chunk.v).collect::<Vec<_>>(),
            )
        })
        .collect();
    Ok(digest(
        &serde_json::to_vec(&(entry.entry_type.clone(), fields)).map_err(|e| e.to_string())?,
    ))
}
impl Metadata {
    pub fn source(&self, path: &str) -> Option<&Source> {
        self.sources.values().find(|s| s.path == path)
    }
    pub fn resolve(&self, id: &str) -> Option<&StoredTag> {
        self.tags.get(id)
    }
    pub fn redirected(&self, id: &str) -> Option<&StoredTag> {
        let mut id = id;
        let mut seen = BTreeSet::new();
        while seen.insert(id) {
            if let Some(tag) = self.tags.get(id) {
                return Some(tag);
            }
            id = self.redirects.get(id)?;
        }
        None
    }
    fn snapshot(&self, path: &str, key: &str) -> TagSnapshot {
        let assigned_ids = self
            .source(path)
            .and_then(|s| s.records.values().find(|r| r.present && r.key == key))
            .map(|r| r.tags.iter().cloned().collect())
            .unwrap_or_default();
        let pending = self
            .sources
            .iter()
            .flat_map(|(sid, s)| {
                s.records
                    .iter()
                    .filter(|(_, r)| !r.present && !r.tags.is_empty())
                    .map(|(rid, r)| PendingAssociation {
                        record_id: rid.clone(),
                        source_id: sid.clone(),
                        path: s.path.clone(),
                        cite_key: r.key.clone(),
                        tags: r.tags.iter().cloned().collect(),
                    })
            })
            .collect();
        let mut tags: Vec<_> = self
            .tags
            .values()
            .map(|t| TagRecord {
                id: t.id.clone(),
                name: t.name.clone(),
            })
            .collect();
        tags.sort_by_key(|t| comparison(&t.name));
        TagSnapshot {
            revision: self.revision,
            tags,
            assigned_ids,
            pending,
            sources: self
                .sources
                .iter()
                .map(|(id, s)| crate::SearchSourceRecord {
                    id: id.clone(),
                    path: s.path.clone(),
                    available: Path::new(&s.path).is_file(),
                })
                .collect(),
        }
    }
    pub fn reconcile(
        &mut self,
        path: &str,
        version: &str,
        references: &[Reference],
    ) -> Result<bool> {
        if self.source(path).is_some_and(|s| s.version == version) {
            return Ok(false);
        }
        let mut incoming = BTreeMap::new();
        for reference in references {
            if incoming
                .insert(reference.cite_key.clone(), fingerprint(reference)?)
                .is_some()
            {
                return Err("DuplicateCiteKey".into());
            }
        }
        let sid = self
            .sources
            .iter()
            .find(|(_, s)| s.path == path)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let source = self.sources.entry(sid).or_insert_with(|| Source {
            path: path.into(),
            version: String::new(),
            records: BTreeMap::new(),
            extra: BTreeMap::new(),
        });
        let mut keys: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
        for (id, record) in &source.records {
            keys.entry(record.key.clone())
                .or_default()
                .push((id.clone(), record.present));
        }
        let existing: BTreeMap<_, _> = keys
            .into_iter()
            .filter_map(|(key, ids)| {
                let present: Vec<_> = ids.iter().filter(|(_, present)| *present).collect();
                if present.len() == 1 {
                    Some((key, present[0].0.clone()))
                } else if ids.len() == 1 {
                    Some((key, ids[0].0.clone()))
                } else {
                    None
                }
            })
            .collect();
        // Uniqueness is a property of complete snapshots, not just unmatched
        // leftovers after exact-key reconciliation. An unchanged duplicate still
        // makes a purported rename ambiguous.
        let mut old_counts = BTreeMap::new();
        for record in source.records.values() {
            *old_counts.entry(record.fingerprint.clone()).or_insert(0) += 1;
        }
        let mut incoming_counts = BTreeMap::new();
        for fp in incoming.values() {
            *incoming_counts.entry(fp.clone()).or_insert(0) += 1;
        }
        for record in source.records.values_mut() {
            record.present = false;
        }
        let mut remaining = Vec::new();
        for (key, fp) in incoming {
            if let Some(id) = existing.get(&key) {
                if let Some(record) = source.records.get_mut(id) {
                    record.present = true;
                    record.fingerprint = fp;
                }
            } else {
                remaining.push((key, fp));
            }
        }
        let mut old: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (id, r) in &source.records {
            if !r.present {
                old.entry(r.fingerprint.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
        for (key, fp) in &remaining {
            let id = old
                .get(fp)
                .filter(|ids| ids.len() == 1 && old_counts[fp] == 1 && incoming_counts[fp] == 1)
                .map(|ids| ids[0].clone())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let r = source.records.entry(id).or_insert_with(|| Record {
                key: key.clone(),
                fingerprint: fp.clone(),
                present: true,
                tags: BTreeSet::new(),
                extra: BTreeMap::new(),
            });
            r.key = key.clone();
            r.present = true;
        }
        source.version = version.into();
        self.revision = self.revision.checked_add(1).ok_or("RevisionOverflow")?;
        Ok(true)
    }
}
pub(crate) fn load_snapshot(
    path: &str,
    version: &str,
    references: &[Reference],
) -> Result<Metadata> {
    crate::registry::recover_search_transaction()?;
    let _lock = lock()?;
    let mut metadata = read()?;
    if metadata.reconcile(path, version, references)? {
        write(&metadata)?;
    }
    Ok(metadata)
}
pub fn snapshot(path: String, key: String) -> Result<TagSnapshot> {
    crate::registry::recover_search_transaction()?;
    let canonical = if path.is_empty() {
        path
    } else {
        fs::canonicalize(path)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned()
    };
    let _lock = lock()?;
    Ok(read()?.snapshot(&canonical, &key))
}
pub fn mutate(mutation: TagMutation) -> Result<TagSnapshot> {
    crate::registry::recover_search_transaction()?;
    if mutation.action == TagAction::Relocate {
        crate::registry::relocate_search_source(
            &mutation.id,
            &mutation.path,
            mutation.expected_revision,
        )?;
        return snapshot(mutation.path, mutation.cite_key);
    }
    let canonical = if mutation.path.is_empty() {
        String::new()
    } else {
        fs::canonicalize(&mutation.path)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned()
    };
    let _lock = lock()?;
    let mut data = read()?;
    if data.revision != mutation.expected_revision {
        return Err("StaleTagRevision".into());
    }
    match mutation.action {
        TagAction::Create => {
            let name = display(&mutation.name)?;
            if !data
                .tags
                .values()
                .any(|t| comparison(&t.name) == comparison(&name))
            {
                let id = Uuid::new_v4().to_string();
                data.tags.insert(
                    id.clone(),
                    StoredTag {
                        id,
                        name,
                        extra: BTreeMap::new(),
                    },
                );
            }
        }
        TagAction::Rename => {
            let name = display(&mutation.name)?;
            if data
                .tags
                .values()
                .any(|t| t.id != mutation.id && comparison(&t.name) == comparison(&name))
            {
                return Err("TagNameConflict".into());
            }
            data.tags
                .get_mut(&mutation.id)
                .ok_or("MissingTagBinding")?
                .name = name;
        }
        TagAction::Delete | TagAction::Merge => {
            if mutation.action == TagAction::Merge
                && (mutation.id == mutation.target_id
                    || !data.tags.contains_key(&mutation.target_id))
            {
                return Err("InvalidMergeTarget".into());
            }
            data.tags.remove(&mutation.id).ok_or("MissingTagBinding")?;
            for s in data.sources.values_mut() {
                for r in s.records.values_mut() {
                    if r.tags.remove(&mutation.id) && mutation.action == TagAction::Merge {
                        r.tags.insert(mutation.target_id.clone());
                    }
                }
            }
            if mutation.action == TagAction::Merge {
                data.redirects
                    .insert(mutation.id.clone(), mutation.target_id.clone());
            }
        }
        TagAction::Assign | TagAction::Unassign => {
            if !data.tags.contains_key(&mutation.id) {
                return Err("MissingTagBinding".into());
            }
            let r = data
                .sources
                .values_mut()
                .find(|s| s.path == canonical)
                .and_then(|s| {
                    s.records
                        .values_mut()
                        .find(|r| r.present && r.key == mutation.cite_key)
                })
                .ok_or("ReferenceNotFound")?;
            if mutation.action == TagAction::Assign {
                r.tags.insert(mutation.id);
            } else {
                r.tags.remove(&mutation.id);
            }
        }
        TagAction::Relink => {
            let old = data
                .sources
                .values()
                .find_map(|s| s.records.get(&mutation.id))
                .filter(|r| !r.present)
                .cloned()
                .ok_or("PendingReferenceNotFound")?;
            let target = data
                .sources
                .values_mut()
                .find(|s| s.path == canonical)
                .and_then(|s| {
                    s.records
                        .values_mut()
                        .find(|r| r.present && r.key == mutation.cite_key)
                })
                .ok_or("ReferenceNotFound")?;
            target.tags.extend(old.tags);
            for s in data.sources.values_mut() {
                s.records.remove(&mutation.id);
            }
        }
        TagAction::Relocate => {
            if data.source(&canonical).is_some() {
                return Err("SourceAlreadyExists".into());
            }
            let s = data.sources.get_mut(&mutation.id).ok_or("SourceNotFound")?;
            s.path = canonical.clone();
            s.version = String::new();
        }
    }
    data.revision = data.revision.checked_add(1).ok_or("RevisionOverflow")?;
    write(&data)?;
    Ok(data.snapshot(&canonical, &mutation.cite_key))
}

#[cfg(test)]
mod tests;
