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

export async function activate(context: vscode.ExtensionContext) {
    aresOutput = vscode.window.createOutputChannel('ARES');
    aresOutput.appendLine('ARES Memory OS extension activating...\n');
    
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
    if (!fs.existsSync(aresDb)) {
        if (!aresCliCache) {
            aresOutput.appendLine('Workspace not initialized and ares CLI not found. Cannot auto-ingest.');
            vscode.window.showErrorMessage('ARES: Workspace not ingested. Please run `ares ingest .` manually.');
            return;
        }

        aresOutput.appendLine(`Workspace not initialized. Running: ${aresCliCache.path} ingest .`);
        aresOutput.show();

        const { spawnSync } = require('child_process') as typeof import('child_process');
        const result = spawnSync(aresCliCache.path, ['ingest', '.'], {
            cwd: workspace,
            encoding: 'utf-8',
            timeout: 300_000,
        });

        if (result.error) {
            aresOutput.appendLine(`Ingest failed: ${result.error.message}`);
            vscode.window.showErrorMessage(`ARES ingest failed: ${result.error.message}`);
            return;
        }

        if (result.status !== 0) {
            aresOutput.appendLine(`Ingest exited with code ${result.status}`);
            aresOutput.appendLine(result.stderr || result.stdout);
            vscode.window.showErrorMessage(`ARES ingest failed (exit code ${result.status}). Check ARES output channel.`);
            return;
        }

        aresOutput.appendLine('Ingest completed successfully.');
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
}

export function deactivate() {
    if (mcpClient) {
        mcpClient.disconnect();
    }
}
