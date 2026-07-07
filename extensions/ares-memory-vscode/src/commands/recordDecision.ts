import * as vscode from 'vscode';

export async function recordInlineDecision(context: vscode.ExtensionContext, mcpClient: any) {
    if (!mcpClient) {
        vscode.window.showErrorMessage("ARES MCP client is not initialized.");
        return;
    }

    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage("Open a file to record a decision against it.");
        return;
    }

    const currentFilePath = editor.document.uri.fsPath;

    const summary = await vscode.window.showInputBox({
        title: "Record Decision (1/3)",
        prompt: "Summary of the decision",
        placeHolder: "e.g., Use SQLite for graph storage instead of Neo4j",
        ignoreFocusOut: true
    });
    if (!summary) return; // user cancelled

    const rationale = await vscode.window.showInputBox({
        title: "Record Decision (2/3)",
        prompt: "Rationale (Why?)",
        placeHolder: "e.g., SQLite requires no external dependencies and is extremely fast for our use-case",
        ignoreFocusOut: true
    });
    if (!rationale) return; // user cancelled

    const alternatives = await vscode.window.showInputBox({
        title: "Record Decision (3/3)",
        prompt: "Alternatives Considered",
        placeHolder: "e.g., Neo4j (too heavy), Postgres (requires daemon)",
        ignoreFocusOut: true
    });
    if (!alternatives) return; // user cancelled

    // Format the description cleanly
    const description = `**Rationale:**\n${rationale}\n\n**Alternatives Considered:**\n${alternatives}`;

    try {
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: "ARES: Recording Decision...",
            cancellable: false
        }, async () => {
            const result = await mcpClient.callTool({
                name: "ares_record_decision",
                arguments: {
                    title: summary,
                    description: description,
                    status: "accepted",
                    impacted_paths: [currentFilePath],
                    source: "human",
                    confidence: 1.0
                }
            });

            if (result.isError) {
                throw new Error(result.content[0]?.text || "Unknown error");
            }
        });

        vscode.window.showInformationMessage(`Decision recorded and linked to ${vscode.workspace.asRelativePath(currentFilePath)}`);
    } catch (e: any) {
        vscode.window.showErrorMessage(`Failed to record decision: ${e.message}`);
    }
}
