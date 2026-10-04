# User-invoked only. Requires Visual Studio Build Tools + Windows SDK, Rust, .NET 10.
param(
    [Parameter(Mandatory)][ValidateSet('x64', 'ARM64')][string]$Architecture,
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+\.\d+$')][string]$Version,
    [Parameter(Mandatory)][string]$Publisher,
    [Parameter(Mandatory)][string]$CertificateThumbprint,
    [Parameter(Mandatory)][uri]$PublishBaseUri,
    [uri]$PayloadBaseUri = $PublishBaseUri,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [uri]$TimestampServer = $env:WINDOWS_TIMESTAMP_URL
)
$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
if ($PublishBaseUri.Scheme -ne 'https') { throw 'PublishBaseUri must use HTTPS.' }
if ($PayloadBaseUri.Scheme -ne 'https') { throw 'PayloadBaseUri must use HTTPS.' }
if (-not $TimestampServer -or $TimestampServer.Scheme -notin @('http', 'https')) { throw 'Set TimestampServer or WINDOWS_TIMESTAMP_URL to the certificate authority timestamp service.' }
$Certificate = Get-Item "Cert:/CurrentUser/My/$CertificateThumbprint"
if (-not $Certificate.HasPrivateKey -or $Certificate.Subject -ne $Publisher) { throw 'The signing certificate must have a private key and its subject must exactly equal Publisher.' }
if ($Certificate.NotAfter -le (Get-Date)) { throw 'The signing certificate is expired.' }
$OutputDirectory = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
$ManifestPath = Join-Path $Repo 'windows/BibCiTeX/Package.appxmanifest'
$OriginalManifest = [System.IO.File]::ReadAllText($ManifestPath)
try {
    [xml]$Manifest = $OriginalManifest
    $Manifest.Package.Identity.SetAttribute('Publisher', $Publisher)
    $Manifest.Package.Identity.SetAttribute('Version', $Version)
    $Manifest.Save($ManifestPath)
    $Project = Join-Path $Repo 'windows/BibCiTeX.sln'
    & dotnet build $Project -c Release "-p:Platform=$Architecture" "-p:RuntimeIdentifier=win-$($Architecture.ToLowerInvariant())" '-p:GenerateAppxPackageOnBuild=true' '-p:AppxBundle=Never' '-p:UapAppxPackageBuildMode=SideloadOnly' '-p:AppxPackageSigningEnabled=true' "-p:PackageCertificateThumbprint=$CertificateThumbprint" "-p:AppxPackageSigningTimestampServerUrl=$($TimestampServer.AbsoluteUri)" "-p:AppxPackageDir=$OutputDirectory/"
    if ($LASTEXITCODE -ne 0) { throw 'MSIX packaging failed.' }
    $Packages = @(Get-ChildItem $OutputDirectory -Recurse -Filter '*.msix' | Where-Object { $_.Name -match [regex]::Escape($Version) -and $_.Name -notmatch 'Dependencies' })
    if ($Packages.Count -ne 1) { throw 'Expected exactly one versioned MSIX in OutputDirectory. Use an empty directory for each release.' }
    $Package = $Packages[0]
    # Stable release names let the shared validator bind each feed to its
    # architecture without depending on MSBuild's internal output naming.
    $PackagePath = Join-Path $OutputDirectory "BibCiTeX-$Version-$($Architecture.ToLowerInvariant()).msix"
    if ($Package.FullName -ne $PackagePath) {
        Copy-Item -LiteralPath $Package.FullName -Destination $PackagePath
        if ($Package.DirectoryName -eq $OutputDirectory) { Remove-Item -LiteralPath $Package.FullName }
    }
    $Package = Get-Item -LiteralPath $PackagePath
    $AppInstallerName = "BibCiTeX-$($Architecture.ToLowerInvariant()).appinstaller"
    $BaseUri = $PublishBaseUri.AbsoluteUri.TrimEnd('/') + '/'
    $Xml = [xml]'<?xml version="1.0" encoding="utf-8"?><AppInstaller xmlns="http://schemas.microsoft.com/appx/appinstaller/2018" Version="0.0.0.0" Uri=""><MainPackage Name="BibCiTeX" Publisher="" Version="0.0.0.0" ProcessorArchitecture="" Uri="" /><UpdateSettings><OnLaunch HoursBetweenUpdateChecks="0" ShowPrompt="true" UpdateBlocksActivation="false" /><AutomaticBackgroundTask /></UpdateSettings></AppInstaller>'
    $Xml.AppInstaller.SetAttribute('Version', $Version)
    $Xml.AppInstaller.SetAttribute('Uri', $BaseUri + $AppInstallerName)
    $Xml.AppInstaller.MainPackage.SetAttribute('Publisher', $Publisher)
    $Xml.AppInstaller.MainPackage.SetAttribute('Version', $Version)
    $Xml.AppInstaller.MainPackage.SetAttribute('ProcessorArchitecture', $Architecture.ToLowerInvariant())
    $Xml.AppInstaller.MainPackage.SetAttribute('Uri', $PayloadBaseUri.AbsoluteUri.TrimEnd('/') + '/' + [uri]::EscapeDataString($Package.Name))
    $Xml.Save((Join-Path $OutputDirectory $AppInstallerName))
    Write-Output "Publish the MSIX and $AppInstallerName at $BaseUri. Install via the .appinstaller to enable native updates."
} finally { [System.IO.File]::WriteAllText($ManifestPath, $OriginalManifest) }
