//! Linux-only signed update verification and atomic portable executable replacement.
use base64::Engine as _;
use ed25519_dalek::{Signature, VerifyingKey};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

const REPOSITORY: &str = "tangxiangong/bibcitex";
const COS_ROOT: &str = "https://app-release-1302963684.cos.ap-guangzhou.myqcloud.com/bibcitex";
const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: String,
    pub arch: String,
    pub file: String,
    pub size: u64,
    pub sha256: String,
    pub package_file: String,
    pub package_size: u64,
    pub package_sha256: String,
}
#[derive(Clone, Debug)]
pub struct Release {
    pub manifest: Manifest,
    pub url: String,
    pub fallback_base: Option<String>,
}

pub fn architecture() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    }
}
pub fn public_key() -> &'static str {
    option_env!("BIBCITEX_LINUX_UPDATE_PUBLIC_KEY").unwrap_or("")
}
pub fn eligible(current: &str, candidate: &str, channel: &str) -> bool {
    let (Ok(current), Ok(candidate)) = (Version::parse(current), Version::parse(candidate)) else {
        return false;
    };
    if candidate <= current || !candidate.build.is_empty() {
        return false;
    }
    let pre = candidate.pre.as_str();
    pre.is_empty()
        || (channel == "beta" && valid_preview(pre, "beta"))
        || (channel == "alpha" && (valid_preview(pre, "alpha") || valid_preview(pre, "beta")))
}
fn valid_preview(pre: &str, prefix: &str) -> bool {
    pre.strip_prefix(&format!("{prefix}.")).is_some_and(|n| {
        !n.is_empty()
            && n.bytes().all(|b| b.is_ascii_digit())
            && (n.len() == 1 || !n.starts_with('0'))
    })
}

fn catalog_version(bytes: &[u8], channel: &str) -> Result<String, String> {
    let catalog: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let version = catalog["versions"][channel]
        .as_str()
        .ok_or("Missing update channel")?;
    let parsed = Version::parse(version).map_err(|e| e.to_string())?;
    let pre = parsed.pre.as_str();
    if !parsed.build.is_empty()
        || !(pre.is_empty()
            || (channel == "beta" && valid_preview(pre, "beta"))
            || (channel == "alpha" && (valid_preview(pre, "alpha") || valid_preview(pre, "beta"))))
    {
        return Err("Invalid version in update channel".into());
    }
    Ok(version.to_owned())
}
pub fn verify_manifest(
    bytes: &[u8],
    signature: &str,
    key: &str,
    version: &str,
    arch: &str,
) -> Result<Manifest, String> {
    let key: [u8; 32] = base64::engine::general_purpose::STANDARD
        .decode(key.trim())
        .map_err(|e| e.to_string())?
        .try_into()
        .map_err(|_| "Invalid update public key")?;
    let signature = base64::engine::general_purpose::STANDARD
        .decode(signature.trim())
        .map_err(|e| e.to_string())?;
    VerifyingKey::from_bytes(&key)
        .map_err(|e| e.to_string())?
        .verify_strict(
            bytes,
            &Signature::from_slice(&signature).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if manifest.version != version
        || manifest.arch != arch
        || manifest.file != format!("BibCiTeX-{version}-linux-{arch}.tar.gz")
        || manifest.package_file != format!("BibCiTeX-{version}-linux-{arch}.deb")
        || manifest.package_size == 0
        || manifest.package_size > MAX_ARCHIVE
        || manifest.package_sha256.len() != 64
        || !manifest
            .package_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || manifest.size == 0
        || manifest.size > MAX_ARCHIVE
        || manifest.sha256.len() != 64
        || !manifest.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("Linux update identity or size is invalid".into());
    }
    Ok(manifest)
}
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("BibCiTeX-Linux")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
        .https_only(true)
        .build()
        .map_err(|e| e.to_string())
}
fn download(client: &reqwest::blocking::Client, url: &str, limit: u64) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Update response exceeds its size limit".into());
    }
    Ok(bytes)
}
pub fn check(current: &str, channel: &str) -> Result<Option<Release>, String> {
    let key = public_key();
    if key.is_empty() {
        return Err("此构建未配置更新验证公钥".into());
    }
    if !matches!(channel, "stable" | "beta" | "alpha") {
        return Err("Invalid update channel".into());
    }
    let client = client()?;
    if let Ok(release) = check_cos(&client, current, channel, key) {
        return Ok(release);
    }
    let mut candidates = Vec::new();
    for page in 1..=10 {
        let bytes = download(
            &client,
            &format!("https://api.github.com/repos/{REPOSITORY}/releases?per_page=100&page={page}"),
            4 * 1024 * 1024,
        )?;
        let releases: Vec<serde_json::Value> =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let count = releases.len();
        for release in releases {
            if release["draft"].as_bool() != Some(false) {
                continue;
            }
            if let Some(version) = release["tag_name"]
                .as_str()
                .and_then(|s| s.strip_prefix('v'))
                && eligible(current, version, channel)
                && let Ok(parsed) = Version::parse(version)
            {
                let filename = format!("linux-{}.json", architecture());
                if release["assets"]
                    .as_array()
                    .is_some_and(|assets| assets.iter().any(|a| a["name"] == filename))
                {
                    candidates.push(parsed);
                }
            }
        }
        if count < 100 {
            break;
        }
    }
    let Some(version) = candidates.into_iter().max().map(|v| v.to_string()) else {
        return Ok(None);
    };
    let base = format!("https://github.com/{REPOSITORY}/releases/download/v{version}");
    let metadata = download(
        &client,
        &format!("{base}/linux-{}.json", architecture()),
        16 * 1024,
    )?;
    let signature = download(
        &client,
        &format!("{base}/linux-{}.json.sig", architecture()),
        1024,
    )?;
    let signature = std::str::from_utf8(&signature).map_err(|e| e.to_string())?;
    let manifest = verify_manifest(&metadata, signature, key, &version, architecture())?;
    let url = format!("{base}/{}", manifest.file);
    Ok(Some(Release {
        manifest,
        url,
        fallback_base: None,
    }))
}
fn check_cos(
    client: &reqwest::blocking::Client,
    current: &str,
    channel: &str,
    key: &str,
) -> Result<Option<Release>, String> {
    let bytes = download(
        client,
        &format!("{COS_ROOT}/update-feed/channels.json"),
        16 * 1024,
    )?;
    // A malformed pointer is a feed failure, not evidence that we are up to date.
    // Returning an error lets check() try the GitHub fallback.
    let version = catalog_version(&bytes, channel)?;
    if !eligible(current, &version, channel) {
        return Ok(None);
    }
    let base = format!("{COS_ROOT}/releases/v{version}");
    let bytes = download(
        client,
        &format!("{base}/linux-{}.json", architecture()),
        16 * 1024,
    )?;
    let signature = download(
        client,
        &format!("{base}/linux-{}.json.sig", architecture()),
        1024,
    )?;
    let manifest = verify_manifest(
        &bytes,
        std::str::from_utf8(&signature).map_err(|e| e.to_string())?,
        key,
        &version,
        architecture(),
    )?;
    let url = format!("{base}/{}", manifest.file);
    Ok(Some(Release {
        manifest,
        url,
        fallback_base: Some(format!(
            "https://github.com/{REPOSITORY}/releases/download/v{version}"
        )),
    }))
}

fn payload(release: &Release, file: &str, size: u64, checksum: &str) -> Result<Vec<u8>, String> {
    let client = client()?;
    let base = release.url.rsplit_once('/').ok_or("Invalid update URL")?.0;
    let verified = |base: &str| -> Result<Vec<u8>, String> {
        let bytes = download(&client, &format!("{base}/{file}"), size)?;
        let hash: String = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if bytes.len() as u64 != size || !hash.eq_ignore_ascii_case(checksum) {
            return Err("Linux update checksum mismatch".into());
        }
        Ok(bytes)
    };
    match verified(base) {
        Ok(bytes) => Ok(bytes),
        Err(error) => match &release.fallback_base {
            Some(fallback) => verified(fallback),
            None => Err(error),
        },
    }
}

pub fn managed_install(executable: &Path) -> bool {
    executable.starts_with("/usr")
        || executable.starts_with("/opt")
        || executable.starts_with("/app")
}
pub enum InstallOutcome {
    Replaced(std::path::PathBuf),
    Package(std::path::PathBuf),
}
pub fn install(release: &Release) -> Result<InstallOutcome, String> {
    let executable = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if managed_install(&executable) {
        let bytes = payload(
            release,
            &release.manifest.package_file,
            release.manifest.package_size,
            &release.manifest.package_sha256,
        )?;
        let directory = dirs::cache_dir()
            .ok_or("No user cache directory")?
            .join("bibcitex/updates");
        fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let path = directory.join(&release.manifest.package_file);
        let mut file = tempfile::NamedTempFile::new_in(&directory).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(&path).map_err(|e| e.to_string())?;
        return Ok(InstallOutcome::Package(path));
    }
    let bytes = payload(
        release,
        &release.manifest.file,
        release.manifest.size,
        &release.manifest.sha256,
    )?;
    replace_executable(&executable, &bytes).map(InstallOutcome::Replaced)
}
pub fn replace_executable(executable: &Path, archive: &[u8]) -> Result<std::path::PathBuf, String> {
    let parent = executable.parent().ok_or("Invalid executable path")?;
    let mut replacement = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    let mut archive =
        tar::Archive::new(flate2::read::GzDecoder::new(archive).take(MAX_ARCHIVE + 1));
    let mut found = false;
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        if entry.path().map_err(|e| e.to_string())?.as_ref() != Path::new("bin/bibcitex") {
            continue;
        }
        if found || !entry.header().entry_type().is_file() || entry.size() > MAX_ARCHIVE {
            return Err("Invalid executable in update archive".into());
        }
        let mut magic = [0; 4];
        entry.read_exact(&mut magic).map_err(|e| e.to_string())?;
        if magic != *b"\x7fELF" {
            return Err("Update is not a Linux executable".into());
        }
        replacement.write_all(&magic).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut replacement).map_err(|e| e.to_string())?;
        found = true;
    }
    if !found {
        return Err("Update archive is missing the executable".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        replacement
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }
    replacement
        .as_file()
        .sync_all()
        .map_err(|e| e.to_string())?;
    replacement.persist(executable).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|e| e.to_string())?;
    // On Linux current_exe() refers to the unlinked old inode after replacement.
    // Preserve the installed path so the caller can restart the new executable.
    Ok(executable.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    #[test]
    fn malformed_catalog_versions_are_errors_instead_of_no_update() {
        for version in ["invalid", "v1.2.3", "1.2.3+build", "1.2.3-alpha.1"] {
            let bytes = serde_json::to_vec(&serde_json::json!({
                "versions": {"stable": version}
            }))
            .unwrap();
            assert!(catalog_version(&bytes, "stable").is_err(), "{version}");
        }
        assert_eq!(
            catalog_version(br#"{"versions":{"stable":"1.2.3"}}"#, "stable").unwrap(),
            "1.2.3"
        );
        assert_eq!(
            catalog_version(br#"{"versions":{"beta":"1.2.3-beta.2"}}"#, "beta").unwrap(),
            "1.2.3-beta.2"
        );
    }
    #[test]
    fn channels_never_downgrade_and_only_accept_selected_preview_levels() {
        assert!(eligible("1.0.0", "1.1.0-beta.2", "beta"));
        assert!(!eligible("1.0.0", "1.1.0-beta.2", "stable"));
        assert!(!eligible("1.0.0", "1.1.0-alpha.2", "beta"));
        assert!(eligible("1.1.0-beta.2", "1.1.0", "beta"));
        assert!(!eligible("1.2.0", "1.1.1", "alpha"));
        assert!(!eligible("1.0.0", "1.1.0-rc.1", "alpha"));
    }
    #[test]
    fn signature_binds_metadata_identity_and_payload_hash() {
        let key = SigningKey::from_bytes(&[43; 32]);
        let public =
            base64::engine::general_purpose::STANDARD.encode(key.verifying_key().as_bytes());
        let manifest = Manifest {
            version: "1.0.0".into(),
            arch: "arm64".into(),
            file: "BibCiTeX-1.0.0-linux-arm64.tar.gz".into(),
            size: 128,
            sha256: "a".repeat(64),
            package_file: "BibCiTeX-1.0.0-linux-arm64.deb".into(),
            package_size: 3,
            package_sha256: "b".repeat(64),
        };
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let signature =
            base64::engine::general_purpose::STANDARD.encode(key.sign(&bytes).to_bytes());
        assert!(verify_manifest(&bytes, &signature, &public, "1.0.0", "arm64").is_ok());
        assert!(verify_manifest(&bytes, &signature, &public, "1.0.1", "arm64").is_err());
        assert!(verify_manifest(&bytes, &signature, &public, "1.0.0", "x86_64").is_err());
        let mut altered = bytes.clone();
        altered[10] ^= 1;
        assert!(verify_manifest(&altered, &signature, &public, "1.0.0", "arm64").is_err());
    }
    #[test]
    fn invalid_archive_preserves_installed_executable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bibcitex");
        fs::write(&path, "original").unwrap();
        assert!(replace_executable(&path, b"invalid").is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
    }
}

#[cfg(test)]
mod archive_tests {
    use super::*;
    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        let mut builder = tar::Builder::new(gzip);
        for (name, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, name, *bytes).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn running_executable_replacement_returns_a_launchable_restart_path() {
        use std::process::{Command, Stdio};

        let directory = tempfile::tempdir().unwrap();
        let exe = directory.path().join("bibcitex");
        fs::copy("/bin/sh", &exe).unwrap();
        let mut running = Command::new(&exe)
            .args(["-c", "printf ready; read line"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut ready = [0; 5];
        running
            .stdout
            .take()
            .unwrap()
            .read_exact(&mut ready)
            .unwrap();
        let bytes = fs::read("/bin/true").unwrap();
        let restart_path = replace_executable(&exe, &archive(&[("bin/bibcitex", &bytes)])).unwrap();
        let old_image = fs::read_link(format!("/proc/{}/exe", running.id())).unwrap();
        let restarted = Command::new(&restart_path).status().unwrap();
        drop(running.stdin.take());
        running.wait().unwrap();

        assert_eq!(&ready, b"ready");
        assert!(
            !old_image.exists(),
            "The running image should have been unlinked"
        );
        assert_eq!(restart_path, exe);
        assert!(restarted.success());
    }
    #[test]
    fn portable_update_replaces_only_the_binary_and_rejects_duplicate_entries() {
        let directory = tempfile::tempdir().unwrap();
        let exe = directory.path().join("bibcitex");
        fs::write(&exe, b"old").unwrap();
        let payload = b"\x7fELFnew";
        let valid = archive(&[("bin/bibcitex", payload), ("share/unused", b"data")]);
        let restart_path = replace_executable(&exe, &valid).unwrap();
        assert_eq!(restart_path, exe);
        assert_eq!(fs::read(&restart_path).unwrap(), payload);
        let duplicate = archive(&[("bin/bibcitex", payload), ("bin/bibcitex", b"\x7fELFother")]);
        assert!(replace_executable(&exe, &duplicate).is_err());
        assert_eq!(fs::read(&exe).unwrap(), payload);
        assert!(!directory.path().join("share").exists());
    }
}
