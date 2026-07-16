import * as vscode from 'vscode';
import { McpClient } from '../mcp-client';
import { RequestManager } from '../requestManager';
import { AresQueryPanel, AresResponse } from '../queryPanel';
import { aresStatusBar } from '../extension';

export function registerHealthCommands(
    context: vscode.ExtensionContext,
    mcpClient: McpClient,
    output: vscode.OutputChannel,
): void {
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.healthCheck', async () => {
            output.appendLine('\n--- ARES Health Check ---');

            const panel = AresQueryPanel.showLoading(context);

            try {
                const projectId = RequestManager.getWorkspaceName();
                console.log("[Health] request start");
                output.appendLine("[Health] request start");
                
                const result = await mcpClient.callTool('ares_health_check', { project_id: projectId });

                console.log("[Health] MCP response");
                output.appendLine("[Health] MCP response");

                let rawResult: any = result;
                if (result.content?.[0]?.type === 'text') {
                    rawResult = JSON.parse(result.content[0].text);
                }
                
                console.log("[Health] parsed response\n", JSON.stringify(rawResult, null, 2));
                output.appendLine("[Health] parsed response\n" + JSON.stringify(rawResult, null, 2));

                const healthScore = Math.round(rawResult.health_score || 0);
                const scoreBreakdown = rawResult.score_breakdown || {};
                
                // Update status bar
                const icon = healthScore > 70 ? '$(check)' : (healthScore > 40 ? '$(warning)' : '$(error)');
                aresStatusBar.text = `${icon} ARES: ${healthScore}`;
                
                let tooltip = `ARES Repository Health: ${healthScore}/100\n`;
                if (scoreBreakdown.overall !== undefined) {
                    tooltip += `---\n`;
                    tooltip += `Files w/ Decisions (40%): ${Math.round(scoreBreakdown.files_with_decisions_term || 0)}%\n`;
                    tooltip += `Decisions w/ Reqs (30%): ${Math.round(scoreBreakdown.decisions_with_requirements_term || 0)}%\n`;
                    tooltip += `Files w/ Owners (20%): ${Math.round(scoreBreakdown.files_with_owners_term || 0)}%\n`;
                    tooltip += `Fresh Decisions (10%): ${Math.round(scoreBreakdown.fresh_decisions_term || 0)}%`;
                }
                aresStatusBar.tooltip = tooltip;

                const response: AresResponse = {
                    answer: 'ARES Repository Health',
                    confidence: 1.0,
                    evidence: [],
                    related_decisions: [],
                    query_type: 'healthCheck',
                    gaps: rawResult.gaps || [],
                    health_score: rawResult.health_score || 0,
                    score_breakdown: scoreBreakdown,
                    hotspots: rawResult.hotspots || [],
                    recent_queries: context.workspaceState.get('ares.recentQueries', []),
                    execution_time_ms: 0,
                };

                console.log("[Health] render start");
                output.appendLine("[Health] render start");
                AresQueryPanel.show(context, response);
                console.log("[Health] render complete");
                output.appendLine("[Health] render complete");
            } catch (e: any) {
                AresQueryPanel.showError(context, {
                    message: 'Unable to load ARES Health Check',
                    detail: e.message || 'An unexpected error occurred.',
                });
            }
        }),
        vscode.commands.registerCommand('ares.createDecisionFromGap', async (filePath: string, gapDescription: string) => {
            try {
                await mcpClient.callTool('ares_record_decision', {
                    title: `Decision needed: ${gapDescription}`,
                    description: `Identified by ARES health check: ${gapDescription}`,
                    status: 'draft',
                    impacted_paths: [filePath]
                });
                vscode.window.showInformationMessage('Decision draft created for ' + filePath);
                // Refresh health check automatically after creating the decision
                vscode.commands.executeCommand('ares.healthCheck');
            } catch (e: any) {
                vscode.window.showErrorMessage('Failed to create decision: ' + e.message);
            }
        })
    );
}
