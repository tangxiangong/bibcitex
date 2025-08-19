/// All errors that can occur while running the updater.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Endpoints are not sent.
    #[error("Updater does not have any endpoints set.")]
    EmptyEndpoints,
    /// IO errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Semver errors.
    #[error(transparent)]
    Semver(#[from] semver::Error),
    /// Serialization errors.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// Could not fetch a valid response from the server.
    #[error("Could not fetch a valid release JSON from the remote")]
    ReleaseNotFound,
    /// Unsupported app architecture.
    #[error(
        "Unsupported application architecture, expected one of `x86`, `x86_64`, `arm` or `aarch64`."
    )]
    UnsupportedArch,
    /// Operating system is not supported.
    #[error("Unsupported OS, expected one of `linux`, `darwin` or `windows`.")]
    UnsupportedOs,
    /// Failed to determine updater package extract path
    #[error("Failed to determine updater package extract path.")]
    FailedToDetermineExtractPath,
    /// `reqwest` crate errors.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// The platform was not found on the updater JSON response.
    #[error("the platform `{0}` was not found on the response `platforms` object")]
    TargetNotFound(String),
    /// Download failed
    #[error("`{0}`")]
    Network(String),
    /// Temp dir is not on same mount mount. This prevents our updater to rename the AppImage to a temp file.
    #[error("temp directory is not on the same mount point as the AppImage")]
    TempDirNotOnSameMountPoint,
    #[error("binary for the current target not found in the archive")]
    BinaryNotFoundInArchive,
    #[error("failed to create temporary directory")]
    TempDirNotFound,
    #[error("Authentication failed or was cancelled")]
    AuthenticationFailed,
    #[error("Failed to install .deb package")]
    DebInstallFailed,
    #[error("invalid updater binary format")]
    InvalidUpdaterFormat,
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    /// The configured updater endpoint must use a secure protocol like `https`

    #[error(transparent)]
    URLParseError(#[from] url::ParseError),
    /// Zip extraction errors.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
}

pub type Result<T> = std::result::Result<T, Error>;
