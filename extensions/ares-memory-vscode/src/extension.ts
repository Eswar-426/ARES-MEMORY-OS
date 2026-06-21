import * as vscode from 'vscode';
import { McpClient } from './mcp-client';
import { execFile } from 'child_process';
import * as path from 'path';
import { resolveAresCli, resolveAresMcp, ResolvedBinary } from './binary-discovery';
import { RepositoryWatcher } from './watcher';

let mcpClient: McpClient;
let aresOutput: vscode.OutputChannel;
let aresCliCache: ResolvedBinary | undefined;
let aresMcpCache: ResolvedBinary | undefined;

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
            
            execFile(aresCliCache!.path, ['doctor'], { cwd }, (error, stdout, stderr) => {
                if (stdout) aresOutput.append(stdout);
                if (stderr) aresOutput.append(stderr);
                if (error) {
                    aresOutput.appendLine(`\nError running ares doctor: ${error.message}`);
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
            execFile(aresCliCache!.path, ['ingest', '.'], { cwd }, (error, stdout, stderr) => {
                if (stdout) aresOutput.append(stdout);
                if (stderr) aresOutput.append(stderr);
                if (error) {
                    vscode.window.showErrorMessage(`ARES Error: Failed to ingest. See ARES Output for details.`);
                    aresOutput.appendLine(`\nError running ares ingest: ${error.message}`);
                } else {
                    vscode.window.showInformationMessage('ARES: Repository Ingested Successfully!');
                }
            });
        })
    );

    const runToolCommand = async (commandName: string, toolName: string, uri?: vscode.Uri) => {
        aresOutput.show(true);
        aresOutput.appendLine(`\n--- ${commandName} ---`);
        
        logEnvironment();

        let targetId = "";
        
        if (uri) {
            // Invoked from context menu
            const workspaceFolders = vscode.workspace.workspaceFolders;
            if (workspaceFolders) {
                // Try to get relative path
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
            // Invoked from command palette
            const input = await vscode.window.showInputBox({
                prompt: `Enter Target ID for ${commandName}`,
                placeHolder: "e.g., src/main.rs or PROJ-001"
            });
            if (!input) return;
            targetId = input;
        }

        try {
            await mcpClient.callTool(toolName, { id: targetId });
        } catch (e: any) {
            vscode.window.showErrorMessage(`ARES MCP Error: ${e.message}`);
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
