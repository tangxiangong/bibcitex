# No deployment, registry writes, network requests, or applications are launched.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '../Installer/migrate.ps1')
$RemoveNsis = ${function:Remove-LegacyNsisInstallation}
$ReportMsi = ${function:Report-LegacyMsiInstallation}
$GetNsisDirectory = ${function:Get-LegacyNsisDirectory}
$Work = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
New-Item -ItemType Directory $Work | Out-Null
$script:Deployments = 0
$script:Launches = 0
$script:FailDeployment = $false
$script:InstalledPublisher = 'CN=Test'
$PreviousLocalAppData = $env:LOCALAPPDATA
$script:Removals = 0
function Remove-LegacyNsisInstallation { $script:Removals++ }
function Report-LegacyMsiInstallation { }
function Wait-LegacyApplicationExit { }
function Remove-LegacyNsisShortcuts { param($Directory) }
function Add-AppxPackage {
    param([switch]$AppInstallerFile, [string]$Path, [string]$ErrorAction)
    if (-not $AppInstallerFile -or -not (Test-Path -LiteralPath $Path)) { throw 'Expected a bundled App Installer file.' }
    $script:Deployments++
    if ($script:FailDeployment) { throw 'simulated deployment failure' }
}
function Get-AppxPackage {
    param([string]$Name)
    [pscustomobject]@{ Publisher = $script:InstalledPublisher; Version = '0.6.1.0'; PackageFamilyName = 'BibCiTeX_test' }
}
function Start-Process {
    param([string]$FilePath, [string]$ArgumentList)
    if ($FilePath -ne 'explorer.exe' -or $ArgumentList -ne 'shell:AppsFolder\BibCiTeX_test!App') { throw 'Wrong native launch target.' }
    $script:Launches++
}
function Assert-Fails([scriptblock]$Action) {
    $Failed = $false
    try { & $Action } catch { $Failed = $true }
    if (-not $Failed) { throw 'Expected migration failure.' }
}
try {
    $Config = Join-Path $Work 'configuration.json'
    $Installer = Join-Path $Work 'BibCiTeX.appinstaller'
    @{ name='BibCiTeX'; publisher='CN=Test'; version='0.6.1.0'; appInstallerUri='https://example.invalid/BibCiTeX.appinstaller' } | ConvertTo-Json | Set-Content $Config
    '<AppInstaller Uri="https://example.invalid/BibCiTeX.appinstaller"><MainPackage Name="BibCiTeX" Publisher="CN=Test" Version="0.6.1.0" /></AppInstaller>' | Set-Content $Installer
    Invoke-BibCiTeXMigration -Configuration $Config -InstallerPath $Installer
    if ($script:Deployments -ne 1 -or $script:Launches -ne 1 -or $script:Removals -ne 1) { throw 'Successful migration did not deploy, clean up and launch.' }
    $script:FailDeployment = $true
    Assert-Fails { Invoke-BibCiTeXMigration -Configuration $Config -InstallerPath $Installer }
    if ($script:Launches -ne 1 -or $script:Removals -ne 1) { throw 'Failed deployment launched or removed application.' }
    $script:FailDeployment = $false
    $script:InstalledPublisher = 'CN=Other'
    Assert-Fails { Invoke-BibCiTeXMigration -Configuration $Config -InstallerPath $Installer }
    if ($script:Launches -ne 1) { throw 'Wrong publisher launched application.' }
    (Get-Content $Installer -Raw).Replace('CN=Test', 'CN=Other') | Set-Content $Installer
    $Before = $script:Deployments
    Assert-Fails { Invoke-BibCiTeXMigration -Configuration $Config -InstallerPath $Installer }
    if ($script:Deployments -ne $Before) { throw 'Mismatched metadata reached deployment.' }
    (Get-Content $Installer -Raw).Replace('CN=Other', 'CN=Test') | Set-Content $Installer
    $script:InstalledPublisher = 'CN=Test'
    function Report-LegacyMsiInstallation { throw 'simulated cleanup failure' }
    Assert-Fails { Invoke-BibCiTeXMigration -Configuration $Config -InstallerPath $Installer }
    if ($script:Launches -ne 2) { throw 'Successfully installed app was not launched after cleanup failed.' }
    $env:LOCALAPPDATA = $Work
    $Directory = Join-Path $Work 'BibCiTeX'
    $script:SavedDirectory = Join-Path $Work 'custom-install'
    function Get-Item {
        param($LiteralPath, $ErrorAction)
        $Key = [pscustomobject]@{}
        $Key | Add-Member -MemberType ScriptMethod -Name GetValue -Value { param($Name); $script:SavedDirectory }
        $Key | Add-Member -MemberType ScriptMethod -Name Close -Value { }
        $Key
    }
    if ((& $GetNsisDirectory) -ne $script:SavedDirectory) { throw 'Custom registered directory was not accepted.' }
    $script:SavedDirectory = 'relative-path'
    Assert-Fails { & $GetNsisDirectory }
    $script:SavedDirectory = [IO.Path]::GetPathRoot($Work)
    Assert-Fails { & $GetNsisDirectory }
    $script:SavedDirectory = '\\server\share'
    Assert-Fails { & $GetNsisDirectory }
    function Get-LegacyNsisDirectory { $Directory }
    New-Item -ItemType Directory $Directory | Out-Null
    $Uninstaller = Join-Path $Directory 'uninstall.exe'
    Set-Content -LiteralPath $Uninstaller -Value 'test fixture, never executed'
    $script:Registration = [pscustomobject]@{ DisplayName='BibCiTeX'; Publisher='bibcitex'; DisplayVersion='0.6.0'; UninstallString=('"' + $Uninstaller + '"') }
    $script:Uninstallations = 0
    function Get-ItemProperty { param($LiteralPath, $ErrorAction); $script:Registration }
    function Start-Process {
        param($FilePath, $ArgumentList, [switch]$Wait, [switch]$PassThru)
        if ($FilePath -ne $Uninstaller -or $ArgumentList -ne "/S /UPDATE _?=$Directory" -or -not $Wait -or -not $PassThru) { throw 'Unsafe legacy uninstall invocation.' }
        $script:Uninstallations++
        [pscustomobject]@{ ExitCode=0 }
    }
    & $RemoveNsis
    if ($script:Uninstallations -ne 1) { throw 'Validated default NSIS was not removed.' }
    $script:Registration.UninstallString = 'cmd.exe /c unsafe'
    & $RemoveNsis
    $script:Registration.UninstallString = '"' + $Uninstaller + '"'
    $script:Registration.DisplayVersion = '0.7.0'
    & $RemoveNsis
    if ($script:Uninstallations -ne 1) { throw 'Unverified legacy installation was removed.' }
    $script:MsiProduct = [pscustomobject]@{ Code='{01234567-89AB-CDEF-0123-456789ABCDEF}'; Name='BibCiTeX'; Publisher='bibcitex'; Version='0.6.0' }
    function Get-LegacyMsiProducts { $script:MsiProduct }
    $script:MsiWarnings = 0
    function Write-Warning { param($Message); $script:MsiWarnings++ }
    function Start-Process { throw 'Machine-wide MSI must not be uninstalled or elevated.' }
    & $ReportMsi
    if ($script:MsiWarnings -ne 1) { throw 'Shared MSI retention was not reported.' }
    $script:MsiProduct.Publisher = 'other'
    Assert-Fails { & $ReportMsi }
    $script:MsiProduct.Publisher = 'bibcitex'
    $script:MsiProduct.Version = '0.7.0'
    & $ReportMsi
    $script:MsiProduct.Version = '0.6.0'
    $script:MsiProduct.Code = 'cmd.exe /c arbitrary'
    Assert-Fails { & $ReportMsi }
    if ($script:Uninstallations -ne 1 -or $script:MsiWarnings -ne 1) { throw 'Shared MSI installation handling was unsafe.' }
    Write-Output 'PASS migration ordering, failure recovery, publisher/metadata mismatch, NSIS cleanup guards and machine-wide MSI retention'
} finally {
    $env:LOCALAPPDATA = $PreviousLocalAppData
    Remove-Item -LiteralPath $Work -Recurse -Force
}
