[unix]
set shell := ["bash", "-cu"]
[windows]
set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

host_arch := if os() == "windows" { if arch() == "aarch64" { "ARM64" } else { "x64" } } else if arch() == "aarch64" { "arm64" } else { "x86_64" }

# List available commands.
default:
    @just --list

# Build through the Xcode project without distribution signing or launch.
[macos]
build architecture=host_arch profile="release":
    ARCH={{ quote(architecture) }} CONFIGURATION={{ quote(lowercase(profile)) }} BUILD_ONLY=1 bash macos/build.sh

# Compile the Windows app without launching it.
[windows]
build architecture=host_arch profile="release":
    dotnet build windows/BibCiTeX.sln -c '{{ replace(profile, "'", "''") }}' '-p:Platform={{ replace(architecture, "'", "''") }}'

# Compile the GPUI Kit Linux client without launching or packaging it.
[linux]
build profile="release":
    cargo build --locked -p bibcitex-linux --profile {{ if profile == "debug" { "dev" } else { quote(profile) } }}

# Create the macOS .app bundle; never launch it.
[macos]
bundle architecture=host_arch profile="release":
    ARCH={{ quote(architecture) }} CONFIGURATION={{ quote(lowercase(profile)) }} BUILD_ONLY=0 bash macos/build.sh

# Build Linux portable and Debian packages without launching the app.
[linux]
bundle architecture=host_arch:
    cargo build --locked --release -p bibcitex-linux
    cargo run --locked -p xtask -- package-linux target/release/bibcitex target/linux-packages {{ quote(architecture) }}

# Check Rust formatting, lint and tests.
check-rust: fmt clippy test-rust

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --locked --workspace --all-targets --all-features -- -D warnings

test-rust:
    cargo test --locked --workspace --all-targets --all-features

# Test release version ordering and update channel eligibility without a GUI.
[macos]
test-updates:
    bash macos/BibCiTeX/Tests/run-update-tests.sh

# Run update policy and Swift/Rust integration tests after a debug build.
[macos]
test-macos architecture=host_arch: test-updates
    bash macos/BibCiTeX/Tests/run-interop-tests.sh --products {{ quote(env("DERIVED_DATA_PATH", "target/xcode/" + architecture) + "/Build/Products/Debug") }}

# Run the C#/Rust integration tests using a built platform library.
[windows]
test-windows library:
    dotnet run --project windows/Tests/BibCiTeX.IntegrationTests.csproj -c Debug -- '{{ replace(library, "'", "''") }}'

# Test update metadata and publication rules without publishing.
test-ci:
    cargo test --locked -p xtask

# Release workflow helpers; branch builds do not invoke these.
ci-release-metadata:
    cargo run --locked -p xtask -- release-metadata

[macos]
ci-release-macos:
    bash ci/release-macos.sh

[windows]
ci-release-windows:
    & ./ci/release-windows.ps1

ci-publish-release:
    cargo run --locked -p xtask -- publish-release

ci-publish-cos:
    cargo run --locked -p xtask -- publish-cos
