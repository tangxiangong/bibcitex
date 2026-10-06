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

fn fixture(tag: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[17; 32]);
    let version = versions(tag).unwrap();
    for arch in ["arm64", "x86_64"] {
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
    (
        dir,
        base64::engine::general_purpose::STANDARD.encode(key.verifying_key().as_bytes()),
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
fn complete_release_checks_signatures_hashes_and_locales() {
    let (dir, key) = fixture("v0.7.0-beta.1");
    let result = prepare(dir.path(), "v0.7.0-beta.1", "owner/repo", &key).unwrap();
    assert_eq!(result.channel, "beta");
    assert!(dir.path().join("SHA256SUMS").exists());
    fs::write(
        dir.path().join("BibCiTeX-0.7.0-beta.1-macos-arm64.app.zip"),
        b"tampered",
    )
    .unwrap();
    assert!(prepare(dir.path(), "v0.7.0-beta.1", "owner/repo", &key).is_err());
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
    let wrong = base64::engine::general_purpose::STANDARD
        .encode(SigningKey::from_bytes(&[18; 32]).verifying_key().as_bytes());
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
fn upload_failure_leaves_draft_and_never_advances_feeds() {
    let (dir, key) = fixture("v1.0.0");
    let mut calls = Vec::new();
    let result = publish_release("v1.0.0", dir.path(), "owner/repo", &key, |args| {
        calls.push(args.to_vec());
        Ok(if args[0] == "api" || args[1] == "view" {
            CommandResult {
                success: false,
                output: String::new(),
                error: "404".into(),
            }
        } else if args[1] == "upload" {
            CommandResult {
                success: false,
                output: String::new(),
                error: "upload failed".into(),
            }
        } else {
            CommandResult {
                success: true,
                output: String::new(),
                error: String::new(),
            }
        })
    });
    assert!(result.is_err());
    assert!(
        !calls
            .iter()
            .any(|a| a[1] == "edit" || a.iter().any(|x| x == HUB))
    );
}
#[test]
fn untrusted_paths_and_xml_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    assert!(safe_file(dir.path(), "../secret").is_err());
    assert!(safe_file(dir.path(), "C:\\file").is_err());
    assert!(parse_xml("<!DOCTYPE x><rss/>").is_err());
    assert!(parse_xml("<rss>").is_err());
}

#[test]
fn publish_orders_payloads_before_feeds_and_catalog_last() {
    let (dir, key) = fixture("v1.0.0-beta.2");
    let mut calls = Vec::new();
    publish_release("v1.0.0-beta.2", dir.path(), "owner/repo", &key, |args| {
        calls.push(args.to_vec());
        Ok(if args[0] == "api" || args[1] == "view" {
            CommandResult {
                success: false,
                output: String::new(),
                error: "404".into(),
            }
        } else {
            CommandResult {
                success: true,
                output: String::new(),
                error: String::new(),
            }
        })
    })
    .unwrap();
    let full_upload = calls
        .iter()
        .position(|a| a.get(2).is_some_and(|x| x == HUB) && a.iter().any(|x| x.ends_with(".nupkg")))
        .unwrap();
    let first_feed = calls
        .iter()
        .position(|a| a.get(2).is_some_and(|x| x == HUB) && a.iter().any(|x| x.ends_with(".xml")))
        .unwrap();
    assert!(full_upload < first_feed);
    assert!(
        calls
            .last()
            .unwrap()
            .last()
            .unwrap()
            .ends_with("channels.json")
    );
    assert!(
        calls.iter().any(
            |a| a.contains(&"--prerelease=true".into()) && a.contains(&"--latest=false".into())
        )
    );
    assert!(!calls.iter().flatten().any(|a| a.contains("-stable")));
}

#[test]
fn retry_of_published_release_never_overwrites_versioned_assets() {
    let (dir, key) = fixture("v1.0.0");
    let mut calls = Vec::new();
    publish_release("v1.0.0", dir.path(), "owner/repo", &key, |args| {
        calls.push(args.to_vec());
        if args[1] == "view" {
            return Ok(CommandResult {
                success: true,
                output: r#"{"isDraft":false}"#.into(),
                error: String::new(),
            });
        }
        if args[1] == "download" {
            let destination = &args[args.iter().position(|x| x == "--dir").unwrap() + 1];
            fs::copy(
                dir.path().join("release.json"),
                Path::new(destination).join("release.json"),
            )?;
        }
        Ok(if args[0] == "api" {
            CommandResult {
                success: false,
                output: String::new(),
                error: "404".into(),
            }
        } else {
            CommandResult {
                success: true,
                output: String::new(),
                error: String::new(),
            }
        })
    })
    .unwrap();
    assert!(!calls.iter().any(|a| a[1] == "upload" && a[2] == "v1.0.0"));
}

#[test]
fn an_older_stable_release_cannot_replace_github_latest() {
    let (dir, key) = fixture("v1.0.0");
    let mut calls = Vec::new();
    publish_release("v1.0.0", dir.path(), "owner/repo", &key, |args| {
        calls.push(args.to_vec());
        Ok(if args[0] == "api" && args[1].ends_with("/latest") {
            CommandResult {
                success: true,
                output: r#"{"tag_name":"v1.1.0"}"#.into(),
                error: String::new(),
            }
        } else if args[0] == "api" || args[1] == "view" {
            CommandResult {
                success: false,
                output: String::new(),
                error: "404".into(),
            }
        } else {
            CommandResult {
                success: true,
                output: String::new(),
                error: String::new(),
            }
        })
    })
    .unwrap();
    let edit = calls
        .iter()
        .find(|args| args[1] == "edit" && args[2] == "v1.0.0")
        .unwrap();
    assert!(edit.contains(&"--latest=false".into()));
}

#[test]
fn latest_lookup_failure_cannot_start_publication() {
    let (dir, key) = fixture("v1.0.0");
    let mut calls = Vec::new();
    let result = publish_release("v1.0.0", dir.path(), "owner/repo", &key, |args| {
        calls.push(args.to_vec());
        Ok(CommandResult {
            success: false,
            output: String::new(),
            error: "403 rate limited".into(),
        })
    });
    assert!(result.is_err());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0][0], "api");
}
