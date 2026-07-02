import * as vscode from 'vscode';
import { McpClient } from './mcp-client';

/**
 * Central request manager — the single pipeline for all MCP calls.
 *
 * Rules:
 * - No panel calls MCP directly
 * - No panel owns state
 * - Every MCP call goes through this manager
 * - Busy guard prevents duplicate concurrent requests
 *
 * Capabilities:
 * - execute()  — run a tool call with busy guard
 * - cache()    — memoize results for N seconds
 * - cancel()   — cancel in-flight request by key
 * - isBusy()   — check if a key is currently running
 * - telemetry  — timing and call count logging
 */
export class RequestManager {
    private mcpClient: McpClient;
    private output: vscode.OutputChannel;
    private busyCommands = new Set<string>();
    private cache = new Map<string, { data: any; expires: number }>();
    private callCount = 0;

    constructor(mcpClient: McpClient, output: vscode.OutputChannel) {
        this.mcpClient = mcpClient;
        this.output = output;
    }

    /**
     * Execute an MCP tool call with busy-guard protection.
     * If the same commandKey is already running, the call is dropped.
     */
    async callTool<T = any>(commandKey: string, toolName: string, args: Record<string, any>): Promise<T | null> {
        if (this.busyCommands.has(commandKey)) {
            this.output.appendLine(`[RequestManager] Dropping duplicate: ${commandKey}`);
            return null;
        }

        // Check cache
        const cached = this.cache.get(commandKey);
        if (cached && Date.now() < cached.expires) {
            this.output.appendLine(`[RequestManager] Cache hit: ${commandKey}`);
            return cached.data as T;
        }

        this.busyCommands.add(commandKey);
        this.callCount++;
        const callId = this.callCount;
        const start = Date.now();

        try {
            this.output.appendLine(`[RequestManager] #${callId} ${commandKey} → ${toolName}`);
            const result = await this.mcpClient.callTool(toolName, args);

            let parsed: any;
            if (result?.content?.[0]?.type === 'text') {
                parsed = JSON.parse(result.content[0].text);
            } else {
                parsed = result;
            }

            const elapsed = Date.now() - start;
            this.output.appendLine(`[RequestManager] #${callId} ${commandKey} ✓ ${elapsed}ms`);
            return parsed as T;
        } catch (e: any) {
            const elapsed = Date.now() - start;
            this.output.appendLine(`[RequestManager] #${callId} ${commandKey} ✗ ${elapsed}ms — ${e.message}`);
            throw e;
        } finally {
            this.busyCommands.delete(commandKey);
        }
    }

    /**
     * Execute with cache. Results are memoized for `ttlMs` milliseconds.
     */
    async callToolCached<T = any>(commandKey: string, toolName: string, args: Record<string, any>, ttlMs: number = 30_000): Promise<T | null> {
        const result = await this.callTool<T>(commandKey, toolName, args);
        if (result !== null) {
            this.cache.set(commandKey, { data: result, expires: Date.now() + ttlMs });
        }
        return result;
    }

    /**
     * Cancel an in-flight request by removing its busy flag.
     * The MCP call itself can't be cancelled, but the caller will see `null`.
     */
    cancel(commandKey: string): void {
        if (this.busyCommands.has(commandKey)) {
            this.busyCommands.delete(commandKey);
            this.output.appendLine(`[RequestManager] Cancelled: ${commandKey}`);
        }
    }

    /**
     * Invalidate a cached result.
     */
    invalidateCache(commandKey: string): void {
        this.cache.delete(commandKey);
    }

    /**
     * Clear all cached results.
     */
    clearCache(): void {
        this.cache.clear();
    }

    /**
     * Check if a command is currently running.
     */
    isBusy(commandKey: string): boolean {
        return this.busyCommands.has(commandKey);
    }

    /**
     * Get the workspace folder name for display (never raw IDs).
     */
    static getWorkspaceName(): string {
        const folders = vscode.workspace.workspaceFolders;
        if (folders && folders.length > 0) {
            return folders[0].name;
        }
        return 'Unknown';
    }

    /**
     * Get the workspace folder path.
     */
    static getWorkspacePath(): string {
        const folders = vscode.workspace.workspaceFolders;
        if (folders && folders.length > 0) {
            return folders[0].uri.fsPath;
        }
        return process.cwd();
    }
}
