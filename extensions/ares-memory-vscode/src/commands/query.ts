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
export function parseAresResponse(result: any, filePath?: string): AresResponse {
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

    // Map ares_generate_context_file — result is a file path string
    if (typeof raw.result === 'string' && (raw.result as string).endsWith('.md')) {
        raw.query_type = 'context_file';
        raw.answer = 'Context file written to: ' + raw.result;
        raw.file_path = raw.result;
    }

    // Map ares_dead_code — fields are at top level, no result wrapper
    if (raw.dead_files !== undefined) {
        raw.query_type = 'dead_code';
        raw.answer = `${raw.total_dead_files || 0} dead files, ${raw.total_dead_functions || 0} dead functions detected.`;
    }

    // Map ares_who_owns — contributors in result wrapper
    if (raw.query_type === 'who_owns' ||
        (raw.result && Array.isArray(raw.result.contributors)) ||
        (raw.result && Array.isArray(raw.result) && raw.result.length > 0 && raw.result[0].owner)) {
        raw.query_type = 'who_owns';
        raw.owners = raw.result.contributors ? raw.result : raw.result;
    }

    // Map ares_decisions nested result to flat structure
    if (raw.query_type === 'decisions' ||
        (raw.result && Array.isArray(raw.result.decisions)) ||
        (raw.result && Array.isArray(raw.result) && raw.result.length > 0 && raw.result[0].summary)) {
        raw.query_type = 'decisions';
        raw.related_decisions = raw.result.decisions || raw.result || [];
    }

    // Map ares_briefing nested result to flat structure for webview
    if (raw.result?.project !== undefined) {
        raw.query_type = 'briefing';
        raw.answer = raw.result.recommended_first_action || 'Briefing generated';
        raw.project = raw.result.project;
        raw.recent_activity = raw.result.recent_activity;
        raw.agent_handoff = raw.result.agent_handoff;
        raw.critical_gaps = raw.result.critical_gaps;
        raw.context_freshness_hours = raw.result.context_freshness_hours;
        raw.recommended_first_action = raw.result.recommended_first_action;
    }

    // Infer query_type from response content when not explicitly set
    if (!raw.query_type) {
        const ans = typeof raw.answer === 'string' ? raw.answer : '';
        if (ans.includes('**Purpose**') && raw.entity) {
            raw.query_type = 'why_exists';
        } else if (ans.includes('**Blast Radius**')) {
            raw.query_type = 'impact';
        } else if (ans.includes('**Drift Verdict**') || ans.includes('**Stability**')) {
            raw.query_type = 'drift';
        } else if (Array.isArray(raw.decisions) || Array.isArray(raw.related_decisions)) {
            raw.query_type = 'decisions';
        }
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
    paramKey: string = 'id',
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
        DiagnosticsPanel.logMcpTraffic('SEND', toolName, { [paramKey]: targetId });
        const result = await mcpClient.callTool(toolName, { [paramKey]: targetId });
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
    const simple: [string, string, string, string][] = [
        ['ares.whyExists',       'Why Exists',       'ares_why_exists',     'id'],
        ['ares.impactAnalysis',  'Impact Analysis',  'ares_impact',         'id'],
        ['ares.driftAnalysis',   'Drift Analysis',   'ares_drift',          'id'],
        ['ares.whoOwns',         'Who Owns This',    'ares_who_owns',       'file_path'],
        ['ares.decisions',       'Decisions',        'ares_decisions',      'project_id'],
    ];

    for (const [cmd, name, tool, paramKey] of simple) {
        context.subscriptions.push(
            vscode.commands.registerCommand(cmd, (uri?: vscode.Uri) => {
                runToolCommand(context, mcpClient, output, name, tool, paramKey, uri);
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

    // ── Briefing ──────────────────────────────────────────────
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.briefing', async () => {
            output.appendLine('\\n--- ARES Briefing ---');
            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_briefing', {});
                const response = parseAresResponse(result);
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Unable to generate briefing', detail: e.message });
            }
        })
    );

    // ── Dead Code ────────────────────────────────────────────
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.findDeadCode', async () => {
            output.appendLine('\\n--- ARES Dead Code ---');
            const thresholdStr = await vscode.window.showInputBox({
                prompt: 'Threshold in days (files older than this are candidates)',
                placeHolder: '30',
                validateInput: (v) => {
                    const n = parseInt(v);
                    return (isNaN(n) || n < 1) ? 'Must be a positive number' : undefined;
                }
            });
            const args: any = {};
            if (thresholdStr) { args.threshold_days = parseInt(thresholdStr); }
            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_dead_code', args);
                const response = parseAresResponse(result);
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Unable to find dead code', detail: e.message });
            }
        })
    );

    // ── Generate Context File (CLAUDE.md) ─────────────────────
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.generateContextFile', async () => {
            output.appendLine('\\n--- Generating Context File ---');
            const outputPath = await vscode.window.showInputBox({
                prompt: 'Output path for CLAUDE.md (leave empty for default .ares/CLAUDE.md)',
                placeHolder: '.ares/CLAUDE.md'
            });
            const args: any = {};
            if (outputPath && outputPath.trim()) { args.output_path = outputPath.trim(); }
            const panel = AresQueryPanel.showLoading(context);
            try {
                const t = Date.now();
                const result = await mcpClient.callTool('ares_generate_context_file', args);
                const response = parseAresResponse(result);
                response.execution_time_ms = Date.now() - t;
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                AresQueryPanel.showError(context, { message: 'Unable to generate context file', detail: e.message });
            }
        })
    );

}