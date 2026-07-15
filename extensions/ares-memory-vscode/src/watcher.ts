import * as vscode from 'vscode';
import { getState, AresState } from './state';
import { spawn } from 'child_process';
import { ResolvedBinary } from './binary-discovery';

export class RepositoryWatcher {
    private dirtyFiles: Set<string> = new Set();
    private debounceTimer: NodeJS.Timeout | null = null;
    private readonly DEBOUNCE_MS = 2000;

    constructor(
        private aresOutput: vscode.OutputChannel,
        private aresCli: ResolvedBinary
    ) { }

    public watch() {
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.{rs,ts,tsx,js,jsx,md,toml,json,py,go,java,c,cpp,h,hpp,cc,cxx,rb,cs,php,kt,kts}');

        watcher.onDidChange(uri => this.handleFileEvent(uri));
        watcher.onDidCreate(uri => this.handleFileEvent(uri));
        watcher.onDidDelete(uri => this.handleFileEvent(uri));

        this.aresOutput.appendLine('RepositoryWatcher: Listening for file changes...');
    }

    private handleFileEvent(uri: vscode.Uri) {
        if (this.shouldIgnore(uri)) {
            return;
        }

        this.dirtyFiles.add(uri.fsPath);

        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
        }

        this.debounceTimer = setTimeout(() => {
            this.flushQueue();
        }, this.DEBOUNCE_MS);
    }

    private shouldIgnore(uri: vscode.Uri): boolean {
        const pathStr = uri.fsPath.replace(/\\/g, '/');
        const ignorePatterns = [
            '/node_modules/',
            '/target/',
            '/.git/',
            '/dist/',
            '/build/',
            '/out/',
            'package-lock.json',
            'yarn.lock',
            'pnpm-lock.yaml',
            '.ares/CLAUDE.md',
        ];

        return ignorePatterns.some(pattern => pathStr.includes(pattern));
    }

    private flushQueue() {
        if (this.dirtyFiles.size === 0) return;

        const currentState = getState();
        if (currentState !== AresState.READY) {
            this.aresOutput.appendLine(`[Watcher] Skipped ${this.dirtyFiles.size} file(s) — state is ${currentState}, not READY`);
            this.dirtyFiles.clear();
            return;
        }

        if (!this.aresCli || !this.aresCli.path) {
            this.aresOutput.appendLine('[Watcher] Skipped — CLI binary not available');
            this.dirtyFiles.clear();
            return;
        }

        const filesToProcess = Array.from(this.dirtyFiles);
        this.dirtyFiles.clear();

        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return;

        const cwd = workspaceFolders[0].uri.fsPath;

        this.aresOutput.appendLine(`\n--- Incremental Ingestion ---`);
        this.aresOutput.appendLine(`Processing ${filesToProcess.length} changed file(s)...`);
        this.aresOutput.appendLine(`Files: ${filesToProcess.slice(0, 5).join(', ')}${filesToProcess.length > 5 ? '...' : ''}`);

        const args = ['ingest', '.', '--incremental', '--files', filesToProcess.join(',')];

        const child = spawn(this.aresCli.path, args, { cwd });

        child.stdout.on("data", (data) => {
            this.aresOutput.append(data.toString());
        });

        child.stderr.on("data", (data) => {
            this.aresOutput.appendLine(`[stderr] ${data.toString()}`);
        });

        child.on("close", (code) => {
            if (code !== 0) {
                this.aresOutput.appendLine(`Incremental ingest failed with code ${code}`);
            } else {
                this.aresOutput.appendLine(`Incremental ingest completed successfully.`);
            }
        });
    }
}
