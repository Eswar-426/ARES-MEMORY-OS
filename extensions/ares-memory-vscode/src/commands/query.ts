import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { McpClient } from '../mcp-client';
import { RequestManager } from '../requestManager';
import { DiagnosticsPanel } from '../diagnosticsPanel';
import { AresQueryPanel, AresResponse, AresError } from '../queryPanel';

// ─────────────────────────────────────────────────────────────
// Shared MCP response parser
// ─────────────────────────────────────────────────────────────
function parseAresResponse(result: any, filePath?: string): AresResponse {
    let raw: any = {};

    if (result?.content && Array.isArray(result.content)) {
        for (const block of result.content) {
            if (block.type === 'text' && typeof block.text === 'string') {
                try { raw = JSON.parse(block.text); } catch { raw = { answer: block.text }; }
                break;
            }
        }
    } else if (typeof result === 'string') {
        try { raw = JSON.parse(result); } catch { raw = { answer: result }; }
    } else if (result && typeof result === 'object') {
        raw = result;
    }

    return {
        ...raw,
        answer: raw.answer ?? raw.explanation ?? '',
        confidence: (typeof raw.confidence === 'object' && raw.confidence !== null)
            ? raw.confidence
            : { score: typeof raw.confidence === 'number' ? raw.confidence : 0, reasons: [] },
        evidence: Array.isArray(raw.evidence) ? raw.evidence : [],
        related_decisions: Array.isArray(raw.related_decisions) ? raw.related_decisions : [],
        query_type: raw.query_type ?? '',
        file_path: filePath ?? raw.file_path,
    };
}

// ─────────────────────────────────────────────────────────────
// Resolve target ID from URI or prompt
// ─────────────────────────────────────────────────────────────
async function resolveTargetId(commandName: string, uri?: vscode.Uri): Promise<string | undefined> {
    if (uri) {
        const folders = vscode.workspace.workspaceFolders;
        if (folders) {
            const relative = vscode.workspace.asRelativePath(uri, false);
            if (relative && relative !== uri.fsPath && !path.isAbsolute(relative)) {
                return relative.replace(/\\/g, '/');
            }
        }
        return uri.fsPath.replace(/\\/g, '/');
    }
    return vscode.window.showInputBox({
        prompt: `Enter Target ID for ${commandName}`,
        placeHolder: 'e.g., src/main.rs or PROJ-001',
    });
}

// ─────────────────────────────────────────────────────────────
// Generic MCP tool command runner
// ─────────────────────────────────────────────────────────────
async function runToolCommand(
    context: vscode.ExtensionContext,
    mcpClient: McpClient,
    output: vscode.OutputChannel,
    commandName: string,
    toolName: string,
    uri?: vscode.Uri,
): Promise<void> {
    output.appendLine(`\n--- ${commandName} ---`);

    const targetId = await resolveTargetId(commandName, uri);
    if (!targetId) return;

    // Store recent query
    const recent: any[] = context.workspaceState.get('ares.recentQueries', []);
    recent.unshift({ command: commandName, target: targetId, timestamp: new Date().toISOString() });
    context.workspaceState.update('ares.recentQueries', recent.slice(0, 10));

    const panel = AresQueryPanel.showLoading(context);

    try {
        const t = Date.now();
        DiagnosticsPanel.logMcpTraffic('SEND', toolName, { id: targetId });
        const result = await mcpClient.callTool(toolName, { id: targetId });
        const response = parseAresResponse(result, targetId);
        DiagnosticsPanel.logMcpTraffic('RECEIVE', toolName, response);
        response.execution_time_ms = Date.now() - t;
        AresQueryPanel.show(context, response);
    } catch (e: any) {
        AresQueryPanel.showError(context, {
            message: 'Unable to retrieve repository memory',
            detail: e.message || 'An unexpected error occurred.',
        });
    }
}

// ─────────────────────────────────────────────────────────────
// Register all query commands
// ─────────────────────────────────────────────────────────────
export function registerQueryCommands(
    context: vscode.ExtensionContext,
    mcpClient: McpClient,
    output: vscode.OutputChannel,
): void {
    // Simple tool commands
    const simple: [string, string, string][] = [
        ['ares.whyExists',       'Why Exists',       'ares_why_exists'],
        ['ares.impactAnalysis',  'Impact Analysis',  'ares_impact'],
        ['ares.driftAnalysis',   'Drift Analysis',   'ares_drift'],
    ];

    for (const [cmd, name, tool] of simple) {
        context.subscriptions.push(
            vscode.commands.registerCommand(cmd, (uri?: vscode.Uri) => {
                runToolCommand(context, mcpClient, output, name, tool, uri);
            })
        );
    }

    // Traceability (needs depth parameter)
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.traceabilityAnalysis', async (uri?: vscode.Uri) => {
            output.appendLine('\n--- Traceability Analysis ---');

            const targetId = await resolveTargetId('Traceability', uri);
            if (!targetId) return;

            const depthStr = await vscode.window.showInputBox({ prompt: 'Traversal depth (default 3)', placeHolder: '3' });
            const depth = depthStr && !isNaN(parseInt(depthStr)) ? parseInt(depthStr) : 3;

            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_traceability', { entity_id: targetId, depth });
                const response = parseAresResponse(result, targetId);
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Unable to trace entity', detail: e.message });
            }
        })
    );

    // Coverage
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.coverageAnalysis', async () => {
            output.appendLine('\n--- Coverage Analysis ---');
            const projectId = await vscode.window.showInputBox({
                prompt: 'Enter Project ID', placeHolder: 'e.g., PROJ-001',
                value: RequestManager.getWorkspaceName(),
            });
            if (!projectId) return;

            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_coverage', { project_id: projectId });
                const response = parseAresResponse(result, projectId);
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Unable to analyze coverage', detail: e.message });
            }
        })
    );

    // Simulate Change
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.simulateChange', async (uri?: vscode.Uri) => {
            output.appendLine('\n--- Simulate Change ---');

            const targetId = await resolveTargetId('Simulate Removal', uri);
            if (!targetId) return;

            const projectId = await vscode.window.showInputBox({
                prompt: 'Project ID for simulation context', placeHolder: 'e.g., PROJ-001',
                value: RequestManager.getWorkspaceName(),
            });
            if (!projectId) return;

            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_simulate', { project_id: projectId, target_id: targetId, action: 'remove' });
                const response = parseAresResponse(result, targetId);
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Unable to simulate change', detail: e.message });
            }
        })
    );
}
