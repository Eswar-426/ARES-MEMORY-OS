param(
    [string]$RipgrepPath,
    [string]$CargoWatchPath,
    [string]$AutomyraPath,
    [string]$RustBookPath
)

$base_dir = (Get-Location).Path
$temp_dir = Join-Path $env:TEMP "ares-cert"
if (-not (Test-Path $temp_dir)) {
    New-Item -ItemType Directory -Force -Path $temp_dir | Out-Null
}

$ares_exe = "$base_dir\target\debug\ares.exe"
if (-not (Test-Path $ares_exe)) {
    Write-Host "ARES executable not found at $ares_exe. Building..."
    cargo build --bin ares
}

function Remove-AresDb {
    Stop-Process -Name "ares" -Force -ErrorAction SilentlyContinue
    for ($retry = 0; $retry -lt 5; $retry++) {
        if (-not (Test-Path ".ares")) { return }
        Remove-Item -Recurse -Force ".ares" -ErrorAction SilentlyContinue
        if (-not (Test-Path ".ares")) { return }
        Start-Sleep -Seconds 1
    }
}

function Run-RealScenario {
    param([string]$name, [string]$repo_url, [string]$override_path)
    
    Write-Host "`n=================================================="
    Write-Host "  Repository: $name"
    Write-Host "=================================================="
    
    $repo_dir = ""
    if ($override_path -and (Test-Path $override_path)) {
        Write-Host "Using local override path: $override_path"
        $repo_dir = $override_path
    } else {
        $repo_dir = Join-Path $temp_dir $name
        if (-not (Test-Path $repo_dir)) {
            Write-Host "Cloning $repo_url to $repo_dir..."
            Set-Location $temp_dir
            git clone $repo_url $name
            if ($LASTEXITCODE -ne 0) {
                Write-Host "Failed to clone $repo_url"
                return
            }
        } else {
            Write-Host "Repository already cloned at $repo_dir"
            # Optional: git pull
        }
    }
    
    Set-Location $repo_dir
    Write-Host "Cleaning previous ARES state..."
    Remove-AresDb
    
    Write-Host "`n--- Ingesting ---"
    & $ares_exe ingest .
    
    Write-Host "`n--- Metrics ---"
    Write-Host "Memory Coverage:"
    & $ares_exe governance coverage
    
    Write-Host "`nMemory Debt:"
    & $ares_exe governance debt
    
    Write-Host "`nMemory Health:"
    & $ares_exe governance health
    
    Write-Host "`nMemory Maturity:"
    & $ares_exe governance maturity
    
    Write-Host "`nMemory Confidence:"
    & $ares_exe governance confidence
    
    Set-Location $base_dir
}

# 1. ARES (Self-measurement)
Write-Host "`n=================================================="
Write-Host "  Repository: ARES (Self-measurement)"
Write-Host "=================================================="
Set-Location $base_dir
Remove-AresDb
& $ares_exe ingest .
Write-Host "`nMemory Coverage:"
& $ares_exe governance coverage
Write-Host "`nMemory Debt:"
& $ares_exe governance debt
Write-Host "`nMemory Health:"
& $ares_exe governance health
Write-Host "`nMemory Maturity:"
& $ares_exe governance maturity
Write-Host "`nMemory Confidence:"
& $ares_exe governance confidence

# 2. Automyra
# Assuming Automyra is available at https://github.com/eswar-426/Automyra (If not, we can rely on override or it will fail clone gracefully)
Run-RealScenario -name "Automyra" -repo_url "https://github.com/eswar-426/Automyra.git" -override_path $AutomyraPath

# 3. ripgrep
Run-RealScenario -name "ripgrep" -repo_url "https://github.com/BurntSushi/ripgrep.git" -override_path $RipgrepPath

# 4. cargo-watch
Run-RealScenario -name "cargo-watch" -repo_url "https://github.com/watchexec/cargo-watch.git" -override_path $CargoWatchPath

# 5. rust-lang/book
Run-RealScenario -name "rust-book" -repo_url "https://github.com/rust-lang/book.git" -override_path $RustBookPath

Write-Host "`nReal Matrix Certification Complete."
