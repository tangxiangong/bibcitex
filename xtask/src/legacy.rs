use crate::{Result, asset_from_url, ensure, versions};
use base64::{Engine, engine::general_purpose::STANDARD};
use minisign::{PublicKey, PublicKeyBox, SecretKeyBox, SignatureBox};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{BufReader, Read},
    path::{Component, Path},
};

// Public key embedded in the released Tauri v0.6.0 app. Never rotate this key for migration.
const PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDkzQzBFQjBBQjUwMDRCRgpSV1MvQkZDcnNBNDhDUzlvOFNrL1hhVWFxVU5JQnpOR0xkaGVxMnJsQWZESFpCRmJvSVExbFR3WAo=";
pub(crate) const PAYLOADS: [(&str, &str); 4] = [
    ("darwin-aarch64", "BibCiTeX_aarch64.app.tar.gz"),
    ("darwin-x86_64", "BibCiTeX_x64.app.tar.gz"),
    ("windows-aarch64", "BibCiTeX_arm64-setup.exe"),
    ("windows-x86_64", "BibCiTeX_x64-setup.exe"),
];

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: String,
    platforms: BTreeMap<String, Platform>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Platform {
    url: String,
    signature: String,
}
fn decode(value: &str) -> Result<String> {
    Ok(String::from_utf8(STANDARD.decode(value.trim())?)?)
}
pub(crate) fn public_key() -> Result<PublicKey> {
    Ok(PublicKeyBox::from_string(&decode(PUBLIC_KEY)?)?.into_public_key()?)
}
fn release_version(tag: &str) -> Result<crate::Versions> {
    let version = versions(tag)?;
    ensure(
        version.parts > [0, 6, 0],
        "Migration release must be newer than v0.6.0",
    )?;
    Ok(version)
}
fn comment(path: &Path, version: &str) -> Result<String> {
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("Invalid payload filename")?;
    ensure(
        !name.contains(['\t', '\n', '\r']),
        "Invalid payload filename",
    )?;
    Ok(format!("file:{name}\tversion:{version}"))
}
fn verify(path: &Path, signature: &str, version: &str, key: &PublicKey) -> Result<()> {
    let signature = SignatureBox::from_string(&decode(signature)?)?;
    minisign::verify(
        key,
        &signature,
        BufReader::new(fs::File::open(path)?),
        true,
        false,
        false,
    )?;
    ensure(
        signature.trusted_comment()? == comment(path, version)?,
        "Signed payload filename/version mismatch",
    )
}

fn validate_payload(path: &Path) -> Result<()> {
    if path.extension().is_some_and(|ext| ext == "exe") {
        let mut magic = [0; 2];
        fs::File::open(path)?.read_exact(&mut magic)?;
        return ensure(
            magic == *b"MZ",
            "Migration installer is not a Windows executable",
        );
    }
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(fs::File::open(path)?));
    let mut executable = false;
    let mut plist = false;
    let mut paths = BTreeSet::new();
    let mut symlinks = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let name = entry.path()?;
        let components = name.components().collect::<Vec<_>>();
        ensure(
            components.first() == Some(&Component::Normal("BibCiTeX.app".as_ref()))
                && components.iter().all(|c| matches!(c, Component::Normal(_))),
            "Archive must contain only paths rooted at BibCiTeX.app",
        )?;
        let kind = entry.header().entry_type();
        ensure(paths.insert(name.to_path_buf()), "Duplicate archive entry")?;
        ensure(
            kind.is_file() || kind.is_dir() || kind.is_symlink(),
            "Unsupported archive entry",
        )?;
        if let Some(target) = entry.link_name()? {
            symlinks.push(name.to_path_buf());
            let mut depth = components.len() - 1;
            let mut descended = false;
            for component in target.components() {
                match component {
                    Component::Normal(_) => {
                        depth += 1;
                        descended = true;
                    }
                    Component::CurDir => {}
                    // A parent traversal after a symlink component can escape despite
                    // appearing lexically inside the bundle. Framework aliases use
                    // normal paths or leading parent components, never this form.
                    Component::ParentDir if depth > 1 && !descended => depth -= 1,
                    _ => return Err("Archive symlink escapes application bundle".into()),
                }
            }
        }
        executable |= name == Path::new("BibCiTeX.app/Contents/MacOS/BibCiTeX")
            && kind.is_file()
            && entry.header().mode()? & 0o111 != 0;
        plist |= name == Path::new("BibCiTeX.app/Contents/Info.plist") && kind.is_file();
    }
    ensure(
        !paths.iter().any(|path| {
            symlinks
                .iter()
                .any(|link| path != link && path.starts_with(link))
        }),
        "Archive entry descends through a symlink",
    )?;
    ensure(
        executable && plist,
        "Archive is missing executable or Info.plist",
    )
}

pub fn sign(path: &Path) -> Result<()> {
    let version = release_version(&env::var("RELEASE_TAG")?)?;
    validate_payload(path)?;
    let secret = env::var("TAURI_SIGNING_PRIVATE_KEY")
        .map_err(|_| "Missing TAURI_SIGNING_PRIVATE_KEY; use the existing Tauri signing key")?;
    let secret = SecretKeyBox::from_string(&decode(&secret)?)?.into_secret_key(Some(
        env::var("TAURI_SIGNING_PRIVATE_KEY_PASSWORD").unwrap_or_default(),
    ))?;
    let key = public_key()?;
    let signature = minisign::sign(
        Some(&key),
        &secret,
        BufReader::new(fs::File::open(path)?),
        Some(&comment(path, &version.version)?),
        None,
    )?;
    let encoded = STANDARD.encode(signature.to_string());
    verify(path, &encoded, &version.version, &key)?;
    fs::write(path.with_added_extension("sig"), encoded)?;
    Ok(())
}

fn manifest(directory: &Path, repository: &str, tag: &str, key: &PublicKey) -> Result<Manifest> {
    let version = release_version(tag)?;
    let mut platforms = BTreeMap::new();
    for (platform, name) in PAYLOADS {
        let path = directory.join(name);
        let signature = fs::read_to_string(path.with_added_extension("sig"))?;
        verify(&path, &signature, &version.version, key)?;
        validate_payload(&path)?;
        platforms.insert(
            platform.to_owned(),
            Platform {
                url: format!("https://github.com/{repository}/releases/download/{tag}/{name}"),
                signature: signature.trim().to_owned(),
            },
        );
    }
    Ok(Manifest {
        version: version.version,
        platforms,
    })
}
pub fn write_manifest(directory: &Path, repository: &str, tag: &str) -> Result<()> {
    let manifest = manifest(directory, repository, tag, &public_key()?)?;
    fs::write(
        directory.join("latest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}
pub(crate) fn validate(
    directory: &Path,
    repository: &str,
    tag: &str,
    key: &PublicKey,
) -> Result<()> {
    let expected = manifest(directory, repository, tag, key)?;
    let actual: Manifest = serde_json::from_slice(&fs::read(directory.join("latest.json"))?)?;
    ensure(
        actual.version == expected.version,
        "Legacy manifest version mismatch",
    )?;
    ensure(
        actual.platforms.len() == PAYLOADS.len(),
        "Legacy manifest platform mismatch",
    )?;
    let base = format!("https://github.com/{repository}/releases/download/{tag}/");
    for (platform, expected) in expected.platforms {
        let actual = actual
            .platforms
            .get(&platform)
            .ok_or("Missing legacy platform")?;
        ensure(
            actual.url == expected.url && actual.signature == expected.signature,
            "Legacy manifest payload/signature mismatch",
        )?;
        asset_from_url(directory, &actual.url, &base)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use minisign::KeyPair;
    pub(crate) fn fixtures(directory: &Path, tag: &str) -> PublicKey {
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        let version = versions(tag).unwrap();
        for (_, name) in PAYLOADS {
            let path = directory.join(name);
            if name.ends_with(".exe") {
                fs::write(&path, b"MZtest-installer").unwrap();
            } else {
                let gzip = flate2::write::GzEncoder::new(
                    fs::File::create(&path).unwrap(),
                    flate2::Compression::default(),
                );
                let mut archive = tar::Builder::new(gzip);
                for name in [
                    "BibCiTeX.app/Contents/MacOS/BibCiTeX",
                    "BibCiTeX.app/Contents/Info.plist",
                ] {
                    let mut header = tar::Header::new_gnu();
                    header.set_size(4);
                    header.set_mode(0o755);
                    header.set_cksum();
                    archive
                        .append_data(&mut header, name, &b"test"[..])
                        .unwrap();
                }
                archive.into_inner().unwrap().finish().unwrap();
            }
            let sig = minisign::sign(
                Some(&pair.pk),
                &pair.sk,
                fs::File::open(&path).unwrap(),
                Some(&comment(&path, &version.version).unwrap()),
                None,
            )
            .unwrap();
            fs::write(
                path.with_added_extension("sig"),
                STANDARD.encode(sig.to_string()),
            )
            .unwrap();
        }
        let value = manifest(directory, "owner/repo", tag, &pair.pk).unwrap();
        fs::write(
            directory.join("latest.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        pair.pk
    }
    #[test]
    fn signed_migration_validates_all_platforms() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        validate(dir.path(), "owner/repo", "v0.6.1", &key).unwrap();
    }
    #[test]
    fn tauri_2_10_1_verifier_accepts_all_migration_signatures() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        // Match the released updater's base64 envelope and verifier, including
        // its allow_legacy flag. This is deliberately a separate implementation.
        let public_envelope = STANDARD.encode(key.to_box().unwrap().to_string());
        let legacy_key =
            minisign_verify::PublicKey::decode(&decode(&public_envelope).unwrap()).unwrap();
        for (_, name) in PAYLOADS {
            let path = dir.path().join(name);
            let envelope = fs::read_to_string(path.with_added_extension("sig")).unwrap();
            let signature =
                minisign_verify::Signature::decode(&decode(&envelope).unwrap()).unwrap();
            legacy_key
                .verify(&fs::read(path).unwrap(), &signature, true)
                .unwrap();
        }
    }
    #[test]
    fn production_key_rejects_another_signer() {
        let dir = tempfile::tempdir().unwrap();
        fixtures(dir.path(), "v0.6.1");
        assert!(validate(dir.path(), "owner/repo", "v0.6.1", &public_key().unwrap()).is_err());
    }
    #[test]
    fn corrupted_payload_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        fs::write(dir.path().join(PAYLOADS[0].1), "corruption").unwrap();
        assert!(validate(dir.path(), "owner/repo", "v0.6.1", &key).is_err());
    }
    #[test]
    fn missing_payload_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        fs::remove_file(dir.path().join(PAYLOADS[1].1)).unwrap();
        assert!(validate(dir.path(), "owner/repo", "v0.6.1", &key).is_err());
    }
    #[test]
    fn signed_version_cannot_be_relabelled() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        assert!(validate(dir.path(), "owner/repo", "v0.6.2", &key).is_err());
    }
    #[test]
    fn old_or_equal_version_is_rejected() {
        assert!(release_version("v0.6.0").is_err());
        assert!(release_version("v0.5.9").is_err());
    }

    #[test]
    fn missing_signature_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        fs::remove_file(dir.path().join(PAYLOADS[0].1).with_added_extension("sig")).unwrap();
        assert!(validate(dir.path(), "owner/repo", "v0.6.1", &key).is_err());
    }
    #[test]
    fn mismatched_manifest_signature_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let key = fixtures(dir.path(), "v0.6.1");
        let path = dir.path().join("latest.json");
        let mut value: Manifest = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value.platforms.get_mut("darwin-aarch64").unwrap().signature =
            value.platforms["darwin-x86_64"].signature.clone();
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(validate(dir.path(), "owner/repo", "v0.6.1", &key).is_err());
    }
    #[test]
    fn archive_with_wrong_root_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.tar.gz");
        let gzip = flate2::write::GzEncoder::new(
            fs::File::create(&path).unwrap(),
            flate2::Compression::default(),
        );
        let mut archive = tar::Builder::new(gzip);
        let mut header = tar::Header::new_gnu();
        header.set_size(4);
        header.set_mode(0o755);
        header.set_cksum();
        archive
            .append_data(&mut header, "Contents/MacOS/BibCiTeX", &b"test"[..])
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap();
        assert!(validate_payload(&path).is_err());
    }
    #[test]
    fn archive_symlink_outside_bundle_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.tar.gz");
        let gzip = flate2::write::GzEncoder::new(
            fs::File::create(&path).unwrap(),
            flate2::Compression::default(),
        );
        let mut archive = tar::Builder::new(gzip);
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o777);
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_cksum();
        archive
            .append_link(&mut header, "BibCiTeX.app/Contents/link", "../../outside")
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap();
        assert!(validate_payload(&path).is_err());
    }
}
