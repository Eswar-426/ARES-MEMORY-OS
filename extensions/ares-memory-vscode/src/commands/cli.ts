import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { ResolvedBinary } from '../binary-discovery';

/**
 * Register CLI-based commands: Ingest, Doctor, Benchmark, Self Test.
 * These spawn CLI subprocesses — they don't go through MCP.
 */
export function registerCliCommands(
    context: vscode.ExtensionContext,
    output: vscode.OutputChannel,
    cliBinary: ResolvedBinary,
    mcpClient: any,
): void {
    // ── Doctor ────────────────────────────────────────────────
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.doctor', async () => {
            output.show(true);
            output.appendLine('\n--- Running ARES Doctor ---');

            const cwd = getWorkspaceCwd();
            await runCliWithProgress(output, cliBinary.path, ['doctor'], cwd, 'ARES: Running Doctor...');
        })
    );

    // ── Ingest ────────────────────────────────────────────────
    let ingestRunning = false;
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.ingest', async () => {
            if (ingestRunning) {
                vscode.window.showWarningMessage('ARES: Ingest is already running.');
                return;
            }

            const cwd = getWorkspaceCwd();
            if (!cwd) {
                vscode.window.showErrorMessage('ARES: No workspace folder open');
                return;
            }

            ingestRunning = true;
            output.show(true);
            output.appendLine('\n--- Ingesting Repository ---');

            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'ARES: Ingesting Repository...',
                cancellable: false,
            }, async (progress) => {
                return new Promise<void>((resolve) => {
                    const child = spawn(cliBinary.path, ['ingest', '.'], { cwd });

                    child.stdout.on('data', (data) => {
                        const str = data.toString();
                        output.append(str);
                        if (str.includes('Extracting') || str.includes('Scanning') || str.includes('Building')) {
                            progress.report({ message: str.trim().split('\n')[0] });
                        }
                    });
                    child.stderr.on('data', (data) => output.appendLine(`[stderr] ${data}`));
                    child.on('close', (code) => {
                        ingestRunning = false;
                        output.appendLine(`ARES exited with code ${code}`);
                        if (code !== 0) {
                            vscode.window.showErrorMessage('ARES Error: Failed to ingest. See Output.');
                        } else {
                            vscode.window.showInformationMessage('ARES: Repository Ingested Successfully!');
                        }
                        resolve();
                    });
                });
            });
        })
    );

    // ── Benchmark ─────────────────────────────────────────────
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.benchmark', async () => {
            output.show(true);
            output.appendLine('\n--- Benchmarking Repository ---');

            const cwd = getWorkspaceCwd();
            if (!cwd) {
                vscode.window.showErrorMessage('ARES: No workspace folder open');
                return;
            }
            await runCliWithProgress(output, cliBinary.path, ['benchmark'], cwd, 'ARES: Benchmarking...');
        })
    );

    // ── Self Test ─────────────────────────────────────────────
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.selfTest', async () => {
            output.show(true);
            output.appendLine('\n--- Running ARES Self Test ---');

            let passedChecks = 0;
            const totalChecks = 11;

            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'ARES Self Test',
                cancellable: false,
            }, async (progress) => {
                const check = (name: string, passed: boolean) => {
                    output.appendLine(passed ? `✓ ${name}` : `✗ ${name}`);
                    if (passed) passedChecks++;
                };

                check('CLI found', !!cliBinary?.path);
                progress.report({ increment: 10, message: 'Checking MCP...' });

                check('MCP connected', !!mcpClient);
                progress.report({ increment: 10, message: 'Checking Database...' });

                const cwd = getWorkspaceCwd() || '';
                const fs = require('fs');
                const path = require('path');
                check('Database found', fs.existsSync(path.join(cwd, '.ares', 'ares.db')));
                progress.report({ increment: 10, message: 'Checking Repository Index...' });

                try {
                    const result = await mcpClient.callTool('ares_dashboard', { project_id: cwd });
                    const dash = JSON.parse(result.content[0].text);
                    check('Repository indexed', dash.repository.files > 0);
                    progress.report({ increment: 10, message: 'Checking Graph...' });
                    check('Graph exists', dash.knowledge_graph.nodes > 0);
                    progress.report({ increment: 10, message: 'Checking Intelligence...' });
                    check('Intelligence ready', dash.readiness?.find((r: any) => r.engine === 'Graph Connectivity')?.status === 'Ready');
                    progress.report({ increment: 10, message: 'Checking Embeddings...' });
                    check('Embeddings configured', dash.readiness?.find((r: any) => r.engine === 'Context Injection')?.status === 'Ready');
                    check('Dashboard working', true);
                } catch {
                    ['Repository indexed', 'Graph exists', 'Intelligence ready', 'Embeddings configured', 'Dashboard working']
                        .forEach(n => check(n, false));
                }

                progress.report({ increment: 30, message: 'Testing AI...' });
                for (const [name, tool, args] of [
                    ['Why Exists working', 'ares_why_exists', { id: cwd }],
                    ['Impact working', 'ares_impact', { id: cwd }],
                    ['Traceability working', 'ares_traceability', { entity_id: cwd, depth: 1 }],
                ] as const) {
                    try {
                        const r = await mcpClient.callTool(tool, args);
                        check(name, !!r.content);
                    } catch { check(name, false); }
                    progress.report({ increment: 10 });
                }

                output.appendLine(`\nARES Ready\n${passedChecks}/${totalChecks} checks passed\n`);
                if (passedChecks === totalChecks) {
                    vscode.window.showInformationMessage(`ARES Ready: ${passedChecks}/${totalChecks} checks passed.`);
                } else {
                    vscode.window.showErrorMessage(`ARES Self Test Failed: ${passedChecks}/${totalChecks}. See Output.`);
                }
            });
        })
    );
}

// ── Helpers ──────────────────────────────────────────────────
function getWorkspaceCwd(): string | undefined {
    const folders = vscode.workspace.workspaceFolders;
    return folders ? folders[0].uri.fsPath : undefined;
}

async function runCliWithProgress(
    output: vscode.OutputChannel,
    cliPath: string,
    args: string[],
    cwd: string | undefined,
    title: string,
): Promise<void> {
    if (!cwd) {
        vscode.window.showErrorMessage('ARES: No workspace folder open');
        return;
    }
    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title,
        cancellable: false,
    }, async () => {
        return new Promise<void>((resolve) => {
            const child = spawn(cliPath, args, { cwd });
            child.stdout.on('data', (data) => output.append(data.toString()));
            child.stderr.on('data', (data) => output.appendLine(`[stderr] ${data}`));
            child.on('close', (code) => {
                output.appendLine(`ARES exited with code ${code}`);
                if (code !== 0) {
                    vscode.window.showErrorMessage(`ARES: Command failed (exit ${code}). See Output.`);
                }
                resolve();
            });
        });
    });
}
