use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey};
use quick_xml::{XmlVersion, events::Event, name::ResolveResult, reader::NsReader};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error, fs, path::Path, process::Command};
pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
const SPARKLE: &str = "http://www.andymatuschak.org/xml-namespaces/sparkle";
const HUB: &str = "update-feed";
fn ensure(value: bool, message: &str) -> Result<()> {
    if value { Ok(()) } else { Err(message.into()) }
}

#[derive(Debug, PartialEq)]
pub struct Versions {
    pub version: String,
    pub channel: String,
}
pub fn versions(tag: &str) -> Result<Versions> {
    let version = Version::parse(
        tag.strip_prefix('v')
            .ok_or("Expected a v-prefixed SemVer tag")?,
    )?;
    ensure(
        version.build.is_empty(),
        "Build metadata is not a release identity",
    )?;
    let channel = if version.pre.is_empty() {
        "stable"
    } else {
        let (kind, number) = version
            .pre
            .as_str()
            .split_once('.')
            .ok_or("Use alpha.N or beta.N")?;
        ensure(
            matches!(kind, "alpha" | "beta") && number.parse::<u32>().is_ok(),
            "Use alpha.N or beta.N",
        )?;
        kind
    };
    Ok(Versions {
        version: version.to_string(),
        channel: channel.into(),
    })
}
pub fn audiences(channel: &str) -> &[&str] {
    match channel {
        "stable" => &["stable", "beta", "alpha"],
        "beta" => &["beta", "alpha"],
        _ => &["alpha"],
    }
}
fn hash(path: &Path) -> Result<String> {
    use std::fmt::Write;

    let mut encoded = String::with_capacity(64);
    for byte in Sha256::digest(fs::read(path)?) {
        write!(encoded, "{byte:02x}")?;
    }
    Ok(encoded)
}
fn safe_file(directory: &Path, name: &str) -> Result<std::path::PathBuf> {
    ensure(
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
            && name != "."
            && name != "..",
        "Unsafe asset filename",
    )?;
    let path = directory.join(name);
    ensure(
        path.is_file() && !path.is_symlink(),
        &format!("Missing or linked release asset: {}", path.display()),
    )?;
    Ok(path)
}
#[derive(Default)]
struct Element {
    namespace: String,
    name: String,
    attrs: BTreeMap<(String, String), String>,
    children: Vec<Element>,
    text: String,
}

impl Element {
    fn attr(&self, namespace: &str, name: &str) -> &str {
        self.attrs
            .get(&(namespace.to_owned(), name.to_owned()))
            .map_or("", String::as_str)
    }
    fn one(&self, namespace: &str, name: &str) -> Result<&Self> {
        let found = self
            .children
            .iter()
            .filter(|node| node.name == name && node.namespace == namespace)
            .collect::<Vec<_>>();
        ensure(found.len() == 1, &format!("Expected exactly one {name}"))?;
        Ok(found[0])
    }
}

fn namespace(resolved: ResolveResult<'_>) -> Result<String> {
    match resolved {
        ResolveResult::Bound(value) => Ok(value.0.to_owned()),
        ResolveResult::Unbound => Ok(String::new()),
        ResolveResult::Unknown(_) => Err("Undeclared XML namespace".into()),
    }
}

fn parse_xml(text: &str) -> Result<Element> {
    let mut reader = NsReader::from_str(text);
    let mut stack = vec![Element::default()];
    loop {
        let event = reader.read_event()?;
        match event {
            Event::DocType(_) => return Err("DTD is prohibited in update feeds".into()),
            Event::Start(ref start) | Event::Empty(ref start) => {
                let (ns, name) = reader.resolver().resolve_element(start.name());
                let mut node = Element {
                    namespace: namespace(ns)?,
                    name: name.as_ref().to_owned(),
                    ..Element::default()
                };
                for attr in start.attributes() {
                    let attr = attr?;
                    let (ns, name) = reader.resolver().resolve_attribute(attr.key);
                    node.attrs.insert(
                        (namespace(ns)?, name.as_ref().to_owned()),
                        attr.normalized_value(XmlVersion::Implicit1_0)?.into_owned(),
                    );
                }
                if matches!(event, Event::Empty(_)) {
                    stack
                        .last_mut()
                        .ok_or("Invalid XML tree")?
                        .children
                        .push(node);
                } else {
                    stack.push(node);
                }
            }
            Event::End(_) => {
                ensure(stack.len() > 1, "Unexpected XML end tag")?;
                let node = stack.pop().ok_or("Missing XML node")?;
                stack
                    .last_mut()
                    .ok_or("Missing XML parent")?
                    .children
                    .push(node);
            }
            Event::Text(value) => stack
                .last_mut()
                .ok_or("Missing XML node")?
                .text
                .push_str(&value.xml_content(XmlVersion::Implicit1_0)),
            Event::CData(value) => stack
                .last_mut()
                .ok_or("Missing XML node")?
                .text
                .push_str(&value.xml_content(XmlVersion::Implicit1_0)),
            Event::GeneralRef(value) => {
                let escaped = format!("&{};", value.as_ref());
                stack
                    .last_mut()
                    .ok_or("Missing XML node")?
                    .text
                    .push_str(&quick_xml::escape::unescape(&escaped)?);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    ensure(
        stack.len() == 1 && stack[0].children.len() == 1 && stack[0].text.trim().is_empty(),
        "Expected one complete XML document",
    )?;
    Ok(stack
        .pop()
        .ok_or("Missing XML document")?
        .children
        .remove(0))
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub size: u64,
    pub sha256: String,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub channel: String,
    pub assets: BTreeMap<String, Asset>,
}
/// Validate the complete four-target release before any network mutation.
pub fn prepare(
    directory: &Path,
    tag: &str,
    repository: &str,
    public_key: &str,
) -> Result<Manifest> {
    let version = versions(tag)?;
    let key: [u8; 32] = base64::engine::general_purpose::STANDARD
        .decode(public_key.trim())?
        .try_into()
        .map_err(|_| "Invalid Sparkle public key")?;
    let key = VerifyingKey::from_bytes(&key)?;
    for arch in ["arm64", "x86_64"] {
        let filename = format!("BibCiTeX-{}-macos-{arch}.app.zip", version.version);
        let archive = safe_file(directory, &filename)?;
        safe_file(
            directory,
            &format!("BibCiTeX-{}-macos-{arch}.dmg", version.version),
        )?;
        let feed = parse_xml(&fs::read_to_string(safe_file(
            directory,
            &format!("appcast-{arch}.xml"),
        )?)?)?;
        let item = feed.one("", "channel")?.one("", "item")?;
        ensure(
            item.one(SPARKLE, "shortVersionString")?.text == version.version,
            "Sparkle version mismatch",
        )?;
        let enclosure = item.one("", "enclosure")?;
        ensure(
            enclosure.attr("", "url")
                == format!("https://github.com/{repository}/releases/download/{tag}/{filename}"),
            "Unexpected Sparkle payload URL",
        )?;
        ensure(
            enclosure.attr("", "length").parse::<u64>()? == fs::metadata(&archive)?.len(),
            "Sparkle size mismatch",
        )?;
        let signature = Signature::from_slice(
            &base64::engine::general_purpose::STANDARD
                .decode(enclosure.attr(SPARKLE, "edSignature"))?,
        )?;
        key.verify_strict(&fs::read(&archive)?, &signature)?;
    }
    for arch in ["x64", "arm64"] {
        for ext in ["exe", "msi"] {
            safe_file(
                directory,
                &format!("BibCiTeX-{}-windows-{arch}.{ext}", version.version),
            )?;
        }
        let filename = format!("releases.win-{arch}-{}.json", version.channel);
        let feed: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(safe_file(directory, &filename)?)?)?;
        let assets = feed["Assets"].as_array().ok_or("Invalid Velopack feed")?;
        let full = assets
            .iter()
            .filter(|a| a["Type"] == "Full" || a["Type"] == 1)
            .collect::<Vec<_>>();
        ensure(
            full.len() == 1,
            "Expected one full update package per architecture",
        )?;
        let asset = full[0];
        ensure(
            asset["Version"] == version.version && asset["PackageId"] == "BibCiTeX",
            "Velopack identity mismatch",
        )?;
        let name = asset["FileName"]
            .as_str()
            .ok_or("Missing package filename")?;
        ensure(
            name.contains(&format!("win-{arch}")),
            "Velopack architecture missing from filename",
        )?;
        let payload = safe_file(directory, name)?;
        ensure(
            asset["Size"].as_u64() == Some(fs::metadata(&payload)?.len()),
            "Velopack size mismatch",
        )?;
        let checksum = hash(&payload)?;
        ensure(
            asset["SHA256"]
                .as_str()
                .is_some_and(|value| value.eq_ignore_ascii_case(&checksum)),
            "Velopack checksum mismatch",
        )?;
    }
    for locale in ["zh-Hans", "en"] {
        ensure(
            !fs::read_to_string(safe_file(directory, &format!("notes-{locale}.md"))?)?
                .trim()
                .is_empty(),
            "Missing localized release notes",
        )?;
    }
    let mut assets = BTreeMap::new();
    for file in fs::read_dir(directory)? {
        let file = file?;
        let name = file
            .file_name()
            .into_string()
            .map_err(|_| "Invalid asset name")?;
        if matches!(
            name.as_str(),
            "release.json" | "SHA256SUMS" | "release-body.md"
        ) {
            continue;
        }
        let path = safe_file(directory, &name)?;
        let size = fs::metadata(&path)?.len();
        ensure(size > 0, "Empty release asset")?;
        assets.insert(
            name,
            Asset {
                size,
                sha256: hash(&path)?,
            },
        );
    }
    let manifest = Manifest {
        version: version.version,
        channel: version.channel,
        assets,
    };
    fs::write(
        directory.join("SHA256SUMS"),
        manifest
            .assets
            .iter()
            .map(|(name, a)| format!("{}  {name}\n", a.sha256))
            .collect::<String>(),
    )?;
    fs::write(
        directory.join("release.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

#[derive(Default, Serialize, Deserialize)]
pub struct Channels {
    pub versions: BTreeMap<String, String>,
}
/// Generate only forward-moving channel pointers. Package URLs remain version-specific.
pub fn feeds(
    directory: &Path,
    output: &Path,
    repository: &str,
    manifest: &Manifest,
    channels: &mut Channels,
) -> Result<Vec<String>> {
    fs::create_dir_all(output)?;
    let current = Version::parse(&manifest.version)?;
    let mut result = Vec::new();
    for audience in audiences(&manifest.channel) {
        if let Some(previous) = channels.versions.get(*audience)
            && Version::parse(previous)? >= current
        {
            continue;
        }
        for arch in ["arm64", "x86_64"] {
            let source = fs::read_to_string(directory.join(format!("appcast-{arch}.xml")))?;
            let root = parse_xml(&source)?;
            root.one("", "channel")?.one("", "item")?;
            for locale in ["zh-Hans", "en"] {
                let link = format!(
                    "https://github.com/{repository}/releases/download/v{}/notes-{locale}.md",
                    manifest.version
                );
                let xml = source.replace(
                    "</item>",
                    &format!("<sparkle:releaseNotesLink>{link}</sparkle:releaseNotesLink></item>"),
                );
                let name = format!("appcast-{arch}-{audience}-{locale}.xml");
                fs::write(output.join(&name), xml)?;
                result.push(name);
            }
        }
        for arch in ["x64", "arm64"] {
            let name = format!("releases.win-{arch}-{audience}.json");
            let source = directory.join(format!("releases.win-{arch}-{}.json", manifest.channel));
            fs::copy(source, output.join(&name))?;
            result.push(name);
        }
        channels
            .versions
            .insert((*audience).into(), manifest.version.clone());
    }
    fs::write(
        output.join("channels.json"),
        serde_json::to_vec_pretty(channels)?,
    )?;
    result.push("channels.json".into());
    Ok(result)
}

pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub error: String,
}
pub fn github(arguments: &[String]) -> Result<CommandResult> {
    let output = Command::new("gh").args(arguments).output()?;
    Ok(CommandResult {
        success: output.status.success(),
        output: String::from_utf8(output.stdout)?,
        error: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).into()).collect()
}
fn successful(value: CommandResult) -> Result<()> {
    ensure(value.success, &value.error)
}
/// Publish all immutable assets first, then advance the separate channel index release.
pub fn publish_release(
    tag: &str,
    directory: &Path,
    repository: &str,
    public_key: &str,
    mut gh: impl FnMut(&[String]) -> Result<CommandResult>,
) -> Result<()> {
    let manifest = prepare(directory, tag, repository, public_key)?;
    // The version release is published by the maintainer before this workflow runs.
    // Never delete or replace public assets: retry only the missing files.
    let existing = gh(&args(&[
        "api",
        &format!("repos/{repository}/releases/tags/{tag}"),
    ]))?;
    ensure(existing.success, &existing.error)?;
    let value: serde_json::Value = serde_json::from_str(&existing.output)?;
    ensure(value["tag_name"] == tag, "Release tag mismatch")?;
    ensure(
        value["draft"] == false,
        "Publish the release before uploading assets",
    )?;
    ensure(
        value["prerelease"] == (manifest.channel != "stable"),
        "Release prerelease flag does not match tag",
    )?;
    let remote = value["assets"].as_array().ok_or("Invalid release assets")?;
    let mut missing = Vec::new();
    // Validate every existing file before mutating anything. Upload the inventory last.
    for name in manifest
        .assets
        .keys()
        .map(String::as_str)
        .chain(["SHA256SUMS", "release.json"])
    {
        let path = safe_file(directory, name)?;
        if let Some(asset) = remote.iter().find(|asset| asset["name"] == name) {
            ensure(
                asset["size"].as_u64() == Some(fs::metadata(&path)?.len())
                    && asset["digest"] == format!("sha256:{}", hash(&path)?),
                &format!(
                    "Existing release asset differs or has no digest: {name}; refusing overwrite"
                ),
            )?;
        } else {
            missing.push(path);
        }
    }
    for path in missing {
        successful(gh(&args(&[
            "release",
            "upload",
            tag,
            "--repo",
            repository,
            path.to_str().ok_or("Invalid path")?,
        ]))?)?;
    }
    let work = tempfile::tempdir()?;
    let hub = gh(&args(&[
        "api",
        &format!("repos/{repository}/releases/tags/{HUB}"),
    ]))?;
    let mut channels = Channels::default();
    let mut remote_assets = BTreeMap::new();
    if hub.success {
        let value: serde_json::Value = serde_json::from_str(&hub.output)?;
        for asset in value["assets"]
            .as_array()
            .ok_or("Invalid update release response")?
        {
            let name = asset["name"].as_str().ok_or("Invalid update asset")?;
            remote_assets.insert(name.to_owned(), asset.clone());
        }
        if remote_assets.contains_key("channels.json") {
            successful(gh(&args(&[
                "release",
                "download",
                HUB,
                "--repo",
                repository,
                "--pattern",
                "channels.json",
                "--dir",
                work.path().to_str().ok_or("Invalid path")?,
            ]))?)?;
            channels = serde_json::from_slice(&fs::read(work.path().join("channels.json"))?)?;
        } else {
            ensure(
                remote_assets.is_empty(),
                "Update catalog is missing; refuse to reset existing feeds",
            )?;
        }
    } else {
        ensure(
            hub.error.contains("404") || hub.error.to_lowercase().contains("release not found"),
            &hub.error,
        )?;
        // GitHub accepts a branch or commit SHA here, not a release tag.
        let target = gh(&args(&[
            "api",
            &format!("repos/{repository}/commits/{tag}"),
            "--jq",
            ".sha",
        ]))?;
        ensure(target.success, &target.error)?;
        let revision = target.output.trim();
        ensure(
            revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "Invalid release commit SHA",
        )?;
        successful(gh(&args(&[
            "release",
            "create",
            HUB,
            "--repo",
            repository,
            "--target",
            revision,
            "--prerelease",
            "--latest=false",
            "--title",
            "Update feeds",
        ]))?)?;
    }
    if !remote_assets.contains_key("channels.json") {
        fs::write(
            work.path().join("channels.json"),
            serde_json::to_vec_pretty(&channels)?,
        )?;
        successful(gh(&args(&[
            "release",
            "upload",
            HUB,
            "--repo",
            repository,
            "--clobber",
            work.path()
                .join("channels.json")
                .to_str()
                .ok_or("Invalid path")?,
        ]))?)?;
    }
    let names = feeds(directory, work.path(), repository, &manifest, &mut channels)?;
    // Full packages must be reachable at the feed base URL before a feed points at them.
    let mut payloads = args(&["release", "upload", HUB, "--repo", repository]);
    for name in manifest
        .assets
        .keys()
        .filter(|name| name.ends_with(".nupkg"))
    {
        if let Some(remote) = remote_assets.get(name) {
            let expected = &manifest.assets[name];
            ensure(
                remote["size"].as_u64() == Some(expected.size)
                    && remote["digest"] == format!("sha256:{}", expected.sha256),
                "Existing update package differs or has no digest",
            )?;
        } else {
            payloads.push(directory.join(name).to_string_lossy().into_owned());
        }
    }
    if payloads.len() > 5 {
        successful(gh(&payloads)?)?;
    }
    // Publish the catalog last so failed runs can safely regenerate the same pointers.
    for name in names {
        successful(gh(&args(&[
            "release",
            "upload",
            HUB,
            "--repo",
            repository,
            "--clobber",
            work.path().join(name).to_str().ok_or("Invalid path")?,
        ]))?)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests;
