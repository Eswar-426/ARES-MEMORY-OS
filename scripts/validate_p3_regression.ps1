$ErrorActionPreference = "Stop"

Write-Host "Running P3.0-E Regression Suite..."
$dummy_dir = "dummy_repo"
if (Test-Path $dummy_dir) {
    Remove-Item -Recurse -Force $dummy_dir
}
New-Item -ItemType Directory -Force -Path $dummy_dir | Out-Null
Set-Location $dummy_dir

# Initialize a dummy repo with code, requirement, decision, owner
New-Item -ItemType Directory -Force -Path ".github" | Out-Null
New-Item -ItemType Directory -Force -Path "src" | Out-Null
New-Item -ItemType Directory -Force -Path "docs\requirements" | Out-Null
New-Item -ItemType Directory -Force -Path "docs\decisions" | Out-Null

for ($i=0; $i -lt 25; $i++) {
    Set-Content -Path "src\main_$i.rs" -Value "fn main() {}"
    Set-Content -Path "docs\requirements\REQ_$i.md" -Value "# REQ-$i"
    Set-Content -Path "docs\decisions\ADR_$i.md" -Value "# ADR-$i"
}
Set-Content -Path ".github\CODEOWNERS" -Value "* @eswar-426"

Write-Host "Compiling ARES..."
Set-Location "E:\My Projects\ARES_Memory_os"
cargo build --bin ares

$ares_exe = "E:\My Projects\ARES_Memory_os\target\debug\ares.exe"

Set-Location $dummy_dir
Write-Host "Ingesting baseline..."
& $ares_exe ingest .

Write-Host "Exporting baseline snapshot..."
& $ares_exe governance snapshot create --out "baseline.json"

function Run-Check {
    param([string]$expected)
    
    # We run the command and check exit code
    $p = Start-Process -FilePath $ares_exe -ArgumentList "governance","check","--baseline","baseline.json" -Wait -NoNewWindow -PassThru
    $exit_code = $p.ExitCode
    
    if ($expected -eq "HardFail" -and $exit_code -ne 1) {
        Write-Host "FAIL: Expected HardFail (exit 1), but got $exit_code"
        exit 1
    }
    if ($expected -eq "SoftFail" -and $exit_code -eq 1) {
        Write-Host "FAIL: Expected SoftFail (exit 0), but got $exit_code"
        exit 1
    }
    if ($expected -eq "Pass" -and $exit_code -ne 0) {
        Write-Host "FAIL: Expected Pass (exit 0), but got $exit_code"
        exit 1
    }
    Write-Host "OK: Matches expected $expected"
}

Write-Host "Test 1: Delete Decision -> Expected: HardFail"
Remove-Item "docs\decisions\ADR_0.md"
Remove-Item -Force ".ares\ares.db" -ErrorAction SilentlyContinue
& $ares_exe ingest .
Run-Check -expected "HardFail"
# restore
Set-Content -Path "docs\decisions\ADR_0.md" -Value "# ADR-0"

Write-Host "Test 2: Delete Owner -> Expected: HardFail"
Remove-Item ".github\CODEOWNERS"
Remove-Item -Force ".ares\ares.db" -ErrorAction SilentlyContinue
& $ares_exe ingest .
Run-Check -expected "HardFail"
# restore
Set-Content -Path ".github\CODEOWNERS" -Value "* @eswar-426"

Write-Host "Test 3: Delete Requirement -> Expected: HardFail"
Remove-Item "docs\requirements\REQ_0.md"
Remove-Item -Force ".ares\ares.db" -ErrorAction SilentlyContinue
& $ares_exe ingest .
Run-Check -expected "HardFail"
# restore
Set-Content -Path "docs\requirements\REQ_0.md" -Value "# REQ-0"

Write-Host "Test 4: Coverage Drop (SoftFail)"
# Add just 1 code file to a repo of 25 files. Coverage drop will be 1/26 = 3.8% drop (< 5%)
Set-Content -Path "src\extra_soft.rs" -Value "fn extra() {}"
Remove-Item -Force ".ares\ares.db" -ErrorAction SilentlyContinue
& $ares_exe ingest .
Run-Check -expected "SoftFail"

Write-Host "Test 5: Coverage Drop (HardFail)"
# Add even more code to cause severe coverage drop
for ($i=5; $i -lt 50; $i++) {
    Set-Content -Path "src\extra_$i.rs" -Value "fn extra() {}"
}
Remove-Item -Force ".ares\ares.db" -ErrorAction SilentlyContinue
& $ares_exe ingest .
Run-Check -expected "HardFail"

Write-Host "All regression tests passed successfully!"
