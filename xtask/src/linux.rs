//! Linux packages and signed metadata. This module never uploads or launches an app.
use crate::{Result, ensure, hash};
use base64::Engine as _;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::process::Command;
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: String,
    arch: String,
    file: String,
    size: u64,
    sha256: String,
    package_file: String,
    package_size: u64,
    package_sha256: String,
}
#[cfg(unix)]
fn copy(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target.parent().ok_or("Invalid package path")?)?;
    fs::copy(source, target)?;
    Ok(())
}
#[cfg(unix)]
pub fn package(binary: &Path, directory: &Path, arch: &str) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let deb_arch = match arch {
        "arm64" => "arm64",
        "x86_64" => "amd64",
        _ => return Err("Unsupported Linux architecture".into()),
    };
    let bytes = fs::read(binary)?;
    ensure(
        bytes.len() >= 20 && &bytes[..4] == b"\x7fELF" && bytes[4] == 2 && bytes[5] == 1,
        "Expected a little-endian ELF64 executable",
    )?;
    let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    ensure(
        machine == if arch == "arm64" { 183 } else { 62 },
        "Linux package architecture mismatch",
    )?;
    fs::create_dir_all(directory)?;
    let stage = tempfile::tempdir()?;
    copy(binary, &stage.path().join("bin/bibcitex"))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("Missing workspace")?;
    for (source, destination) in [
        (
            "linux/resources/io.github.tangxiangong.bibcitex.desktop",
            "share/applications/io.github.tangxiangong.bibcitex.desktop",
        ),
        (
            "linux/resources/io.github.tangxiangong.bibcitex.metainfo.xml",
            "share/metainfo/io.github.tangxiangong.bibcitex.metainfo.xml",
        ),
        (
            "assets/app-icons/128x128.png",
            "share/icons/hicolor/128x128/apps/io.github.tangxiangong.bibcitex.png",
        ),
        (
            "linux/gnome-extension/extension.js",
            "share/gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io/extension.js",
        ),
        (
            "linux/gnome-extension/metadata.json",
            "share/gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io/metadata.json",
        ),
        ("linux/resources/install.sh", "install.sh"),
        (
            "linux/resources/STIX-OFL.txt",
            "share/licenses/bibcitex/STIX-OFL.txt",
        ),
    ] {
        copy(&root.join(source), &stage.path().join(destination))?;
    }
    let filename = format!("BibCiTeX-{version}-linux-{arch}.tar.gz");
    let output = fs::File::create(directory.join(&filename))?;
    let gzip = flate2::write::GzEncoder::new(output, flate2::Compression::default());
    let mut archive = tar::Builder::new(gzip);
    archive.append_path_with_name(stage.path().join("bin/bibcitex"), "bin/bibcitex")?;
    archive.append_path_with_name(stage.path().join("install.sh"), "install.sh")?;
    archive.append_dir_all("share", stage.path().join("share"))?;
    archive.into_inner()?.finish()?.sync_all()?;
    let deb = tempfile::tempdir()?;
    copy(binary, &deb.path().join("usr/lib/bibcitex/bibcitex"))?;
    for destination in [
        "gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io/extension.js",
        "gnome-shell/extensions/bibcitex-tray@tangxiangong.github.io/metadata.json",
        "licenses/bibcitex/STIX-OFL.txt",
        "applications/io.github.tangxiangong.bibcitex.desktop",
        "metainfo/io.github.tangxiangong.bibcitex.metainfo.xml",
        "icons/hicolor/128x128/apps/io.github.tangxiangong.bibcitex.png",
    ] {
        copy(
            &stage.path().join("share").join(destination),
            &deb.path().join("usr/share").join(destination),
        )?;
    }
    fs::create_dir_all(deb.path().join("usr/bin"))?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        "../lib/bibcitex/bibcitex",
        deb.path().join("usr/bin/bibcitex"),
    )?;
    fs::create_dir_all(deb.path().join("DEBIAN"))?;
    fs::write(
        deb.path().join("DEBIAN/control"),
        format!(
            "Package: bibcitex\nVersion: {}\nArchitecture: {deb_arch}\nMaintainer: BibCiTeX <tangxiangong@gmail.com>\nSection: utils\nPriority: optional\nDepends: libc6 (>= 2.39), libgcc-s1, libstdc++6, libfontconfig1, libfreetype6, libxkbcommon0, libxkbcommon-x11-0, libwayland-client0, libxcb1, libx11-xcb1, libvulkan1, libssl3t64 | libssl3\nDescription: BibTeX reference search and citation helper\n",
            version.replace('-', "~")
        ),
    )?;
    let status = Command::new("dpkg-deb")
        .args(["--build", "--root-owner-group"])
        .arg(deb.path())
        .arg(directory.join(format!("BibCiTeX-{version}-linux-{arch}.deb")))
        .status()?;
    ensure(status.success(), "dpkg-deb failed")?;
    let manifest = Manifest {
        version: version.into(),
        arch: arch.into(),
        file: filename.clone(),
        size: fs::metadata(directory.join(&filename))?.len(),
        sha256: hash(&directory.join(filename))?,
        package_file: format!("BibCiTeX-{version}-linux-{arch}.deb"),
        package_size: fs::metadata(directory.join(format!("BibCiTeX-{version}-linux-{arch}.deb")))?
            .len(),
        package_sha256: hash(&directory.join(format!("BibCiTeX-{version}-linux-{arch}.deb")))?,
    };
    fs::write(
        directory.join(format!("linux-{arch}.json")),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}
#[cfg(not(unix))]
pub fn package(_: &Path, _: &Path, _: &str) -> Result<()> {
    Err("Linux packaging requires a Unix build host".into())
}
pub fn sign(directory: &Path, arch: &str) -> Result<()> {
    ensure(
        matches!(arch, "arm64" | "x86_64"),
        "Unsupported Linux architecture",
    )?;
    let secret = std::env::var("LINUX_UPDATE_PRIVATE_KEY")?;
    let bytes: [u8; 32] = base64::engine::general_purpose::STANDARD
        .decode(secret.trim())?
        .try_into()
        .map_err(|_| "Linux signing key must be a base64 Ed25519 seed")?;
    let key = SigningKey::from_bytes(&bytes);
    let public = std::env::var("BIBCITEX_LINUX_UPDATE_PUBLIC_KEY")?;
    ensure(
        base64::engine::general_purpose::STANDARD.decode(public.trim())?
            == key.verifying_key().as_bytes(),
        "Linux signing public key mismatch",
    )?;
    let file = directory.join(format!("linux-{arch}.json"));
    let signature = key.sign(&fs::read(file)?);
    fs::write(
        directory.join(format!("linux-{arch}.json.sig")),
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
    )?;
    Ok(())
}
pub fn verify(directory: &Path, version: &str, public: &str) -> Result<()> {
    let key: [u8; 32] = base64::engine::general_purpose::STANDARD
        .decode(public.trim())?
        .try_into()
        .map_err(|_| "Invalid Linux public key")?;
    let key = VerifyingKey::from_bytes(&key)?;
    for arch in ["arm64", "x86_64"] {
        let bytes = fs::read(directory.join(format!("linux-{arch}.json")))?;
        let signature = base64::engine::general_purpose::STANDARD
            .decode(fs::read_to_string(directory.join(format!("linux-{arch}.json.sig")))?.trim())?;
        key.verify_strict(&bytes, &Signature::from_slice(&signature)?)?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        ensure(
            manifest.version == version
                && manifest.arch == arch
                && manifest.file == format!("BibCiTeX-{version}-linux-{arch}.tar.gz"),
            "Linux manifest identity mismatch",
        )?;
        let file = crate::safe_file(directory, &manifest.file)?;
        ensure(
            manifest.size > 0
                && fs::metadata(&file)?.len() == manifest.size
                && hash(&file)? == manifest.sha256,
            "Linux payload hash or size mismatch",
        )?;
        ensure(
            manifest.package_file == format!("BibCiTeX-{version}-linux-{arch}.deb"),
            "Linux package identity mismatch",
        )?;
        let package = crate::safe_file(directory, &manifest.package_file)?;
        ensure(
            manifest.package_size > 0
                && fs::metadata(&package)?.len() == manifest.package_size
                && hash(&package)? == manifest.package_sha256,
            "Linux package checksum mismatch",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_linux_release_requires_both_architectures_and_exact_payloads() {
        let directory = tempfile::tempdir().unwrap();
        let key = SigningKey::from_bytes(&[21; 32]);
        let public =
            base64::engine::general_purpose::STANDARD.encode(key.verifying_key().as_bytes());
        for arch in ["arm64", "x86_64"] {
            let filename = format!("BibCiTeX-1.2.3-linux-{arch}.tar.gz");
            fs::write(directory.path().join(&filename), b"signed-payload").unwrap();
            fs::write(
                directory
                    .path()
                    .join(format!("BibCiTeX-1.2.3-linux-{arch}.deb")),
                b"deb",
            )
            .unwrap();
            let manifest = Manifest {
                version: "1.2.3".into(),
                arch: arch.into(),
                file: filename.clone(),
                size: 14,
                sha256: hash(&directory.path().join(&filename)).unwrap(),
                package_file: format!("BibCiTeX-1.2.3-linux-{arch}.deb"),
                package_size: 3,
                package_sha256: hash(
                    &directory
                        .path()
                        .join(format!("BibCiTeX-1.2.3-linux-{arch}.deb")),
                )
                .unwrap(),
            };
            let bytes = serde_json::to_vec(&manifest).unwrap();
            let signature =
                base64::engine::general_purpose::STANDARD.encode(key.sign(&bytes).to_bytes());
            fs::write(directory.path().join(format!("linux-{arch}.json")), &bytes).unwrap();
            fs::write(
                directory.path().join(format!("linux-{arch}.json.sig")),
                signature,
            )
            .unwrap();
        }
        assert!(verify(directory.path(), "1.2.3", &public).is_ok());
        fs::write(
            directory.path().join("BibCiTeX-1.2.3-linux-arm64.tar.gz"),
            b"altered",
        )
        .unwrap();
        assert!(verify(directory.path(), "1.2.3", &public).is_err());
    }
}
