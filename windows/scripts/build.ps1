# User-invoked only. This script builds; the coding agent must not run it.
param(
    [ValidateSet('x64', 'ARM64')][string]$Architecture = 'x64',
    [ValidateSet('Debug', 'Release')][string]$Configuration = 'Debug'
)
$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$Target = if ($Architecture -eq 'ARM64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$Profile = if ($Configuration -eq 'Release') { 'release' } else { 'debug' }
Push-Location $Repo
try {
    $CargoArgs = @('build', '--locked', '-p', 'bibcitex-csharp', '--target', $Target)
    if ($Configuration -eq 'Release') { $CargoArgs += '--release' }
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed.' }
    & (Join-Path $Repo 'bindings/generate-csharp.ps1') -Library (Join-Path $Repo "target/$Target/$Profile/bibcitex_csharp.dll")
    & dotnet build (Join-Path $Repo 'windows/BibCiTeX/BibCiTeX.csproj') -c $Configuration "-p:Platform=$Architecture" "-p:RuntimeIdentifier=win-$($Architecture.ToLowerInvariant())"
    if ($LASTEXITCODE -ne 0) { throw 'WinUI build failed.' }
} finally { Pop-Location }
