use crate::error::{Error, Result};
use futures_util::StreamExt;
use http::{HeaderName, header::ACCEPT};
use octocrab::Octocrab;
use reqwest::{
    ClientBuilder,
    header::{HeaderMap, HeaderValue},
};
use semver::Version;
use serde::Deserialize;
#[cfg(not(target_os = "macos"))]
use std::ffi::OsStr;
use std::{
    env::current_exe,
    ffi::OsString,
    io::Cursor,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use url::Url;

const UPDATER_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// 系统硬件信息检测结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub target: String,
}

impl SystemInfo {
    /// 获取当前系统的硬件信息
    pub fn current() -> Option<Self> {
        let os = get_os_name()?;
        let arch = get_arch_name()?;
        let target = format!("{}-{}", os, arch);

        Some(SystemInfo { os, arch, target })
    }

    /// 检查是否为macOS系统
    pub fn is_macos(&self) -> bool {
        self.os == "macos"
    }

    /// 检查是否为Windows系统
    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }

    /// 检查是否为Linux系统
    pub fn is_linux(&self) -> bool {
        self.os == "linux"
    }

    /// 检查是否为ARM架构
    pub fn is_arm(&self) -> bool {
        self.arch.contains("arm") || self.arch == "aarch64"
    }

    /// 检查是否为x86_64架构
    pub fn is_x86_64(&self) -> bool {
        self.arch == "x86_64"
    }
}

/// 获取操作系统名称
fn get_os_name() -> Option<String> {
    if cfg!(target_os = "linux") {
        Some("linux".to_string())
    } else if cfg!(target_os = "macos") {
        Some("macos".to_string())
    } else if cfg!(target_os = "windows") {
        Some("windows".to_string())
    } else {
        None
    }
}

/// 获取架构名称
fn get_arch_name() -> Option<String> {
    if cfg!(target_arch = "x86") {
        Some("i686".to_string())
    } else if cfg!(target_arch = "x86_64") {
        Some("x86_64".to_string())
    } else if cfg!(target_arch = "arm") {
        Some("armv7".to_string())
    } else if cfg!(target_arch = "aarch64") {
        Some("aarch64".to_string())
    } else if cfg!(target_arch = "riscv64") {
        Some("riscv64".to_string())
    } else {
        None
    }
}

/// GitHub Release客户端，使用octocrab库
#[derive(Debug, Clone)]
pub struct GitHubReleaseClient {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubReleaseClient {
    /// 创建新的GitHub Release客户端
    pub fn new(owner: String, repo: String) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .build()
            .map_err(|e| Error::Network(format!("Failed to create GitHub client: {}", e)))?;

        Ok(Self {
            octocrab,
            owner,
            repo,
        })
    }

    /// 获取最新的Release信息
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let release = self
            .octocrab
            .repos(&self.owner, &self.repo)
            .releases()
            .get_latest()
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch latest release: {}", e)))?;

        // 转换octocrab的Release到我们的GitHubRelease结构体
        let version =
            Version::parse(release.tag_name.trim_start_matches('v')).map_err(Error::Semver)?;

        let assets = release
            .assets
            .into_iter()
            .map(|asset| GitHubAsset {
                name: asset.name,
                browser_download_url: asset.browser_download_url,
                size: asset.size as u64,
            })
            .collect();

        Ok(GitHubRelease {
            tag_name: version,
            name: release.name,
            body: release.body,
            published_at: release.published_at.map(|dt| dt.to_rfc3339()),
            assets,
            _extra: serde_json::Value::Null,
        })
    }

    /// 根据系统信息查找匹配的资源文件
    pub fn find_asset_for_system<'a>(
        &self,
        release: &'a GitHubRelease,
        system_info: &SystemInfo,
    ) -> Option<&'a GitHubAsset> {
        println!(
            "[DEBUG] find_asset_for_system: Looking for assets matching os={}, arch={}",
            system_info.os, system_info.arch
        );

        let result = release.assets.iter().find(|asset| {
            let name = asset.name.to_lowercase();
            println!("[DEBUG] Checking asset: {}", name);

            // 检查操作系统匹配
            let os_match = match system_info.os.as_str() {
                "macos" => {
                    let matches =
                        name.contains("macos") || name.contains("darwin") || name.contains("osx");
                    println!("[DEBUG]   OS match (macos): {}", matches);
                    matches
                }
                "windows" => {
                    let matches = name.contains("windows") || name.contains("win");
                    println!("[DEBUG]   OS match (windows): {}", matches);
                    matches
                }
                "linux" => {
                    let matches = name.contains("linux");
                    println!("[DEBUG]   OS match (linux): {}", matches);
                    matches
                }
                _ => {
                    println!("[DEBUG]   OS match (unknown): false");
                    false
                }
            };

            // 检查架构匹配
            let arch_match = match system_info.arch.as_str() {
                "x86_64" => {
                    let matches = name.contains("x86_64") || name.contains("amd64");
                    println!("[DEBUG]   Arch match (x86_64): {}", matches);
                    matches
                }
                "aarch64" => {
                    let matches = name.contains("aarch64") || name.contains("arm64");
                    println!("[DEBUG]   Arch match (aarch64): {}", matches);
                    matches
                }
                "armv7" => {
                    let matches = name.contains("armv7") || name.contains("arm");
                    println!("[DEBUG]   Arch match (armv7): {}", matches);
                    matches
                }
                "i686" => {
                    let matches = name.contains("i686") || name.contains("x86");
                    println!("[DEBUG]   Arch match (i686): {}", matches);
                    matches
                }
                _ => {
                    println!("[DEBUG]   Arch match (unknown): false");
                    false
                }
            };

            // 检查文件扩展名
            let ext_match = match system_info.os.as_str() {
                "macos" => {
                    let matches = name.ends_with(".dmg")
                        || name.ends_with(".app.zip")
                        || name.ends_with(".zip");
                    println!("[DEBUG]   Ext match (macos): {}", matches);
                    matches
                }
                "windows" => {
                    let matches = name.ends_with(".exe") || name.ends_with(".msi");
                    println!("[DEBUG]   Ext match (windows): {}", matches);
                    matches
                }
                "linux" => {
                    let matches = name.ends_with(".tar.gz")
                        || name.ends_with(".deb")
                        || name.ends_with(".appimage");
                    println!("[DEBUG]   Ext match (linux): {}", matches);
                    matches
                }
                _ => {
                    println!("[DEBUG]   Ext match (unknown): false");
                    false
                }
            };

            let overall_match = os_match && arch_match && ext_match;
            println!(
                "[DEBUG]   Overall match: {} (os={}, arch={}, ext={})",
                overall_match, os_match, arch_match, ext_match
            );
            overall_match
        });

        if let Some(asset) = result {
            println!(
                "[DEBUG] find_asset_for_system: Found matching asset: {}",
                asset.name
            );
        } else {
            println!("[DEBUG] find_asset_for_system: No matching asset found");
        }

        result
    }

    /// 检查资源文件是否为macOS的.zip格式应用包
    pub fn is_macos_zip_app(&self, asset: &GitHubAsset, system_info: &SystemInfo) -> bool {
        system_info.is_macos() && asset.name.to_lowercase().ends_with(".app.zip")
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: Url,
    pub size: u64,
}

/// GitHub API release response structure
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    /// Version to install.
    #[serde(deserialize_with = "parse_version")]
    pub tag_name: Version,
    /// Release name.
    pub name: Option<String>,
    /// Release notes.
    pub body: Option<String>,
    /// Release date.
    pub published_at: Option<String>,
    /// Assets.
    pub assets: Vec<GitHubAsset>,
    /// Ignore all other fields from GitHub API
    #[serde(flatten)]
    pub _extra: serde_json::Value,
}

/// Information about a release returned by GitHub API.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteRelease {
    /// Version to install.
    pub version: Version,
    /// Release notes.
    pub notes: Option<String>,
    /// Release date.
    pub pub_date: Option<String>,
    /// GitHub assets.
    pub assets: Vec<GitHubAsset>,
}

/// Convert GitHub release to RemoteRelease
impl From<GitHubRelease> for RemoteRelease {
    fn from(github_release: GitHubRelease) -> Self {
        RemoteRelease {
            version: github_release.tag_name,
            notes: github_release.body,
            pub_date: github_release.published_at,
            assets: github_release.assets,
        }
    }
}

impl RemoteRelease {
    /// Get the version of this release
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Get the release notes
    pub fn notes(&self) -> &Option<String> {
        &self.notes
    }

    /// Get the publication date
    pub fn pub_date(&self) -> &Option<String> {
        &self.pub_date
    }

    /// The release's download URL for the given target.
    pub fn download_url(&self, target: &str) -> Result<Url> {
        println!("[DEBUG] download_url called with target: {}", target);
        println!(
            "[DEBUG] Available assets: {:?}",
            self.assets.iter().map(|a| &a.name).collect::<Vec<_>>()
        );

        // Use SystemInfo for better asset matching
        if let Some(system_info) = SystemInfo::current() {
            println!(
                "[DEBUG] SystemInfo: os={}, arch={}, target={}",
                system_info.os, system_info.arch, system_info.target
            );

            // Create a temporary GitHubReleaseClient for asset matching
            let temp_client = GitHubReleaseClient::new("temp".to_string(), "temp".to_string())?;
            let github_release = GitHubRelease {
                tag_name: self.version.clone(),
                name: None,
                body: self.notes.clone(),
                published_at: self.pub_date.clone(),
                assets: self.assets.clone(),
                _extra: serde_json::Value::Null,
            };

            if let Some(asset) = temp_client.find_asset_for_system(&github_release, &system_info) {
                println!("[DEBUG] Found matching asset: {}", asset.name);
                return Ok(asset.browser_download_url.clone());
            } else {
                println!("[DEBUG] No matching asset found using SystemInfo");
            }
        } else {
            println!("[DEBUG] SystemInfo not available");
        }

        // Fallback to simple name matching if SystemInfo is not available or no match found
        let asset = self.assets.iter().find(|asset| {
            let name = asset.name.to_lowercase();
            // Match based on target architecture and OS
            if target.contains("darwin") || target.contains("macos") {
                name.contains("macos")
                    && ((target.contains("arm64") && name.contains("arm64"))
                        || (target.contains("x86_64") && name.contains("x86_64")))
                    && (name.ends_with(".dmg")
                        || name.ends_with(".app.zip")
                        || name.ends_with(".zip"))
            } else if target.contains("windows") {
                name.contains("windows")
                    && ((target.contains("arm64") && name.contains("arm64"))
                        || (target.contains("x86_64") && name.contains("x86_64")))
                    && (name.ends_with(".exe") || name.ends_with(".msi"))
            } else if target.contains("linux") {
                name.contains("linux")
                    && ((target.contains("arm64") && name.contains("arm64"))
                        || (target.contains("x86_64") && name.contains("x86_64")))
                    && (name.ends_with(".tar.gz")
                        || name.ends_with(".deb")
                        || name.ends_with(".appimage"))
            } else {
                false
            }
        });

        // If still no match, try a more relaxed matching
        let asset = asset.or_else(|| {
            self.assets.iter().find(|asset| {
                let name = asset.name.to_lowercase();
                // More relaxed matching - just check OS and common file extensions
                if target.contains("darwin") || target.contains("macos") {
                    (name.contains("macos") || name.contains("darwin") || name.contains("osx"))
                        && (name.ends_with(".dmg") || name.ends_with(".zip"))
                } else if target.contains("windows") {
                    (name.contains("windows") || name.contains("win"))
                        && (name.ends_with(".exe") || name.ends_with(".msi"))
                } else if target.contains("linux") {
                    name.contains("linux")
                        && (name.ends_with(".tar.gz")
                            || name.ends_with(".deb")
                            || name.ends_with(".appimage"))
                } else {
                    false
                }
            })
        });

        asset
            .map(|a| a.browser_download_url.clone())
            .ok_or_else(|| {
                println!(
                    "Available assets: {:?}",
                    self.assets.iter().map(|a| &a.name).collect::<Vec<_>>()
                );
                println!("Target: {}", target);
                Error::TargetNotFound(target.to_string())
            })
    }
}

pub type OnBeforeExit = Arc<dyn Fn() + Send + Sync + 'static>;
pub type OnBeforeRequest = Arc<dyn Fn(ClientBuilder) -> ClientBuilder + Send + Sync + 'static>;
pub type VersionComparator = Arc<dyn Fn(Version, RemoteRelease) -> bool + Send + Sync>;

pub struct UpdaterBuilder {
    app_name: String,
    current_version: Version,
    pub(crate) version_comparator: Option<VersionComparator>,
    executable_path: Option<PathBuf>,
    target: Option<String>,
    headers: HeaderMap,
    timeout: Option<Duration>,
    proxy: Option<Url>,
    installer_args: Vec<OsString>,
    current_exe_args: Vec<OsString>,
    on_before_exit: Option<OnBeforeExit>,
    configure_client: Option<OnBeforeRequest>,
    // GitHub相关配置
    github_owner: Option<String>,
    github_repo: Option<String>,
}

impl UpdaterBuilder {
    pub fn new(app_name: String, current_version: Version) -> Self {
        Self {
            installer_args: Vec::new(),
            current_exe_args: Vec::new(),
            app_name,
            current_version,
            version_comparator: None,
            executable_path: None,
            target: None,
            headers: HeaderMap::new(),
            timeout: None,
            proxy: None,
            on_before_exit: None,
            configure_client: None,
            github_owner: None,
            github_repo: None,
        }
    }

    pub fn version_comparator<F: Fn(Version, RemoteRelease) -> bool + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.version_comparator = Some(Arc::new(f));
        self
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target.replace(target.into());
        self
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

    pub fn on_before_exit<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_before_exit.replace(Arc::new(f));
        self
    }

    /// Allows you to modify the `reqwest` client builder before the HTTP request is sent.
    ///
    /// Note that `reqwest` crate may be updated in minor releases of tauri-plugin-updater.
    /// Therefore it's recommended to pin the plugin to at least a minor version when you're using `configure_client`.
    ///
    pub fn configure_client<F: Fn(ClientBuilder) -> ClientBuilder + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.configure_client.replace(Arc::new(f));
        self
    }

    pub fn github_repo(mut self, owner: String, repo: String) -> Self {
        self.github_owner = Some(owner);
        self.github_repo = Some(repo);
        self
    }

    pub fn build(self) -> Result<Updater> {
        let arch = get_updater_arch().ok_or(Error::UnsupportedArch)?;
        let (target, json_target) = if let Some(target) = self.target {
            (target.clone(), target)
        } else {
            let target = get_updater_target().ok_or(Error::UnsupportedOs)?;
            (target.to_string(), format!("{target}-{arch}"))
        };

        let executable_path = self.executable_path.clone().unwrap_or(current_exe()?);

        // Get the extract_path from the provided executable_path
        let extract_path = if cfg!(target_os = "linux") {
            executable_path
        } else {
            extract_path_from_executable(&executable_path)?
        };

        let github_client = if let (Some(owner), Some(repo)) = (self.github_owner, self.github_repo)
        {
            Some(GitHubReleaseClient::new(owner, repo)?)
        } else {
            None
        };

        Ok(Updater {
            app_name: self.app_name,
            current_version: self.current_version,
            version_comparator: self.version_comparator,
            proxy: self.proxy,

            installer_args: self.installer_args,
            current_exe_args: self.current_exe_args,
            target,
            json_target,
            headers: self.headers,
            extract_path,
            configure_client: self.configure_client,
            github_client,
        })
    }
}

pub struct Updater {
    app_name: String,
    current_version: Version,
    version_comparator: Option<VersionComparator>,
    proxy: Option<Url>,
    // The `{{target}}` variable we replace in the endpoint
    target: String,
    // The value we search if the updater server returns a JSON with the `platforms` object
    json_target: String,
    headers: HeaderMap,
    extract_path: PathBuf,
    configure_client: Option<OnBeforeRequest>,
    #[allow(unused)]
    installer_args: Vec<OsString>,
    #[allow(unused)]
    current_exe_args: Vec<OsString>,
    github_client: Option<GitHubReleaseClient>,
}

impl Updater {
    pub async fn check(&self) -> Result<Option<(Update, Option<u64>)>> {
        // we want JSON only
        let mut headers = self.headers.clone();
        if !headers.contains_key(ACCEPT) {
            headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        }

        // Set SSL certs for linux if they aren't available.
        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("SSL_CERT_FILE").is_none() {
                std::env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
            }
            if std::env::var_os("SSL_CERT_DIR").is_none() {
                std::env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
            }
        }

        // Only use GitHub API
        let github_client = self.github_client.as_ref().ok_or(Error::ReleaseNotFound)?;
        let github_release = github_client.get_latest_release().await?;
        let release: RemoteRelease = github_release.into();

        let should_update = match self.version_comparator.as_ref() {
            Some(comparator) => comparator(self.current_version.clone(), release.clone()),
            None => release.version() > &self.current_version,
        };

        let update = if should_update {
            // Try to get download URL and asset size
            match release.download_url(&self.json_target) {
                Ok(download_url) => {
                    // Find the matching asset to get its size
                    let asset_size =
                        if let Some(system_info) = crate::updater::SystemInfo::current() {
                            // Create a temporary GitHubRelease for asset matching
                            let github_release = crate::updater::GitHubRelease {
                                tag_name: release.version().clone(),
                                name: None,
                                body: release.notes().clone(),
                                published_at: release.pub_date().clone(),
                                assets: release.assets.clone(),
                                _extra: serde_json::Value::Null,
                            };

                            github_client
                                .find_asset_for_system(&github_release, &system_info)
                                .map(|asset| asset.size)
                        } else {
                            None
                        };

                    Some((
                        Update {
                            app_name: self.app_name.clone(),
                            current_version: self.current_version.to_string(),
                            target: self.target.clone(),
                            extract_path: self.extract_path.clone(),
                            version: release.version().to_string(),
                            date: release.pub_date().clone(),
                            download_url,
                            body: release.notes().clone(),

                            timeout: None,
                            proxy: self.proxy.clone(),
                            headers: self.headers.clone(),
                            installer_args: self.installer_args.clone(),
                            current_exe_args: self.current_exe_args.clone(),
                            configure_client: self.configure_client.clone(),
                            octocrab: Some(github_client.octocrab.clone()),
                        },
                        asset_size,
                    ))
                }
                Err(Error::TargetNotFound(_)) => {
                    println!("No compatible asset found for target: {}", self.json_target);
                    return Err(Error::TargetNotFound(format!(
                        "No compatible asset found for target: {}",
                        self.json_target
                    )));
                }
                Err(e) => return Err(e), // Other errors should still be propagated
            }
        } else {
            None
        };

        Ok(update)
    }

    /// Check for updates and download/install if available.
    ///
    /// This is a convenience method that combines `check()` and `download_and_install()`.
    /// Returns `Ok(true)` if an update was found and installed, `Ok(false)` if no update was needed.
    pub async fn update<C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<bool> {
        if let Some((update, _size)) = self.check().await? {
            update
                .download_and_install(on_chunk, on_download_finish)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone)]
pub struct Update {
    /// Update description
    pub body: Option<String>,
    /// Version used to check for update
    pub current_version: String,
    /// Version announced
    pub version: String,
    /// Update publish date
    pub date: Option<String>,
    /// Target
    pub target: String,
    /// Download URL announced
    pub download_url: Url,
    /// The raw version of server's JSON response. Useful if the response contains additional fields that the updater doesn't handle.

    /// Request timeout
    pub timeout: Option<Duration>,
    /// Request proxy
    pub proxy: Option<Url>,
    /// Request headers
    pub headers: HeaderMap,
    /// Extract path
    #[allow(unused)]
    extract_path: PathBuf,
    /// App name, used for creating named tempfiles on Windows
    #[allow(unused)]
    app_name: String,
    #[allow(unused)]
    installer_args: Vec<OsString>,
    #[allow(unused)]
    current_exe_args: Vec<OsString>,
    configure_client: Option<OnBeforeRequest>,
    /// Octocrab client for GitHub API operations
    octocrab: Option<Octocrab>,
}

impl Update {
    /// Downloads the updater package, verifies it then return it as bytes.
    ///
    /// Use [`Update::install`] to install it
    pub async fn download<C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        mut on_chunk: C,
        on_download_finish: D,
    ) -> Result<Vec<u8>> {
        // Try to use octocrab for download if available
        if let Some(ref _octocrab) = self.octocrab {
            // For now, skip octocrab and use reqwest for proper streaming progress
            // This ensures we get real-time download progress instead of jumping to 100%
            // TODO: Implement proper streaming download with octocrab in the future
        }

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
        if let Some(ref configure_client) = self.configure_client {
            request = configure_client(request);
        }
        let response = request
            .build()?
            .get(self.download_url.clone())
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Download request failed with status: {}",
                response.status()
            )));
        }

        let content_length: Option<u64> = response
            .headers()
            .get("Content-Length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());

        let mut buffer = Vec::new();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            on_chunk(chunk.len(), content_length);
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
    pub async fn download_and_install<C: FnMut(usize, Option<u64>), D: FnOnce()>(
        &self,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<()> {
        let bytes = self.download(on_chunk, on_download_finish).await?;
        self.install(bytes)
    }
}

#[cfg(windows)]
enum WindowsUpdaterType {
    Nsis {
        path: PathBuf,
        #[allow(unused)]
        temp: Option<tempfile::TempPath>,
    },
    Msi {
        path: PathBuf,
        #[allow(unused)]
        temp: Option<tempfile::TempPath>,
    },
}

#[cfg(windows)]
impl WindowsUpdaterType {
    fn nsis(path: PathBuf, temp: Option<tempfile::TempPath>) -> Self {
        Self::Nsis { path, temp }
    }

    fn msi(path: PathBuf, temp: Option<tempfile::TempPath>) -> Self {
        Self::Msi {
            path: path.wrap_in_quotes(),
            temp,
        }
    }
}

// TODO:
/// Windows
#[cfg(windows)]
impl Update {
    /// ### Expected structure:
    /// ├── [AppName]-[version]-windows-x86_64.msi              # Application MSI (x86_64)
    /// ├── [AppName]-[version]-windows-arm64.msi               # Application MSI (arm64)
    /// ├── [AppName]-[version]-windows-x86_64.exe              # NSIS installer (x86_64)
    /// ├── [AppName]-[version]-windows-arm64.exe               # NSIS installer (arm64)
    fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        use std::iter::once;
        use windows_sys::{
            Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOW},
            w,
        };

        let updater_type = self.extract(bytes)?;

        let install_mode = crate::config::WindowsUpdateInstallMode::default();
        let current_args = &self.current_exe_args()[1..];
        let msi_args;
        let nsis_args;

        let installer_args: Vec<&OsStr> = match &updater_type {
            WindowsUpdaterType::Nsis { .. } => {
                nsis_args = current_args
                    .iter()
                    .map(escape_nsis_current_exe_arg)
                    .collect::<Vec<_>>();

                install_mode
                    .nsis_args()
                    .iter()
                    .map(OsStr::new)
                    .chain(once(OsStr::new("/UPDATE")))
                    .chain(once(OsStr::new("/RESTART")))
                    .chain(once(OsStr::new("/CLOSEAPPLICATIONS"))) // 强制关闭应用
                    .chain(once(OsStr::new("/RESTARTAPPLICATIONS"))) // 重启后恢复
                    .chain(once(OsStr::new("/ARGS")))
                    .chain(nsis_args.iter().map(OsStr::new))
                    .chain(self.installer_args())
                    .collect()
            }
            WindowsUpdaterType::Msi { path, .. } => {
                let escaped_args = current_args
                    .iter()
                    .map(escape_msi_property_arg)
                    .collect::<Vec<_>>()
                    .join(" ");
                msi_args = OsString::from(format!("LAUNCHAPPARGS=\"{escaped_args}\""));

                [OsStr::new("/i"), path.as_os_str()]
                    .into_iter()
                    .chain(install_mode.msiexec_args().iter().map(OsStr::new))
                    .chain(once(OsStr::new("/forcerestart"))) // 强制重启
                    .chain(self.installer_args())
                    .chain(once(OsStr::new("AUTOLAUNCHAPP=True")))
                    .chain(once(msi_args.as_os_str()))
                    .collect()
            }
        };

        let file = match &updater_type {
            WindowsUpdaterType::Nsis { path, .. } => path.as_os_str().to_os_string(),
            WindowsUpdaterType::Msi { .. } => std::env::var("SYSTEMROOT").as_ref().map_or_else(
                |_| OsString::from("msiexec.exe"),
                |p| OsString::from(format!("{p}\\System32\\msiexec.exe")),
            ),
        };
        let file = encode_wide(file);

        // 正确处理包含空格的参数，避免参数解析错误
        let parameters = installer_args
            .iter()
            .map(|arg| {
                let arg_str = arg.to_string_lossy();
                if arg_str.contains(' ') && !arg_str.starts_with('"') {
                    format!("\"{}\"", arg_str)
                } else {
                    arg_str.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let parameters = encode_wide(parameters);

        // 使用 runas 动词请求管理员权限
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                w!("runas"), // 请求管理员权限
                file.as_ptr(),
                parameters.as_ptr(),
                std::ptr::null(),
                SW_SHOW,
            )
        };

        // 检查启动结果并返回具体错误
        if result as i32 <= 32 {
            return match result as i32 {
                5 => Err(Error::InsufficientPrivileges), // ERROR_ACCESS_DENIED
                1223 => Err(Error::UserCancelledElevation), // ERROR_CANCELLED (用户取消UAC)
                32 => Err(Error::FileInUse), // ERROR_SHARING_VIOLATION
                2 => Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Installer file not found"
                ))),
                _ => Err(Error::InstallerExecutionFailed(result as i32)),
            };
        }

        // 延迟退出，给安装程序更多时间启动
        std::thread::sleep(std::time::Duration::from_millis(2000));
        std::process::exit(0);
    }

    fn installer_args(&self) -> Vec<&OsStr> {
        self.installer_args
            .iter()
            .map(OsStr::new)
            .collect::<Vec<_>>()
    }

    fn current_exe_args(&self) -> Vec<&OsStr> {
        self.current_exe_args
            .iter()
            .map(OsStr::new)
            .collect::<Vec<_>>()
    }

    fn extract(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        
        if infer::archive::is_zip(bytes) {
            return self.extract_zip(bytes);
        }

        self.extract_exe(bytes)
    }

    fn make_temp_dir(&self) -> Result<PathBuf> {
        Ok(tempfile::Builder::new()
            .prefix(&format!("{}-{}-updater-", self.app_name, self.version))
            .tempdir()?
            .keep())
    }

    fn extract_zip(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        let temp_dir = self.make_temp_dir()?;

        let archive = Cursor::new(bytes);
        let mut extractor = zip::ZipArchive::new(archive)?;
        extractor.extract(&temp_dir)?;

        let paths = std::fs::read_dir(&temp_dir)?;
        for path in paths {
            let path = path?.path();
            let ext = path.extension();
            if ext == Some(OsStr::new("exe")) {
                return Ok(WindowsUpdaterType::nsis(path, None));
            } else if ext == Some(OsStr::new("msi")) {
                return Ok(WindowsUpdaterType::msi(path, None));
            }
        }

        Err(crate::Error::BinaryNotFoundInArchive)
    }

    fn extract_exe(&self, bytes: &[u8]) -> Result<WindowsUpdaterType> {
        if infer::app::is_exe(bytes) {
            let (path, temp) = self.write_to_temp(bytes, ".exe")?;
            Ok(WindowsUpdaterType::nsis(path, temp))
        } else if infer::archive::is_msi(bytes) {
            let (path, temp) = self.write_to_temp(bytes, ".msi")?;
            Ok(WindowsUpdaterType::msi(path, temp))
        } else {
            Err(crate::Error::InvalidUpdaterFormat)
        }
    }

    fn write_to_temp(
        &self,
        bytes: &[u8],
        ext: &str,
    ) -> Result<(PathBuf, Option<tempfile::TempPath>)> {
        use std::io::Write;

        let temp_dir = self.make_temp_dir()?;
        let mut temp_file = tempfile::Builder::new()
            .prefix(&format!("{}-{}-installer", self.app_name, self.version))
            .suffix(ext)
            .rand_bytes(0)
            .tempfile_in(temp_dir)?;
        temp_file.write_all(bytes)?;

        let temp = temp_file.into_temp_path();
        Ok((temp.to_path_buf(), Some(temp)))
    }
}

/// Linux (AppImage and Deb)
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
impl Update {
    /// ### Expected structure:
    /// ├── [AppName]_[version]_amd64.AppImage.tar.gz    # GZ generated by tauri-bundler
    /// │   └──[AppName]_[version]_amd64.AppImage        # Application AppImage
    /// ├── [AppName]_[version]_amd64.deb                # Debian package (amd64)
    /// ├── [AppName]_[version]_arm64.deb                # Debian package (arm64)
    /// └── ...
    ///
    fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        if self.is_deb_package() {
            self.install_deb(bytes)
        } else {
            // Handle AppImage or other formats
            self.install_appimage(bytes)
        }
    }

    fn install_appimage(&self, bytes: &[u8]) -> Result<()> {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let extract_path_metadata = self.extract_path.metadata()?;

        let tmp_dir_locations = vec![
            Box::new(|| Some(std::env::temp_dir())) as Box<dyn FnOnce() -> Option<PathBuf>>,
            Box::new(dirs::cache_dir),
            Box::new(|| Some(self.extract_path.parent().unwrap().to_path_buf())),
        ];

        for tmp_dir_location in tmp_dir_locations {
            if let Some(tmp_dir_location) = tmp_dir_location() {
                let tmp_dir = tempfile::Builder::new()
                    .prefix("tauri_current_app")
                    .tempdir_in(tmp_dir_location)?;
                let tmp_dir_metadata = tmp_dir.path().metadata()?;

                if extract_path_metadata.dev() == tmp_dir_metadata.dev() {
                    let mut perms = tmp_dir_metadata.permissions();
                    perms.set_mode(0o700);
                    std::fs::set_permissions(tmp_dir.path(), perms)?;

                    let tmp_app_image = &tmp_dir.path().join("current_app.AppImage");

                    let permissions = std::fs::metadata(&self.extract_path)?.permissions();

                    // create a backup of our current app image
                    std::fs::rename(&self.extract_path, tmp_app_image)?;

                    #[cfg(feature = "zip")]
                    if infer::archive::is_gz(bytes) {
                        log::debug!("extracting AppImage");
                        // extract the buffer to the tmp_dir
                        // we extract our signed archive into our final directory without any temp file
                        let archive = Cursor::new(bytes);
                        let decoder = flate2::read::GzDecoder::new(archive);
                        let mut archive = tar::Archive::new(decoder);
                        for mut entry in archive.entries()?.flatten() {
                            if let Ok(path) = entry.path() {
                                if path.extension() == Some(OsStr::new("AppImage")) {
                                    // if something went wrong during the extraction, we should restore previous app
                                    if let Err(err) = entry.unpack(&self.extract_path) {
                                        std::fs::rename(tmp_app_image, &self.extract_path)?;
                                        return Err(err.into());
                                    }
                                    // early finish we have everything we need here
                                    return Ok(());
                                }
                            }
                        }
                        // if we have not returned early we should restore the backup
                        std::fs::rename(tmp_app_image, &self.extract_path)?;
                        return Err(Error::BinaryNotFoundInArchive);
                    }

                    log::debug!("rewriting AppImage");
                    return match std::fs::write(&self.extract_path, bytes)
                        .and_then(|_| std::fs::set_permissions(&self.extract_path, permissions))
                    {
                        Err(err) => {
                            // if something went wrong during the extraction, we should restore previous app
                            std::fs::rename(tmp_app_image, &self.extract_path)?;
                            Err(err.into())
                        }
                        Ok(_) => Ok(()),
                    };
                }
            }
        }

        Err(Error::TempDirNotOnSameMountPoint)
    }

    fn is_deb_package(&self) -> bool {
        // First check if we're in a typical Debian installation path
        let in_system_path = self
            .extract_path
            .to_str()
            .map(|p| p.starts_with("/usr"))
            .unwrap_or(false);

        if !in_system_path {
            return false;
        }

        // Then verify it's actually a Debian-based system by checking for dpkg
        let dpkg_exists = std::path::Path::new("/var/lib/dpkg").exists();
        let apt_exists = std::path::Path::new("/etc/apt").exists();

        // Additional check for the package in dpkg database
        let package_in_dpkg = if let Ok(output) = std::process::Command::new("dpkg")
            .args(["-S", &self.extract_path.to_string_lossy()])
            .output()
        {
            output.status.success()
        } else {
            false
        };

        // Consider it a deb package only if:
        // 1. We're in a system path AND
        // 2. We have Debian package management tools AND
        // 3. The binary is tracked by dpkg
        dpkg_exists && apt_exists && package_in_dpkg
    }

    fn install_deb(&self, bytes: &[u8]) -> Result<()> {
        // First verify the bytes are actually a .deb package
        if !infer::archive::is_deb(bytes) {
            log::warn!("update is not a valid deb package");
            return Err(Error::InvalidUpdaterFormat);
        }

        // Try different temp directories
        let tmp_dir_locations = vec![
            Box::new(|| Some(std::env::temp_dir())) as Box<dyn FnOnce() -> Option<PathBuf>>,
            Box::new(dirs::cache_dir),
            Box::new(|| Some(self.extract_path.parent().unwrap().to_path_buf())),
        ];

        // Try writing to multiple temp locations until one succeeds
        for tmp_dir_location in tmp_dir_locations {
            if let Some(path) = tmp_dir_location() {
                if let Ok(tmp_dir) = tempfile::Builder::new()
                    .prefix("tauri_deb_update")
                    .tempdir_in(path)
                {
                    let deb_path = tmp_dir.path().join("package.deb");

                    // Try writing the .deb file
                    if std::fs::write(&deb_path, bytes).is_ok() {
                        // If write succeeds, proceed with installation
                        return self.try_install_with_privileges(&deb_path);
                    }
                    // If write fails, continue to next temp location
                }
            }
        }

        // If we get here, all temp locations failed
        Err(Error::TempDirNotFound)
    }

    fn try_install_with_privileges(&self, deb_path: &Path) -> Result<()> {
        // 1. First try using pkexec (graphical sudo prompt)
        if let Ok(status) = std::process::Command::new("pkexec")
            .arg("dpkg")
            .arg("-i")
            .arg(deb_path)
            .status()
        {
            if status.success() {
                log::debug!("installed deb with pkexec");
                return Ok(());
            }
        }

        // 2. Try zenity or kdialog for a graphical sudo experience
        if let Ok(password) = self.get_password_graphically() {
            if self.install_with_sudo(deb_path, &password)? {
                log::debug!("installed deb with GUI sudo");
                return Ok(());
            }
        }

        // 3. Final fallback: terminal sudo
        let status = std::process::Command::new("sudo")
            .arg("dpkg")
            .arg("-i")
            .arg(deb_path)
            .status()?;

        if status.success() {
            log::debug!("installed deb with sudo");
            Ok(())
        } else {
            Err(Error::DebInstallFailed)
        }
    }

    fn get_password_graphically(&self) -> Result<String> {
        // Try zenity first
        let zenity_result = std::process::Command::new("zenity")
            .args([
                "--password",
                "--title=Authentication Required",
                "--text=Enter your password to install the update:",
            ])
            .output();

        if let Ok(output) = zenity_result {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        // Fall back to kdialog if zenity fails or isn't available
        let kdialog_result = std::process::Command::new("kdialog")
            .args(["--password", "Enter your password to install the update:"])
            .output();

        if let Ok(output) = kdialog_result {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        Err(Error::AuthenticationFailed)
    }

    fn install_with_sudo(&self, deb_path: &Path, password: &str) -> Result<bool> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("sudo")
            .arg("-S") // read password from stdin
            .arg("dpkg")
            .arg("-i")
            .arg(deb_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            // Write password to stdin
            writeln!(stdin, "{}", password)?;
        }

        let status = child.wait()?;
        Ok(status.success())
    }
}

/// MacOS
#[cfg(target_os = "macos")]
impl Update {
    /// Detect if the bytes represent a ZIP file
    fn is_zip_file(bytes: &[u8]) -> bool {
        bytes.len() >= 4 && &bytes[0..4] == b"PK\x03\x04"
    }

    /// Extract ZIP file for macOS .app bundles
    fn extract_zip(&self, bytes: &[u8]) -> Result<Vec<PathBuf>> {
        use std::os::unix::fs::PermissionsExt;
        use zip::ZipArchive;

        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;
        let mut extracted_files = Vec::new();

        // Create temp directory for extraction
        let tmp_extract_dir = tempfile::Builder::new()
            .prefix("tauri_updated_app")
            .tempdir()?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => tmp_extract_dir.path().join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                // Directory
                std::fs::create_dir_all(&outpath)?;
                // Set directory permissions if available
                if let Some(mode) = file.unix_mode() {
                    let permissions = std::fs::Permissions::from_mode(mode);
                    std::fs::set_permissions(&outpath, permissions)?;
                }
            } else {
                // File
                if let Some(p) = outpath.parent()
                    && !p.exists()
                {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;

                // Set file permissions if available, otherwise use default executable permissions for binaries
                if let Some(mode) = file.unix_mode() {
                    let permissions = std::fs::Permissions::from_mode(mode);
                    std::fs::set_permissions(&outpath, permissions)?;
                } else {
                    // If no permissions in ZIP, check if this is likely an executable file
                    let file_name = outpath.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let path_str = outpath.to_string_lossy();

                    // Check if this is a binary executable in Contents/MacOS/
                    if path_str.contains("Contents/MacOS/")
                        || (!file_name.contains('.') && !path_str.contains("Contents/Resources/"))
                    {
                        // Set executable permissions (0o755 = rwxr-xr-x)
                        let permissions = std::fs::Permissions::from_mode(0o755);
                        std::fs::set_permissions(&outpath, permissions)?;
                        println!(
                            "[DEBUG] Set executable permissions for: {}",
                            outpath.display()
                        );
                    }
                }
            }
            extracted_files.push(outpath);
        }

        // For .app.zip files, we need to find the .app bundle and move it to the correct location
        let app_bundle = extracted_files
            .iter()
            .find(|path| path.extension().and_then(|s| s.to_str()) == Some("app"))
            .cloned();

        if let Some(app_path) = app_bundle {
            // Move the .app bundle to the target location
            self.move_app_bundle(&app_path)?;
        } else {
            // If no .app bundle found, try to move the entire extracted directory
            self.move_extracted_files(tmp_extract_dir.path())?;
        }

        Ok(extracted_files)
    }

    /// Move .app bundle to target location
    fn move_app_bundle(&self, app_path: &Path) -> Result<()> {
        // Create temp directory for backup
        let tmp_backup_dir = tempfile::Builder::new()
            .prefix("tauri_current_app")
            .tempdir()?;

        // Try to move the current app to backup
        let move_result = std::fs::rename(
            &self.extract_path,
            tmp_backup_dir.path().join("current_app"),
        );

        let need_authorization = if let Err(err) = move_result {
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                true
            } else {
                return Err(err.into());
            }
        } else {
            false
        };

        if need_authorization {
            // Use AppleScript to perform atomic move with admin privileges
            // First create a backup, then move new app, then remove backup
            let backup_path = format!("{}.backup", self.extract_path.display());
            let apple_script = format!(
                "do shell script \"mv '{src}' '{backup}' && mv '{new}' '{src}' && rm -rf '{backup}'\" with administrator privileges",
                src = self.extract_path.display(),
                new = app_path.display(),
                backup = backup_path
            );

            let mut script =
                osakit::Script::new_from_source(osakit::Language::AppleScript, &apple_script);
            script.compile().expect("invalid AppleScript");
            let result = script.execute();

            if result.is_err() {
                // Try to restore from backup if it exists
                let restore_script = format!(
                    "do shell script \"if [ -d '{backup}' ]; then rm -rf '{src}' && mv '{backup}' '{src}'; fi\" with administrator privileges",
                    src = self.extract_path.display(),
                    backup = backup_path
                );
                let mut restore_script =
                    osakit::Script::new_from_source(osakit::Language::AppleScript, &restore_script);
                restore_script.compile().expect("invalid AppleScript");
                let _ = restore_script.execute(); // Best effort restore

                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Failed to move the new app into place",
                )));
            }
        } else {
            // Atomic move: backup current app, move new app, remove backup
            let backup_path = self.extract_path.with_extension("backup");

            // Step 1: Move current app to backup (if it exists)
            if self.extract_path.exists() {
                std::fs::rename(&self.extract_path, &backup_path)?;
            }

            // Step 2: Move new app to target location
            let move_result = std::fs::rename(app_path, &self.extract_path);

            if let Err(err) = move_result {
                // If move failed, try to restore from backup
                if backup_path.exists() {
                    let _ = std::fs::rename(&backup_path, &self.extract_path); // Best effort restore
                }
                return Err(err.into());
            }

            // Step 3: Remove backup if everything succeeded
            if backup_path.exists() {
                let _ = std::fs::remove_dir_all(&backup_path); // Best effort cleanup
            }
        }

        Ok(())
    }

    /// Move extracted files to target location (fallback for non-.app bundles)
    fn move_extracted_files(&self, extract_dir: &Path) -> Result<()> {
        // Create temp directory for backup
        let tmp_backup_dir = tempfile::Builder::new()
            .prefix("tauri_current_app")
            .tempdir()?;

        // Try to move the current app to backup
        let move_result = std::fs::rename(
            &self.extract_path,
            tmp_backup_dir.path().join("current_app"),
        );

        let need_authorization = if let Err(err) = move_result {
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                true
            } else {
                return Err(err.into());
            }
        } else {
            false
        };

        if need_authorization {
            // Use AppleScript to perform moves with admin privileges
            let apple_script = format!(
                "do shell script \"rm -rf '{src}' && mv -f '{new}' '{src}'\" with administrator privileges",
                src = self.extract_path.display(),
                new = extract_dir.display()
            );

            let mut script =
                osakit::Script::new_from_source(osakit::Language::AppleScript, &apple_script);
            script.compile().expect("invalid AppleScript");
            let result = script.execute();

            if result.is_err() {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Failed to move the new app into place",
                )));
            }
        } else {
            // Remove existing directory if it exists
            if self.extract_path.exists() {
                std::fs::remove_dir_all(&self.extract_path)?;
            }
            // Move the new app to the target path
            std::fs::rename(extract_dir, &self.extract_path)?;
        }

        Ok(())
    }
    /// ### Expected structure:
    /// ├── [AppName]-[version]-macos-x86_64.app.tar.gz     # TAR.GZ (x86_64)
    /// ├── [AppName]-[version]-macos-arm64.app.tar.gz      # TAR.GZ (arm64)
    /// ├── [AppName]-[version]-macos-x86_64.dmg            # DMG (x86_64)
    /// ├── [AppName]-[version]-macos-arm64.dmg             # DMG (arm64)
    /// │   └──[AppName].app                         # Main application
    /// │      └── Contents                          # Application contents...
    /// │          └── ...
    /// └── ...
    fn install_inner(&self, bytes: &[u8]) -> Result<()> {
        // Detect file type and handle accordingly
        if Self::is_zip_file(bytes) {
            // Handle ZIP files (typically .app.zip bundles)
            self.extract_zip(bytes)?;
        } else {
            // Handle TAR.GZ files (traditional format)
            self.extract_tar_gz(bytes)?;
        }

        // Touch the app to update modification time
        let _ = std::process::Command::new("touch")
            .arg(&self.extract_path)
            .status();

        // Restart the application after successful update
        println!(
            "[DEBUG] Restarting application: {}",
            self.extract_path.display()
        );

        // Use 'open' command to launch the updated app in the background
        // The -n flag opens a new instance even if one is already running
        // The -a flag specifies the application to open
        let restart_result = std::process::Command::new("open")
            .arg("-n")
            .arg(&self.extract_path)
            .spawn();

        match restart_result {
            Ok(_) => {
                println!("[DEBUG] Successfully launched updated application");
            }
            Err(e) => {
                eprintln!(
                    "[WARNING] Failed to restart application automatically: {}. Please restart manually.",
                    e
                );
                // Don't return error here as the update itself was successful
            }
        }

        Ok(())
    }

    /// Extract TAR.GZ file (traditional format)
    fn extract_tar_gz(&self, bytes: &[u8]) -> Result<()> {
        use flate2::read::GzDecoder;

        let cursor = Cursor::new(bytes);
        let mut extracted_files: Vec<PathBuf> = Vec::new();

        // Create temp directories for backup and extraction
        let tmp_backup_dir = tempfile::Builder::new()
            .prefix("tauri_current_app")
            .tempdir()?;

        let tmp_extract_dir = tempfile::Builder::new()
            .prefix("tauri_updated_app")
            .tempdir()?;

        let decoder = GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(decoder);

        // Extract files to temporary directory
        for entry in archive.entries()? {
            let mut entry = entry?;
            let collected_path: PathBuf = entry.path()?.iter().skip(1).collect();
            let extraction_path = tmp_extract_dir.path().join(&collected_path);

            // Ensure parent directories exist
            if let Some(parent) = extraction_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            if let Err(err) = entry.unpack(&extraction_path) {
                // Cleanup on error
                std::fs::remove_dir_all(tmp_extract_dir.path()).ok();
                return Err(err.into());
            }
            extracted_files.push(extraction_path);
        }

        // Try to move the current app to backup
        let move_result = std::fs::rename(
            &self.extract_path,
            tmp_backup_dir.path().join("current_app"),
        );
        let need_authorization = if let Err(err) = move_result {
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                true
            } else {
                std::fs::remove_dir_all(tmp_extract_dir.path()).ok();
                return Err(err.into());
            }
        } else {
            false
        };

        if need_authorization {
            // Use AppleScript to perform moves with admin privileges
            let apple_script = format!(
                "do shell script \"rm -rf '{src}' && mv -f '{new}' '{src}'\" with administrator privileges",
                src = self.extract_path.display(),
                new = tmp_extract_dir.path().display()
            );

            let mut script =
                osakit::Script::new_from_source(osakit::Language::AppleScript, &apple_script);
            script.compile().expect("invalid AppleScript");
            let result = script.execute();

            if result.is_err() {
                std::fs::remove_dir_all(tmp_extract_dir.path()).ok();
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Failed to move the new app into place",
                )));
            }
        } else {
            // Remove existing directory if it exists
            if self.extract_path.exists() {
                std::fs::remove_dir_all(&self.extract_path)?;
            }
            // Move the new app to the target path
            std::fs::rename(tmp_extract_dir.path(), &self.extract_path)?;
        }

        Ok(())
    }
}

/// Gets the target string used on the updater.
pub fn target() -> Option<String> {
    if let (Some(target), Some(arch)) = (get_updater_target(), get_updater_arch()) {
        Some(format!("{target}-{arch}"))
    } else {
        None
    }
}

pub(crate) fn get_updater_target() -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "macos") {
        Some("macos")
    } else if cfg!(target_os = "windows") {
        Some("windows")
    } else {
        None
    }
}

pub(crate) fn get_updater_arch() -> Option<&'static str> {
    if cfg!(target_arch = "x86") {
        Some("i686")
    } else if cfg!(target_arch = "x86_64") {
        Some("x86_64")
    } else if cfg!(target_arch = "arm") {
        Some("armv7")
    } else if cfg!(target_arch = "aarch64") {
        Some("arm64")
    } else if cfg!(target_arch = "riscv64") {
        Some("riscv64")
    } else {
        None
    }
}

pub fn extract_path_from_executable(executable_path: &Path) -> Result<PathBuf> {
    // Return the path of the current executable by default
    // Example C:\Program Files\My App\
    let extract_path = executable_path
        .parent()
        .map(PathBuf::from)
        .ok_or(Error::FailedToDetermineExtractPath)?;

    // MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
    // We need to get /Applications/<app>.app
    // TODO(lemarier): Need a better way here
    // Maybe we could search for <*.app> to get the right path
    #[cfg(target_os = "macos")]
    if extract_path
        .display()
        .to_string()
        .contains("Contents/MacOS")
    {
        return extract_path
            .parent()
            .map(PathBuf::from)
            .ok_or(Error::FailedToDetermineExtractPath)?
            .parent()
            .map(PathBuf::from)
            .ok_or(Error::FailedToDetermineExtractPath);
    }

    Ok(extract_path)
}

fn parse_version<'de, D>(deserializer: D) -> std::result::Result<Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;

    Version::from_str(str.trim_start_matches('v')).map_err(serde::de::Error::custom)
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
