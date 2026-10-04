$ErrorActionPreference = 'Stop'
$Required = @('WINDOWS_CERTIFICATE_PFX', 'WINDOWS_CERTIFICATE_PASSWORD', 'WINDOWS_PUBLISHER', 'WINDOWS_TIMESTAMP_URL', 'WINDOWS_VERSION', 'ARCH', 'GITHUB_REPOSITORY', 'RUNNER_TEMP', 'RELEASE_TAG', 'TAURI_SIGNING_PRIVATE_KEY')
foreach ($Name in $Required) {
    if ([string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($Name))) { throw "Missing release configuration: $Name" }
}
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Work = Join-Path $env:RUNNER_TEMP ([guid]::NewGuid().ToString())
New-Item -ItemType Directory $Work | Out-Null
$CertificatePath = Join-Path $Work 'signing.pfx'
$Certificate = $null
try {
    [IO.File]::WriteAllBytes($CertificatePath, [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE_PFX))
    $Password = ConvertTo-SecureString $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
    $Certificate = Import-PfxCertificate -FilePath $CertificatePath -CertStoreLocation Cert:/CurrentUser/My -Password $Password
    if ($Certificate.Count -gt 1) { throw 'The signing PFX must contain exactly one signing certificate.' }
    $Out = Join-Path $Repo "dist/release/windows-$($env:ARCH.ToLowerInvariant())"
    & (Join-Path $Repo 'windows/scripts/publish.ps1') -Architecture $env:ARCH -Version $env:WINDOWS_VERSION -Publisher $env:WINDOWS_PUBLISHER -CertificateThumbprint $Certificate.Thumbprint -PublishBaseUri "https://github.com/$env:GITHUB_REPOSITORY/releases/latest/download/" -PayloadBaseUri "https://github.com/$env:GITHUB_REPOSITORY/releases/download/$env:RELEASE_TAG/" -OutputDirectory $Out
    if ($LASTEXITCODE -ne 0) { throw 'Windows release packaging failed.' }
    $Packages = @(Get-ChildItem $Out -File -Filter '*.msix')
    if ($Packages.Count -ne 1) { throw 'Expected one root MSIX release asset.' }
    if ((Get-AuthenticodeSignature $Packages[0].FullName).Status -ne 'Valid') { throw 'MSIX signature is not trusted or invalid.' }
    & (Join-Path $Repo 'windows/scripts/build-migration.ps1') -Architecture $env:ARCH -Version $env:WINDOWS_VERSION -Publisher $env:WINDOWS_PUBLISHER -CertificateThumbprint $Certificate.Thumbprint -AppInstallerUri "https://github.com/$env:GITHUB_REPOSITORY/releases/latest/download/BibCiTeX-$($env:ARCH.ToLowerInvariant()).appinstaller" -OutputDirectory $Out
    $Migration = Join-Path $Out "BibCiTeX_$($env:ARCH.ToLowerInvariant())-setup.exe"
    & cargo run --locked --manifest-path (Join-Path $Repo 'Cargo.toml') -p xtask -- legacy-sign $Migration
    if ($LASTEXITCODE -ne 0) { throw 'Legacy updater signature generation failed.' }
    & just --justfile (Join-Path $Repo 'Justfile') ci-verify-release windows $Out $env:WINDOWS_VERSION $env:GITHUB_REPOSITORY $env:RELEASE_TAG
    if ($LASTEXITCODE -ne 0) { throw 'App Installer metadata verification failed.' }
} finally {
    if ($null -ne $Certificate) {
        foreach ($Item in @($Certificate)) { Remove-Item "Cert:/CurrentUser/My/$($Item.Thumbprint)" -DeleteKey -ErrorAction SilentlyContinue }
    }
    Remove-Item $Work -Recurse -Force -ErrorAction SilentlyContinue
}
