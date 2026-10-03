use super::*;
use tempfile::{TempDir, tempdir};

fn mac_feed(signature: &str, size: u64, version: &str) -> TempDir {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("BibCiTeX.zip"), b"zip").unwrap();
    fs::write(dir.path().join("appcast-arm64.xml"), format!(r#"<rss xmlns:sparkle="{SPARKLE}"><channel><item><sparkle:shortVersionString>{version}</sparkle:shortVersionString><enclosure url="https://github.com/owner/repo/releases/download/v0.6.0/BibCiTeX.zip" length="{size}" sparkle:edSignature="{signature}" /></item></channel></rss>"#)).unwrap();
    dir
}
fn check_mac(dir: &Path) -> Result<()> {
    verify_release("macos", dir, "0.6.0", "owner/repo", "v0.6.0")
}
fn windows_feed() -> TempDir {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("BibCiTeX-x64.msix"), b"msix").unwrap();
    fs::write(dir.path().join("BibCiTeX-x64.appinstaller"), format!(r#"<AppInstaller xmlns="{INSTALLER}" Uri="https://github.com/owner/repo/releases/latest/download/BibCiTeX-x64.appinstaller"><MainPackage Version="0.6.0.0" Uri="https://github.com/owner/repo/releases/latest/download/BibCiTeX-x64.msix" /></AppInstaller>"#)).unwrap();
    dir
}
fn check_windows(dir: &Path) -> Result<()> {
    verify_release("windows", dir, "0.6.0.0", "owner/repo", "v0.6.0")
}
fn publish_assets() -> TempDir {
    let dir = tempdir().unwrap();
    for name in [
        "BibCiTeX-0.6.0-macos-arm64.zip",
        "BibCiTeX-0.6.0-macos-x86_64.zip",
        "appcast-arm64.xml",
        "appcast-x86_64.xml",
        "BibCiTeX-x64.appinstaller",
        "BibCiTeX-arm64.appinstaller",
        "BibCiTeX-x64.msix",
        "BibCiTeX-arm64.msix",
    ] {
        fs::write(dir.path().join(name), b"asset").unwrap();
    }
    dir
}
fn mock_publish(latest: &str, draft: bool, fail_upload: bool) -> (Result<()>, Vec<String>) {
    let dir = publish_assets();
    let mut calls = Vec::new();
    let result = publish_release("v0.6.0", dir.path(), "owner/repo", |args| {
        let verb = if args[0] == "api" { "latest" } else { &args[1] };
        calls.push(verb.to_owned());
        let output = match verb {
            "latest" => serde_json::json!({"tag_name": latest}).to_string(),
            "view" => serde_json::json!({"isDraft": draft, "isPrerelease": false}).to_string(),
            _ => String::new(),
        };
        Ok(CommandResult {
            success: !(verb == "upload" && fail_upload),
            output,
            error: "upload failed".into(),
        })
    });
    (result, calls)
}

#[test]
fn platform_versions_include_zero_major() {
    assert_eq!(versions("v1.2.34").unwrap().windows, "1.2.34.0");
    assert_eq!(versions("v0.6.0").unwrap().version, "0.6.0");
    assert_eq!(versions("v0.6.0").unwrap().windows, "0.6.0.0");
}
#[test]
fn prerelease_and_noncanonical_versions_are_rejected() {
    for tag in [
        "v1.0.0-beta.1",
        "v1.0.0+dev",
        "1.0.0",
        "v01.0.0",
        "v1.2.3\n",
        "../v1.2.3",
        "v1..3",
    ] {
        assert!(versions(tag).is_err(), "{tag}");
    }
}
#[test]
fn msix_version_bounds() {
    assert_eq!(
        versions("v65535.65535.65535").unwrap().windows,
        "65535.65535.65535.0"
    );
    for tag in [
        "v65536.0.0",
        "v1.65536.0",
        "v1.0.65536",
        "v999999999999999999.0.0",
    ] {
        assert!(versions(tag).is_err());
    }
}
#[test]
fn signed_archive_metadata() {
    check_mac(mac_feed("signed", 3, "0.6.0").path()).unwrap();
}
#[test]
fn unsigned_archive_rejected() {
    assert!(
        check_mac(mac_feed("", 3, "0.6.0").path())
            .unwrap_err()
            .to_string()
            .contains("Unsigned")
    );
}
#[test]
fn wrong_archive_size_rejected() {
    assert!(
        check_mac(mac_feed("signed", 99, "0.6.0").path())
            .unwrap_err()
            .to_string()
            .contains("size mismatch")
    );
}
#[test]
fn wrong_version_rejected() {
    assert!(
        check_mac(mac_feed("signed", 3, "0.5.0").path())
            .unwrap_err()
            .to_string()
            .contains("version mismatch")
    );
}
#[test]
fn app_installer_resolves_payload() {
    check_windows(windows_feed().path()).unwrap();
}
#[test]
fn missing_msix_rejected() {
    let dir = windows_feed();
    fs::remove_file(dir.path().join("BibCiTeX-x64.msix")).unwrap();
    assert!(
        check_windows(dir.path())
            .unwrap_err()
            .to_string()
            .contains("Missing release payload")
    );
}
#[test]
fn draft_publishes_only_after_upload() {
    let (result, calls) = mock_publish("v0.5.0", true, false);
    result.unwrap();
    assert_eq!(calls, ["latest", "view", "upload", "edit"]);
}
#[test]
fn upload_failure_preserves_stable_release() {
    let (result, calls) = mock_publish("v0.5.0", true, true);
    assert!(result.unwrap_err().to_string().contains("upload failed"));
    assert_eq!(calls, ["latest", "view", "upload"]);
}
#[test]
fn downgrade_creates_no_release() {
    let (result, calls) = mock_publish("v0.7.0", true, false);
    assert!(result.unwrap_err().to_string().contains("newer"));
    assert_eq!(calls, ["latest"]);
}
#[test]
fn published_release_cannot_be_overwritten() {
    let (result, calls) = mock_publish("v0.5.0", false, false);
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("already published")
    );
    assert_eq!(calls, ["latest", "view"]);
}
#[test]
fn xml_rejects_dtd_and_incomplete_documents() {
    for xml in [
        "<!DOCTYPE rss SYSTEM 'file:///etc/passwd'><rss/>",
        "<rss>",
        "<rss/><rss/>",
    ] {
        assert!(parse_xml(xml).is_err());
    }
}
#[test]
fn asset_url_cannot_escape_directory() {
    let dir = tempdir().unwrap();
    for name in [
        "../secret",
        "%2e%2e%2fsecret",
        "%2fsecret",
        "foo%5cbar",
        "%00",
        "%XX",
    ] {
        assert!(
            asset_from_url(
                dir.path(),
                &format!("https://example.com/{name}"),
                "https://example.com/"
            )
            .is_err()
        );
    }
}
#[test]
fn xml_resolves_namespace_instead_of_trusting_prefix() {
    let dir = mac_feed("signed", 3, "0.6.0");
    let file = dir.path().join("appcast-arm64.xml");
    let xml = fs::read_to_string(&file)
        .unwrap()
        .replace("sparkle:", "s:")
        .replace("xmlns:sparkle=", "xmlns:s=");
    fs::write(&file, &xml).unwrap();
    check_mac(dir.path()).unwrap();
    fs::write(file, xml.replace(SPARKLE, "https://wrong.example")).unwrap();
    assert!(check_mac(dir.path()).is_err());
}
#[test]
fn malformed_github_json_stops_before_mutation() {
    let dir = publish_assets();
    let mut calls = 0;
    assert!(
        publish_release("v0.6.0", dir.path(), "owner/repo", |_| {
            calls += 1;
            Ok(CommandResult {
                success: true,
                output: "{}".into(),
                error: String::new(),
            })
        })
        .is_err()
    );
    assert_eq!(calls, 1);
}
