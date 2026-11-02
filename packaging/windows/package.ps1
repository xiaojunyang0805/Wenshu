<#
.SYNOPSIS
  Build and package Wenshu (文枢) for Windows.

.DESCRIPTION
  Produces, in $env:DIST (default: dist):
    wenshu-<version>-windows-<arch>-portable.zip   wenshu-desktop.exe + wenshu.exe (CLI) + notices
    wenshu-<version>-windows-<arch>.msi            per-user installer, when WiX v5 is installed

  Like WordCraft's packaging (upstream/wordcraft/packaging/windows), the binaries link the C runtime
  statically (+crt-static), so neither package needs the Visual C++ redistributable. The packages
  carry Wenshu's licences, WordCraft's NOTICE, ATTRIBUTION and licences, and the OFL licences of
  the fonts built into the app.

  Needs: Rust (MSVC toolchain), and `scripts/sync-upstream.sh` run first. For the MSI also WiX v5:
    dotnet tool install --global wix --version 5.0.2

.EXAMPLE
  powershell -File packaging/windows/package.ps1
  powershell -File packaging/windows/package.ps1 -SkipBuild
#>
param(
  [ValidateSet('x64', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Up = Join-Path $Root 'upstream\wordcraft'

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version: [workspace.package] version in the root Cargo.toml.
$Version = $null
$inPkg = $false
foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
  if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
  if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }

$Target = switch ($Arch) { 'x64' { 'x86_64-pc-windows-msvc' } 'arm64' { 'aarch64-pc-windows-msvc' } }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'eval\target\wordcraft' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $SkipBuild) {
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p wenshu-desktop -p wenshu-cli --target $Target }
}
$Bin = Join-Path $TargetDir "$Target\release"

# PE headers: the right machine, the app a GUI program (2), the CLI a console one (3).
function Get-PeHeader([string] $Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  $pe = [BitConverter]::ToInt32($bytes, 0x3C)
  return @{ Machine = [BitConverter]::ToUInt16($bytes, $pe + 4); Subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C) }
}
$Machine = switch ($Arch) { 'x64' { 0x8664 } 'arm64' { 0xAA64 } }
foreach ($check in @(@('wenshu-desktop.exe', 2), @('wenshu.exe', 3))) {
  $h = Get-PeHeader (Join-Path $Bin $check[0])
  if ($h.Machine -ne $Machine) { throw "$($check[0]) is for machine 0x$('{0:X}' -f $h.Machine), expected $Arch" }
  if ($h.Subsystem -ne $check[1]) { throw "$($check[0]) has PE subsystem $($h.Subsystem), expected $($check[1])" }
  Write-Output "ok $($check[0]): $Arch, PE subsystem $($h.Subsystem)"
}

$Stage = Join-Path $TargetDir "wenshu-package\$Arch"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item (Join-Path $Bin 'wenshu-desktop.exe'), (Join-Path $Bin 'wenshu.exe') $Stage
Copy-Item (Join-Path $Root 'LICENSE-MIT'), (Join-Path $Root 'LICENSE-APACHE') $Stage
$Notices = Join-Path $Stage 'notices'
New-Item -ItemType Directory -Force -Path $Notices | Out-Null
Copy-Item (Join-Path $Up 'NOTICE') (Join-Path $Notices 'WordCraft-NOTICE')
Copy-Item (Join-Path $Up 'ATTRIBUTION.md') (Join-Path $Notices 'WordCraft-ATTRIBUTION.md')
Copy-Item (Join-Path $Up 'LICENSE-MIT') (Join-Path $Notices 'WordCraft-LICENSE-MIT')
Copy-Item (Join-Path $Up 'LICENSE-APACHE') (Join-Path $Notices 'WordCraft-LICENSE-APACHE')
Copy-Item (Join-Path $Up 'assets\fonts\OFL-*.txt') $Notices

$Zip = Join-Path $Dist "wenshu-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
# Windows' bsdtar writes a standard zip (forward-slash paths); PowerShell 5.1's Compress-Archive
# writes backslashes that other systems' unzip tools reject.
Invoke-Native 'zip' { & (Join-Path $env:SystemRoot 'System32\tar.exe') -a -c -f $Zip -C $Stage . }
Write-Output "wrote $Zip"

if (Get-Command wix -ErrorAction SilentlyContinue) {
  # wenshu.wxs names each notice file (a per-user component can't harvest a folder): the staged
  # notices must be exactly those, or an upstream font licence would silently miss the MSI.
  $Wxs = Get-Content -Raw -Encoding UTF8 (Join-Path $PSScriptRoot 'wenshu.wxs')
  $Listed = [regex]::Matches($Wxs, '\\notices\\([^"]+)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object
  $Staged = Get-ChildItem -Name $Notices | Sort-Object
  if (Compare-Object @($Listed) @($Staged)) {
    throw "wenshu.wxs lists notices [$($Listed -join ', ')] but the package has [$($Staged -join ', ')]"
  }
  $Msi = Join-Path $Dist "wenshu-$Version-windows-$Arch.msi"
  $MsiVersion = ($Version -split '-')[0]
  Invoke-Native "wix build" {
    wix build (Join-Path $PSScriptRoot 'wenshu.wxs') -arch $Arch -d "Version=$MsiVersion" -d "Stage=$Stage" `
      -d "Icon=$(Join-Path $Root 'apps\wenshu-desktop\assets\wenshu.ico')" -o $Msi
  }
  Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))
  Write-Output "wrote $Msi"
} else {
  Write-Warning 'WiX v5 not found: no MSI (dotnet tool install --global wix --version 5.0.2)'
}
