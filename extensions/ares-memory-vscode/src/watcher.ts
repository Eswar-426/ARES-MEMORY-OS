import * as vscode from 'vscode';
import { execFile } from 'child_process';
import { ResolvedBinary } from './binary-discovery';

export class RepositoryWatcher {
    private dirtyFiles: Set<string> = new Set();
    private debounceTimer: NodeJS.Timeout | null = null;
    private readonly DEBOUNCE_MS = 2000;
    
    constructor(
        private aresOutput: vscode.OutputChannel,
        private aresCli: ResolvedBinary
    ) {}

    public watch() {
        const watcher = vscode.workspace.createFileSystemWatcher('**/*.{rs,ts,tsx,js,jsx,md,toml,json}');
        
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
            '/dist/',
            '/build/',
            '/.git/',
            '/.ares/'
        ];
        
        return ignorePatterns.some(pattern => pathStr.includes(pattern));
    }

    private flushQueue() {
        if (this.dirtyFiles.size === 0) return;

        const filesToProcess = Array.from(this.dirtyFiles);
        this.dirtyFiles.clear();

        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return;
        
        const cwd = workspaceFolders[0].uri.fsPath;

        this.aresOutput.appendLine(`\n--- Incremental Ingestion ---`);
        this.aresOutput.appendLine(`Processing ${filesToProcess.length} changed file(s)...`);

        const args = ['ingest', '.', '--incremental', '--files', filesToProcess.join(',')];

        execFile(this.aresCli.path, args, { cwd }, (error, stdout, stderr) => {
            if (stdout) this.aresOutput.append(stdout);
            if (stderr) this.aresOutput.append(stderr);
            if (error) {
                this.aresOutput.appendLine(`Incremental ingest failed: ${error.message}`);
            } else {
                this.aresOutput.appendLine(`Incremental ingest completed successfully.`);
            }
        });
    }
}
