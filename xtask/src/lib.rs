use quick_xml::{XmlVersion, events::Event, name::ResolveResult, reader::NsReader};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
const SPARKLE: &str = "http://www.andymatuschak.org/xml-namespaces/sparkle";
const INSTALLER: &str = "http://schemas.microsoft.com/appx/appinstaller/2018";

#[derive(Debug, PartialEq, Eq)]
pub struct Versions {
    pub version: String,
    pub windows: String,
    parts: [u16; 3],
}

pub fn versions(tag: &str) -> Result<Versions> {
    let version = tag.strip_prefix('v').ok_or("Expected vMAJOR.MINOR.PATCH")?;
    let parts = version.split('.').collect::<Vec<_>>();
    ensure(
        parts.len() == 3,
        "Expected stable vMAJOR.MINOR.PATCH; prereleases cannot replace stable feeds",
    )?;
    let mut numbers = [0; 3];
    for (index, part) in parts.iter().enumerate() {
        ensure(
            !part.is_empty()
                && part.bytes().all(|b| b.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0')),
            "Noncanonical stable version",
        )?;
        numbers[index] = part
            .parse()
            .map_err(|_| "MSIX version components must be <= 65535")?;
    }
    Ok(Versions {
        version: version.to_owned(),
        windows: format!("{version}.0"),
        parts: numbers,
    })
}

fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
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

fn matching_files(directory: &Path, matches: impl Fn(&str) -> bool) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && matches(&entry.file_name().to_string_lossy()) {
            found.push(entry.path());
        }
    }
    Ok(found)
}

fn asset_from_url(directory: &Path, url: &str, base: &str) -> Result<PathBuf> {
    let name = url
        .strip_prefix(base)
        .ok_or("Unexpected release asset URL")?;
    let mut decoded = Vec::new();
    let mut bytes = name.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes
                .next()
                .and_then(|b| char::from(b).to_digit(16))
                .ok_or("Invalid URL escape")?;
            let low = bytes
                .next()
                .and_then(|b| char::from(b).to_digit(16))
                .ok_or("Invalid URL escape")?;
            decoded.push((high * 16 + low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    let name = String::from_utf8(decoded)?;
    ensure(
        !name.is_empty()
            && name != "."
            && name != ".."
            && !name.contains(['/', '\\', '?', '#', '\0']),
        "Invalid release asset name",
    )?;
    let path = directory.join(name);
    ensure(path.is_file(), "Missing release payload")?;
    Ok(path)
}

pub fn verify_release(
    platform: &str,
    directory: &Path,
    version: &str,
    repository: &str,
    tag: &str,
) -> Result<()> {
    match platform {
        "macos" => {
            let feeds = matching_files(directory, |name| {
                name.starts_with("appcast-") && name.ends_with(".xml")
            })?;
            ensure(
                feeds.len() == 1,
                "Expected one architecture-specific appcast",
            )?;
            let root = parse_xml(&fs::read_to_string(&feeds[0])?)?;
            ensure(
                root.name == "rss" && root.namespace.is_empty(),
                "Expected RSS feed",
            )?;
            let item = root.one("", "channel")?.one("", "item")?;
            let enclosure = item.one("", "enclosure")?;
            let item_version = item
                .one(SPARKLE, "shortVersionString")
                .ok()
                .map(|n| n.text.as_str());
            ensure(
                item_version == Some(version)
                    || enclosure.attr(SPARKLE, "shortVersionString") == version,
                "App version mismatch",
            )?;
            ensure(
                !enclosure.attr(SPARKLE, "edSignature").is_empty(),
                "Unsigned Sparkle update",
            )?;
            let base = format!("https://github.com/{repository}/releases/download/{tag}/");
            let asset = asset_from_url(directory, enclosure.attr("", "url"), &base)?;
            let size: u64 = enclosure.attr("", "length").parse()?;
            ensure(fs::metadata(asset)?.len() == size, "Archive size mismatch")
        }
        "windows" => {
            let feeds = matching_files(directory, |name| name.ends_with(".appinstaller"))?;
            ensure(feeds.len() == 1, "Expected one App Installer file")?;
            let root = parse_xml(&fs::read_to_string(&feeds[0])?)?;
            ensure(
                root.name == "AppInstaller" && root.namespace == INSTALLER,
                "Expected App Installer namespace",
            )?;
            let package = root.one(INSTALLER, "MainPackage")?;
            ensure(
                package.attr("", "Version") == version,
                "MSIX version mismatch",
            )?;
            let base = format!("https://github.com/{repository}/releases/latest/download/");
            let feed_name = feeds[0]
                .file_name()
                .ok_or("Missing feed filename")?
                .to_string_lossy();
            ensure(
                root.attr("", "Uri") == format!("{base}{feed_name}"),
                "Unexpected App Installer URL",
            )?;
            let asset = asset_from_url(directory, package.attr("", "Uri"), &base)?;
            ensure(
                asset
                    .extension()
                    .is_some_and(|extension| extension == "msix"),
                "Missing MSIX payload",
            )
        }
        _ => Err("Unknown release platform".into()),
    }
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

fn arguments(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}
fn command_ok(result: CommandResult) -> Result<()> {
    ensure(result.success, &result.error)
}

#[derive(Deserialize)]
struct LatestRelease {
    tag_name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExistingRelease {
    is_draft: bool,
    is_prerelease: bool,
}

pub fn publish_release(
    tag: &str,
    directory: &Path,
    repository: &str,
    mut gh: impl FnMut(&[String]) -> Result<CommandResult>,
) -> Result<()> {
    let version = versions(tag)?;
    let assets = matching_files(directory, |_| true)?;
    let names = assets
        .iter()
        .filter_map(|p| p.file_name())
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>();
    let expected = [
        format!("BibCiTeX-{}-macos-arm64.zip", version.version),
        format!("BibCiTeX-{}-macos-x86_64.zip", version.version),
        "appcast-arm64.xml".into(),
        "appcast-x86_64.xml".into(),
        "BibCiTeX-x64.appinstaller".into(),
        "BibCiTeX-arm64.appinstaller".into(),
    ];
    ensure(
        assets.len() == 8
            && expected.iter().all(|name| names.iter().any(|n| n == name))
            && names.iter().filter(|n| n.ends_with(".msix")).count() == 2,
        "Missing or unexpected release artifacts",
    )?;
    for asset in &assets {
        ensure(fs::metadata(asset)?.len() > 0, "Empty release artifact")?;
    }
    let latest = gh(&arguments(&[
        "api",
        &format!("repos/{repository}/releases/latest"),
    ]))?;
    if latest.success {
        let latest: LatestRelease = serde_json::from_str(&latest.output)?;
        if let Ok(previous) = versions(&latest.tag_name) {
            ensure(
                version.parts > previous.parts,
                "The stable release must be newer than the currently published stable version",
            )?;
        }
    } else {
        ensure(latest.error.contains("404"), &latest.error)?;
    }
    let release = gh(&arguments(&[
        "release",
        "view",
        tag,
        "--json",
        "isDraft,isPrerelease",
    ]))?;
    if release.success {
        let release: ExistingRelease = serde_json::from_str(&release.output)?;
        ensure(
            release.is_draft && !release.is_prerelease,
            "Refusing to replace an already published or prerelease release",
        )?;
    } else {
        ensure(
            release.error.contains("404")
                || release.error.to_lowercase().contains("release not found"),
            &release.error,
        )?;
        command_ok(gh(&arguments(&[
            "release",
            "create",
            tag,
            "--verify-tag",
            "--draft",
            "--title",
            tag,
            "--generate-notes",
        ]))?)?;
    }
    let mut upload = arguments(&["release", "upload", tag]);
    for asset in &assets {
        upload.push(asset.to_str().ok_or("Non-UTF-8 release path")?.to_owned());
    }
    upload.push("--clobber".into());
    // A failed upload leaves a draft; all previously published feeds stay intact.
    command_ok(gh(&upload)?)?;
    command_ok(gh(&arguments(&[
        "release",
        "edit",
        tag,
        "--draft=false",
        "--prerelease=false",
        "--latest",
    ]))?)
}

#[cfg(test)]
mod tests;
