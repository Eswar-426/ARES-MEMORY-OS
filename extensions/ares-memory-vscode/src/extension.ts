import * as path from 'path';
import * as fs from 'fs';
import * as vscode from 'vscode';
import { McpClient } from './mcp-client';
import { resolveAresCli, resolveAresMcp, ResolvedBinary } from './binary-discovery';
import { RepositoryWatcher } from './watcher';
import { RequestManager } from './requestManager';
import { registerGraphCommand } from './commands/graph';
import { registerCliCommands } from './commands/cli';
import { registerQueryCommands } from './commands/query';
import { registerDashboardCommand } from './commands/dashboard';
import { registerDiagnosticsCommand } from './diagnosticsPanel';

let mcpClient: McpClient;
let requestManager: RequestManager;
let aresOutput: vscode.OutputChannel;
let aresCliCache: ResolvedBinary | undefined;
let aresMcpCache: ResolvedBinary | undefined;

export async function activate(context: vscode.ExtensionContext) {
    aresOutput = vscode.window.createOutputChannel('ARES');
    aresOutput.appendLine('ARES Memory OS extension activating...\n');
    aresOutput.appendLine('--- ARES Startup Validation ---');

    // ── Resolve Binaries ─────────────────────────────────────
    aresCliCache = await resolveAresCli(context);
    if (aresCliCache) {
        aresOutput.appendLine(`✓ CLI:  ${aresCliCache.path}  (${aresCliCache.source})`);
    } else {
        aresOutput.appendLine('✗ CLI:  not found');
    }

    aresMcpCache = await resolveAresMcp(context);
    if (aresMcpCache) {
        aresOutput.appendLine(`✓ MCP:  ${aresMcpCache.path}  (${aresMcpCache.source})`);
    } else {
        aresOutput.appendLine('✗ MCP:  not found');
    }

    if (!aresCliCache || !aresMcpCache) {
        aresOutput.appendLine('\nActivation Status: ABORTED (Missing Binaries)');
        vscode.window.showErrorMessage('ARES: Could not find CLI or MCP binary.');
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
            aresOutput.appendLine('Workspace not initialized and ares CLI not found. Cannot auto-scan.');
            vscode.window.showErrorMessage('ARES: Workspace not scanned. Please run `ares scan .` manually.');
            return;
        }

        aresOutput.appendLine(`Workspace not initialized. Running: ${aresCliCache.path} scan .`);
        aresOutput.show();

        const { spawnSync } = require('child_process') as typeof import('child_process');
        const result = spawnSync(aresCliCache.path, ['scan'], {
            cwd: workspace,
            encoding: 'utf-8',
            timeout: 120_000,
        });

        if (result.error) {
            aresOutput.appendLine(`Scan failed: ${result.error.message}`);
            vscode.window.showErrorMessage(`ARES scan failed: ${result.error.message}`);
            return;
        }

        if (result.status !== 0) {
            aresOutput.appendLine(`Scan exited with code ${result.status}`);
            aresOutput.appendLine(result.stderr || result.stdout);
            vscode.window.showErrorMessage(`ARES scan failed (exit code ${result.status}). Check ARES output channel.`);
            return;
        }

        aresOutput.appendLine('Scan completed successfully.');
    } else {
        aresOutput.appendLine(`Database found: ${aresDb}`);
    }

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

    // ── Initialize Services ──────────────────────────────────
    requestManager = new RequestManager(mcpClient, aresOutput);

    const watcher = new RepositoryWatcher(aresOutput, aresCliCache);
    watcher.watch();

    // ── Register Commands ────────────────────────────────────
    registerGraphCommand(context, requestManager, aresOutput);
    registerCliCommands(context, aresOutput, aresCliCache, mcpClient);
    registerQueryCommands(context, mcpClient, aresOutput);
    registerDashboardCommand(context, mcpClient, aresOutput);
    registerDiagnosticsCommand(context, mcpClient, aresOutput);
}

export function deactivate() {
    if (mcpClient) {
        mcpClient.disconnect();
    }
}
