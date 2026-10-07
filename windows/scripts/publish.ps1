# User-invoked or release workflow. Uses the same MSBuild graph as Visual Studio.
param(
    [Parameter(Mandatory)][ValidateSet('x64', 'ARM64')][string]$Architecture,
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][ValidateSet('stable', 'beta', 'alpha')][string]$Channel,
    [Parameter(Mandatory)][uint32]$BuildNumber,
    [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$Arch = $Architecture.ToLowerInvariant()
$Rid = "win-$Arch"
# Installer OS guard matches TargetPlatformMinVersion; update channels keep their existing names.
$InstallerRuntime = "win10.0.17763-$Arch"
# Isolate publish output so obsolete files from another version cannot enter a package.
$Publish = Join-Path $Repo "target/publish/$Rid-$([guid]::NewGuid().ToString('N'))"
if ((Test-Path $OutputDirectory) -and @(Get-ChildItem -Force $OutputDirectory).Count -ne 0) {
    throw 'OutputDirectory must be empty; refusing to mix release artifacts.'
}
$Tools = Join-Path $Repo 'target/release-tools'
$Notes = Join-Path $Repo "release-notes/$Version"
foreach ($Language in @('zh-Hans', 'en')) {
    if (-not (Test-Path (Join-Path $Notes "$Language.md"))) { throw "Missing $Language release notes for $Version" }
}
& dotnet publish (Join-Path $Repo 'windows/BibCiTeX/BibCiTeX.csproj') -c Release "-p:Platform=$Architecture" "-p:RuntimeIdentifier=$Rid" "-p:Version=$Version" -o $Publish
if ($LASTEXITCODE -ne 0) { throw 'WinUI publish failed.' }
foreach ($BundledRuntime in @('coreclr.dll', 'Microsoft.ui.xaml.dll', 'DirectML.dll', 'onnxruntime.dll')) {
    if (Test-Path (Join-Path $Publish $BundledRuntime)) { throw "Unexpected bundled runtime: $BundledRuntime" }
}
$RuntimeConfig = Get-Content -Raw (Join-Path $Publish 'BibCiTeX.runtimeconfig.json') | ConvertFrom-Json
if ($RuntimeConfig.runtimeOptions.framework.name -ne 'Microsoft.NETCore.App') { throw 'Expected framework-dependent .NET publish output.' }
if (-not (Test-Path (Join-Path $Publish 'Microsoft.WindowsAppRuntime.Bootstrap.dll'))) { throw 'Windows App Runtime bootstrap DLL is missing.' }
if (-not (Test-Path (Join-Path $Tools 'vpk.exe'))) {
    & dotnet tool install vpk --version 1.2.161 --tool-path $Tools
    if ($LASTEXITCODE -ne 0) { throw 'Velopack tool installation failed.' }
}
New-Item -ItemType Directory -Force $OutputDirectory | Out-Null
$Combined = Join-Path $Publish 'release-notes.md'
$Text = "<!-- locale:zh-Hans -->`n" + (Get-Content -Raw (Join-Path $Notes 'zh-Hans.md')) + "`n<!-- /locale -->`n<!-- locale:en -->`n" + (Get-Content -Raw (Join-Path $Notes 'en.md')) + "`n<!-- /locale -->"
[IO.File]::WriteAllText($Combined, $Text)
# MSI has numeric versions; the CI run sequence is independent from display SemVer.
$MsiVersion = "1.$([math]::Floor($BuildNumber / 65536)).$($BuildNumber % 65536)"
if ($BuildNumber -ge 16777216) { throw 'MSI build sequence exhausted.' }
& (Join-Path $Tools 'vpk.exe') pack --packId BibCiTeX --packVersion $Version --packDir $Publish --mainExe BibCiTeX.exe --runtime $InstallerRuntime --framework "net10.0-$Arch-runtime,vcredist143-$Arch" --channel "$Rid-$Channel" --outputDir $OutputDirectory --icon (Join-Path $Repo 'assets/app-icons/icon.ico') --releaseNotes $Combined --msi --msiVersion $MsiVersion
if ($LASTEXITCODE -ne 0) { throw 'Velopack packaging failed.' }
foreach ($Extension in @('exe', 'msi')) {
    $Installers = @(Get-ChildItem $OutputDirectory -File -Filter "*.$Extension")
    if ($Installers.Count -ne 1) { throw "Expected one $Extension installer." }
    Move-Item $Installers[0].FullName (Join-Path $OutputDirectory "BibCiTeX-$Version-windows-$Arch.$Extension")
}
# Portable archives and legacy RELEASES files are not part of this distribution.
Get-ChildItem $OutputDirectory -File | Where-Object { $_.Extension -eq '.zip' -or $_.Name -like 'RELEASES*' -and $_.Extension -ne '.json' } | Remove-Item

# Runtime updates only consume full packages; installer names above are public downloads.
$FeedPath = Join-Path $OutputDirectory "releases.$Rid-$Channel.json"
$Feed = Get-Content -Raw $FeedPath | ConvertFrom-Json
$Feed.Assets = @($Feed.Assets | Where-Object { $_.Type -eq 'Full' -or $_.Type -eq 1 })
if ($Feed.Assets.Count -ne 1) { throw 'Expected one full Velopack package.' }
$Feed | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8NoBOM $FeedPath
