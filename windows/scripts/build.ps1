param(
    [ValidateSet('x64', 'ARM64')][string]$Architecture = 'x64',
    [ValidateSet('Debug', 'Release')][string]$Configuration = 'Debug'
)
$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
# Visual Studio and this CLI entry share the same MSBuild-owned Rust/binding pipeline.
& dotnet build (Join-Path $Repo 'windows/BibCiTeX.sln') -c $Configuration "-p:Platform=$Architecture"
if ($LASTEXITCODE -ne 0) { throw 'BibCiTeX solution build failed.' }
