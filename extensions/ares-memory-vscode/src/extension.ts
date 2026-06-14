import * as vscode from 'vscode';
import { SidebarProvider } from './sidebar-provider';
import { ApiClient } from './api-client';

export function activate(context: vscode.ExtensionContext) {
    console.log('ARES Memory OS extension is now active!');

    const apiClient = new ApiClient('http://localhost:3000/api/v1');
    const sidebarProvider = new SidebarProvider(context.extensionUri, apiClient);

    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider(
            "aresMemorySidebar",
            sidebarProvider
        )
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('ares.saveProjectMemory', async () => {
            vscode.window.showInformationMessage('ARES: Saving Project Memory...');
            try {
                await apiClient.saveMemory(vscode.workspace.name || "Unknown");
                vscode.window.showInformationMessage('ARES: Memory Saved Successfully');
            } catch (e: any) {
                vscode.window.showErrorMessage(`ARES Error: ${e.message}`);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('ares.restoreProjectMemory', async () => {
            vscode.window.showInformationMessage('ARES: Restoring Project Memory Context...');
            // This would normally fetch the context and display it in a webview or chat panel
            sidebarProvider.refresh();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('ares.generateSnapshot', async () => {
            vscode.window.showInformationMessage('ARES: Generating Snapshot...');
            try {
                const workspaceFolders = vscode.workspace.workspaceFolders;
                if (!workspaceFolders) {
                    vscode.window.showErrorMessage('ARES: No workspace folder open');
                    return;
                }
                const path = workspaceFolders[0].uri.fsPath;
                await apiClient.generateSnapshot(path);
                vscode.window.showInformationMessage('ARES: Snapshot Generated Successfully');
            } catch (e: any) {
                vscode.window.showErrorMessage(`ARES Error: ${e.message}`);
            }
        })
    );
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.copyContextToClipboard', async () => {
            vscode.window.showInformationMessage('ARES: Exporting Portable Context...');
            try {
                // In a real scenario, we would get the project ID from the workspace or config.
                // For now, we'll ask the user or use a default mock ID.
                const projectId = await vscode.window.showInputBox({
                    prompt: "Enter Project ID to export context for",
                    placeHolder: "e.g., proj_12345"
                });
                
                if (!projectId) return;

                const data = await apiClient.exportContext(projectId);
                await vscode.env.clipboard.writeText(data.text);
                
                vscode.window.showInformationMessage(`ARES: Portable Context copied to clipboard! (~${data.estimated_tokens} tokens)`);
            } catch (e: any) {
                vscode.window.showErrorMessage(`ARES Error: Failed to export context. Is ARES daemon running? ${e.message}`);
            }
        })
    );
}

export function deactivate() {}
