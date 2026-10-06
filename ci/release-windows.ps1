$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
& (Join-Path $Repo 'windows/scripts/publish.ps1') -Architecture $env:ARCH -Version $env:APP_VERSION -Channel $env:RELEASE_CHANNEL -BuildNumber $env:BUILD_NUMBER -OutputDirectory (Join-Path $Repo "dist/release/windows-$($env:ARCH.ToLowerInvariant())")
