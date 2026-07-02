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
