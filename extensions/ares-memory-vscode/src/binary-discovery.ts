import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { spawnSync } from 'child_process';

export interface ResolvedBinary {
    path: string;
    source: string;
}

/**
 * Checks if a command exists in the global PATH by attempting to spawn it.
 */
function isCommandInPath(commandName: string): boolean {
    try {
        const result = spawnSync(commandName, ['--help'], { stdio: 'ignore', windowsHide: true });
        return !result.error;
    } catch (e) {
        return false;
    }
}

/**
 * Common discovery logic for both ares and ares-mcp
 */
async function discoverBinary(
    context: vscode.ExtensionContext,
    binaryName: string,
    settingKey: string
): Promise<ResolvedBinary | undefined> {
    const isWindows = process.platform === 'win32';
    const executableName = isWindows ? `${binaryName}.exe` : binaryName;

    // 1. VS Code Settings
    const config = vscode.workspace.getConfiguration('ares');
    const configuredPath = config.get<string>(settingKey);
    if (configuredPath && fs.existsSync(configuredPath)) {
        return { path: configuredPath, source: 'Settings' };
    }

    // 2. Workspace .ares/bin
    if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
        for (const folder of vscode.workspace.workspaceFolders) {
            const workspaceBin = path.join(folder.uri.fsPath, '.ares', 'bin', executableName);
            if (fs.existsSync(workspaceBin)) {
                return { path: workspaceBin, source: 'Workspace' };
            }
        }
    }

    // 3. Extension Bundled Binaries
    const bundledBin = path.join(context.extensionPath, 'bin', executableName);
    if (fs.existsSync(bundledBin)) {
        return { path: bundledBin, source: 'Bundled' };
    }

    // 4. Global PATH
    if (isCommandInPath(executableName)) {
        return { path: executableName, source: 'PATH' };
    }

    // 5. File Picker
    const result = await vscode.window.showWarningMessage(
        `ARES: Could not locate '${executableName}'. Please select the executable.`,
        'Browse'
    );

    if (result === 'Browse') {
        const fileUris = await vscode.window.showOpenDialog({
            canSelectFiles: true,
            canSelectFolders: false,
            canSelectMany: false,
            title: `Select ${executableName}`,
            filters: isWindows ? { 'Executables': ['exe'] } : undefined
        });

        if (fileUris && fileUris.length > 0) {
            const selectedPath = fileUris[0].fsPath;
            // Save to settings
            await config.update(settingKey, selectedPath, vscode.ConfigurationTarget.Global);
            return { path: selectedPath, source: 'Settings (User Selected)' };
        }
    }

    return undefined;
}

export async function resolveAresCli(context: vscode.ExtensionContext): Promise<ResolvedBinary | undefined> {
    return discoverBinary(context, 'ares', 'cliPath');
}

export async function resolveAresMcp(context: vscode.ExtensionContext): Promise<ResolvedBinary | undefined> {
    return discoverBinary(context, 'ares-mcp', 'mcpPath');
}
