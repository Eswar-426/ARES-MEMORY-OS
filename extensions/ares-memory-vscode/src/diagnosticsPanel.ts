import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export class DiagnosticsPanel {
    public static currentPanel: DiagnosticsPanel | undefined;
    public static readonly viewType = 'aresDiagnostics';

    private readonly panel: vscode.WebviewPanel;
    private readonly context: vscode.ExtensionContext;
    private disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, context: vscode.ExtensionContext, mcpClient: any) {
        this.panel = panel;
        this.context = context;

        this.panel.webview.html = this.getHtml(this.panel.webview, this.context.extensionUri);

        this.panel.onDidDispose(() => this.dispose(), null, this.disposables);

        this.updateState(mcpClient);
    }

    public static show(context: vscode.ExtensionContext, mcpClient: any): DiagnosticsPanel {
        if (DiagnosticsPanel.currentPanel) {
            DiagnosticsPanel.currentPanel.panel.reveal(vscode.ViewColumn.Beside);
            return DiagnosticsPanel.currentPanel;
        }

        const panel = vscode.window.createWebviewPanel(
            DiagnosticsPanel.viewType,
            'ARES Diagnostics',
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'media')]
            }
        );

        DiagnosticsPanel.currentPanel = new DiagnosticsPanel(panel, context, mcpClient);
        return DiagnosticsPanel.currentPanel;
    }

    private async updateState(mcpClient: any) {
        const folders = vscode.workspace.workspaceFolders;
        const workspaceRoot = folders ? folders[0].uri.fsPath : 'No workspace open';
        const dbPath = folders ? path.join(workspaceRoot, '.ares', 'ares.db') : 'No DB';
        
        // Execute a simple MCP tool call to test connection
        let mcpTestResult = 'Not executed';
        try {
            if (mcpClient && typeof mcpClient.callTool === 'function') {
                // Testing with ares_dashboard since it's a simple, reliable command
                const result = await mcpClient.callTool('ares_dashboard', { project_id: 'default' });
                mcpTestResult = JSON.stringify(result, null, 2);
            } else {
                mcpTestResult = 'MCP Client not available';
            }
        } catch (e: any) {
            mcpTestResult = 'Error: ' + e.message;
        }

        this.panel.webview.postMessage({
            type: 'STATE_UPDATE',
            data: {
                workspaceRoot,
                dbPath,
                mcpTestResult
            }
        });
    }

    public static logMcpTraffic(direction: 'SEND' | 'RECEIVE', method: string, payload: any) {
        if (DiagnosticsPanel.currentPanel) {
            DiagnosticsPanel.currentPanel.panel.webview.postMessage({
                type: 'MCP_TRAFFIC',
                data: {
                    direction,
                    method,
                    payload: typeof payload === 'string' ? payload : JSON.stringify(payload, null, 2),
                    timestamp: new Date().toISOString()
                }
            });
        }
    }

    private getHtml(webview: vscode.Webview, extensionUri: vscode.Uri): string {
        const htmlPath = path.join(extensionUri.fsPath, 'media', 'diagnostics.html');
        let html = fs.readFileSync(htmlPath, 'utf8');
        html = html.replace(/\$\{webview\.cspSource\}/g, webview.cspSource);
        return html;
    }

    public dispose() {
        DiagnosticsPanel.currentPanel = undefined;
        this.panel.dispose();
        while (this.disposables.length) {
            const x = this.disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }
}

export function registerDiagnosticsCommand(context: vscode.ExtensionContext, mcpClient: any, output: vscode.OutputChannel) {
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.showDiagnostics', () => {
            output.appendLine('--- Opening ARES Diagnostics ---');
            DiagnosticsPanel.show(context, mcpClient);
        })
    );
}
