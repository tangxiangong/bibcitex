# User-invoked only. This compiles the generator; the agent must not execute it.
param(
    [Parameter(Mandatory = $true)]
    [string]$Library
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path -LiteralPath $Library -PathType Leaf)) {
    throw "Missing Rust library: $Library. Build bibcitex-csharp first."
}
Push-Location $repo
try {
    # The inventory is compiled from the same Rust declarations as the DLL.
    # It does not load a target-architecture DLL into this host process.
    & cargo run --locked --release -p bibcitex-csharp-bindgen -- (Join-Path $PSScriptRoot 'generated/csharp')
    if ($LASTEXITCODE -ne 0) { throw 'C# binding generation failed.' }
} finally {
    Pop-Location
}
