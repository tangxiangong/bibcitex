use super::*;
fn refs(source: &str) -> Vec<Reference> {
    crate::core::utils::read_bibliography(biblatex::Bibliography::parse(source).unwrap())
}
#[test]
fn identities_follow_exact_keys_or_unique_full_fingerprints_only() {
    let mut m = Metadata {
        schema: 2,
        ..Metadata::default()
    };
    let a = refs("@article{A,title={same},note={x}} @article{B,title={same},note={y}}");
    m.reconcile("/one-uid", "/one", "v1", &a).unwrap();
    let id = m
        .source("/one")
        .unwrap()
        .records
        .iter()
        .find(|(_, r)| r.key == "A")
        .unwrap()
        .0
        .clone();
    let source = m.sources.values_mut().next().unwrap();
    source
        .records
        .get_mut(&id)
        .unwrap()
        .tags
        .insert("tag".into());
    m.reconcile(
        "/one-uid",
        "/one",
        "v2",
        &refs("@article{RENAMED,note={x},title={same}} @article{B,title={same},note={changed}}"),
    )
    .unwrap();
    let source = m.source("/one").unwrap();
    assert_eq!(source.records[&id].key, "RENAMED");
    assert!(source.records[&id].tags.contains("tag"));
    m.reconcile("/copy-uid", "/copy", "v2", &a).unwrap();
    assert!(
        m.source("/copy")
            .unwrap()
            .records
            .values()
            .all(|r| r.tags.is_empty())
    );
    m.reconcile(
        "/one-uid",
        "/one",
        "v3",
        &refs("@article{UNKNOWN,title={changed},note={new}}"),
    )
    .unwrap();
    assert!(!m.source("/one").unwrap().records[&id].present);
}
#[test]
fn duplicate_content_does_not_guess_a_rename() {
    let mut m = Metadata {
        schema: 2,
        ..Metadata::default()
    };
    m.reconcile(
        "/x-uid",
        "/x",
        "1",
        &refs("@article{A,title={same}} @article{B,title={same}}"),
    )
    .unwrap();
    let old: Vec<_> = m.source("/x").unwrap().records.keys().cloned().collect();
    m.reconcile(
        "/x-uid",
        "/x",
        "2",
        &refs("@article{C,title={same}} @article{D,title={same}}"),
    )
    .unwrap();
    assert!(
        old.iter()
            .all(|id| !m.source("/x").unwrap().records[id].present)
    );
}
#[test]
fn unicode_casefold_and_atomic_unknown_fields() {
    assert_eq!(comparison("  STRASSE \t x "), comparison("Straße x"));
    let _guard = lock().unwrap();
    let mut data = read().unwrap();
    data.extra
        .insert("future".into(), serde_json::json!({"keep":true}));
    write(&data).unwrap();
    assert_eq!(read().unwrap().extra["future"]["keep"], true);
    let original = fs::read(path()).unwrap();
    fs::write(path(), b"{broken").unwrap();
    assert!(read().is_err());
    assert_eq!(fs::read(path()).unwrap(), b"{broken");
    fs::write(path(), original).unwrap();
}
#[test]
fn metadata_process_worker() {
    let Ok(name) = std::env::var("BIBCITEX_TAG_WORKER") else {
        return;
    };
    for _ in 0..100 {
        let snapshot = snapshot(String::new(), String::new()).unwrap();
        let result = mutate(TagMutation {
            library_uid: String::new(),
            record_uid: String::new(),
            color: None,
            action: TagAction::Create,
            id: String::new(),
            name: name.clone(),
            target_id: String::new(),
            path: String::new(),
            cite_key: String::new(),
            expected_revision: snapshot.revision,
        });
        match result {
            Ok(_) => return,
            Err(e) if e == "StaleTagRevision" => continue,
            Err(e) => panic!("{e}"),
        }
    }
    panic!("Too many metadata conflicts");
}
#[test]
fn independent_process_tag_writes_do_not_lose_updates() {
    let directory = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut children = Vec::new();
    for i in 0..6 {
        children.push(
            std::process::Command::new(&executable)
                .args([
                    "--exact",
                    "tags::tests::metadata_process_worker",
                    "--nocapture",
                ])
                .env("BIBCITEX_TEST_REGISTRY_DIRECTORY", directory.path())
                .env("BIBCITEX_TAG_WORKER", format!("tag-{i}"))
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
    }
    for mut child in children {
        assert!(child.wait().unwrap().success());
    }
    let data: Metadata =
        serde_json::from_slice(&fs::read(directory.path().join("search-metadata.json")).unwrap())
            .unwrap();
    assert_eq!(data.tags.len(), 6);
    assert_eq!(data.revision, 6);
}

#[test]
fn unchanged_duplicate_still_prevents_automatic_fingerprint_relink() {
    let mut m = Metadata {
        schema: 2,
        ..Metadata::default()
    };
    m.reconcile(
        "/x-uid",
        "/x",
        "1",
        &refs("@article{A,title={same}} @article{B,title={same}}"),
    )
    .unwrap();
    let old = m
        .source("/x")
        .unwrap()
        .records
        .iter()
        .find(|(_, r)| r.key == "A")
        .unwrap()
        .0
        .clone();
    m.reconcile(
        "/x-uid",
        "/x",
        "2",
        &refs("@article{C,title={same}} @article{B,title={same}}"),
    )
    .unwrap();
    assert!(!m.source("/x").unwrap().records[&old].present);
    assert_eq!(m.source("/x").unwrap().records[&old].key, "A");
}

fn command(
    action: TagAction,
    snapshot: &TagSnapshot,
    id: &str,
    name: &str,
    color: Option<&str>,
) -> TagMutation {
    TagMutation {
        library_uid: snapshot.library_uid.clone(),
        record_uid: snapshot.record_uid.clone(),
        color: color.map(str::to_owned),
        action,
        id: id.into(),
        name: name.into(),
        target_id: String::new(),
        path: String::new(),
        cite_key: String::new(),
        expected_revision: snapshot.revision,
    }
}

#[test]
fn uid_associations_survive_relocation_with_changed_content_and_share_tag_colors_across_libraries()
{
    let first = tempfile::NamedTempFile::new().unwrap();
    let second = tempfile::NamedTempFile::new().unwrap();
    let moved = tempfile::NamedTempFile::new().unwrap();
    fs::write(first.path(), "@article{A,title={original}}").unwrap();
    fs::write(second.path(), "@article{A,title={original}}").unwrap();
    fs::write(moved.path(), "@article{A,title={edited during move}}").unwrap();
    let library =
        crate::add_library("one".into(), first.path().to_str().unwrap().into(), None).unwrap();
    let other =
        crate::add_library("two".into(), second.path().to_str().unwrap().into(), None).unwrap();
    let original = crate::search(library.path.clone(), "".into(), "all".into(), "all".into())
        .unwrap()
        .remove(0);
    let copy = crate::search(other.path.clone(), "".into(), "all".into(), "all".into())
        .unwrap()
        .remove(0);
    // Compare identities without exposing their values in failure logs.
    assert!(
        original.id != copy.id,
        "records in separate libraries must have distinct identities"
    );
    assert!(
        library.uid != other.uid,
        "separate libraries must have distinct identities"
    );
    let initial = crate::tag_snapshot_uid(library.uid.clone(), original.id.clone()).unwrap();
    let created = crate::mutate_tag(command(
        TagAction::Create,
        &initial,
        "",
        "Read",
        Some("#3b82f6"),
    ))
    .unwrap();
    let tag = created.tags[0].id.clone();
    assert_eq!(created.assigned_ids.as_slice(), std::slice::from_ref(&tag));
    let copy_snapshot = crate::tag_snapshot_uid(other.uid.clone(), copy.id.clone()).unwrap();
    assert!(copy_snapshot.assigned_ids.is_empty());
    crate::mutate_tag(command(TagAction::Assign, &copy_snapshot, &tag, "", None)).unwrap();
    let updated = crate::update_library(
        "one".into(),
        "renamed".into(),
        Some(moved.path().to_str().unwrap().into()),
        None,
    )
    .unwrap();
    assert!(
        updated.uid == library.uid,
        "relocating a library must preserve its identity"
    );
    // The original file is still present, and content differs at the new location.
    assert!(first.path().exists());
    let relocated = crate::tag_snapshot_uid(library.uid.clone(), original.id.clone()).unwrap();
    assert_eq!(
        relocated.assigned_ids.as_slice(),
        std::slice::from_ref(&tag)
    );
    crate::mutate_tag(command(
        TagAction::Recolor,
        &relocated,
        &tag,
        "",
        Some("#22c55e"),
    ))
    .unwrap();
    for path in [&updated.path, &other.path] {
        let rows = crate::search(path.clone(), "".into(), "all".into(), "all".into()).unwrap();
        assert_eq!(rows[0].tags[0].color, "#22C55E");
        assert_eq!(rows[0].tags[0].id, tag);
    }
    let stored = fs::read_to_string(path()).unwrap();
    assert!(!stored.contains("\"path\""));
    assert!(!stored.contains(first.path().to_str().unwrap()));
    assert!(!stored.contains(moved.path().to_str().unwrap()));
    let stable =
        crate::search(updated.path.clone(), "".into(), "all".into(), "all".into()).unwrap();
    assert!(
        stable[0].id == original.id,
        "relocating a library must preserve record identities"
    );
    assert!(
        stable[0].library_uid == library.uid,
        "relocated records must retain their library identity"
    );
    let current = crate::tag_snapshot_uid(library.uid.clone(), original.id.clone()).unwrap();
    let before = fs::read(path()).unwrap();
    let mut invalid = command(TagAction::Recolor, &current, &tag, "", Some("red"));
    assert!(crate::mutate_tag(invalid.clone()).is_err());
    invalid.color = Some("#FFFFFF".into());
    invalid.expected_revision = current.revision.saturating_sub(1);
    assert!(crate::mutate_tag(invalid).is_err());
    assert_eq!(fs::read(path()).unwrap(), before);
}

#[test]
fn cross_library_record_pairs_reject_wrong_assignments_and_share_global_deletion() {
    let first = tempfile::NamedTempFile::new().unwrap();
    let second = tempfile::NamedTempFile::new().unwrap();
    for file in [&first, &second] {
        fs::write(file.path(), "@article{A,title={same cite key}}").unwrap();
    }
    let a = crate::tag_snapshot(first.path().to_string_lossy().into(), "A".into()).unwrap();
    let b = crate::tag_snapshot(second.path().to_string_lossy().into(), "A".into()).unwrap();
    let made = crate::mutate_tag(command(TagAction::Create, &b, "", "Shared", None)).unwrap();
    let tag = &made.tags[0].id;
    let mut wrong = command(TagAction::Assign, &made, tag, "", None);
    wrong.record_uid = a.record_uid.clone();
    let before = fs::read(path()).unwrap();
    assert!(
        crate::mutate_tag(wrong)
            .unwrap_err()
            .to_string()
            .contains("ReferenceNotFound")
    );
    assert_eq!(fs::read(path()).unwrap(), before);
    let a = crate::tag_snapshot_uid(a.library_uid, a.record_uid).unwrap();
    assert!(a.assigned_ids.is_empty());
    let assigned = crate::mutate_tag(command(TagAction::Assign, &a, tag, "", None)).unwrap();
    assert_eq!(assigned.assigned_ids, made.assigned_ids);
    let deleted = crate::mutate_tag(command(TagAction::Delete, &assigned, tag, "", None)).unwrap();
    assert!(deleted.assigned_ids.is_empty());
    assert!(
        crate::tag_snapshot_uid(b.library_uid, b.record_uid)
            .unwrap()
            .assigned_ids
            .is_empty()
    );
}

#[test]
fn stale_uid_location_cannot_create_a_second_library_identity() {
    let original = tempfile::NamedTempFile::new().unwrap();
    let target = tempfile::NamedTempFile::new().unwrap();
    fs::write(original.path(), "@article{A,title={original}}").unwrap();
    fs::write(target.path(), "@article{A,title={updated}}").unwrap();
    let library = crate::add_library(
        "Original".into(),
        original.path().to_string_lossy().into(),
        None,
    )
    .unwrap();
    let old_source = crate::api::references(&library.path).unwrap();
    let snapshot = crate::tag_snapshot_uid(library.uid.clone(), String::new()).unwrap();
    crate::update_library(
        library.name.clone(),
        library.name,
        Some(target.path().to_string_lossy().into()),
        None,
    )
    .unwrap();
    let before = fs::read(crate::registry::settings_path()).unwrap();
    assert_eq!(
        load_snapshot_uid(
            &library.uid,
            &library.path,
            &old_source.version,
            &old_source.references
        )
        .unwrap_err(),
        "SourceLocationChanged"
    );
    assert_eq!(fs::read(crate::registry::settings_path()).unwrap(), before);
    assert_eq!(
        crate::tag_snapshot_uid(library.uid, String::new())
            .unwrap()
            .library_uid,
        snapshot.library_uid
    );
}

#[test]
fn missing_tag_color_uses_default_without_changing_identity_or_assignments() {
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), "@article{A,title={Color default}}").unwrap();
    let selected = crate::tag_snapshot(file.path().to_string_lossy().into(), "A".into()).unwrap();
    let created =
        crate::mutate_tag(command(TagAction::Create, &selected, "", "Read", None)).unwrap();
    let tag_uid = created.tags[0].id.clone();
    let mut document: Value = serde_json::from_slice(&fs::read(path()).unwrap()).unwrap();
    document["tags"][&tag_uid]
        .as_object_mut()
        .unwrap()
        .remove("color");
    atomic_write(&path(), &document).unwrap();
    let original = fs::read(path()).unwrap();
    let loaded =
        crate::tag_snapshot_uid(created.library_uid.clone(), created.record_uid.clone()).unwrap();
    assert_eq!(loaded.library_uid, created.library_uid);
    assert_eq!(loaded.record_uid, created.record_uid);
    assert_eq!(loaded.assigned_ids, created.assigned_ids);
    assert_eq!(loaded.tags[0].id, tag_uid);
    assert_eq!(loaded.tags[0].color, default_color());
    assert_eq!(fs::read(path()).unwrap(), original);
    crate::mutate_tag(command(
        TagAction::Rename,
        &loaded,
        &tag_uid,
        "Read later",
        None,
    ))
    .unwrap();
    let saved: Value = serde_json::from_slice(&fs::read(path()).unwrap()).unwrap();
    assert_eq!(saved["tags"][&tag_uid]["color"], default_color());
    assert_eq!(saved["sources"], document["sources"]);
    document["tags"][&tag_uid]["color"] = serde_json::json!("invalid");
    atomic_write(&path(), &document).unwrap();
    assert_eq!(read().unwrap_err(), "InvalidTagColor");
}

#[test]
fn merge_and_relink_keep_global_tags_and_cross_library_records_distinct() {
    let a = tempfile::NamedTempFile::new().unwrap();
    let b = tempfile::NamedTempFile::new().unwrap();
    for file in [&a, &b] {
        fs::write(
            file.path(),
            "@article{A,title={same key}} @article{B,title={kept}}",
        )
        .unwrap();
    }
    let first = crate::tag_snapshot(a.path().to_string_lossy().into(), "A".into()).unwrap();
    let second = crate::tag_snapshot(b.path().to_string_lossy().into(), "A".into()).unwrap();
    let mut current =
        crate::tag_snapshot_uid(first.library_uid.clone(), first.record_uid.clone()).unwrap();
    current = crate::mutate_tag(command(TagAction::Create, &current, "", "Source", None)).unwrap();
    let source_tag = current.assigned_ids[0].clone();
    let mut target =
        crate::tag_snapshot_uid(second.library_uid.clone(), second.record_uid.clone()).unwrap();
    target = crate::mutate_tag(command(
        TagAction::Create,
        &target,
        "",
        "Destination",
        Some("#123456"),
    ))
    .unwrap();
    let target_tag = target.assigned_ids[0].clone();
    let mut merge = command(TagAction::Merge, &target, &source_tag, "", None);
    merge.target_id = target_tag.clone();
    crate::mutate_tag(merge).unwrap();
    let first_after =
        crate::tag_snapshot_uid(first.library_uid.clone(), first.record_uid.clone()).unwrap();
    assert_eq!(first_after.assigned_ids, vec![target_tag.clone()]);
    assert_eq!(first_after.tags[0].color, "#123456");
    assert_eq!(
        read().unwrap().redirected(&source_tag).unwrap().id,
        target_tag
    );
    fs::write(a.path(), "@article{B,title={kept}}").unwrap();
    let pending = crate::tag_snapshot(a.path().to_string_lossy().into(), "B".into()).unwrap();
    assert_eq!(pending.pending[0].record_id, first.record_uid);
    let target =
        crate::tag_snapshot_uid(second.library_uid.clone(), second.record_uid.clone()).unwrap();
    let result = crate::mutate_tag(command(
        TagAction::Relink,
        &target,
        &first.record_uid,
        "",
        None,
    ))
    .unwrap();
    assert!(result.pending.is_empty());
    assert_eq!(result.library_uid, second.library_uid);
    assert_eq!(result.record_uid, second.record_uid);
    assert_eq!(result.assigned_ids, vec![target_tag]);
    let data = read().unwrap();
    assert!(
        !data.sources[&first.library_uid]
            .records
            .contains_key(&first.record_uid)
    );
    assert!(
        data.sources[&second.library_uid]
            .records
            .contains_key(&second.record_uid)
    );
}

#[test]
fn batch_tags_are_atomic_deduplicated_and_cross_library() {
    let first = tempfile::NamedTempFile::new().unwrap();
    let second = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        first.path(),
        "@article{same,title={one}} @article{B,title={two}}",
    )
    .unwrap();
    fs::write(second.path(), "@article{same,title={three}}").unwrap();
    let a = crate::add_library(
        "batch-a".into(),
        first.path().to_str().unwrap().into(),
        None,
    )
    .unwrap();
    let b = crate::add_library(
        "batch-b".into(),
        second.path().to_str().unwrap().into(),
        None,
    )
    .unwrap();
    let rows = crate::search(a.path.clone(), "".into(), "all".into(), "all".into()).unwrap();
    let other = crate::search(b.path.clone(), "".into(), "all".into(), "all".into()).unwrap();
    let one = TagTarget {
        library_uid: a.uid.clone(),
        record_uid: rows[0].id.clone(),
    };
    let two = TagTarget {
        library_uid: b.uid.clone(),
        record_uid: other[0].id.clone(),
    };
    let targets = vec![one.clone(), two.clone(), one.clone()];
    let snapshot = crate::batch_tag_snapshot(targets.clone()).unwrap();
    assert_eq!(snapshot.count, 2);
    let create = BatchTagMutation {
        targets: targets.clone(),
        action: TagAction::Create,
        id: String::new(),
        name: "Batch".into(),
        expected_revision: snapshot.revision,
    };
    let assigned = crate::mutate_batch_tag(create.clone()).unwrap();
    let id = assigned
        .tags
        .iter()
        .find(|tag| tag.name == "Batch")
        .unwrap()
        .id
        .clone();
    assert_eq!(assigned.assigned_ids, vec![id.clone()]);
    assert!(assigned.partial_ids.is_empty());
    let bytes = fs::read(path()).unwrap();
    assert!(
        crate::mutate_batch_tag(create)
            .unwrap_err()
            .to_string()
            .contains("StaleTagRevision")
    );
    assert_eq!(bytes, fs::read(path()).unwrap());
    let wrong = TagTarget {
        library_uid: a.uid.clone(),
        record_uid: other[0].id.clone(),
    };
    for (action, tag, selection) in [
        (TagAction::Unassign, id.clone(), vec![one.clone(), wrong]),
        (TagAction::Assign, "missing-tag".into(), targets.clone()),
        (TagAction::Delete, id.clone(), targets.clone()),
        (TagAction::Create, String::new(), Vec::new()),
    ] {
        let result = crate::mutate_batch_tag(BatchTagMutation {
            targets: selection,
            action,
            id: tag,
            name: "Must not exist".into(),
            expected_revision: assigned.revision,
        });
        assert!(result.is_err());
        assert_eq!(bytes, fs::read(path()).unwrap());
    }
    crate::mutate_batch_tag(BatchTagMutation {
        targets: vec![one.clone()],
        action: TagAction::Unassign,
        id: id.clone(),
        name: String::new(),
        expected_revision: assigned.revision,
    })
    .unwrap();
    let partial = crate::batch_tag_snapshot(targets.clone()).unwrap();
    assert!(partial.assigned_ids.is_empty());
    assert_eq!(partial.partial_ids, vec![id.clone()]);
    let result = crate::mutate_batch_tag(BatchTagMutation {
        targets: targets.clone(),
        action: TagAction::Assign,
        id: id.clone(),
        name: String::new(),
        expected_revision: partial.revision,
    })
    .unwrap();
    assert_eq!(result.assigned_ids, vec![id.clone()]);
    let untouched = crate::tag_snapshot_uid(a.uid.clone(), rows[1].id.clone()).unwrap();
    assert!(untouched.assigned_ids.is_empty());
    let removed = crate::mutate_batch_tag(BatchTagMutation {
        targets,
        action: TagAction::Unassign,
        id,
        name: String::new(),
        expected_revision: result.revision,
    })
    .unwrap();
    assert!(removed.assigned_ids.is_empty() && removed.partial_ids.is_empty());
    assert_eq!(
        removed.tags.len(),
        1,
        "removing associations never deletes the global tag"
    );
}

#[test]
fn batch_targets_survive_library_relocation_and_reject_deleted_records() {
    let original = tempfile::NamedTempFile::new().unwrap();
    let moved = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        original.path(),
        "@article{A,title={one}} @article{B,title={two}}",
    )
    .unwrap();
    fs::copy(original.path(), moved.path()).unwrap();
    let library = crate::add_library(
        "batch-move".into(),
        original.path().to_str().unwrap().into(),
        None,
    )
    .unwrap();
    let rows = crate::search(library.path.clone(), "".into(), "all".into(), "all".into()).unwrap();
    let targets: Vec<_> = rows
        .iter()
        .map(|r| TagTarget {
            library_uid: r.library_uid.clone(),
            record_uid: r.id.clone(),
        })
        .collect();
    crate::update_library(
        library.name.clone(),
        library.name,
        Some(moved.path().to_str().unwrap().into()),
        None,
    )
    .unwrap();
    let snapshot = crate::batch_tag_snapshot(targets.clone()).unwrap();
    let mutation = BatchTagMutation {
        targets: targets.clone(),
        action: TagAction::Create,
        id: String::new(),
        name: "Stable".into(),
        expected_revision: snapshot.revision,
    };
    let tagged = crate::mutate_batch_tag(mutation.clone()).unwrap();
    assert_eq!(tagged.assigned_ids.len(), 1);
    fs::write(moved.path(), "@article{A,title={one}}").unwrap();
    assert!(crate::batch_tag_snapshot(targets).is_err());
    let bytes = fs::read(path()).unwrap();
    let mut mutation = mutation;
    mutation.expected_revision = read().unwrap().revision;
    mutation.name = "Never partially created".into();
    assert!(crate::mutate_batch_tag(mutation).is_err());
    assert_eq!(bytes, fs::read(path()).unwrap());
}
