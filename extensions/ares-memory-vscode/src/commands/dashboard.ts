import * as vscode from 'vscode';
import { McpClient } from '../mcp-client';
import { RequestManager } from '../requestManager';
import { AresQueryPanel, AresResponse, AresError } from '../queryPanel';

/**
 * Register the ARES Home dashboard command.
 */
export function registerDashboardCommand(
    context: vscode.ExtensionContext,
    mcpClient: McpClient,
    output: vscode.OutputChannel,
): void {
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.aresHome', async () => {
            output.appendLine('\n--- ARES Home ---');

            const panel = AresQueryPanel.showLoading(context);

            try {
                const projectId = RequestManager.getWorkspaceName();
                console.log("[Dashboard] request start");
                output.appendLine("[Dashboard] request start");
                const result = await mcpClient.callTool('ares_dashboard', { project_id: projectId });

                console.log("[Dashboard] MCP response");
                output.appendLine("[Dashboard] MCP response");

                let rawResult: any = result;
                if (result.content?.[0]?.type === 'text') {
                    rawResult = JSON.parse(result.content[0].text);
                }
                
                console.log("[Dashboard] parsed response\n", JSON.stringify(rawResult, null, 2));
                output.appendLine("[Dashboard] parsed response\n" + JSON.stringify(rawResult, null, 2));

                const response: AresResponse = {
                    answer: 'ARES Home',
                    confidence: 1.0,
                    evidence: [],
                    related_decisions: [],
                    query_type: 'ARES Home',
                    dashboard: rawResult,
                    recent_queries: context.workspaceState.get('ares.recentQueries', []),
                    execution_time_ms: 0,
                };

                console.log("[Dashboard] render start");
                output.appendLine("[Dashboard] render start");
                AresQueryPanel.show(context, response);
                console.log("[Dashboard] render complete");
                output.appendLine("[Dashboard] render complete");
            } catch (e: any) {
                AresQueryPanel.showError(context, {
                    message: 'Unable to load ARES Home',
                    detail: e.message || 'An unexpected error occurred.',
                });
            }
        })
    );
}
