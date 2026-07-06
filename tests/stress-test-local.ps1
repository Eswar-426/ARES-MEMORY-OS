#!/usr/bin/env pwsh
<#
.SYNOPSIS
    ARES Memory OS - Local Repository Stress Test
.DESCRIPTION
    Scans all git repos found in a directory tree, runs ares-cli,
    measures ingest, DB size, and validates queries.
.PARAMETER ReposDir
    Directory containing git repositories (default: datasets/repositories)
.PARAMETER ReportPath
    Output report path
.PARAMETER AresCli
    Path to ares-cli binary
.PARAMETER AresMcp
    Path to ares-mcp binary
.PARAMETER Filter
    Only test repos matching this name pattern (e.g., "tokio" or "axum")
.PARAMETER SkipQuery
    Skip MCP query tests
.PARAMETER KeepDbs
    Don't delete .ares directories after testing
#>

param(
    [string]$ReposDir = "datasets/repositories",
    [string]$ReportPath = "tests/stress-test-local-report.md",
    [string]$AresCli = "ares-cli",
    [string]$AresMcp = "ares-mcp",
    [string]$Filter = "",
    [switch]$SkipQuery,
    [switch]$KeepDbs
)

$ErrorActionPreference = "Continue"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# ═══════════════════════════════════════════════════════════════
#  UTILITIES
# ═══════════════════════════════════════════════════════════════

function Write-Section($title) {
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "  $title" -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════════" -ForegroundColor Cyan
}
function Write-Ok($msg) { Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "  [FAIL] $msg" -ForegroundColor Red }
function Write-Warn($msg) { Write-Host "  [WARN]  $msg" -ForegroundColor Yellow }
function Write-Info($msg) { Write-Host "  [INFO]  $msg" -ForegroundColor Gray }
function Format-Bytes($bytes) {
    if ($bytes -gt 1MB) { "{0:N1} MB" -f ($bytes / 1MB) }
    elseif ($bytes -gt 1KB) { "{0:N1} KB" -f ($bytes / 1KB) }
    else { "$bytes B" }
}
function Format-Duration($ms) {
    if ($ms -gt 60000) { "{0:N1} min" -f ($ms / 60000) }
    elseif ($ms -gt 1000) { "{0:N1} sec" -f ($ms / 1000) }
    else { "$ms ms" }
}

# ═══════════════════════════════════════════════════════════════
#  PHASE 0: Pre-flight
# ═══════════════════════════════════════════════════════════════

Write-Section "PHASE 0: Pre-flight Checks"

$cliExists = Get-Command $AresCli -ErrorAction SilentlyContinue
if (-not $cliExists) { Write-Fail "$AresCli not found in PATH"; exit 1 }
else { Write-Ok "$AresCli at $($cliExists.Source)" }

$mcpExists = Get-Command $AresMcp -ErrorAction SilentlyContinue
if ($mcpExists) { Write-Ok "$AresMcp at $($mcpExists.Source)" }
else { Write-Warn "$AresMcp not found - query tests will be skipped" }

$sqliteExists = Get-Command sqlite3 -ErrorAction SilentlyContinue
if ($sqliteExists) { Write-Ok "sqlite3 at $($sqliteExists.Source)" }
else { Write-Warn "sqlite3 not found - DB stats will be limited" }

# ═══════════════════════════════════════════════════════════════
#  PHASE 1: Discover Repos
# ═══════════════════════════════════════════════════════════════

Write-Section "PHASE 1: Discovering Repositories"

$resolvedDir = (Resolve-Path $ReposDir -ErrorAction SilentlyContinue).Path
if (-not $resolvedDir) {
    Write-Fail "Directory not found: $ReposDir"
    exit 1
}
Write-Info "Scanning: $resolvedDir"

$repoDirs = Get-ChildItem -Path $resolvedDir -Directory | Where-Object {
    Test-Path (Join-Path $_.FullName ".git")
}

if ($Filter) {
    $repoDirs = $repoDirs | Where-Object { $_.Name -match $Filter }
}

if ($repoDirs.Count -eq 0) {
    Write-Fail "No git repositories found"
    exit 1
}

Write-Ok "Found $($repoDirs.Count) repositories:"
$repoDirs | ForEach-Object {
    $fileCount = (Get-ChildItem -Path $_.FullName -Recurse -File -ErrorAction SilentlyContinue).Count
    $size = (Get-ChildItem -Path $_.FullName -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    Write-Host "    $($_.Name) - $fileCount files, $(Format-Bytes $size)" -ForegroundColor Gray
}

# ═══════════════════════════════════════════════════════════════
#  PHASE 2: Scan Each Repo
# ═══════════════════════════════════════════════════════════════

Write-Section "PHASE 2: Scanning Repositories"

$Results = @()

foreach ($dir in $repoDirs) {
    $repoName = $dir.Name
    $repoPath = $dir.FullName

    Write-Host ""
    Write-Host "  ── $repoName ──" -ForegroundColor Yellow

    # Detect language from file extensions
    $extStats = @{}
    Get-ChildItem -Path $repoPath -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
        $ext = $_.Extension.TrimStart('.').ToLower()
        if ($ext) {
            if ($null -ne $extStats[$ext]) { $extStats[$ext]++ } else { $extStats[$ext] = 1 }
        }
    }
    $topLang = ($extStats.GetEnumerator() | Sort-Object Value -Descending | Select-Object -First 3 | ForEach-Object { "$($_.Key)($($_.Value))" }) -join ", "

    # Remove old .ares
    $aresDir = Join-Path $repoPath ".ares"
    if (Test-Path $aresDir) { Remove-Item $aresDir -Recurse -Force }

    # Run scan
    Write-Host "  Scanning..." -NoNewline
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    Push-Location $repoPath
    & $AresCli scan 2>&1 | Tee-Object -Variable scanOutput | Out-Null
    $exitCode = $LASTEXITCODE
    Pop-Location
    $sw.Stop()

    $dbPath = Join-Path $aresDir "ares.db"

    if ($exitCode -ne 0) {
        Write-Fail "CRASH (exit $exitCode) in $(Format-Duration $sw.ElapsedMilliseconds)"
        $scanOutput | Select-Object -Last 15 | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
        $Results += [PSCustomObject]@{
            Repo = $repoName; Lang = $topLang; Path = $repoPath
            IngestResult = "CRASH (exit $exitCode)"; IngestTime = $sw.ElapsedMilliseconds
            DbSize = 0; NodeCount = 0; EdgeCount = 0; CommitCount = 0
            FileCount = 0; StructCount = 0; FuncCount = 0
            WhyExistsResult = "SKIP"; ImpactResult = "SKIP"; DriftResult = "SKIP"
            ErrorSnippet = ($scanOutput | Select-Object -Last 5) -join "`n"
            DbPath = $null
        }
        continue
    }

    Write-Ok "Done in $(Format-Duration $sw.ElapsedMilliseconds)"

    # Query DB stats
    $dbSize = 0; $nodeCount = 0; $edgeCount = 0; $commitCount = 0
    $fileCount = 0; $structCount = 0; $funcCount = 0

    if (Test-Path $dbPath) {
        $dbSize = (Get-Item $dbPath).Length
        if ($sqliteExists) {
            $nodeCount = [int](& sqlite3 $dbPath "SELECT COUNT(*) FROM graph_nodes" 2>$null)
            $edgeCount = [int](& sqlite3 $dbPath "SELECT COUNT(*) FROM graph_edges" 2>$null)
            $commitCount = [int](& sqlite3 $dbPath "SELECT COUNT(*) FROM graph_nodes WHERE node_type='commit'" 2>$null)
            $fileCount = [int](& sqlite3 $dbPath "SELECT COUNT(*) FROM graph_nodes WHERE node_type='file'" 2>$null)
            $structCount = [int](& sqlite3 $dbPath "SELECT COUNT(*) FROM graph_nodes WHERE node_type='struct'" 2>$null)
            $funcCount = [int](& sqlite3 $dbPath "SELECT COUNT(*) FROM graph_nodes WHERE node_type='function'" 2>$null)
        }
    } else {
        Write-Fail "Database not created"
    }

    Write-Info "DB: $(Format-Bytes $dbSize) | Files: $fileCount | Structs: $structCount | Funcs: $funcCount | Commits: $commitCount"

    $Results += [PSCustomObject]@{
        Repo = $repoName; Lang = $topLang; Path = $repoPath
        IngestResult = "OK"; IngestTime = $sw.ElapsedMilliseconds
        DbSize = $dbSize; NodeCount = $nodeCount; EdgeCount = $edgeCount; CommitCount = $commitCount
        FileCount = $fileCount; StructCount = $structCount; FuncCount = $funcCount
        WhyExistsResult = "PENDING"; ImpactResult = "PENDING"; DriftResult = "PENDING"
        ErrorSnippet = ""; DbPath = $dbPath
    }
}

# ═══════════════════════════════════════════════════════════════
#  PHASE 3: Query Tests
# ═══════════════════════════════════════════════════════════════

if ($SkipQuery -or -not $mcpExists) {
    Write-Section "PHASE 3: Skipped"
} else {
    Write-Section "PHASE 3: Intelligence Query Tests"

    function Invoke-McpQuery($dbPath, $tool, $args) {
        $body = @{
            jsonrpc = "2.0"; id = [Guid]::NewGuid().ToString("N")
            method = "tools/call"
            params = @{ name = $tool; arguments = $args }
        } | ConvertTo-Json -Depth 5

        Push-Location (Split-Path (Split-Path $dbPath))
        $psi = [System.Diagnostics.ProcessStartInfo]::new()
        $psi.FileName = (Get-Command $AresMcp).Source
        $psi.Arguments = "--db `"$dbPath`""
        $psi.WorkingDirectory = (Split-Path (Split-Path $dbPath))
        $psi.RedirectStandardInput = $true
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        $psi.UseShellExecute = $false
        $psi.CreateNoWindow = $true

        $proc = [System.Diagnostics.Process]::Start($psi)

        $init = @{ jsonrpc = "2.0"; id = "1"; method = "initialize"; params = @{
            protocolVersion = "2024-11-05"; capabilities = @{}; clientInfo = @{ name = "stress-test"; version = "1.0" }
        }} | ConvertTo-Json -Depth 3
        $proc.StandardInput.WriteLine($init)
        $notif = @{ jsonrpc = "2.0"; method = "notifications/initialized" } | ConvertTo-Json
        $proc.StandardInput.WriteLine($notif)
        Start-Sleep -Milliseconds 500
        $proc.StandardInput.WriteLine($body)
        $proc.StandardInput.Close()
        
        # Wait a short time for the LLM/Engine to process
        $proc.WaitForExit(8000) | Out-Null
        if (-not $proc.HasExited) { $proc.Kill() }
        
        $output = $proc.StandardOutput.ReadToEnd()
        $stderr = $proc.StandardError.ReadToEnd()
        Pop-Location

        if ($output) {
            try {
                $resp = $output | ConvertFrom-Json
                if ($resp.result.content) { return $resp.result.content[0].text }
                if ($resp.error) { return "MCP ERROR: $($resp.error.message)" }
                return "EMPTY RESPONSE"
            } catch { return "PARSE ERROR" }
        }
        if ($stderr) { return "STDERR: $($stderr.Substring(0, [Math]::Min(300, $stderr.Length)))" }
        return "NO OUTPUT"
    }

    function Get-TestFile($dbPath) {
        if (-not $sqliteExists) { return "README.md" }
        # Pick a real source file with symbols
        $file = & sqlite3 $dbPath "SELECT file_path FROM graph_nodes WHERE node_type='file' AND file_path LIKE '%.rs' LIMIT 1" 2>$null
        if ($file) { return $file.Trim() }
        $file = & sqlite3 $dbPath "SELECT file_path FROM graph_nodes WHERE node_type='file' AND file_path LIKE '%.py' LIMIT 1" 2>$null
        if ($file) { return $file.Trim() }
        $file = & sqlite3 $dbPath "SELECT file_path FROM graph_nodes WHERE node_type='file' AND file_path LIKE '%.ts' LIMIT 1" 2>$null
        if ($file) { return $file.Trim() }
        $file = & sqlite3 $dbPath "SELECT file_path FROM graph_nodes WHERE node_type='file' LIMIT 1" 2>$null
        if ($file) { return $file.Trim() }
        return "README.md"
    }

    foreach ($r in $Results) {
        if ($r.WhyExistsResult -eq "SKIP" -or -not $r.DbPath) { continue }

        Write-Host ""
        Write-Host "  ── $($r.Repo) queries ──" -ForegroundColor Yellow

        $testFile = Get-TestFile $r.DbPath
        Write-Info "Test file: $testFile"

        # Why Exists
        Write-Host "  Why Exists..." -NoNewline
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $res = Invoke-McpQuery -dbPath $r.DbPath -tool "ares_why_exists" -args @{ target_path = $testFile }
        $sw.Stop()
        if ($res -match "ERROR|STDERR|PARSE|EMPTY|NO OUTPUT") { Write-Fail "$(Format-Duration $sw.ElapsedMilliseconds) - $res"; $r.WhyExistsResult = "FAIL" }
        elseif ($res.Length -lt 30) { Write-Warn "$(Format-Duration $sw.ElapsedMilliseconds) - weak ($($res.Length) chars)"; $r.WhyExistsResult = "WEAK" }
        else { Write-Ok "$(Format-Duration $sw.ElapsedMilliseconds) - $($res.Length) chars"; $r.WhyExistsResult = "OK" }

        # Impact
        Write-Host "  Impact..." -NoNewline
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $res = Invoke-McpQuery -dbPath $r.DbPath -tool "ares_impact" -args @{ target_path = $testFile }
        $sw.Stop()
        if ($res -match "ERROR|STDERR|PARSE|EMPTY|NO OUTPUT") { Write-Fail "$(Format-Duration $sw.ElapsedMilliseconds) - $res"; $r.ImpactResult = "FAIL" }
        elseif ($res.Length -lt 30) { Write-Warn "$(Format-Duration $sw.ElapsedMilliseconds) - weak ($($res.Length) chars)"; $r.ImpactResult = "WEAK" }
        else { Write-Ok "$(Format-Duration $sw.ElapsedMilliseconds) - $($res.Length) chars"; $r.ImpactResult = "OK" }

        # Drift
        Write-Host "  Drift..." -NoNewline
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $res = Invoke-McpQuery -dbPath $r.DbPath -tool "ares_drift" -args @{ target_path = $testFile }
        $sw.Stop()
        if ($res -match "ERROR|STDERR|PARSE|EMPTY|NO OUTPUT") { Write-Fail "$(Format-Duration $sw.ElapsedMilliseconds) - $res"; $r.DriftResult = "FAIL" }
        elseif ($res.Length -lt 30) { Write-Warn "$(Format-Duration $sw.ElapsedMilliseconds) - weak ($($res.Length) chars)"; $r.DriftResult = "WEAK" }
        else { Write-Ok "$(Format-Duration $sw.ElapsedMilliseconds) - $($res.Length) chars"; $r.DriftResult = "OK" }
    }
}

# ═══════════════════════════════════════════════════════════════
#  PHASE 4: Report
# ═══════════════════════════════════════════════════════════════

Write-Section "PHASE 4: Report"

$report = @()
$report += "# ARES Local Stress Test Report"
$report += ""
$report += "**Date:** $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
$report += "**Repos scanned:** $($Results.Count)"
$report += ""
$report += "| Repo | Lang | Ingest | Time | DB | Files | Structs | Funcs | Commits | Why | Impact | Drift |"
$report += "|------|------|--------|------|-----|-------|---------|-------|---------|-----|--------|-------|"

$pass = 0; $fail = 0
foreach ($r in $Results) {
    $ing = if ($r.IngestResult -eq "OK") { "[OK]" } else { "[FAIL]" }
    $why = if ($r.WhyExistsResult -match "^OK") { "[OK]" } elseif ($r.WhyExistsResult -eq "SKIP") { "[SKIP]" } else { "[FAIL]" }
    $imp = if ($r.ImpactResult -match "^OK") { "[OK]" } elseif ($r.ImpactResult -eq "SKIP") { "[SKIP]" } else { "[FAIL]" }
    $dft = if ($r.DriftResult -match "^OK") { "[OK]" } elseif ($r.DriftResult -eq "SKIP") { "[SKIP]" } else { "[FAIL]" }

    if ($r.IngestResult -eq "OK" -and $r.WhyExistsResult -match "^OK" -and $r.ImpactResult -match "^OK") { $pass++ } else { $fail++ }

    $report += "| $($r.Repo) | $($r.Lang) | $ing | $(Format-Duration $r.IngestTime) | $(Format-Bytes $r.DbSize) | $($r.FileCount) | $($r.StructCount) | $($r.FuncCount) | $($r.CommitCount) | $why | $imp | $dft |"
}

$report += ""
$report += "**$pass PASSED / $fail FAILED**"
$report += ""
$report += "---"
$report += ""

# Per-repo details
foreach ($r in $Results) {
    $report += "## $($r.Repo)"
    $report += "- **Path:** $($r.Path)"
    $report += "- **Language:** $($r.Lang)"
    $report += "- **Ingest:** $($r.IngestResult) ($(Format-Duration $r.IngestTime))"
    $report += "- **DB:** $(Format-Bytes $r.DbSize)"
    $report += "- **Nodes:** $($r.NodeCount) (files: $($r.FileCount), structs: $($r.StructCount), funcs: $($r.FuncCount), commits: $($r.CommitCount))"
    $report += "- **Edges:** $($r.EdgeCount)"
    $report += "- **Why Exists:** $($r.WhyExistsResult)"
    $report += "- **Impact:** $($r.ImpactResult)"
    $report += "- **Drift:** $($r.DriftResult)"
    if ($r.ErrorSnippet) {
        $report += "- **Error:**"
        $report += '```'
        $report += $r.ErrorSnippet
        $report += '```'
    }
    $report += ""
}

New-Item -ItemType Directory -Force -Path (Split-Path $ReportPath) | Out-Null
$report | Set-Content $ReportPath -Encoding UTF8
Write-Ok "Report: $ReportPath"

# Cleanup
if (-not $KeepDbs) {
    Write-Section "Cleanup"
    foreach ($r in $Results) {
        $aresDir = Join-Path $r.Path ".ares"
        if (Test-Path $aresDir) {
            Remove-Item $aresDir -Recurse -Force
            Write-Info "Removed $aresDir"
        }
    }
}

# Verdict
Write-Section "VERDICT"
if ($fail -eq 0) { Write-Ok "ALL $pass PASSED"; exit 0 }
else { Write-Fail "$fail FAILED / $pass PASSED"; exit 1 }

