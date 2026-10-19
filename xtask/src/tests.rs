use super::*;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;

#[test]
fn release_hash_matches_standard_sha256_encoding() {
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), b"abc").unwrap();
    assert_eq!(
        hash(file.path()).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

pub(crate) fn fixture(tag: &str) -> (tempfile::TempDir, UpdateKeys) {
    fixture_with_macos(tag, &["universal"])
}

pub(crate) fn fixture_with_macos(
    tag: &str,
    architectures: &[&str],
) -> (tempfile::TempDir, UpdateKeys) {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[17; 32]);
    let version = versions(tag).unwrap();
    for arch in architectures {
        let name = format!("BibCiTeX-{}-macos-{arch}.app.zip", version.version);
        let bytes = format!("fixture-{arch}").into_bytes();
        fs::write(dir.path().join(&name), &bytes).unwrap();
        fs::write(
            dir.path()
                .join(format!("BibCiTeX-{}-macos-{arch}.dmg", version.version)),
            b"disk-image",
        )
        .unwrap();
        let sig = base64::engine::general_purpose::STANDARD.encode(key.sign(&bytes).to_bytes());
        fs::write(dir.path().join(format!("appcast-{arch}.xml")), format!(r#"<?xml version="1.0"?><rss xmlns:sparkle="{SPARKLE}"><channel><item><sparkle:version>42</sparkle:version><sparkle:shortVersionString>{}</sparkle:shortVersionString><enclosure url="https://github.com/owner/repo/releases/download/{tag}/{name}" length="{}" sparkle:edSignature="{sig}" /></item></channel></rss>"#, version.version, bytes.len())).unwrap();
    }
    for arch in ["arm64", "x64"] {
        for ext in ["exe", "msi"] {
            fs::write(
                dir.path()
                    .join(format!("BibCiTeX-{}-windows-{arch}.{ext}", version.version)),
                b"installer",
            )
            .unwrap();
        }
        let name = format!(
            "BibCiTeX-{}-win-{arch}-{}-full.nupkg",
            version.version, version.channel
        );
        let path = dir.path().join(&name);
        fs::write(&path, b"package").unwrap();
        fs::write(dir.path().join(format!("releases.win-{arch}-{}.json", version.channel)), json!({"Assets":[{"PackageId":"BibCiTeX","Version":version.version,"Type":"Full","FileName":name,"Size":7,"SHA256":hash(&path).unwrap()}]}).to_string()).unwrap();
    }
    fs::write(
        dir.path().join("notes-en.md"),
        "# Updates\n\n- Fixed citations",
    )
    .unwrap();
    fs::write(dir.path().join("notes-zh-Hans.md"), "# 更新\n\n- 修复引用").unwrap();
    let windows = crate::windows::test_sign(dir.path(), &version.version, &version.channel);
    (
        dir,
        UpdateKeys {
            sparkle: base64::engine::general_purpose::STANDARD
                .encode(key.verifying_key().as_bytes()),
            windows,
        },
    )
}
#[test]
fn release_versions_accept_channels_and_reject_ambiguous_tags() {
    for (tag, channel) in [
        ("v0.6.0", "stable"),
        ("v0.7.0-alpha.10", "alpha"),
        ("v0.7.0-beta.2", "beta"),
    ] {
        assert_eq!(versions(tag).unwrap().channel, channel);
    }
    for tag in [
        "0.6.0",
        "v01.0.0",
        "v1.0.0-rc.1",
        "v1.0.0-beta",
        "v1.0.0-beta.01",
        "v1.0.0+build.1",
        "v1.0.0-alpha.1.2",
    ] {
        assert!(versions(tag).is_err(), "{tag}");
    }
}

#[test]
fn missing_windows_signatures_block_publication_before_network_writes() {
    let (directory, keys) = fixture("v1.0.0");
    fs::remove_file(directory.path().join("releases.win-x64-stable.signed.json")).unwrap();
    assert!(
        publish_release("v1.0.0", directory.path(), "owner/repo", &keys, |_| {
            panic!("Unverified Windows metadata must not reach GitHub")
        })
        .is_err()
    );
}

#[test]
fn stable_windows_signatures_are_preserved_in_every_promoted_audience() {
    let (directory, keys) = fixture("v1.0.0");
    let manifest = prepare(directory.path(), "v1.0.0", "owner/repo", &keys).unwrap();
    let output = tempfile::tempdir().unwrap();
    feeds(
        directory.path(),
        output.path(),
        "owner/repo",
        &manifest,
        &mut Channels::default(),
    )
    .unwrap();
    for arch in ["x64", "arm64"] {
        let signed = fs::read(
            directory
                .path()
                .join(format!("releases.win-{arch}-stable.signed.json")),
        )
        .unwrap();
        for audience in ["stable", "beta", "alpha"] {
            assert_eq!(
                signed,
                fs::read(
                    output
                        .path()
                        .join(format!("releases.win-{arch}-{audience}.signed.json"))
                )
                .unwrap()
            );
            assert!(
                output
                    .path()
                    .join(format!("releases.win-{arch}-{audience}.json"))
                    .exists()
            );
        }
    }
}
#[test]
fn complete_release_checks_signatures_hashes_and_locales() {
    let (dir, key) = fixture("v0.7.0-beta.1");
    let result = prepare(dir.path(), "v0.7.0-beta.1", "owner/repo", &key).unwrap();
    assert_eq!(result.channel, "beta");
    assert!(dir.path().join("SHA256SUMS").exists());
    fs::write(
        dir.path()
            .join("BibCiTeX-0.7.0-beta.1-macos-universal.app.zip"),
        b"corrupt-universal",
    )
    .unwrap();
    assert!(prepare(dir.path(), "v0.7.0-beta.1", "owner/repo", &key).is_err());
}

#[test]
fn universal_archive_reaches_both_architectures_in_every_channel_and_locale() {
    let (dir, key) = fixture("v1.0.0");
    let manifest = prepare(dir.path(), "v1.0.0", "owner/repo", &key).unwrap();
    let macos_packages: Vec<_> = manifest
        .assets
        .keys()
        .filter(|name| name.contains("-macos-"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        macos_packages,
        [
            "BibCiTeX-1.0.0-macos-universal.app.zip",
            "BibCiTeX-1.0.0-macos-universal.dmg"
        ]
    );
    let source =
        parse_xml(&fs::read_to_string(dir.path().join("appcast-universal.xml")).unwrap()).unwrap();
    let signature = source
        .one("", "channel")
        .unwrap()
        .one("", "item")
        .unwrap()
        .one("", "enclosure")
        .unwrap()
        .attr(SPARKLE, "edSignature");
    let output = tempfile::tempdir().unwrap();
    let names = feeds(
        dir.path(),
        output.path(),
        "owner/repo",
        &manifest,
        &mut Channels::default(),
    )
    .unwrap();
    assert_eq!(
        names.iter().filter(|name| name.ends_with(".xml")).count(),
        12
    );
    for arch in ["arm64", "x86_64"] {
        for audience in ["stable", "beta", "alpha"] {
            for locale in ["en", "zh-Hans"] {
                let name = format!("appcast-{arch}-{audience}-{locale}.xml");
                assert!(names.contains(&name));
                let xml =
                    parse_xml(&fs::read_to_string(output.path().join(&name)).unwrap()).unwrap();
                let item = xml.one("", "channel").unwrap().one("", "item").unwrap();
                let enclosure = item.one("", "enclosure").unwrap();
                assert_eq!(
                    enclosure.attr("", "url"),
                    "https://github.com/owner/repo/releases/download/v1.0.0/BibCiTeX-1.0.0-macos-universal.app.zip"
                );
                assert_eq!(enclosure.attr(SPARKLE, "edSignature"), signature);
                assert_eq!(
                    item.one(SPARKLE, "releaseNotesLink").unwrap().text,
                    format!(
                        "https://github.com/owner/repo/releases/download/v1.0.0/notes-{locale}.md"
                    )
                );
            }
        }
    }
}

#[test]
fn legacy_macos_archives_keep_their_signed_architecture_feeds() {
    let (dir, key) = fixture_with_macos("v0.8.2", &["arm64", "x86_64"]);
    let manifest = prepare(dir.path(), "v0.8.2", "owner/repo", &key).unwrap();
    let output = tempfile::tempdir().unwrap();
    feeds(
        dir.path(),
        output.path(),
        "owner/repo",
        &manifest,
        &mut Channels::default(),
    )
    .unwrap();
    for arch in ["arm64", "x86_64"] {
        let feed = fs::read_to_string(output.path().join(format!("appcast-{arch}-stable-en.xml")))
            .unwrap();
        assert!(feed.contains(&format!("/v0.8.2/BibCiTeX-0.8.2-macos-{arch}.app.zip")));
    }
    fs::write(
        dir.path().join("BibCiTeX-0.8.2-macos-x86_64.app.zip"),
        b"corrupt-x86_64",
    )
    .unwrap();
    assert!(prepare(dir.path(), "v0.8.2", "owner/repo", &key).is_err());
}

#[test]
fn missing_universal_assets_block_publication_before_network_writes() {
    for name in [
        "appcast-universal.xml",
        "BibCiTeX-1.0.0-macos-universal.app.zip",
        "BibCiTeX-1.0.0-macos-universal.dmg",
    ] {
        let (dir, key) = fixture("v1.0.0");
        fs::remove_file(dir.path().join(name)).unwrap();
        let error = publish_release("v1.0.0", dir.path(), "owner/repo", &key, |_| {
            panic!("Incomplete Universal releases must not reach GitHub")
        })
        .unwrap_err();
        assert!(error.to_string().contains(name), "{error}");
    }
}

#[test]
fn universal_feed_cannot_substitute_a_different_payload() {
    let (dir, key) = fixture("v1.0.0");
    let path = dir.path().join("appcast-universal.xml");
    let xml = fs::read_to_string(&path)
        .unwrap()
        .replace("macos-universal.app.zip", "macos-arm64.app.zip");
    fs::write(path, xml).unwrap();
    assert!(
        publish_release("v1.0.0", dir.path(), "owner/repo", &key, |_| {
            panic!("Mismatched Universal metadata must not reach GitHub")
        })
        .is_err()
    );
}

#[test]
fn corrupted_windows_package_and_missing_installer_block_release() {
    let (dir, key) = fixture("v1.0.0");
    let path = dir
        .path()
        .join("BibCiTeX-1.0.0-win-arm64-stable-full.nupkg");
    fs::write(&path, b"corrupt").unwrap();
    assert!(
        prepare(dir.path(), "v1.0.0", "owner/repo", &key)
            .unwrap_err()
            .to_string()
            .contains("checksum")
    );
    fs::write(path, b"package").unwrap();
    fs::remove_file(dir.path().join("BibCiTeX-1.0.0-windows-x64.msi")).unwrap();
    assert!(prepare(dir.path(), "v1.0.0", "owner/repo", &key).is_err());
}
#[test]
fn missing_language_and_wrong_sparkle_key_block_release() {
    let (dir, key) = fixture("v1.0.0");
    let wrong = UpdateKeys {
        sparkle: base64::engine::general_purpose::STANDARD
            .encode(SigningKey::from_bytes(&[18; 32]).verifying_key().as_bytes()),
        ..key.clone()
    };
    assert!(prepare(dir.path(), "v1.0.0", "owner/repo", &wrong).is_err());
    fs::write(dir.path().join("notes-en.md"), " ").unwrap();
    assert!(prepare(dir.path(), "v1.0.0", "owner/repo", &key).is_err());
}
#[test]
fn channels_promote_stable_but_never_downgrade_newer_previews() {
    let (dir, key) = fixture("v1.0.0");
    let manifest = prepare(dir.path(), "v1.0.0", "owner/repo", &key).unwrap();
    let output = tempfile::tempdir().unwrap();
    let mut channels = Channels {
        versions: BTreeMap::from([
            ("alpha".into(), "1.1.0-alpha.2".into()),
            ("beta".into(), "1.0.0-beta.9".into()),
        ]),
    };
    let names = feeds(
        dir.path(),
        output.path(),
        "owner/repo",
        &manifest,
        &mut channels,
    )
    .unwrap();
    assert!(!names.iter().any(|name| name.contains("-alpha")));
    assert!(names.contains(&"releases.win-arm64-stable.json".into()));
    assert_eq!(channels.versions["beta"], "1.0.0");
    assert!(
        fs::read_to_string(output.path().join("appcast-arm64-beta-zh-Hans.xml"))
            .unwrap()
            .contains("notes-zh-Hans.md")
    );
}
#[test]
fn beta_does_not_touch_stable_and_numeric_precedence_is_respected() {
    let (dir, key) = fixture("v1.0.0-beta.10");
    let manifest = prepare(dir.path(), "v1.0.0-beta.10", "owner/repo", &key).unwrap();
    let mut channels = Channels {
        versions: BTreeMap::from([("beta".into(), "1.0.0-beta.9".into())]),
    };
    let output = tempfile::tempdir().unwrap();
    let names = feeds(
        dir.path(),
        output.path(),
        "owner/repo",
        &manifest,
        &mut channels,
    )
    .unwrap();
    assert!(!names.iter().any(|name| name.contains("stable")));
    assert_eq!(channels.versions["beta"], "1.0.0-beta.10");
}
#[test]
fn untrusted_paths_and_xml_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    assert!(safe_file(dir.path(), "../secret").is_err());
    assert!(safe_file(dir.path(), "C:\\file").is_err());
    assert!(parse_xml("<!DOCTYPE x><rss/>").is_err());
    assert!(parse_xml("<rss>").is_err());
}

// Stateful fake GitHub: exercise interruption and retry against a public release.
fn simulate_upload(fail_after: Option<usize>, conflict: bool) {
    let (dir, key) = fixture("v1.0.0");
    let mut remote = Vec::<serde_json::Value>::new();
    if conflict {
        remote.push(json!({"name":"notes-en.md","size":1,"digest":"sha256:wrong"}));
    }
    let mut version_uploads = Vec::new();
    let mut calls = Vec::new();
    let mut attempts = 0;
    for attempt in 0..2 {
        let result = publish_release("v1.0.0", dir.path(), "owner/repo", &key, |a| {
            calls.push(a.to_vec());
            let mut response = CommandResult {
                success: true,
                output: String::new(),
                error: String::new(),
            };
            if a[0] == "api" && a[1].ends_with("/tags/v1.0.0") {
                response.output =
                    json!({"tag_name":"v1.0.0","draft":false,"prerelease":false,"assets":remote})
                        .to_string();
            } else if a[0] == "api" && a[1].ends_with("/commits/v1.0.0") {
                response.output = "1234567890123456789012345678901234567890\n".into();
            } else if a[1] == "create" && a[2] == HUB {
                let target = a.iter().position(|arg| arg == "--target").unwrap() + 1;
                assert_eq!(a[target], "1234567890123456789012345678901234567890");
            } else if a[0] == "api" {
                response.success = false;
                response.error = "404".into();
            } else if a[1] == "upload" && a[2] == "v1.0.0" {
                assert!(!a.iter().any(|x| x == "--clobber"));
                if attempt == 0 && fail_after == Some(attempts) {
                    response.success = false;
                    response.error = "network interrupted".into();
                    return Ok(response);
                }
                attempts += 1;
                let path = Path::new(a.last().unwrap());
                let name = path.file_name().unwrap().to_str().unwrap();
                assert!(
                    !version_uploads.contains(&name.to_owned()),
                    "duplicate upload"
                );
                version_uploads.push(name.to_owned());
                remote.push(json!({"name":name,"size":fs::metadata(path)?.len(),"digest":format!("sha256:{}",hash(path)?)}));
            }
            Ok(response)
        });
        if conflict || (attempt == 0 && fail_after.is_some()) {
            assert!(result.is_err());
            assert!(!calls.iter().any(|a| a.iter().any(|s| s == HUB)));
        } else {
            result.unwrap();
        }
        if conflict {
            assert!(version_uploads.is_empty());
            break;
        }
    }
    if !conflict {
        assert_eq!(version_uploads.last().unwrap(), "release.json");
        let payload = calls
            .iter()
            .position(|a| {
                a.get(2).is_some_and(|x| x == HUB) && a.iter().any(|x| x.ends_with(".nupkg"))
            })
            .unwrap();
        let feed = calls
            .iter()
            .position(|a| {
                a.get(2).is_some_and(|x| x == HUB) && a.iter().any(|x| x.ends_with(".xml"))
            })
            .unwrap();
        assert!(payload < feed);
        assert!(
            calls
                .last()
                .unwrap()
                .last()
                .unwrap()
                .ends_with("channels.json")
        );
        assert!(
            !calls
                .iter()
                .any(|a| a[1] == "edit" || a[1] == "create" && a[2] == "v1.0.0")
        );
    }
}

#[test]
fn published_empty_release_accepts_assets_and_identical_retry_skips_them() {
    simulate_upload(None, false);
}
#[test]
fn interrupted_upload_resumes_missing_assets_before_advancing_feeds() {
    simulate_upload(Some(3), false);
}
#[test]
fn different_public_asset_is_never_overwritten() {
    simulate_upload(None, true);
}
#[test]
fn incorrect_release_identity_or_state_blocks_upload() {
    let (dir, key) = fixture("v1.0.0");
    for state in [
        json!({"tag_name":"v1.1.0","draft":false,"prerelease":false,"assets":[]}),
        json!({"tag_name":"v1.0.0","draft":true,"prerelease":false,"assets":[]}),
        json!({"tag_name":"v1.0.0","draft":false,"prerelease":true,"assets":[]}),
    ] {
        let mut calls = 0;
        assert!(
            publish_release("v1.0.0", dir.path(), "owner/repo", &key, |_| {
                calls += 1;
                Ok(CommandResult {
                    success: true,
                    output: state.to_string(),
                    error: String::new(),
                })
            })
            .is_err()
        );
        assert_eq!(calls, 1);
    }
}
#[test]
fn missing_asset_error_names_the_file() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        safe_file(dir.path(), "missing.exe")
            .unwrap_err()
            .to_string()
            .contains("missing.exe")
    );
}

#[test]
fn linux_payloads_without_signed_metadata_cannot_be_published() {
    let (dir, public) = fixture("v0.6.0");
    fs::write(
        dir.path().join("BibCiTeX-0.6.0-linux-arm64.tar.gz"),
        b"unsigned",
    )
    .unwrap();
    assert!(prepare(dir.path(), "v0.6.0", "owner/repo", &public).is_err());
}
