import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import * as vscode from "vscode";
import * as fs from "fs";

export class McpClient {
    private client: Client;
    private transport?: StdioClientTransport;
    private outputChannel: vscode.OutputChannel;

    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
        this.client = new Client({
            name: "ares-vscode-extension",
            version: "0.1.0"
        }, {
            capabilities: {}
        });
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
            this.outputChannel.appendLine("Successfully connected to ARES MCP.");
            return true;
        } catch (e: any) {
            this.outputChannel.appendLine(`Failed to connect to ARES MCP: ${e.message}`);
            return false;
        }
    }

    async disconnect() {
        if (this.transport) {
            await this.transport.close();
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
