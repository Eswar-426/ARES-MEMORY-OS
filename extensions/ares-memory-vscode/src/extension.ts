import * as vscode from 'vscode';
import { McpClient } from './mcp-client';
import { execFile, spawn } from 'child_process';
import * as path from 'path';
import { resolveAresCli, resolveAresMcp, ResolvedBinary } from './binary-discovery';
import { RepositoryWatcher } from './watcher';
import { AresQueryPanel, AresResponse, AresError } from './queryPanel';

let mcpClient: McpClient;
let aresOutput: vscode.OutputChannel;
let aresCliCache: ResolvedBinary | undefined;
let aresMcpCache: ResolvedBinary | undefined;

/**
 * Parse the raw MCP tool result into an AresResponse.
 * The MCP SDK returns { content: [{ type: "text", text: "..." }] }.
 * The text payload is expected to be JSON matching AresResponse.
 */
function parseAresResponse(result: any, filePath?: string): AresResponse {
    let raw: any = {};

    if (result && result.content && Array.isArray(result.content)) {
        for (const block of result.content) {
            if (block.type === 'text' && typeof block.text === 'string') {
                try {
                    raw = JSON.parse(block.text);
                } catch {
                    // If the text isn't JSON, treat it as the answer
                    raw = { answer: block.text };
                }
                break;
            }
        }
    } else if (typeof result === 'string') {
        try {
            raw = JSON.parse(result);
        } catch {
            raw = { answer: result };
        }
    } else if (result && typeof result === 'object') {
        raw = result;
    }

    return {
        answer: raw.answer ?? raw.explanation ?? '',
        confidence: typeof raw.confidence === 'number' ? raw.confidence : 0,
        evidence: Array.isArray(raw.evidence) ? raw.evidence : [],
        related_decisions: Array.isArray(raw.related_decisions) ? raw.related_decisions : [],
        query_type: raw.query_type ?? '',
        file_path: filePath ?? raw.file_path,
        ...raw,
    };
}

export async function activate(context: vscode.ExtensionContext) {
    aresOutput = vscode.window.createOutputChannel("ARES");
    aresOutput.appendLine('ARES Memory OS extension activating...\n');
    aresOutput.appendLine('--- ARES Startup Validation ---');

    mcpClient = new McpClient(aresOutput);

    // Resolve binaries
    const cliBinary = await resolveAresCli(context);
    const mcpBinary = await resolveAresMcp(context);
    
    aresCliCache = cliBinary;
    aresMcpCache = mcpBinary;

    let aborted = false;

    if (cliBinary) {
        aresOutput.appendLine(`✓ CLI binary found (${cliBinary.source})`);
    } else {
        aresOutput.appendLine(`✗ CLI binary missing`);
        aborted = true;
    }

    if (mcpBinary) {
        aresOutput.appendLine(`✓ MCP binary found (${mcpBinary.source})`);
    } else {
        aresOutput.appendLine(`✗ MCP binary missing`);
        aborted = true;
    }

    if (aborted) {
        aresOutput.appendLine('\nActivation Status: ABORTED');
        vscode.window.showErrorMessage("ARES Initialization Failed: Binary resolution failed. See ARES Output for details.");
        return;
    }

    aresOutput.appendLine(`✓ CLI executable`);
    aresOutput.appendLine(`✓ MCP executable`);

    const connected = await mcpClient.connect(mcpBinary!.path, mcpBinary!.source);
    if (!connected) {
        aresOutput.appendLine('\nActivation Status: ABORTED (MCP Connection Failed)');
        vscode.window.showErrorMessage("ARES MCP failed to connect. See ARES Output for details.");
        return;
    }

    aresOutput.appendLine('\nActivation Status: READY\n');

    const logEnvironment = () => {
        aresOutput.appendLine(`\nARES Environment`);
        aresOutput.appendLine(`CLI Path: ${aresCliCache!.path}`);
        aresOutput.appendLine(`MCP Path: ${aresMcpCache!.path}`);
        aresOutput.appendLine(`Source: ${aresCliCache!.source} | ${aresMcpCache!.source}\n`);
    };

    // Initialize the Repository Watcher
    const watcher = new RepositoryWatcher(aresOutput, aresCliCache!);
    watcher.watch();

    // ARES: Doctor
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.doctor', async () => {
            aresOutput.show(true);
            aresOutput.appendLine('\n--- Running ARES Doctor ---');
            
            logEnvironment();

            const workspaceFolders = vscode.workspace.workspaceFolders;
            const cwd = workspaceFolders ? workspaceFolders[0].uri.fsPath : process.cwd();
            
            const child = spawn(aresCliCache!.path, ['doctor'], { cwd });

            child.stdout.on("data", (data) => {
                aresOutput.append(data.toString());
            });

            child.stderr.on("data", (data) => {
                aresOutput.appendLine(`[stderr] ${data.toString()}`);
            });

            child.on("close", (code) => {
                aresOutput.appendLine(`ARES exited with code ${code}`);
                if (code !== 0) {
                    aresOutput.appendLine(`\nError running ares doctor (exit code ${code})`);
                }
            });
        })
    );

    // ARES: Ingest Repository
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.ingest', async () => {
            aresOutput.show(true);
            aresOutput.appendLine('\n--- Ingesting Repository ---');
            
            logEnvironment();

            const workspaceFolders = vscode.workspace.workspaceFolders;
            if (!workspaceFolders) {
                vscode.window.showErrorMessage('ARES: No workspace folder open');
                return;
            }
            const cwd = workspaceFolders[0].uri.fsPath;
            
            vscode.window.showInformationMessage('ARES: Ingesting Repository...');
            const child = spawn(aresCliCache!.path, ['ingest', '.'], { cwd });

            child.stdout.on("data", (data) => {
                aresOutput.append(data.toString());
            });

            child.stderr.on("data", (data) => {
                aresOutput.appendLine(`[stderr] ${data.toString()}`);
            });

            child.on("close", (code) => {
                aresOutput.appendLine(`ARES exited with code ${code}`);
                if (code !== 0) {
                    vscode.window.showErrorMessage(`ARES Error: Failed to ingest. See ARES Output for details.`);
                } else {
                    vscode.window.showInformationMessage('ARES: Repository Ingested Successfully!');
                }
            });
        })
    );

    const runToolCommand = async (commandName: string, toolName: string, uri?: vscode.Uri) => {
        aresOutput.appendLine(`\n--- ${commandName} ---`);
        logEnvironment();

        let targetId = "";
        
        if (uri) {
            const workspaceFolders = vscode.workspace.workspaceFolders;
            if (workspaceFolders) {
                const rootPath = workspaceFolders[0].uri.fsPath;
                if (uri.fsPath.startsWith(rootPath)) {
                    targetId = path.relative(rootPath, uri.fsPath).replace(/\\/g, '/');
                } else {
                    targetId = uri.fsPath;
                }
            } else {
                targetId = uri.fsPath;
            }
        } else {
            const input = await vscode.window.showInputBox({
                prompt: `Enter Target ID for ${commandName}`,
                placeHolder: "e.g., src/main.rs or PROJ-001"
            });
            if (!input) return;
            targetId = input;
        }

        // Store recent query
        const recentQueries: any[] = context.workspaceState.get('ares.recentQueries', []);
        recentQueries.unshift({
            command: commandName,
            target: targetId,
            timestamp: new Date().toISOString()
        });
        context.workspaceState.update('ares.recentQueries', recentQueries.slice(0, 10));

        // Open panel immediately with loading state
        const panel = AresQueryPanel.showLoading(context);

        // Register the message handler for this panel
        panel.webview.onDidReceiveMessage(
            (message: { command: string; path: string; line?: number; column?: number }) => {
                if (message.command === 'openFile') {
                    const workspaceFolders = vscode.workspace.workspaceFolders;
                    let fileUri: vscode.Uri;
                    if (workspaceFolders && !path.isAbsolute(message.path)) {
                        fileUri = vscode.Uri.file(
                            path.join(workspaceFolders[0].uri.fsPath, message.path)
                        );
                    } else {
                        fileUri = vscode.Uri.file(message.path);
                    }

                    const options: vscode.TextDocumentShowOptions = { preview: true };
                    if (typeof message.line === 'number') {
                        const line = Math.max(0, message.line - 1);
                        const col = typeof message.column === 'number' ? Math.max(0, message.column - 1) : 0;
                        options.selection = new vscode.Range(line, col, line, col);
                    }
                    vscode.window.showTextDocument(fileUri, options);
                }
            },
            undefined,
            context.subscriptions
        );

        try {
            const result = await mcpClient.callTool(toolName, { id: targetId });
            const response = parseAresResponse(result, targetId);
            AresQueryPanel.show(context, response);
        } catch (e: any) {
            const aresError: AresError = {
                message: 'Unable to retrieve repository memory',
                detail: e.message || 'An unexpected error occurred.',
            };
            AresQueryPanel.showError(context, aresError);
        }
    };

    // ARES: Why Exists
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.whyExists', (uri?: vscode.Uri) => {
            runToolCommand('Why Exists', 'ares_why_exists', uri);
        })
    );

    // ARES: Impact Analysis
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.impactAnalysis', (uri?: vscode.Uri) => {
            runToolCommand('Impact Analysis', 'ares_impact', uri);
        })
    );

    // ARES: Drift Analysis
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.driftAnalysis', (uri?: vscode.Uri) => {
            runToolCommand('Drift Analysis', 'ares_drift', uri);
        })
    );

    // ARES: Traceability Analysis
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.traceabilityAnalysis', async (uri?: vscode.Uri) => {
            // For trace, we'll use a modified runToolCommand logic or simply pass a wrapper since input structure is different.
            // But TraceabilityInput expects entity_id, let's use the generic flow but construct correct arguments.
            aresOutput.appendLine(`\n--- Traceability Analysis ---`);
            logEnvironment();

            let targetId = "";
            if (uri) {
                const workspaceFolders = vscode.workspace.workspaceFolders;
                if (workspaceFolders) {
                    const rootPath = workspaceFolders[0].uri.fsPath;
                    if (uri.fsPath.startsWith(rootPath)) {
                        targetId = path.relative(rootPath, uri.fsPath).replace(/\\/g, '/');
                    } else {
                        targetId = uri.fsPath;
                    }
                } else {
                    targetId = uri.fsPath;
                }
            } else {
                const input = await vscode.window.showInputBox({
                    prompt: `Enter Target ID for Traceability`,
                    placeHolder: "e.g., src/main.rs or PROJ-001"
                });
                if (!input) return;
                targetId = input;
            }

            const depthInput = await vscode.window.showInputBox({
                prompt: `Enter traversal depth (default 3)`,
                placeHolder: "3"
            });
            let depth = 3;
            if (depthInput && !isNaN(parseInt(depthInput))) {
                depth = parseInt(depthInput);
            }

            const panel = AresQueryPanel.showLoading(context);
            
            panel.webview.onDidReceiveMessage(
                (message: { command: string; path: string; line?: number; column?: number }) => {
                    if (message.command === 'openFile') {
                        const workspaceFolders = vscode.workspace.workspaceFolders;
                        let fileUri: vscode.Uri;
                        if (workspaceFolders && !path.isAbsolute(message.path)) {
                            fileUri = vscode.Uri.file(
                                path.join(workspaceFolders[0].uri.fsPath, message.path)
                            );
                        } else {
                            fileUri = vscode.Uri.file(message.path);
                        }
    
                        const options: vscode.TextDocumentShowOptions = { preview: true };
                        if (typeof message.line === 'number') {
                            const line = Math.max(0, message.line - 1);
                            const col = typeof message.column === 'number' ? Math.max(0, message.column - 1) : 0;
                            options.selection = new vscode.Range(line, col, line, col);
                        }
                        vscode.window.showTextDocument(fileUri, options);
                    }
                },
                undefined,
                context.subscriptions
            );

            try {
                const result = await mcpClient.callTool('ares_traceability', { entity_id: targetId, depth: depth });
                const response = parseAresResponse(result, targetId);
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                const aresError: AresError = {
                    message: 'Unable to trace entity',
                    detail: e.message || 'An unexpected error occurred.',
                };
                AresQueryPanel.showError(context, aresError);
            }
        })
    );

    // ARES: Coverage Analysis
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.coverageAnalysis', async (uri?: vscode.Uri) => {
            aresOutput.show(true);
            aresOutput.appendLine('\n--- Coverage Analysis ---');
            
            logEnvironment();

            const input = await vscode.window.showInputBox({
                prompt: `Enter Project ID for Coverage Analysis`,
                placeHolder: "e.g., PROJ-001"
            });
            if (!input) return;

            try {
                await mcpClient.callTool('ares_coverage', { project_id: input });
            } catch (e: any) {
                vscode.window.showErrorMessage(`ARES MCP Error: ${e.message}`);
            }
        })
    );

    // ARES: ARES Home
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.repositoryDashboard', async () => {
            aresOutput.appendLine(`\n--- ARES Home ---`);
            logEnvironment();

            const panel = AresQueryPanel.showLoading(context);
            
            panel.webview.onDidReceiveMessage(
                async (message: { command: string; path?: string; line?: number; column?: number; args?: any[] }) => {
                    if (message.command === 'openFile' && message.path) {
                        const workspaceFolders = vscode.workspace.workspaceFolders;
                        let fileUri: vscode.Uri;
                        if (workspaceFolders && !path.isAbsolute(message.path)) {
                            fileUri = vscode.Uri.file(
                                path.join(workspaceFolders[0].uri.fsPath, message.path)
                            );
                        } else {
                            fileUri = vscode.Uri.file(message.path);
                        }
    
                        const options: vscode.TextDocumentShowOptions = { preview: true };
                        if (typeof message.line === 'number') {
                            const line = Math.max(0, message.line - 1);
                            const col = typeof message.column === 'number' ? Math.max(0, message.column - 1) : 0;
                            options.selection = new vscode.Range(line, col, line, col);
                        }
                        vscode.window.showTextDocument(fileUri, options);
                    } else if (message.command === 'executeCommand' && message.args && message.args.length > 0) {
                        vscode.commands.executeCommand(message.args[0], ...message.args.slice(1));
                    } else if (message.command === 'refreshDashboard') {
                        try {
                            const result = await mcpClient.callTool('ares_dashboard', { project_id: 'TEST' });
                            let rawResult: any = result;
                            if (result.content && result.content[0] && result.content[0].type === 'text') {
                                rawResult = JSON.parse(result.content[0].text);
                            }
                            const response: AresResponse = {
                                answer: "ARES Home",
                                confidence: 1.0,
                                evidence: [],
                                related_decisions: [],
                                query_type: "ARES Home",
                                dashboard: rawResult,
                                recent_queries: context.workspaceState.get('ares.recentQueries', [])
                            };
                            panel.webview.postMessage(response);
                        } catch (e) {
                            // ignore background errors
                        }
                    }
                },
                undefined,
                context.subscriptions
            );

            try {
                // Fetch the dashboard from ares-mcp
                const result = await mcpClient.callTool('ares_dashboard', { project_id: 'TEST' });
                
                let rawResult: any = result;
                if (result.content && result.content[0] && result.content[0].type === 'text') {
                    rawResult = JSON.parse(result.content[0].text);
                }

                // Format as AresResponse so the panel knows how to render it
                const response: AresResponse = {
                    answer: "ARES Home",
                    confidence: 1.0,
                    evidence: [],
                    related_decisions: [],
                    query_type: "ARES Home",
                    dashboard: rawResult,
                    recent_queries: context.workspaceState.get('ares.recentQueries', [])
                };
                
                AresQueryPanel.show(context, response);
            } catch (e: any) {
                const aresError: AresError = {
                    message: 'Unable to load ARES Home',
                    detail: e.message || 'An unexpected error occurred.',
                };
                AresQueryPanel.showError(context, aresError);
            }
        })
    );

    // ARES: Simulate Change
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.simulateChange', async (uri?: vscode.Uri) => {
            aresOutput.show(true);
            aresOutput.appendLine(`\n--- Simulate Change ---`);
            
            logEnvironment();

            let targetId = "";
            if (uri) {
                const workspaceFolders = vscode.workspace.workspaceFolders;
                if (workspaceFolders) {
                    const rootPath = workspaceFolders[0].uri.fsPath;
                    if (uri.fsPath.startsWith(rootPath)) {
                        targetId = path.relative(rootPath, uri.fsPath).replace(/\\/g, '/');
                    } else {
                        targetId = uri.fsPath;
                    }
                } else {
                    targetId = uri.fsPath;
                }
            } else {
                const input = await vscode.window.showInputBox({
                    prompt: `Enter Target ID to Simulate Removal`,
                    placeHolder: "e.g., src/main.rs"
                });
                if (!input) return;
                targetId = input;
            }

            const projectId = await vscode.window.showInputBox({
                prompt: `Enter Project ID for Simulation context`,
                placeHolder: "e.g., PROJ-001",
                value: "PROJ-001"
            });
            if (!projectId) return;

            try {
                await mcpClient.callTool('ares_simulate', { 
                    project_id: projectId,
                    target_id: targetId,
                    action: "remove" 
                });
            } catch (e: any) {
                vscode.window.showErrorMessage(`ARES MCP Error: ${e.message}`);
            }
        })
    );
}

export function deactivate() {
    if (mcpClient) {
        mcpClient.disconnect();
    }
}
