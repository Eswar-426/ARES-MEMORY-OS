import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";

export class McpClient {
    private client: Client;
    private transport?: StdioClientTransport;
    private outputChannel: vscode.OutputChannel;
    private pidFile?: string;
    public lastError?: string;

    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
        this.client = new Client({
            name: "ares-vscode-extension",
            version: "0.1.0"
        }, {
            capabilities: {}
        });
    }

    /** Kill any previously orphaned MCP process recorded in the PID file. */
    static killOrphan(workspace: string, outputChannel: vscode.OutputChannel) {
        const pidFile = path.join(workspace, '.ares', 'ares-mcp.pid');
        try {
            if (!fs.existsSync(pidFile)) { return; }
            const pid = parseInt(fs.readFileSync(pidFile, 'utf-8').trim(), 10);
            if (isNaN(pid)) { return; }
            try {
                // process.kill(pid, 0) throws if PID doesn't exist
                process.kill(pid, 0);
                // PID is alive — kill it
                process.kill(pid);
                outputChannel.appendLine(`Killed orphaned ares-mcp.exe (PID ${pid})`);
            } catch (_e) {
                // PID already gone — nothing to do
                outputChannel.appendLine(`Orphan PID ${pid} already gone`);
            }
        } catch (_e) {
            // Ignore any file I/O errors
        } finally {
            try { fs.rmSync(pidFile, { force: true }); } catch (_e) {}
        }
    }

    async connect(mcpPath: string, source: string): Promise<boolean> {
        const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        
        this.outputChannel.appendLine("========== ARES MCP ==========");
        this.outputChannel.appendLine(`Executable : ${mcpPath}`);
        this.outputChannel.appendLine(`Exists     : ${fs.existsSync(mcpPath)}`);
        this.outputChannel.appendLine(`Workspace  : ${workspace}`);
        this.outputChannel.appendLine(`Source     : ${source}`);
        this.outputChannel.appendLine("==============================");
        
        if (source !== 'PATH' && !fs.existsSync(mcpPath)) {
            this.outputChannel.appendLine(`ARES MCP binary not found:\n${mcpPath}`);
            return false;
        }

        try {
            this.transport = new StdioClientTransport({
                command: mcpPath,
                args: [],
                cwd: workspace,
                env: process.env as Record<string, string>
            });

            await this.client.connect(this.transport);

            // Write PID file so next activation can clean up if we crash
            if (workspace) {
                const proc = (this.transport as any).process;
                if (proc?.pid) {
                    this.pidFile = path.join(workspace, '.ares', 'ares-mcp.pid');
                    try {
                        fs.mkdirSync(path.dirname(this.pidFile), { recursive: true });
                        fs.writeFileSync(this.pidFile, String(proc.pid), 'utf-8');
                        this.outputChannel.appendLine(`MCP PID ${proc.pid} written to ${this.pidFile}`);
                    } catch (_e) {
                        // Non-fatal — PID file is a safety net, not required
                    }
                }
            }

            this.outputChannel.appendLine("Successfully connected to ARES MCP.");
            return true;
        } catch (e: any) {
            this.lastError = e.message || String(e);
            this.outputChannel.appendLine(`Failed to connect to ARES MCP: ${e.message}`);
            return false;
        }
    }

    async disconnect() {
        // Delete PID file first — we are doing a clean shutdown
        if (this.pidFile) {
            try { fs.rmSync(this.pidFile, { force: true }); } catch (_e) {}
            this.pidFile = undefined;
        }

        if (this.transport) {
            try {
                // Force-kill the child process to prevent orphans.
                // No signal argument — on Windows this maps to TerminateProcess.
                const proc = (this.transport as any).process;
                if (proc && typeof proc.kill === 'function') {
                    proc.kill();
                }
            } catch (_e) {
                // Ignore — process may already be dead
            }
            try {
                await this.transport.close();
            } catch (_e) {
                // Ignore — transport may already be closed
            }
            this.transport = undefined;
        }
    }

    async callTool(name: string, args: any): Promise<any> {
        const ts = new Date().toISOString();
        this.outputChannel.appendLine(`\n[${ts}] MCP Request: ${name}`);
        this.outputChannel.appendLine(`  Args: ${JSON.stringify(args)}`);
        const start = Date.now();
        try {
            const result = await this.client.callTool({
                name,
                arguments: args
            });
            const elapsed = Date.now() - start;
            this.outputChannel.appendLine(`  Response (${elapsed}ms): ${JSON.stringify(result, null, 2)}`);
            return result;
        } catch (e: any) {
            const elapsed = Date.now() - start;
            this.outputChannel.appendLine(`  Error (${elapsed}ms): ${e.message}`);
            throw e;
        }
    }
}
