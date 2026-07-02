import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

export interface GraphResponse {
    nodes: any[];
    edges: any[];
}

export class GraphPanel {
    public static currentPanel: GraphPanel | undefined;
    public static readonly viewType = 'aresGraphExplorer';

    private readonly panel: vscode.WebviewPanel;
    private readonly context: vscode.ExtensionContext;
    private disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, context: vscode.ExtensionContext) {
        this.panel = panel;
        this.context = context;

        this.panel.webview.html = this.getHtml(this.panel.webview, this.context.extensionUri);

        this.panel.onDidDispose(() => this.dispose(), null, this.disposables);
    }

    public static show(context: vscode.ExtensionContext): GraphPanel {
        if (GraphPanel.currentPanel) {
            GraphPanel.currentPanel.panel.reveal(vscode.ViewColumn.Beside);
            return GraphPanel.currentPanel;
        }

        const panel = vscode.window.createWebviewPanel(
            GraphPanel.viewType,
            'ARES Graph Explorer',
            vscode.ViewColumn.Beside,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
            }
        );

        GraphPanel.currentPanel = new GraphPanel(panel, context);
        return GraphPanel.currentPanel;
    }

    public get webview(): vscode.Webview {
        return this.panel.webview;
    }

    public dispose() {
        GraphPanel.currentPanel = undefined;
        this.panel.dispose();
        while (this.disposables.length) {
            const x = this.disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    private getHtml(webview: vscode.Webview, extensionUri: vscode.Uri): string {
        const htmlPath = path.join(extensionUri.fsPath, 'media', 'nebula-graph.html');
        let html = fs.readFileSync(htmlPath, 'utf8');
        
        // Inject the CSP source so the webview can load fonts/styles securely
        html = html.replace(/\$\{webview\.cspSource\}/g, webview.cspSource);
        
        return html;
    }
}