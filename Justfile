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

# Create the macOS .app bundle; never launch it.
[macos]
bundle architecture=host_arch profile="release":
    ARCH={{ quote(architecture) }} CONFIGURATION={{ quote(lowercase(profile)) }} BUILD_ONLY=0 bash macos/build.sh

# Check Rust formatting, lint and tests.
check-rust: fmt clippy test-rust

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --locked --workspace --all-targets --all-features -- -D warnings

test-rust:
    cargo test --locked --workspace --all-targets --all-features

# Test shared translation resources and language fallback without a GUI.
[macos]
test-localization:
    bash macos/BibCiTeX/Tests/run-localization-tests.sh

# Run the helper state tests without launching the app.
[macos]
test-helper:
    bash macos/BibCiTeX/Tests/run-regressions.sh

# Run workbench binding and search state tests without launching the app.
[macos]
test-workbench:
    bash macos/BibCiTeX/Tests/run-workbench-tests.sh

# Test the independent tray browser without launching a GUI.
[macos]
test-tray:
    bash macos/BibCiTeX/Tests/run-tray-tests.sh

# Test release version ordering and update channel eligibility without a GUI.
[macos]
test-updates:
    bash macos/BibCiTeX/Tests/run-update-tests.sh

# Run model and Swift/Rust integration tests after a debug build.
[macos]
test-macos architecture=host_arch: test-localization test-helper test-workbench test-tray test-updates
    bash macos/BibCiTeX/Tests/run-interop-tests.sh --products {{ quote(env("DERIVED_DATA_PATH", "target/xcode/" + architecture) + "/Build/Products/Debug") }}

# Run the C#/Rust and formula tests using a built platform library.
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
