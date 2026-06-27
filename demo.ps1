param(
    [Parameter(Mandatory=$false)]
    [string]$TargetRepo = "payment-service"
)

$ValidRepos = @("payment-service", "inventory-system", "auth-service")

if ($ValidRepos -notcontains $TargetRepo) {
    Write-Host "Error: TargetRepo must be one of: $($ValidRepos -join ', ')" -ForegroundColor Red
    exit 1
}

$DemoPath = Join-Path $PWD "demo-repositories\$TargetRepo"

Write-Host "`n🚀 ARES MemoryOS Demo Orchestrator" -ForegroundColor Cyan
Write-Host "====================================`n"

# 1. Ingestion
Write-Host "1. Ingesting $TargetRepo..." -ForegroundColor Yellow
$CliExe = Join-Path $PWD "target\debug\ares-cli.exe"
if (-not (Test-Path $CliExe)) {
    Write-Host "Building ares-cli..." -ForegroundColor DarkGray
    cargo build -p ares-cli
}

Set-Location $DemoPath
& $CliExe ingest

# 2. Benchmark
Write-Host "`n2. Benchmarking Graph..." -ForegroundColor Yellow
& $CliExe benchmark

# 3. Launch Extension
Write-Host "`n3. Launching VS Code & MCP Server..." -ForegroundColor Yellow
Write-Host "Opening $DemoPath in VS Code..."
code $DemoPath

Write-Host "`n🎉 Demo Environment Ready!" -ForegroundColor Green
Write-Host "============================"
Write-Host "Try the following queries in the ARES VS Code extension:`n"

if ($TargetRepo -eq "payment-service") {
    Write-Host "- Impact: `"What happens if I change the PaymentProvider trait?`"" -ForegroundColor Cyan
    Write-Host "- Traceability: `"Show me everything implementing REQ-12.`"" -ForegroundColor Cyan
}
elseif ($TargetRepo -eq "inventory-system") {
    Write-Host "- Drift: `"Are there any architecture violations of ADR-3?`"" -ForegroundColor Cyan
    Write-Host "- Simulation: `"Delete the DbConnection struct and simulate the impact.`"" -ForegroundColor Cyan
}
elseif ($TargetRepo -eq "auth-service") {
    Write-Host "- Why Exists: `"Why does constant_time_compare exist?`"" -ForegroundColor Cyan
}

Write-Host "`nNote: Ensure the MCP server is running in your VS Code extension host.`n"
