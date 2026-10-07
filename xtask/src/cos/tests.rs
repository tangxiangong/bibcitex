use super::*;
use std::cell::RefCell;

#[derive(Default)]
struct MemoryStore {
    objects: RefCell<BTreeMap<String, Vec<u8>>>,
    uploads: RefCell<Vec<String>>,
    fail: RefCell<Option<String>>,
}
impl Store for MemoryStore {
    fn catalog(&self) -> Result<Option<Vec<u8>>> {
        Ok(self.objects.borrow().get(CATALOG).cloned())
    }
    fn matches(&self, key: &str, path: &Path) -> Result<bool> {
        match self.objects.borrow().get(key) {
            None => Ok(false),
            Some(bytes) => {
                ensure(bytes == &fs::read(path)?, "Existing object differs")?;
                Ok(true)
            }
        }
    }
    fn upload(&self, key: &str, path: &Path, immutable: bool) -> Result<()> {
        if self.fail.borrow().as_deref() == Some(key) {
            return Err("Injected upload failure".into());
        }
        ensure(
            !immutable || !self.objects.borrow().contains_key(key),
            "Immutable overwrite",
        )?;
        self.objects
            .borrow_mut()
            .insert(key.into(), fs::read(path)?);
        self.uploads.borrow_mut().push(key.into());
        Ok(())
    }
}

#[test]
fn cos_verifies_payloads_then_reserves_versions_before_feeds() {
    let (dir, key) = crate::tests::fixture("v1.2.3");
    let store = MemoryStore::default();
    publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).unwrap();
    let uploads = store.uploads.borrow();
    let first_feed = uploads
        .iter()
        .position(|s| s.starts_with("update-feed/appcast-"))
        .unwrap();
    assert!(
        uploads
            .iter()
            .enumerate()
            .filter(|(_, s)| s.ends_with(".nupkg") || s.starts_with("releases/"))
            .all(|(index, _)| index < first_feed)
    );
    let catalog = uploads.iter().position(|s| s == CATALOG).unwrap();
    assert_eq!(catalog + 1, first_feed);
    assert!(
        uploads[..catalog]
            .iter()
            .all(|s| s.starts_with("releases/") || s.ends_with(".nupkg"))
    );
    let objects = store.objects.borrow();
    let feed =
        String::from_utf8(objects["update-feed/appcast-arm64-stable-zh-Hans.xml"].clone()).unwrap();
    assert!(feed.contains(&format!(
        "{PUBLIC_ROOT}/releases/v1.2.3/BibCiTeX-1.2.3-macos-arm64.app.zip"
    )));
    assert!(feed.contains(&format!("{PUBLIC_ROOT}/releases/v1.2.3/notes-zh-Hans.md")));
    assert!(!feed.contains("github.com"));
    assert!(feed.contains("sparkle:edSignature="));
}

#[test]
fn payload_failure_does_not_advance_feeds_and_retry_skips_existing_payloads() {
    let (dir, key) = crate::tests::fixture("v1.2.3");
    let store = MemoryStore::default();
    *store.fail.borrow_mut() = Some("releases/v1.2.3/notes-en.md".into());
    assert!(publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).is_err());
    assert!(
        !store
            .objects
            .borrow()
            .keys()
            .any(|s| s.starts_with("update-feed/appcast-"))
    );
    let before = store.uploads.borrow().clone();
    *store.fail.borrow_mut() = None;
    publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).unwrap();
    assert!(
        before.iter().filter(|s| *s != CATALOG).all(|s| store
            .uploads
            .borrow()
            .iter()
            .filter(|v| *v == s)
            .count()
            == 1)
    );
}

#[test]
fn partial_feed_upload_can_resume_without_replacing_packages() {
    let (dir, key) = crate::tests::fixture("v1.2.3");
    let store = MemoryStore::default();
    *store.fail.borrow_mut() = Some("update-feed/releases.win-x64-stable.json".into());
    assert!(publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).is_err());
    let catalog: Channels = serde_json::from_slice(&store.objects.borrow()[CATALOG]).unwrap();
    assert_eq!(catalog.versions["stable"], "1.2.3");
    *store.fail.borrow_mut() = None;
    publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).unwrap();
    let catalog: Channels = serde_json::from_slice(&store.objects.borrow()[CATALOG]).unwrap();
    assert_eq!(catalog.versions["stable"], "1.2.3");
}

#[test]
fn conflicting_payload_blocks_all_writes() {
    let (dir, key) = crate::tests::fixture("v1.2.3");
    let store = MemoryStore::default();
    store.objects.borrow_mut().insert(
        "releases/v1.2.3/notes-en.md".into(),
        b"conflicting".to_vec(),
    );
    assert!(publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).is_err());
    assert!(store.uploads.borrow().is_empty());
}

#[test]
fn older_release_does_not_replace_newer_channels() {
    let store = MemoryStore::default();
    for tag in ["v2.0.0-beta.1", "v1.2.3"] {
        let (dir, key) = crate::tests::fixture(tag);
        publish_to(tag, dir.path(), "owner/repo", &key, &store).unwrap();
    }
    let objects = store.objects.borrow();
    let catalog: Channels = serde_json::from_slice(&objects[CATALOG]).unwrap();
    assert_eq!(catalog.versions["stable"], "1.2.3");
    assert_eq!(catalog.versions["beta"], "2.0.0-beta.1");
    let feed = String::from_utf8(objects["update-feed/appcast-arm64-beta-en.xml"].clone()).unwrap();
    assert!(feed.contains("/v2.0.0-beta.1/"));
}

#[test]
fn invalid_catalog_is_not_reset() {
    let (dir, key) = crate::tests::fixture("v1.2.3");
    let store = MemoryStore::default();
    store
        .objects
        .borrow_mut()
        .insert(CATALOG.into(), b"bad json".to_vec());
    assert!(publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).is_err());
    assert!(store.uploads.borrow().is_empty());
}

#[test]
fn hmac_matches_rfc_2202_and_encoding_preserves_object_names() {
    assert_eq!(
        hmac(&[0x0b; 20], "Hi There").unwrap(),
        "b617318655057264e28bc0b6fb378c8ef146be00"
    );
    assert_eq!(encode("a b+中"), "a%20b%2B%E4%B8%AD");
}

#[test]
fn authorization_matches_independent_cos_v5_vector() {
    let headers = BTreeMap::from([
        ("content-type".into(), "application/octet-stream".into()),
        (
            "host".into(),
            "example.cos.ap-guangzhou.myqcloud.com".into(),
        ),
        ("x-cos-acl".into(), "public-read".into()),
    ]);
    let signature = authorization(
        "test-id",
        "test-secret",
        "PUT",
        "/bibcitex/test file.zip",
        &headers,
        1000,
    )
    .unwrap();
    assert_eq!(
        signature,
        "q-sign-algorithm=sha1&q-ak=test-id&q-sign-time=940;4600&q-key-time=940;4600&q-header-list=content-type;host;x-cos-acl&q-url-param-list=&q-signature=67b63dfbbe37d1b8d1190eaeef859731143d683d"
    );
}

#[test]
fn older_repair_cannot_regress_a_partially_published_newer_release() {
    let store = MemoryStore::default();
    let (newer, key) = crate::tests::fixture("v2.0.0");
    *store.fail.borrow_mut() = Some("update-feed/releases.win-x64-stable.json".into());
    assert!(publish_to("v2.0.0", newer.path(), "owner/repo", &key, &store).is_err());
    let partial = store.objects.borrow().clone();
    *store.fail.borrow_mut() = None;
    let (older, key) = crate::tests::fixture("v1.2.3");
    publish_to("v1.2.3", older.path(), "owner/repo", &key, &store).unwrap();
    for (name, bytes) in partial
        .iter()
        .filter(|(name, _)| name.starts_with("update-feed/appcast-"))
    {
        assert_eq!(&store.objects.borrow()[name], bytes);
    }
    assert!(
        !store
            .objects
            .borrow()
            .contains_key("update-feed/releases.win-x64-stable.json")
    );
    publish_to("v2.0.0", newer.path(), "owner/repo", &key, &store).unwrap();
    let objects = store.objects.borrow();
    let feed: serde_json::Value =
        serde_json::from_slice(&objects["update-feed/releases.win-x64-stable.json"]).unwrap();
    assert_eq!(feed["Assets"][0]["Version"], "2.0.0");
}

#[test]
fn failed_version_reservation_prevents_any_client_feed_write() {
    let (dir, key) = crate::tests::fixture("v1.2.3");
    let store = MemoryStore::default();
    *store.fail.borrow_mut() = Some(CATALOG.into());
    assert!(publish_to("v1.2.3", dir.path(), "owner/repo", &key, &store).is_err());
    assert!(
        store
            .objects
            .borrow()
            .keys()
            .all(|name| name.starts_with("releases/") || name.ends_with(".nupkg"))
    );
}
