$ErrorActionPreference = "Stop"

Write-Host "P3.1 Governance Certification - Synthetic Matrix Execution"
Write-Host "=========================================================="

$base_dir = "E:\My Projects\ARES_Memory_os"
$ares_exe = "$base_dir\target\debug\ares.exe"

Write-Host "Compiling ARES..."
Set-Location $base_dir
cargo build --bin ares

Stop-Process -Name "ares" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

$test_dir = "$base_dir\cert_synthetic"
if (Test-Path $test_dir) { Remove-Item -Recurse -Force $test_dir -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Force -Path $test_dir | Out-Null
Set-Location $test_dir

function Remove-AresDb {
    Stop-Process -Name "ares" -Force -ErrorAction SilentlyContinue
    for ($retry = 0; $retry -lt 5; $retry++) {
        if (-not (Test-Path ".ares")) { return }
        Remove-Item -Recurse -Force ".ares" -ErrorAction SilentlyContinue
        if (-not (Test-Path ".ares")) { return }
        Start-Sleep -Seconds 1
    }
}

function Run-Scenario {
    param([string]$name, [scriptblock]$setup, [string]$expected_status)
    
    Write-Host "`n--- Archetype: $name ---"
    $scenario_dir = "$test_dir\$name"
    New-Item -ItemType Directory -Force -Path $scenario_dir | Out-Null
    Set-Location $scenario_dir
    Remove-AresDb
    
    # Run setup
    & $setup
    
    # Ingest baseline if not already created
    if (-not (Test-Path "baseline.json")) {
        & $ares_exe ingest .
        & $ares_exe governance snapshot create --out "baseline.json"
    }
    
    # Run check
    $p = Start-Process -FilePath $ares_exe -ArgumentList "governance","check","--baseline","baseline.json" -Wait -NoNewWindow -PassThru
    $exit_code = $p.ExitCode
    
    $status = if ($exit_code -eq 0) { "Pass" } else { "HardFail" }
    
    if ($status -ne $expected_status) {
        Write-Host "FAIL: Expected $expected_status, got $status (Exit code $exit_code)" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "OK: Matches expected $expected_status" -ForegroundColor Green
    }
}

$MemoryNative = {
    New-Item -ItemType Directory -Force -Path ".github", "src", "docs\requirements", "docs\decisions" | Out-Null
    for ($i=0; $i -lt 10; $i++) {
        Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
        Set-Content -Path "docs\requirements\REQ_$i.md" -Value "# REQ-$i`nThis is a requirement."
        Set-Content -Path "docs\decisions\ADR_$i.md" -Value "# ADR-$i`nDecision: Use Rust."
    }
    Set-Content -Path ".github\CODEOWNERS" -Value "* @eswar-426"
}

$Healthy = {
    New-Item -ItemType Directory -Force -Path ".github", "src", "docs\requirements", "docs\decisions" | Out-Null
    for ($i=0; $i -lt 10; $i++) {
        Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
        if ($i -lt 9) {
            Set-Content -Path "docs\requirements\REQ_$i.md" -Value "# REQ-$i"
            Set-Content -Path "docs\decisions\ADR_$i.md" -Value "# ADR-$i"
        }
    }
    Set-Content -Path ".github\CODEOWNERS" -Value "* @eswar-426"
}

$Moderate = {
    # Moderate debt, triggers SoftFail but passes gatekeeper (Exit 0)
    New-Item -ItemType Directory -Force -Path ".github", "src", "docs\requirements", "docs\decisions" | Out-Null
    for ($i=0; $i -lt 10; $i++) {
        Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
        if ($i -lt 5) {
            Set-Content -Path "docs\requirements\REQ_$i.md" -Value "# REQ-$i"
            Set-Content -Path "docs\decisions\ADR_$i.md" -Value "# ADR-$i"
        }
    }
    Set-Content -Path ".github\CODEOWNERS" -Value "* @eswar-426"
}

$Critical = {
    # Coverage drop HardFail
    New-Item -ItemType Directory -Force -Path ".github", "src", "docs\requirements", "docs\decisions" | Out-Null
    for ($i=0; $i -lt 10; $i++) {
        Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
    }
    Set-Content -Path ".github\CODEOWNERS" -Value "* @eswar-426"
    & $ares_exe ingest .
    & $ares_exe governance snapshot create --out "baseline.json"
    
    # Introduce the breaking change (drop coverage)
    Remove-Item ".github\CODEOWNERS"
    Remove-AresDb
    & $ares_exe ingest .
}

$Chaos = {
    # High debt -> Critical -> HardFail
    New-Item -ItemType Directory -Force -Path "src", "docs\requirements", "docs\decisions" | Out-Null
    for ($i=0; $i -lt 20; $i++) {
        Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
    }
    # No owners, no ADRs, no requirements = Massive Debt & 0% Coverage
    & $ares_exe ingest .
    & $ares_exe governance snapshot create --out "baseline.json"
    
    # Add more code with no docs
    for ($i=20; $i -lt 30; $i++) {
        Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
    }
    Remove-AresDb
    & $ares_exe ingest .
}

$BrokenEnterpriseRepo = {
    New-Item -ItemType Directory -Force -Path "src" | Out-Null
    for ($i=0; $i -lt 50; $i++) {
        Set-Content -Path "src\legacy_$i.rs" -Value "fn legacy() {}"
    }
    & $ares_exe ingest .
    & $ares_exe governance snapshot create --out "baseline.json"
    
    for ($i=50; $i -lt 65; $i++) {
        Set-Content -Path "src\legacy_$i.rs" -Value "fn legacy() {}"
    }
    Remove-AresDb
    & $ares_exe ingest .
}

Run-Scenario -name "MemoryNative" -setup $MemoryNative -expected_status "Pass"
Run-Scenario -name "Healthy" -setup $Healthy -expected_status "Pass"
Run-Scenario -name "Moderate" -setup $Moderate -expected_status "Pass"
Run-Scenario -name "Critical" -setup $Critical -expected_status "HardFail"
Run-Scenario -name "Chaos" -setup $Chaos -expected_status "HardFail"
Run-Scenario -name "BrokenEnterpriseRepo" -setup $BrokenEnterpriseRepo -expected_status "HardFail"

Write-Host "`nAll Synthetic Archetypes Certified." -ForegroundColor Green
