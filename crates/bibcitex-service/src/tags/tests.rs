use super::*;
fn refs(source: &str) -> Vec<Reference> {
    crate::core::utils::read_bibliography(biblatex::Bibliography::parse(source).unwrap())
}
#[test]
fn identities_follow_exact_keys_or_unique_full_fingerprints_only() {
    let mut m = Metadata {
        schema: 1,
        ..Metadata::default()
    };
    let a = refs("@article{A,title={same},note={x}} @article{B,title={same},note={y}}");
    m.reconcile("/one", "v1", &a).unwrap();
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
        "/one",
        "v2",
        &refs("@article{RENAMED,note={x},title={same}} @article{B,title={same},note={changed}}"),
    )
    .unwrap();
    let source = m.source("/one").unwrap();
    assert_eq!(source.records[&id].key, "RENAMED");
    assert!(source.records[&id].tags.contains("tag"));
    m.reconcile("/copy", "v2", &a).unwrap();
    assert!(
        m.source("/copy")
            .unwrap()
            .records
            .values()
            .all(|r| r.tags.is_empty())
    );
    m.reconcile(
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
        schema: 1,
        ..Metadata::default()
    };
    m.reconcile(
        "/x",
        "1",
        &refs("@article{A,title={same}} @article{B,title={same}}"),
    )
    .unwrap();
    let old: Vec<_> = m.source("/x").unwrap().records.keys().cloned().collect();
    m.reconcile(
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
        schema: 1,
        ..Metadata::default()
    };
    m.reconcile(
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
        "/x",
        "2",
        &refs("@article{C,title={same}} @article{B,title={same}}"),
    )
    .unwrap();
    assert!(!m.source("/x").unwrap().records[&old].present);
    assert_eq!(m.source("/x").unwrap().records[&old].key, "A");
}
