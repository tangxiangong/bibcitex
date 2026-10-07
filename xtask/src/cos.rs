//! COS is the primary mirror; GitHub retains the same signed payloads as a fallback.
use crate::{Channels, Result, ensure, feeds_at, hash, prepare};
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Method, StatusCode, blocking::Client};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::Read,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const PUBLIC_ROOT: &str =
    "https://app-release-1302963684.cos.ap-guangzhou.myqcloud.com/bibcitex";
const UPLOAD_HOST: &str = "app-release-1302963684.cos.accelerate.myqcloud.com";
const CATALOG: &str = "update-feed/channels.json";
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";
const FEED_CACHE: &str = "no-cache, max-age=60";

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hmac(key: &[u8], value: &str) -> Result<String> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key)?;
    mac.update(value.as_bytes());
    Ok(hex(&mac.finalize().into_bytes()))
}

// COS v5 requires the hexadecimal SignKey as the second HMAC key, not raw bytes.
fn authorization(
    secret_id: &str,
    secret_key: &str,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    now: u64,
) -> Result<String> {
    let time = format!("{};{}", now.saturating_sub(60), now + 3600);
    let names = headers.keys().cloned().collect::<Vec<_>>().join(";");
    let values = headers
        .iter()
        .map(|(key, value)| format!("{}={}", encode(key), encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let request = format!("{}\n{path}\n\n{values}\n", method.to_ascii_lowercase());
    let request_hash = hex(&Sha1::digest(request.as_bytes()));
    let string_to_sign = format!("sha1\n{time}\n{request_hash}\n");
    let sign_key = hmac(secret_key.as_bytes(), &time)?;
    let signature = hmac(sign_key.as_bytes(), &string_to_sign)?;
    Ok(format!(
        "q-sign-algorithm=sha1&q-ak={secret_id}&q-sign-time={time}&q-key-time={time}&q-header-list={names}&q-url-param-list=&q-signature={signature}"
    ))
}

trait Store {
    fn catalog(&self) -> Result<Option<Vec<u8>>>;
    fn matches(&self, key: &str, path: &Path) -> Result<bool>;
    fn upload(&self, key: &str, path: &Path, immutable: bool) -> Result<()>;
}

struct Cos {
    client: Client,
    secret_id: String,
    secret_key: String,
}

impl Cos {
    fn new() -> Result<Self> {
        let secret_id = env::var("COS_SECRET_ID")?;
        let secret_key = env::var("COS_SECRET_KEY")?;
        ensure(
            !secret_id.is_empty() && !secret_key.is_empty(),
            "Missing COS credentials",
        )?;
        Ok(Self {
            client: Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(1800))
                .build()?,
            secret_id,
            secret_key,
        })
    }

    fn request(
        &self,
        method: Method,
        key: &str,
        file: Option<(&Path, bool)>,
    ) -> Result<reqwest::blocking::Response> {
        let path = format!("/bibcitex/{key}");
        let encoded = path.split('/').map(encode).collect::<Vec<_>>().join("/");
        for attempt in 0..4 {
            let mut headers = BTreeMap::from([("host".into(), UPLOAD_HOST.into())]);
            if let Some((file, immutable)) = file {
                headers.insert(
                    "content-length".into(),
                    fs::metadata(file)?.len().to_string(),
                );
                headers.insert("content-type".into(), content_type(key).into());
                headers.insert(
                    "cache-control".into(),
                    if immutable && key != CATALOG {
                        IMMUTABLE_CACHE
                    } else {
                        FEED_CACHE
                    }
                    .into(),
                );
                headers.insert("x-cos-acl".into(), "public-read".into());
                if immutable {
                    headers.insert("x-cos-forbid-overwrite".into(), "true".into());
                }
            }
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let signature = authorization(
                &self.secret_id,
                &self.secret_key,
                method.as_str(),
                &path,
                &headers,
                now,
            )?;
            let mut request = self
                .client
                .request(method.clone(), format!("https://{UPLOAD_HOST}{encoded}"));
            for (key, value) in headers {
                request = request.header(key, value);
            }
            request = request.header("authorization", signature);
            if let Some((file, _)) = file {
                request = request.body(fs::File::open(file)?);
            }
            match request.send() {
                Ok(response)
                    if attempt < 3
                        && (response.status().is_server_error()
                            || response.status() == StatusCode::TOO_MANY_REQUESTS
                            || response.status() == StatusCode::REQUEST_TIMEOUT) => {}
                Ok(response) => return Ok(response),
                Err(error) if attempt == 3 => return Err(error.into()),
                Err(_) => {}
            }
            std::thread::sleep(Duration::from_secs(1 << attempt));
        }
        Err("COS request retry limit exceeded".into())
    }

    fn public_get(&self, key: &str) -> Result<reqwest::blocking::Response> {
        let encoded = key.split('/').map(encode).collect::<Vec<_>>().join("/");
        Ok(self
            .client
            .get(format!("{PUBLIC_ROOT}/{encoded}"))
            .header("cache-control", "no-cache")
            .send()?
            .error_for_status()?)
    }
}

impl Store for Cos {
    fn catalog(&self) -> Result<Option<Vec<u8>>> {
        let response = self.request(Method::HEAD, CATALOG, None)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        response.error_for_status()?;
        Ok(Some(self.public_get(CATALOG)?.bytes()?.to_vec()))
    }

    fn matches(&self, key: &str, path: &Path) -> Result<bool> {
        let response = self.request(Method::HEAD, key, None)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(false);
        }
        response.error_for_status()?;
        // Check the bytes through the anonymous endpoint used by clients, not just metadata.
        let mut response = self.public_get(key)?;
        let mut digest = Sha256::new();
        let mut size = 0u64;
        let mut buffer = [0; 64 * 1024];
        loop {
            let count = response.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
            size += count as u64;
        }
        ensure(
            size == fs::metadata(path)?.len()
                && digest
                    .finalize()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
                    == hash(path)?,
            &format!("COS object differs: {key}; refusing overwrite"),
        )?;
        Ok(true)
    }

    fn upload(&self, key: &str, path: &Path, immutable: bool) -> Result<()> {
        let response = self.request(Method::PUT, key, Some((path, immutable)))?;
        // A previous attempt may have succeeded before its response was lost.
        if response.status() != StatusCode::CONFLICT || !immutable {
            response.error_for_status()?;
        }
        ensure(
            self.matches(key, path)?,
            &format!("COS object is not publicly available: {key}"),
        )?;
        println!("Published COS object: {key}");
        Ok(())
    }
}

fn content_type(key: &str) -> &'static str {
    if key.ends_with(".xml") {
        "application/xml; charset=utf-8"
    } else if key.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if key.ends_with(".md") || key.ends_with("SHA256SUMS") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

pub fn publish(tag: &str, directory: &Path, repository: &str, public_key: &str) -> Result<()> {
    publish_to(tag, directory, repository, public_key, &Cos::new()?)
}

fn publish_to(
    tag: &str,
    directory: &Path,
    repository: &str,
    public_key: &str,
    store: &impl Store,
) -> Result<()> {
    let manifest = prepare(directory, tag, repository, public_key)?;
    let catalog = store.catalog()?;
    let mut channels: Channels = match &catalog {
        Some(bytes) => serde_json::from_slice(bytes)?,
        None => Channels::default(),
    };
    // Equal versions must regenerate pointers after an interrupted publication. Higher
    // reservations remain intact so an older repair cannot downgrade a partial feed.
    let affected = crate::audiences(&manifest.channel);
    channels.versions.retain(|audience, version| {
        !affected.contains(&audience.as_str()) || version != &manifest.version
    });
    let work = tempfile::tempdir()?;
    let release_url = format!("{PUBLIC_ROOT}/releases/{tag}");
    let names = feeds_at(
        directory,
        work.path(),
        &release_url,
        &manifest,
        &mut channels,
    )?;
    // Only the COS appcasts change URLs; signatures still authenticate identical archives.
    for name in names.iter().filter(|name| name.ends_with(".xml")) {
        let path = work.path().join(name);
        let xml = fs::read_to_string(&path)?.replace(
            &format!("https://github.com/{repository}/releases/download/{tag}/"),
            &format!("{release_url}/"),
        );
        fs::write(path, xml)?;
    }
    let mut missing = Vec::new();
    for name in manifest
        .assets
        .keys()
        .map(String::as_str)
        .chain(["SHA256SUMS", "release.json"])
    {
        let path = directory.join(name);
        let mut keys = vec![format!("releases/{tag}/{name}")];
        if name.ends_with(".nupkg") {
            keys.push(format!("update-feed/{name}"));
        }
        for key in keys {
            if !store.matches(&key, &path)? {
                missing.push((key, path.clone()));
            }
        }
    }
    for (key, path) in missing {
        store.upload(&key, &path, true)?;
    }
    // Reserve the highest target versions before any client-facing pointer changes.
    // Otherwise an interrupted publication followed by an older tag could regress feeds.
    store.upload(CATALOG, &work.path().join("channels.json"), false)?;
    for name in names.into_iter().filter(|name| name != "channels.json") {
        store.upload(
            &format!("update-feed/{name}"),
            &work.path().join(name),
            false,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
