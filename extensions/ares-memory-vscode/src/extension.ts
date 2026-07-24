import * as path from 'path';
import * as fs from 'fs';
import * as vscode from 'vscode';
import { McpClient } from './mcp-client';
import { resolveAresCli, resolveAresMcp, ResolvedBinary } from './binary-discovery';
import { RepositoryWatcher } from './watcher';
import { RequestManager } from './requestManager';
import { registerGraphCommand } from './commands/graph';
import { registerCliCommands } from './commands/cli';
import { registerQueryCommands, parseAresResponse } from './commands/query';
import { AresQueryPanel } from './queryPanel';
import { registerDashboardCommand } from './commands/dashboard';
import { registerHealthCommands } from './commands/health';
import { registerDiagnosticsCommand } from './diagnosticsPanel';
import { recordInlineDecision } from './commands/recordDecision';
import { ensureBinaries, getPlatformInfo } from './binaryDownloader';
import { setState, AresState } from './state';
let mcpClient: McpClient;
let requestManager: RequestManager;
let aresOutput: vscode.OutputChannel;
let aresCliCache: ResolvedBinary | undefined;
let aresMcpCache: ResolvedBinary | undefined;

export let aresStatusBar: vscode.StatusBarItem;

// ── Platform Binary Cleanup ──
// Runs once on activation. Deletes binaries for non-matching OS to save disk space.
// The VSIX ships windows/, linux/, darwin/ — user only needs one.
function cleanupNonPlatformBinaries(extensionPath: string): void {
    const fs = require('fs') as typeof import('fs');
    const path = require('path') as typeof import('path');

    const platform = process.platform; // 'win32' | 'linux' | 'darwin'
    const platformDir = platform === 'win32' ? 'windows' : platform; // our dir naming

    const binariesDir = path.join(extensionPath, 'binaries');
    if (!fs.existsSync(binariesDir)) return;

    const entries = fs.readdirSync(binariesDir);
    for (const entry of entries) {
        // Skip the current platform's directory and any non-directory files
        if (entry === platformDir) continue;
        const fullPath = path.join(binariesDir, entry);
        if (!fs.statSync(fullPath).isDirectory()) continue;

        // Delete the entire directory for the other platform
        try {
            fs.rmSync(fullPath, { recursive: true, force: true });
            console.log(`[ARES] Cleaned up non-platform binaries: ${entry}/`);
        } catch (e) {
            console.error(`[ARES] Failed to clean up ${entry}/:`, e);
        }
    }
}

function configureMcpAccess(binaryPath: string, workspace: string): void {
    const fs = require('fs') as typeof import('fs');
    const path = require('path') as typeof import('path');
    const vscode = require('vscode') as typeof import('vscode');
    const os = require('os') as typeof import('os');

    console.log(`[ARES] Configuring MCP access for: ${binaryPath}`);

    // ── 1. VS Code Global Settings (for VS Code Copilot) ──
    try {
        const config = vscode.workspace.getConfiguration('ares');
        config.update('mcpPath', binaryPath, vscode.ConfigurationTarget.Global).then(() => {
            console.log('[ARES] Wrote ares.mcpPath to VS Code Global settings');
        }, () => {});
    } catch { /* no workspace during activation */ }

    // ── 2. Workspace .mcp.json (for Claude Code, Cursor, Windsurf) ──
    if (workspace) {
        const workspaceMcpPath = path.join(workspace, '.mcp.json');
        writeMcpJson(workspaceMcpPath, binaryPath, 'workspace .mcp.json');
    }

    // ── 3. .cursor/mcp.json (Cursor-specific, if .cursor/ exists) ──
    if (workspace) {
        const cursorDir = path.join(workspace, '.cursor');
        if (fs.existsSync(cursorDir)) {
            writeMcpJson(path.join(cursorDir, 'mcp.json'), binaryPath, '.cursor/mcp.json');
        }
    }

    // ── 4. Codex config.toml (OpenAI Codex) ──
    const codexDir = path.join(os.homedir(), '.codex');
    const codexTomlPath = path.join(codexDir, 'config.toml');
    if (fs.existsSync(codexDir)) {
        writeMcpToml(codexTomlPath, binaryPath);
    }

    // ── 5. Claude Desktop config.json (if exists) ──
    const claudeConfigDir = path.join(
        os.homedir(),
        process.platform === 'win32' ? 'AppData\\Roaming\\Claude' : '.config/Claude'
    );
    const claudeConfigPath = path.join(claudeConfigDir, 'claude_desktop_config.json');
    if (fs.existsSync(claudeConfigDir)) {
        writeMcpJson(claudeConfigPath, binaryPath, 'Claude Desktop config', true);
    }

    // ── 6. Antigravity IDE config (with cwd for workspace detection) ──
    const antigravityConfigDir = path.join(os.homedir(), '.gemini', 'config');
    const antigravityConfigPath = path.join(antigravityConfigDir, 'mcp_config.json');
    writeMcpJson(antigravityConfigPath, binaryPath, 'Antigravity IDE config', true, workspace);
}

function writeMcpJson(
    filePath: string,
    binaryPath: string,
    label: string,
    merge: boolean = false,
    workspace?: string
): void {
    const fs = require('fs') as typeof import('fs');
    const path = require('path') as typeof import('path');
    const entry: any = { command: binaryPath, args: [] };
    if (workspace) {
        entry.cwd = workspace;
    }

    try {
        const dir = path.dirname(filePath);
        if (!fs.existsSync(dir)) {
            try { fs.mkdirSync(dir, { recursive: true }); } catch { return; }
        }

        if (fs.existsSync(filePath)) {
            const existing = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
            const currentAres = existing.mcpServers?.ares;
            if (currentAres?.command === binaryPath && currentAres?.cwd === (workspace || undefined)) {
                console.log(`[ARES] ${label} already configured: ${filePath}`);
                return; // Already correct, don't touch
            }
            if (merge || existing.mcpServers) {
                // Merge: keep other servers, update/add ares
                existing.mcpServers = existing.mcpServers || {};
                existing.mcpServers.ares = entry;
                fs.writeFileSync(filePath, JSON.stringify(existing, null, 2));
                console.log(`[ARES] Merged into ${label}: ${filePath}`);
            } else {
                // Different config entirely, don't overwrite
                console.log(`[ARES] ${label} exists with non-MCP content, skipping: ${filePath}`);
            }
        } else {
            // Fresh file
            const config = { mcpServers: { ares: entry } };
            fs.writeFileSync(filePath, JSON.stringify(config, null, 2));
            console.log(`[ARES] Wrote ${label}: ${filePath}`);
        }
    } catch (e) {
        console.error(`[ARES] Failed to write ${label}:`, e);
    }
}

function writeMcpToml(tomlPath: string, binaryPath: string): void {
    const fs = require('fs') as typeof import('fs');

    // Escape backslashes for TOML: C:\Users\ → C:\\Users\\
    const escapedPath = binaryPath.replace(/\\/g, '\\\\');

    // The TOML block to inject
    const aresBlock = `\n[mcp_servers.ares]\ncommand = "${escapedPath}"\nargs = []\n`;

    try {
        if (fs.existsSync(tomlPath)) {
            const content = fs.readFileSync(tomlPath, 'utf-8');

            // Already has ares entry?
            if (content.includes('[mcp_servers.ares]')) {
                // Check if command matches — if not, replace the block
                const aresRegex = /\[mcp_servers\.ares\]\ncommand\s*=\s*"[^"]*"\nargs\s*=\s*\[[^\]]*\]\n?/;
                if (aresRegex.test(content)) {
                    const updated = content.replace(aresRegex, aresBlock.trimStart());
                    fs.writeFileSync(tomlPath, updated);
                    console.log(`[ARES] Updated Codex config.toml: ${tomlPath}`);
                } else {
                    // Malformed existing entry — append after any [mcp_servers] section
                    console.log(`[ARES] Codex config.toml has malformed ares entry, appending corrected version`);
                    fs.appendFileSync(tomlPath, aresBlock);
                }
                return;
            }

            // Has [mcp_servers] section but no ares? Append inside it.
            if (content.includes('[mcp_servers]')) {
                // Insert after the [mcp_servers] header line
                const lines = content.split('\n');
                const idx = lines.findIndex(l => l.trim() === '[mcp_servers]');
                if (idx !== -1) {
                    lines.splice(idx + 1, 0, ...aresBlock.trimStart().split('\n'));
                    fs.writeFileSync(tomlPath, lines.join('\n'));
                    console.log(`[ARES] Inserted ares into existing [mcp_servers] in Codex config.toml`);
                    return;
                }
            }

            // No mcp_servers section at all — append at end
            fs.appendFileSync(tomlPath, aresBlock);
            console.log(`[ARES] Appended ares to Codex config.toml: ${tomlPath}`);
        }
        // If file doesn't exist, don't create it — Codex manages its own config
    } catch (e) {
        console.error(`[ARES] Failed to write Codex config.toml:`, e);
    }
}
function startBackgroundIngest(
    workspace: string,
    cliPath: string,
    aresDir: string,
    markerPath: string,
    output: vscode.OutputChannel
): void {
    try {
        const { spawn } = require('child_process') as typeof import('child_process');
        const child = spawn(cliPath, ['ingest', '.'], { cwd: workspace });

        if (!child.pid) {
            output.appendLine('Failed to start ingest process.');
            return;
        }

        let stderrBuf = '';
        child.stderr?.on('data', (data: Buffer) => {
            stderrBuf += data.toString();
            const lines = data.toString().split('\n');
            for (const line of lines) {
                if (line.includes('Progress:') || line.includes('Completed in')) {
                    output.appendLine(line.trim());
                }
            }
        });

        child.on('close', (code) => {
            if (code === 0) {
                output.appendLine('Ingest completed successfully.');
                try {
                    if (!fs.existsSync(aresDir)) {
                        fs.mkdirSync(aresDir, { recursive: true });
                    }
                    fs.writeFileSync(markerPath, new Date().toISOString());
                } catch (e: any) {
                    output.appendLine(`Warning: Could not write ingest marker: ${e}`);
                }
            } else {
                output.appendLine(`Ingest exited with code ${code}.`);
                if (stderrBuf) {
                    output.appendLine(stderrBuf.trimEnd());
                }
            }
        });
    } catch (e: any) {
        output.appendLine(`Ingest error: ${e.message}`);
    }
}

export async function activate(context: vscode.ExtensionContext) {
    aresOutput = vscode.window.createOutputChannel('ARES');
    aresOutput.appendLine('ARES Memory OS extension activating...\n');
    try {
    // Clean up binaries for other platforms (saves ~20-30MB)
    cleanupNonPlatformBinaries(context.extensionPath);
    
    aresStatusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    aresStatusBar.command = 'ares.healthCheck';
    aresStatusBar.text = '$(check) ARES: --';
    aresStatusBar.tooltip = 'ARES Repository Health';
    aresStatusBar.show();
    context.subscriptions.push(aresStatusBar);
    
    aresOutput.appendLine('--- ARES Startup Validation ---');

    // ── Resolve Binaries ─────────────────────────────────────
    let binariesEnsured = false;
    let binaryEnsureSource = 'none';
    try {
        const ensureResult = await ensureBinaries(context);
        binariesEnsured = true;
        binaryEnsureSource = ensureResult.source;
        aresOutput.appendLine(`Binary ensure: ${ensureResult.source} → ${ensureResult.path}`);
    } catch (e) {
        aresOutput.appendLine(`Auto-download failed: ${e}`);
        // Continue to fallback discovery
    }

    if (binariesEnsured) {
        const info = getPlatformInfo();
        const binDir = path.join(context.extensionPath, 'binaries', info.dir);
        const cliName = info.binaryName.replace('-mcp', ''); // 'ares.exe' or 'ares'
        aresCliCache = { path: path.join(binDir, cliName), source: binaryEnsureSource === 'bundled' ? 'Bundled' : 'Auto-Downloaded' };
        aresMcpCache = { path: path.join(binDir, info.binaryName), source: binaryEnsureSource === 'bundled' ? 'Bundled' : 'Auto-Downloaded' };
    } else {
        aresCliCache = await resolveAresCli(context);
        aresMcpCache = await resolveAresMcp(context);
    }
    if (aresCliCache) {
        aresOutput.appendLine(`✓ CLI:  ${aresCliCache.path}  (${aresCliCache.source})`);
    } else {
        aresOutput.appendLine('✗ CLI:  not found');
    }

    if (aresMcpCache) {
        aresOutput.appendLine(`✓ MCP:  ${aresMcpCache.path}  (${aresMcpCache.source})`);
    } else {
        aresOutput.appendLine('✗ MCP:  not found');
    }

    if (!aresCliCache || !aresMcpCache) {
        aresOutput.appendLine('\nActivation Status: ABORTED (Missing Binaries)');
        vscode.window.showErrorMessage(
            'ARES binaries (ares.exe, ares-mcp.exe) are missing. You need to build them.',
            'View Build Instructions'
        ).then(selection => {
            if (selection === 'View Build Instructions') {
                const instructions = `
# ARES Binaries Missing

The ARES extension requires the \`ares\` and \`ares-mcp\` binaries to function. These were not found in the extension folder or in your system PATH.

## How to Build

1. Open a terminal in the \`ARES_Memory_os\` repository root.
2. Run the packaging script:
   \`\`\`powershell
   .\\package.ps1
   \`\`\`
3. This will compile the Rust binaries in release mode and package the extension.

Alternatively, compile them manually:
\`\`\`bash
cargo build --release
\`\`\`
And copy the resulting executables from \`target/release/\` into the \`extensions/ares-memory-vscode/binaries/windows/\` folder.
`;
                vscode.workspace.openTextDocument({ content: instructions, language: 'markdown' })
                    .then(doc => vscode.window.showTextDocument(doc));
            }
        });
        return;
    }

    // ── Connect MCP ──────────────────────────────────────────
    const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!workspace) {
        aresOutput.appendLine('No workspace folder open. ARES requires an open workspace.');
        vscode.window.showErrorMessage('ARES requires an open workspace folder.');
        return;
    }

    // ── Auto-Initialize Repository ─────────────────────────
    const aresDir = path.join(workspace, '.ares');
    const aresDb = path.join(aresDir, 'ares.db');
    const ingestMarker = path.join(aresDir, '.ingest_complete');
    const needsIngest = !fs.existsSync(ingestMarker);

    if (needsIngest) {
        if (!aresCliCache) {
            aresOutput.appendLine('Workspace not initialized and ares CLI not found. Cannot auto-ingest.');
            vscode.window.showErrorMessage('ARES: Workspace not ingested. Please run `ares ingest .` manually.');
            return;
        }

        aresOutput.appendLine(`Workspace not initialized. Starting background ingest...`);
        aresOutput.show();
        startBackgroundIngest(workspace, aresCliCache.path, aresDir, ingestMarker, aresOutput);
    } else {
        aresOutput.appendLine(`Database found: ${aresDb}`);
        aresOutput.appendLine(`Checking database integrity...`);
        const { spawnSync } = require('child_process') as typeof import('child_process');
        const doctorResult = spawnSync(aresCliCache.path, ['doctor'], {
            cwd: workspace,
            encoding: 'utf-8',
            timeout: 10_000,
        });

        if (doctorResult.status !== 0 || (doctorResult.stdout && doctorResult.stdout.includes('(Corrupted)'))) {
            aresOutput.appendLine(`Database is corrupted. Output: ${doctorResult.stdout || doctorResult.stderr}`);
            vscode.window.showErrorMessage(
                'ARES database is corrupted. Would you like to rebuild it now?',
                'Rebuild Now'
            ).then(selection => {
                if (selection === 'Rebuild Now') {
                    try {
                        fs.rmSync(aresDir, { recursive: true, force: true });
                        aresOutput.appendLine(`Deleted corrupted database at ${aresDir}`);
                        vscode.commands.executeCommand('workbench.action.reloadWindow');
                    } catch (e: any) {
                        vscode.window.showErrorMessage(`Failed to delete corrupted database: ${e.message}`);
                    }
                }
            });
            return;
        } else {
            aresOutput.appendLine(`Database integrity OK.`);
        }
    }

    // ── Kill any orphaned MCP from a previous crash/uninstall ──
    McpClient.killOrphan(workspace, aresOutput);

    // ── Connect MCP ──────────────────────────────────────────
    aresOutput.appendLine('\n--- Connecting to ARES MCP ---');
    mcpClient = new McpClient(aresOutput);
    const connected = await mcpClient.connect(aresMcpCache.path, aresMcpCache.source);
    if (!connected) {
        aresOutput.appendLine('\nActivation Status: ABORTED (MCP Connection Failed)');
        vscode.window.showErrorMessage(`ARES MCP failed to connect: ${mcpClient.lastError}`);
        return;
    }

    // ── Write .mcp.json for zero-config IDE agent connection ──
    if (workspace && aresMcpCache?.path) {
        configureMcpAccess(aresMcpCache.path, workspace);
    }

    aresOutput.appendLine('\nActivation Status: READY\n');
    setState(AresState.READY);

    // ── Initialize Services ──────────────────────────────────
    requestManager = new RequestManager(mcpClient, aresOutput);

    const watcher = new RepositoryWatcher(aresOutput, aresCliCache);
    watcher.watch();

    // ── Register Commands ────────────────────────────────────
    registerGraphCommand(context, requestManager, aresOutput);
    registerCliCommands(context, aresOutput, aresCliCache, mcpClient);
    registerQueryCommands(context, mcpClient, aresOutput);
    registerDashboardCommand(context, mcpClient, aresOutput);
    registerHealthCommands(context, mcpClient, aresOutput);
    registerDiagnosticsCommand(context, mcpClient, aresOutput);
    context.subscriptions.push(vscode.commands.registerCommand('ares.recordDecision', async () => {
        await recordInlineDecision(context, mcpClient);
    }));

    context.subscriptions.push(
        vscode.commands.registerCommand('ares.architecture', async () => {
            aresOutput.appendLine('\n--- Architecture Map ---');
            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_architecture', {});
                const response = parseAresResponse(result);
                response.query_type = 'architecture';
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Architecture analysis failed', detail: e.message });
            }
        })
    );
    } catch (e: any) {
        aresOutput.appendLine(`Activation error: ${e.message}`);
        return;
    }
}

export function deactivate() {
    if (mcpClient) {
        mcpClient.disconnect();
    }
}
