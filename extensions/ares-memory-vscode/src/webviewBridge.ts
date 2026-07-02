import * as vscode from 'vscode';

export class WebviewBridge {
    private ready = false;
    private pendingMessages: any[] = [];
    private panel: vscode.WebviewPanel;

    constructor(panel: vscode.WebviewPanel) {
        this.panel = panel;
    }

    public send(message: any) {
        if (!this.ready) {
            this.pendingMessages.push(message);
            return;
        }

        this.panel.webview.postMessage(message);
    }

    public markReady() {
        this.ready = true;
        for (const msg of this.pendingMessages) {
            this.panel.webview.postMessage(msg);
        }
        this.pendingMessages.length = 0;
    }
}
