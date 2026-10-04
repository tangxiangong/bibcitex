# Runs inside the Tauri-compatible signed bootstrapper, in the installing user's session.
param([string]$Configuration)
function Get-LegacyMsiProducts {
    # Tauri v2 derives this UpgradeCode using UUIDv5(DNS, "BibCiTeX.exe.app.x64")
    # for both x64 and ARM64. Enumerate only this product family, not all MSI apps.
    $Installer = New-Object -ComObject WindowsInstaller.Installer
    try {
        foreach ($Code in @($Installer.RelatedProducts('{7165612C-F089-5C3E-A3D9-B3D8782CA8D8}'))) {
            [pscustomobject]@{
                Code = $Code
                Name = $Installer.ProductInfo($Code, 'ProductName')
                Publisher = $Installer.ProductInfo($Code, 'Publisher')
                Version = $Installer.ProductInfo($Code, 'VersionString')
            }
        }
    } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($Installer) }
}
function Report-LegacyMsiInstallation {
    foreach ($Product in @(Get-LegacyMsiProducts)) {
        if ($Product.Code -notmatch '^\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}$' -or
            $Product.Name -ne 'BibCiTeX' -or $Product.Publisher -ne 'bibcitex') {
            throw 'Unexpected legacy MSI product identity.'
        }
        $Version = $null
        if (-not [version]::TryParse($Product.Version, [ref]$Version)) { throw 'Invalid legacy MSI version.' }
        if ($Version -gt [version]'0.6.0') { continue }
        Write-Warning 'BibCiTeX 已为当前用户安装。旧 MSI 安装供所有用户使用，已保留。'
    }
}
function Get-LegacyNsisDirectory {
    $Key = Get-Item -LiteralPath 'HKCU:\Software\bibcitex\BibCiTeX' -ErrorAction SilentlyContinue
    try { $Directory = if ($null -ne $Key) { $Key.GetValue('') } else { $null } }
    finally { if ($null -ne $Key) { $Key.Close() } }
    if ([string]::IsNullOrWhiteSpace($Directory)) { $Directory = Join-Path $env:LOCALAPPDATA 'BibCiTeX' }
    if (-not [IO.Path]::IsPathRooted($Directory) -or $Directory.StartsWith('\\')) { throw 'Invalid legacy NSIS installation directory.' }
    $Directory = [IO.Path]::GetFullPath($Directory).TrimEnd('\', '/')
    if ($Directory -eq [IO.Path]::GetPathRoot($Directory).TrimEnd('\', '/')) { throw 'Legacy installation directory cannot be a drive root.' }
    return $Directory
}
function Wait-LegacyApplicationExit {
    $KnownPaths = @(
        (Join-Path (Get-LegacyNsisDirectory) 'bibcitex.exe'),
        (Join-Path $env:ProgramFiles 'BibCiTeX/bibcitex.exe')
    )
    $Deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        $Running = @(Get-Process -Name bibcitex -ErrorAction SilentlyContinue | Where-Object { $_.Path -in $KnownPaths })
        if ($Running.Count -eq 0) { return }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $Deadline)
    throw 'The previous BibCiTeX process is still running; close it before removing the old installation.'
}
function Remove-LegacyNsisShortcuts {
    param([Parameter(Mandatory)][string]$Directory)
    $Roots = @([Environment]::GetFolderPath('DesktopDirectory'), [Environment]::GetFolderPath('StartMenu'))
    $Links = @($Roots | Where-Object { $_ -and (Test-Path -LiteralPath $_) } | ForEach-Object {
        Get-ChildItem -LiteralPath $_ -Filter 'BibCiTeX.lnk' -File -Recurse -ErrorAction Stop
    })
    if ($Links.Count -eq 0) { return }
    $Shell = New-Object -ComObject WScript.Shell
    try {
        foreach ($Link in $Links) {
            $Shortcut = $Shell.CreateShortcut($Link.FullName)
            try {
                if ($Shortcut.TargetPath -eq (Join-Path $Directory 'bibcitex.exe')) {
                    Remove-Item -LiteralPath $Link.FullName -ErrorAction Stop
                }
            } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($Shortcut) }
        }
    } finally { [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($Shell) }
}
function Remove-LegacyNsisInstallation {
    $Directory = Get-LegacyNsisDirectory
    $Uninstaller = Join-Path $Directory 'uninstall.exe'
    $Registration = Get-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BibCiTeX' -ErrorAction SilentlyContinue
    # Only a verified per-user NSIS installation is eligible. Never execute
    # an arbitrary uninstall command from the registry, or remove an MSI installation.
    if ($null -eq $Registration -or $Registration.DisplayName -ne 'BibCiTeX' -or
        $Registration.Publisher -ne 'bibcitex' -or
        $Registration.UninstallString -ne ('"' + $Uninstaller + '"') -or
        -not (Test-Path -LiteralPath $Uninstaller)) { return }
    $Version = $null
    if (-not [version]::TryParse($Registration.DisplayVersion, [ref]$Version) -or $Version -gt [version]'0.6.0') { return }
    $Process = Start-Process -FilePath $Uninstaller -ArgumentList "/S /UPDATE _?=$Directory" -Wait -PassThru
    if ($Process.ExitCode -ne 0) { throw 'The native package is installed, but removing the old installation failed.' }
    # /UPDATE preserves shortcuts for a replacement EXE; our replacement has a
    # package identity, so remove only links still targeting the verified old EXE.
    Remove-LegacyNsisShortcuts -Directory $Directory
}
function Invoke-BibCiTeXMigration {
    param(
        [Parameter(Mandatory)][string]$Configuration,
        [string]$InstallerPath = (Join-Path $PSScriptRoot 'BibCiTeX.appinstaller')
    )
    $ErrorActionPreference = 'Stop'
    $Config = Get-Content -LiteralPath $Configuration -Raw | ConvertFrom-Json
    $Source = [uri]$Config.appInstallerUri
    if ($Source.Scheme -ne 'https') { throw 'The update source must use HTTPS.' }
    if ($Config.name -ne 'BibCiTeX') { throw 'Unexpected update package identity.' }
    [xml]$Installer = Get-Content -LiteralPath $InstallerPath -Raw
    if ($Installer.AppInstaller.Uri -ne $Source.AbsoluteUri -or
        $Installer.AppInstaller.MainPackage.Name -ne $Config.name -or
        $Installer.AppInstaller.MainPackage.Publisher -ne $Config.publisher -or
        $Installer.AppInstaller.MainPackage.Version -ne $Config.version) {
        throw 'The bundled App Installer identity does not match the release.'
    }
    # Installing through App Installer registers its update association as well as the package.
    # Windows validates the MSIX publisher signature; never import a certificate into user trust.
    Add-AppxPackage -AppInstallerFile -Path $InstallerPath -ErrorAction Stop
    $Package = @(Get-AppxPackage -Name $Config.name | Where-Object {
        $_.Publisher -eq $Config.publisher -and [version]$_.Version -ge [version]$Config.version
    })
    if ($Package.Count -ne 1) { throw 'The expected signed package was not registered.' }
    # /UPDATE skips the old Tauri uninstaller's optional application-data deletion.
    $CleanupError = $null
    try {
        Wait-LegacyApplicationExit
        Remove-LegacyNsisInstallation
        Report-LegacyMsiInstallation
    } catch { $CleanupError = $_ }
    # A successfully installed package must remain usable even if legacy cleanup
    # is cancelled or fails. Surface that failure after launching the new app.
    Start-Process explorer.exe -ArgumentList "shell:AppsFolder\$($Package[0].PackageFamilyName)!App"
    if ($null -ne $CleanupError) { throw $CleanupError }
}
if ($MyInvocation.InvocationName -ne '.') {
    try {
        Invoke-BibCiTeXMigration -Configuration $Configuration
        exit 0
    } catch {
        [Console]::Error.WriteLine($_.Exception.Message)
        exit 1
    }
}
