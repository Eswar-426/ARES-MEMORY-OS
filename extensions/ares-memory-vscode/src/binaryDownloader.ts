import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import * as https from 'https';
import * as zlib from 'zlib';
import { GITHUB_OWNER, GITHUB_REPO, GITHUB_RELEASES_URL } from './constants';

interface PlatformInfo {
    dir: string;
    binaryName: string;
    assetPrefix: string;
}

export interface EnsureResult {
    path: string;
    source: 'bundled' | 'downloaded';
}

export function getPlatformInfo(): PlatformInfo {
    const platform = process.platform;
    const arch = process.arch;
    
    if (platform === 'win32') {
        return { dir: 'windows', binaryName: 'ares-mcp.exe', assetPrefix: 'ares-windows' };
    } else if (platform === 'darwin' && arch === 'arm64') {
        return { dir: 'mac-arm', binaryName: 'ares-mcp', assetPrefix: 'ares-mac-arm' };
    } else if (platform === 'darwin') {
        return { dir: 'mac-x64', binaryName: 'ares-mcp', assetPrefix: 'ares-mac-x64' };
    } else if (platform === 'linux') {
        return { dir: 'linux', binaryName: 'ares-mcp', assetPrefix: 'ares-linux' };
    }
    
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
}

function getHeaders(): any {
    const headers: any = { 'User-Agent': 'ARES-VSCode' };
    const token = process.env.GITHUB_TOKEN || vscode.workspace.getConfiguration('ares').get<string>('githubToken');
    if (token) headers['Authorization'] = `token ${token}`;
    return headers;
}

function fetchJson(url: string): Promise<any> {
    return new Promise((resolve, reject) => {
        https.get(url, { headers: getHeaders() }, (res) => {
            if (res.statusCode === 301 || res.statusCode === 302) {
                return resolve(fetchJson(res.headers.location as string));
            }
            if (res.statusCode !== 200) {
                return reject(new Error(`Failed to fetch: ${res.statusCode} ${res.statusMessage}`));
            }
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(JSON.parse(data)));
        }).on('error', reject);
    });
}

function downloadBuffer(url: string): Promise<Buffer> {
    return new Promise((resolve, reject) => {
        https.get(url, { headers: getHeaders() }, (res) => {
            if (res.statusCode === 301 || res.statusCode === 302) {
                return resolve(downloadBuffer(res.headers.location as string));
            }
            if (res.statusCode !== 200) {
                return reject(new Error(`Failed to download: ${res.statusCode} ${res.statusMessage}`));
            }
            const chunks: Buffer[] = [];
            res.on('data', chunk => chunks.push(chunk));
            res.on('end', () => resolve(Buffer.concat(chunks)));
        }).on('error', reject);
    });
}

function parseTar(buffer: Buffer): Map<string, Buffer> {
    const files = new Map<string, Buffer>();
    let offset = 0;
    while (offset < buffer.length - 512) {
        const header = buffer.slice(offset, offset + 512);
        const name = header.slice(0, 100).toString('ascii').replace(/\0/g, '');
        const sizeStr = header.slice(124, 136).toString('ascii').replace(/\0/g, '').trim();
        const size = parseInt(sizeStr, 8);
        if (!name || isNaN(size)) break;
        offset += 512;
        if (size > 0) {
            files.set(name, buffer.slice(offset, offset + size));
            offset += Math.ceil(size / 512) * 512;
        }
    }
    return files;
}

export async function ensureBinaries(context: vscode.ExtensionContext): Promise<EnsureResult> {
    const info = getPlatformInfo();
    const binariesDir = path.join(context.extensionPath, 'binaries', info.dir);
    const binaryPath = path.join(binariesDir, info.binaryName);
    
    if (fs.existsSync(binaryPath)) {
        return { path: binaryPath, source: 'bundled' };
    }
    
    fs.mkdirSync(binariesDir, { recursive: true });
    
    try {
        await vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Notification,
                title: 'ARES: Downloading engine...',
                cancellable: false,
            },
            async (progress) => {
                progress.report({ message: 'Fetching latest release...' });
                
                const releaseData = await fetchJson(GITHUB_RELEASES_URL);
                const asset = releaseData.assets.find(
                    (a: any) => a.name.startsWith(info.assetPrefix) && (a.name.endsWith('.tar.gz') || a.name.endsWith('.zip'))
                );
                
                if (!asset) {
                    throw new Error(`No binary found for ${info.assetPrefix}`);
                }
                
                progress.report({ message: `Downloading ${asset.name}...` });
                const archiveBuffer = await downloadBuffer(asset.browser_download_url);
                
                progress.report({ message: 'Extracting binaries...' });
                if (asset.name.endsWith('.tar.gz')) {
                    const tarBuffer = zlib.gunzipSync(archiveBuffer);
                    const files = parseTar(tarBuffer);
                    for (const [name, content] of files.entries()) {
                        const targetPath = path.join(binariesDir, path.basename(name));
                        fs.writeFileSync(targetPath, content);
                        if (process.platform !== 'win32') {
                            fs.chmodSync(targetPath, 0o755);
                        }
                    }
                } else if (asset.name.endsWith('.zip')) {
                    // Extremely basic fallback to zip if required (but we standardized on tar.gz or simple downloads, wait no we didn't).
                    // As planned, windows archive will be zip. But we don't have zip extraction cleanly. 
                    // Let's rely on standard child process for windows zip
                    const tempZip = path.join(binariesDir, 'temp.zip');
                    fs.writeFileSync(tempZip, archiveBuffer);
                    const cp = require('child_process');
                    const escapedZip = tempZip.replace(/'/g, "''");
                    const escapedDir = binariesDir.replace(/'/g, "''");
                    cp.execSync(`powershell -command "Expand-Archive -Force -LiteralPath '${escapedZip}' -DestinationPath '${escapedDir}'"`, { windowsHide: true });
                    fs.unlinkSync(tempZip);
                }
                
                vscode.window.showInformationMessage('ARES: Engine ready');
            }
        );
        return { path: binaryPath, source: 'downloaded' };
    } catch (e: any) {
        const action = await vscode.window.showErrorMessage(
            `ARES engine download failed: ${e.message}`,
            'View Install Instructions'
        );
        if (action === 'View Install Instructions') {
            vscode.env.openExternal(vscode.Uri.parse(`https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}#manual-install`));
        }
        throw e;
    }
}
