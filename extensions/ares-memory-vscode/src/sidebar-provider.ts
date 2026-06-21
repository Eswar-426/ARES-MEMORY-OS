import * as vscode from 'vscode';

export class SidebarProvider implements vscode.WebviewViewProvider {
    _view?: vscode.WebviewView;
    _doc?: vscode.TextDocument;

    constructor(private readonly _extensionUri: vscode.Uri) {}

    public resolveWebviewView(webviewView: vscode.WebviewView) {
        this._view = webviewView;

        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this._extensionUri],
        };

        webviewView.webview.html = this._getHtmlForWebview(webviewView.webview);

        webviewView.webview.onDidReceiveMessage(async (data) => {
            switch (data.type) {
                case 'onInfo': {
                    if (!data.value) {
                        return;
                    }
                    vscode.window.showInformationMessage(data.value);
                    break;
                }
                case 'onError': {
                    if (!data.value) {
                        return;
                    }
                    vscode.window.showErrorMessage(data.value);
                    break;
                }
                case 'searchMemory': {
                    try {
                        const results = { error: "Search disabled in this version. Use Command Palette." };
                        this._view?.webview.postMessage({ type: 'searchResults', value: results });
                    } catch (err: any) {
                        vscode.window.showErrorMessage("ARES Search Failed: " + err.message);
                    }
                    break;
                }
            }
        });
    }

    public refresh() {
        if (this._view) {
            this._view.webview.html = this._getHtmlForWebview(this._view.webview);
        }
    }

    private _getHtmlForWebview(webview: vscode.Webview) {
        return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>ARES Memory OS</title>
                <style>
                    body {
                        font-family: var(--vscode-font-family);
                        color: var(--vscode-editor-foreground);
                        padding: 10px;
                    }
                    button {
                        background: var(--vscode-button-background);
                        color: var(--vscode-button-foreground);
                        border: none;
                        padding: 8px 12px;
                        margin-top: 10px;
                        width: 100%;
                        cursor: pointer;
                    }
                    button:hover {
                        background: var(--vscode-button-hoverBackground);
                    }
                    input {
                        background: var(--vscode-input-background);
                        color: var(--vscode-input-foreground);
                        border: 1px solid var(--vscode-input-border);
                        padding: 8px;
                        width: 100%;
                        box-sizing: border-box;
                    }
                    .card {
                        background: var(--vscode-editorWidget-background);
                        border: 1px solid var(--vscode-widget-border);
                        padding: 10px;
                        margin-top: 10px;
                        border-radius: 4px;
                    }
                </style>
            </head>
            <body>
                <h2>ARES Memory</h2>
                <div class="card">
                    <h3>Search Project Memory</h3>
                    <input type="text" id="searchInput" placeholder="Ask ARES..." />
                    <button id="searchBtn">Search</button>
                </div>
                
                <div class="card">
                    <h3>Actions</h3>
                    <button id="snapshotBtn">Generate Project Snapshot</button>
                    <button id="contextBtn">Load AI Context</button>
                </div>

                <div id="results"></div>

                <script>
                    const vscode = acquireVsCodeApi();
                    
                    document.getElementById('searchBtn').addEventListener('click', () => {
                        const val = document.getElementById('searchInput').value;
                        vscode.postMessage({ type: 'searchMemory', value: val });
                    });

                    window.addEventListener('message', event => {
                        const message = event.data;
                        switch (message.type) {
                            case 'searchResults':
                                document.getElementById('results').innerHTML = '<pre>' + JSON.stringify(message.value, null, 2) + '</pre>';
                                break;
                        }
                    });
                </script>
            </body>
            </html>`;
    }
}
