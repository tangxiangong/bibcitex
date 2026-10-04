param(
    [Parameter(Mandatory)][ValidateSet('x64', 'ARM64')][string]$Architecture,
    [Parameter(Mandatory)][version]$Version,
    [Parameter(Mandatory)][string]$Publisher,
    [Parameter(Mandatory)][uri]$AppInstallerUri,
    [Parameter(Mandatory)][string]$CertificateThumbprint,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [uri]$TimestampServer = $env:WINDOWS_TIMESTAMP_URL
)
$ErrorActionPreference = 'Stop'
if ($AppInstallerUri.Scheme -ne 'https') { throw 'AppInstallerUri must use HTTPS.' }
if (-not $TimestampServer -or $TimestampServer.Scheme -notin @('http', 'https')) { throw 'A timestamp server is required.' }
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$Compiler = Get-Command makensis.exe -ErrorAction SilentlyContinue
if (-not $Compiler) {
    $Nsis = Join-Path ${env:ProgramFiles(x86)} 'NSIS/makensis.exe'
    if (-not (Test-Path $Nsis)) { throw 'NSIS must be installed to produce legacy updater migration installers.' }
} else { $Nsis = $Compiler.Source }
$Sdk = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
$SignTool = Get-ChildItem $Sdk -Filter signtool.exe -Recurse | Where-Object { $_.Directory.Name -eq 'x64' } | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $SignTool) { throw 'Windows SDK signtool.exe is required.' }
$Work = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $Work | Out-Null
try {
    $Config = Join-Path $Work 'configuration.json'
    @{ name = 'BibCiTeX'; publisher = $Publisher; version = $Version.ToString(); appInstallerUri = $AppInstallerUri.AbsoluteUri } | ConvertTo-Json | Set-Content -LiteralPath $Config -Encoding UTF8
    $Output = Join-Path ([IO.Path]::GetFullPath($OutputDirectory)) "BibCiTeX_$($Architecture.ToLowerInvariant())-setup.exe"
    $Installer = Join-Path ([IO.Path]::GetFullPath($OutputDirectory)) "BibCiTeX-$($Architecture.ToLowerInvariant()).appinstaller"
    if (-not (Test-Path -LiteralPath $Installer)) { throw 'Build the signed MSIX and App Installer metadata first.' }
    & $Nsis "/DOUTPUT=$Output" "/DSCRIPT=$(Join-Path $Repo 'windows/Installer/migrate.ps1')" "/DCONFIGURATION=$Config" "/DAPPINSTALLER=$Installer" (Join-Path $Repo 'windows/Installer/migration.nsi')
    if ($LASTEXITCODE -ne 0) { throw 'Migration installer compilation failed.' }
    & $SignTool.FullName sign /sha1 $CertificateThumbprint /fd SHA256 /tr $TimestampServer.AbsoluteUri /td SHA256 $Output
    if ($LASTEXITCODE -ne 0) { throw 'Migration installer Authenticode signing failed.' }
    if ((Get-AuthenticodeSignature $Output).Status -ne 'Valid') { throw 'Migration installer signature verification failed.' }
} finally { Remove-Item -LiteralPath $Work -Recurse -Force }
