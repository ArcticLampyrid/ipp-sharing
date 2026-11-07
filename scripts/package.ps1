[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet("i686-pc-windows-msvc", "x86_64-pc-windows-msvc")]
    [string] $Target,

    [Parameter(Mandatory)]
    [ValidateSet("x86", "x64")]
    [string] $Architecture,

    [Parameter(Mandatory)]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Leaf })]
    [string] $BonjourPath,

    [Parameter(Mandatory)]
    [string] $OutputDirectory
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$releaseDirectory = Join-Path $root "target\$Target\release"
$buildDirectory = Join-Path $releaseDirectory "build"
$packageName = "ipp-sharing-windows-$Architecture"
$packageDirectory = Join-Path $OutputDirectory $packageName
$sevenZipCommand = Get-Command 7z.exe -ErrorAction SilentlyContinue
$sevenZipPath = $null
if ($null -ne $sevenZipCommand) {
    $sevenZipPath = $sevenZipCommand.Source
}
if ([string]::IsNullOrWhiteSpace($sevenZipPath)) {
    $sevenZipPath = @(
        (Join-Path ${env:ProgramFiles} "7-Zip\7z.exe"),
        (Join-Path ${env:ProgramFiles(x86)} "7-Zip\7z.exe")
    ) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}
if ([string]::IsNullOrWhiteSpace($sevenZipPath)) {
    throw "7-Zip was not found. Install 7-Zip and make 7z.exe available on PATH."
}

if (-not (Test-Path -LiteralPath (Join-Path $releaseDirectory "ipp-sharing.exe"))) {
    throw "The CLI binary was not found in $releaseDirectory."
}
if (-not (Test-Path -LiteralPath (Join-Path $releaseDirectory "ipp-sharing-gui.exe"))) {
    throw "The GUI binary was not found in $releaseDirectory."
}

$pdfium = Get-ChildItem -LiteralPath $buildDirectory -Filter "pdfium.dll" -File -Recurse |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if ($null -eq $pdfium) {
    throw "pdfium.dll was not found below $buildDirectory. The package must include the PDFium runtime."
}

$metadata = (& cargo metadata --locked --no-deps --format-version 1 | ConvertFrom-Json)
$version = ($metadata.packages | Where-Object { $_.name -eq "ipp-sharing-gui" } | Select-Object -First 1).version
if ([string]::IsNullOrWhiteSpace($version)) {
    throw "Could not determine the package version from Cargo metadata."
}

if (Test-Path -LiteralPath $packageDirectory) {
    Remove-Item -LiteralPath $packageDirectory -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $packageDirectory | Out-Null

Copy-Item (Join-Path $releaseDirectory "ipp-sharing.exe") $packageDirectory
Copy-Item (Join-Path $releaseDirectory "ipp-sharing-gui.exe") $packageDirectory
Copy-Item $pdfium.FullName (Join-Path $packageDirectory "pdfium.dll")
Copy-Item $BonjourPath (Join-Path $packageDirectory "Bonjour.msi")
Copy-Item (Join-Path $root "LICENSE.md") $packageDirectory

$readme = (Get-Content (Join-Path $root "packaging\README.md") -Raw).
    Replace("@ARCHITECTURE@", $Architecture).
    Replace("@VERSION@", $version)
Set-Content -LiteralPath (Join-Path $packageDirectory "README.md") -Value $readme -Encoding utf8
Copy-Item (Join-Path $root "packaging\config.example.yaml") (Join-Path $packageDirectory "config.example.yaml")

$checksums = Get-ChildItem -LiteralPath $packageDirectory -File |
    Sort-Object Name |
    ForEach-Object {
        $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $($_.Name)"
    }
Set-Content -LiteralPath (Join-Path $packageDirectory "SHA256SUMS.txt") -Value $checksums -Encoding ascii

$archive = Join-Path $OutputDirectory "$packageName-$version.7z"
if (Test-Path -LiteralPath $archive) {
    Remove-Item -LiteralPath $archive -Force
}
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
Push-Location $packageDirectory
try {
    & $sevenZipPath a -t7z -ms=on -mmt=on -mx=9 $archive "*"
    if ($LASTEXITCODE -ne 0) {
        throw "7-Zip failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}
Write-Host "Created $archive"
