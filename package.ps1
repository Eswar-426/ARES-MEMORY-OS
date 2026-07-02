# package.ps1 - Build release binaries and package the VS Code extension

$ErrorActionPreference = "Stop"

# Ensure we're in the workspace root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

Write-Host "1. Building ARES in release mode..."
cargo build --release --bin ares --bin ares-mcp

$ExtDir = "extensions\ares-memory-vscode"
$WinBinDir = "$ExtDir\binaries\windows"
$LinuxBinDir = "$ExtDir\binaries\linux"
$MacOsBinDir = "$ExtDir\binaries\macos"

Write-Host "2. Creating binary directories..."
New-Item -ItemType Directory -Force -Path $WinBinDir | Out-Null
New-Item -ItemType Directory -Force -Path $LinuxBinDir | Out-Null
New-Item -ItemType Directory -Force -Path $MacOsBinDir | Out-Null

Write-Host "3. Copying binaries..."
Copy-Item -Path "target\release\ares.exe" -Destination "$WinBinDir\ares.exe" -Force
Copy-Item -Path "target\release\ares-mcp.exe" -Destination "$WinBinDir\ares-mcp.exe" -Force

# Note: For Linux/macOS, we would need to cross-compile. Since this is Windows, we just copy the Windows ones for now.
# Real pipelines would copy all 3 platforms.

Write-Host "4. Packaging VS Code Extension..."
Push-Location $ExtDir
npm install
npm run compile
npx @vscode/vsce package --no-dependencies
Pop-Location

Write-Host "Done!"
