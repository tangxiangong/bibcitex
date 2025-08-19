use crate::{
    Error, GitHubAsset, GitHubClient, GitHubRelease, Result, extract_path_from_executable,
};
use futures_util::StreamExt;
use http::{HeaderName, header::ACCEPT};
use reqwest::{
    ClientBuilder,
    header::{HeaderMap, HeaderValue},
};
use semver::Version;
#[cfg(not(target_os = "macos"))]
use std::ffi::OsStr;
use std::{
    env::current_exe,
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

const UPDATER_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct UpdaterBuilder {
    app_name: String,
    github_owner: String,
    github_repo: String,
    current_version: Version,
    executable_path: Option<PathBuf>,
    headers: HeaderMap,
    timeout: Option<Duration>,
    proxy: Option<Url>,
    installer_args: Vec<OsString>,
    current_exe_args: Vec<OsString>,
}

impl UpdaterBuilder {
    pub fn new(
        app_name: &str,
        current_version: Version,
        github_owner: &str,
        github_repo: &str,
    ) -> Self {
        Self {
            installer_args: Vec::new(),
            current_exe_args: Vec::new(),
            app_name: app_name.to_owned(),
            current_version,
            executable_path: None,
            github_owner: github_owner.to_owned(),
            github_repo: github_repo.to_owned(),
            headers: HeaderMap::new(),
            timeout: None,
            proxy: None,
        }
    }

    pub fn executable_path<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.executable_path.replace(p.as_ref().into());
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let key: std::result::Result<HeaderName, http::Error> = key.try_into().map_err(Into::into);
        let value: std::result::Result<HeaderValue, http::Error> =
            value.try_into().map_err(Into::into);
        self.headers.insert(key?, value?);

        Ok(self)
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    pub fn clear_headers(mut self) -> Self {
        self.headers.clear();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn proxy(mut self, proxy: Url) -> Self {
        self.proxy.replace(proxy);
        self
    }

    pub fn installer_arg<S>(mut self, arg: S) -> Self
    where
        S: Into<OsString>,
    {
        self.installer_args.push(arg.into());
        self
    }

    pub fn installer_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.installer_args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn clear_installer_args(mut self) -> Self {
        self.installer_args.clear();
        self
    }

    pub fn build(self) -> Result<Updater> {
        let executable_path = self.executable_path.clone().unwrap_or(current_exe()?);

        // Get the extract_path from the provided executable_path
        let extract_path = if cfg!(target_os = "linux") {
            executable_path
        } else {
            extract_path_from_executable(&executable_path)?
        };

        let github_client = GitHubClient::new(&self.github_owner, &self.github_repo);

        Ok(Updater {
            app_name: self.app_name,
            current_version: self.current_version,
            proxy: self.proxy,
            installer_args: self.installer_args,
            current_exe_args: self.current_exe_args,
            headers: self.headers,
            timeout: self.timeout,
            extract_path,
            github_client,
            latest_release: None,
            proper_asset: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Updater {
    pub app_name: String,
    pub current_version: Version,
    pub proxy: Option<Url>,
    pub github_client: GitHubClient,
    pub headers: HeaderMap,
    pub extract_path: PathBuf,
    pub timeout: Option<Duration>,
    pub installer_args: Vec<OsString>,
    pub current_exe_args: Vec<OsString>,
    pub latest_release: Option<GitHubRelease>,
    pub proper_asset: Option<GitHubAsset>,
}

impl Updater {
    pub async fn latest_release(&self) -> Result<GitHubRelease> {
        self.github_client.get_latest_release().await?.try_into()
    }

    pub fn latest_version(&self) -> Option<Version> {
        self.latest_release
            .as_ref()
            .map(|release| release.version.clone())
    }

    pub fn asset_size(&self) -> Option<u64> {
        self.proper_asset.as_ref().map(|asset| asset.size)
    }

    pub async fn proper_asset(&self) -> Result<GitHubAsset> {
        let release = self.latest_release().await?;
        release.find_proper_asset()
    }

    pub async fn check(&self) -> Result<Option<Updater>> {
        let updater = self.clone();
        let latest_release = self.latest_release().await?;
        if latest_release.version > self.current_version {
            let asset = latest_release.find_proper_asset()?;
            Ok(Some(Self {
                latest_release: Some(latest_release),
                proper_asset: Some(asset),
                ..updater
            }))
        } else {
            Ok(None)
        }
    }

    /// Check for updates and download/install if available.
    ///
    /// This is a convenience method that combines `check()` and `download_and_install()`.
    /// Returns `Ok(true)` if an update was found and installed, `Ok(false)` if no update was needed.
    pub async fn update<C: FnMut(usize, u64), D: FnOnce()>(
        &self,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<bool> {
        if let Some(updater) = self.check().await? {
            updater
                .download_and_install(on_chunk, on_download_finish)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Updater {
    /// Downloads the updater package, verifies it then return it as bytes.
    ///
    /// Use [`Update::install`] to install it
    pub async fn download<C: FnMut(usize, u64), D: FnOnce()>(
        &self,
        mut on_chunk: C,
        on_download_finish: D,
    ) -> Result<Vec<u8>> {
        // Fallback to reqwest if octocrab is not available
        let mut headers = self.headers.clone();
        if !headers.contains_key(ACCEPT) {
            headers.insert(ACCEPT, HeaderValue::from_static("application/octet-stream"));
        }

        let mut request = ClientBuilder::new().user_agent(UPDATER_USER_AGENT);
        if let Some(timeout) = self.timeout {
            request = request.timeout(timeout);
        }
        if let Some(ref proxy) = self.proxy {
            let proxy = reqwest::Proxy::all(proxy.as_str())?;
            request = request.proxy(proxy);
        }

        let download_url = self
            .proper_asset
            .clone()
            .ok_or(Error::AssetNotFound)?
            .browser_download_url
            .clone();

        let response = request
            .build()?
            .get(download_url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Download request failed with status: {}",
                response.status()
            )));
        }

        let mut buffer = Vec::new();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            on_chunk(chunk.len(), self.asset_size().unwrap());
            buffer.extend(chunk);
        }
        on_download_finish();
        Ok(buffer)
    }

    /// Installs the updater package downloaded by [`Update::download`]
    pub fn install(&self, bytes: impl AsRef<[u8]>) -> Result<()> {
        self.install_inner(bytes.as_ref())
    }

    /// Downloads and installs the updater package
    pub async fn download_and_install<C: FnMut(usize, u64), D: FnOnce()>(
        &self,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<()> {
        let bytes = self.download(on_chunk, on_download_finish).await?;
        self.install(bytes)
    }
}

#[cfg(windows)]
fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    string
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
trait PathExt {
    fn wrap_in_quotes(&self) -> Self;
}

#[cfg(windows)]
impl PathExt for PathBuf {
    fn wrap_in_quotes(&self) -> Self {
        let mut msi_path = OsString::from("\"");
        msi_path.push(self.as_os_str());
        msi_path.push("\"");
        PathBuf::from(msi_path)
    }
}

// adapted from https://github.com/rust-lang/rust/blob/1c047506f94cd2d05228eb992b0a6bbed1942349/library/std/src/sys/args/windows.rs#L174
#[cfg(windows)]
fn escape_nsis_current_exe_arg(arg: &&OsStr) -> String {
    let arg = arg.to_string_lossy();
    let mut cmd: Vec<char> = Vec::new();

    // compared to std we additionally escape `/` so that nsis won't interpret them as a beginning of an nsis argument.
    let quote = arg.chars().any(|c| c == ' ' || c == '\t' || c == '/') || arg.is_empty();
    let escape = true;
    if quote {
        cmd.push('"');
    }
    let mut backslashes: usize = 0;
    for x in arg.chars() {
        if escape {
            if x == '\\' {
                backslashes += 1;
            } else {
                if x == '"' {
                    // Add n+1 backslashes to total 2n+1 before internal '"'.
                    cmd.extend((0..=backslashes).map(|_| '\\'));
                }
                backslashes = 0;
            }
        }
        cmd.push(x);
    }
    if quote {
        // Add n backslashes to total 2n before ending '"'.
        cmd.extend((0..backslashes).map(|_| '\\'));
        cmd.push('"');
    }
    cmd.into_iter().collect()
}

#[cfg(windows)]
fn escape_msi_property_arg(arg: impl AsRef<OsStr>) -> String {
    let mut arg = arg.as_ref().to_string_lossy().to_string();

    // Otherwise this argument will get lost in ShellExecute
    if arg.is_empty() {
        return "\"\"\"\"".to_string();
    } else if !arg.contains(' ') && !arg.contains('"') {
        return arg;
    }

    if arg.contains('"') {
        arg = arg.replace('"', r#""""""#);
    }

    if arg.starts_with('-') {
        if let Some((a1, a2)) = arg.split_once('=') {
            format!("{a1}=\"\"{a2}\"\"")
        } else {
            format!("\"\"{arg}\"\"")
        }
    } else {
        format!("\"\"{arg}\"\"")
    }
}
