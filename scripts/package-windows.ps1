$ErrorActionPreference = "Stop"

$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$manifestPath = Join-Path $root "Cargo.toml"
$cargoToml = Get-Content -Raw $manifestPath

if ($cargoToml -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "Could not read package version from Cargo.toml."
}

$version = $Matches[1]
$packageName = "ipcalc-gui-$version-windows-x86_64"
$distPath = Join-Path $root "dist"
$packagePath = Join-Path $distPath $packageName
$zipPath = Join-Path $distPath "$packageName.zip"
$distFullPath = [System.IO.Path]::GetFullPath($distPath)
$packageFullPath = [System.IO.Path]::GetFullPath($packagePath)

if (-not $packageFullPath.StartsWith($distFullPath + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to package outside the project dist directory."
}

cargo build --release --manifest-path $manifestPath

if (Test-Path $packagePath) {
    Remove-Item -LiteralPath $packagePath -Recurse -Force
}

New-Item -ItemType Directory -Force $packagePath | Out-Null
Copy-Item -LiteralPath (Join-Path $root "target\release\ipcalc.exe") -Destination $packagePath
Copy-Item -LiteralPath (Join-Path $root "README.md") -Destination $packagePath
Copy-Item -LiteralPath (Join-Path $root "LICENSE") -Destination $packagePath
Copy-Item -LiteralPath (Join-Path $root "CHANGELOG.md") -Destination $packagePath

if (Test-Path $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

Compress-Archive -Path $packagePath -DestinationPath $zipPath -Force
Write-Host "Created $zipPath"
