import * as vscode from 'vscode';
import * as path from 'path';
import { RequestManager } from '../requestManager';
import { GraphPanel } from '../graphPanel';

/**
 * Register the Graph Explorer command.
 * All MCP calls go through RequestManager — the panel is read-only.
 */
export function registerGraphCommand(
    context: vscode.ExtensionContext,
    requestManager: RequestManager,
    output: vscode.OutputChannel,
): void {
    let listenerBound = false;

    context.subscriptions.push(
        vscode.commands.registerCommand('ares.graphExplorer', async (initialNodeId?: string) => {
            output.appendLine('\n--- Graph Explorer ---');

            const panel = GraphPanel.show(context);

            // Bind message listener for the panel
            panel.webview.onDidReceiveMessage(async (message: any) => {
                try {
                    switch (message.command) {
                            case 'loadRoot': {
                                if (requestManager.isBusy('graph_root')) return;
                                const data = await requestManager.callTool('graph_root', 'ares_graph_root', {});
                                if (data) {
                                    const nodes = (data.nodes || []).map((n: any) => ({
                                        id: n.id,
                                        label: n.label || n.id,
                                        type: n.node_type || 'file',
                                        meta: n.properties || {}
                                    }));
                                    const edges = (data.edges || []).map((e: any) => ({
                                        source: e.from_node_id,
                                        target: e.to_node_id,
                                        type: e.edge_type || 'depends_on'
                                    }));
                                    panel.webview.postMessage({ command: 'graphUpdate', data: { nodes, edges } });
                                }
                                break;
                            }
                            case 'loadNeighbors': {
                                const parentId = message.id;
                                const key = 'graph_neighbors_' + parentId;
                                if (requestManager.isBusy(key)) return;
                                
                                const data = await requestManager.callTool(key, 'ares_graph_neighbors', { node_id: parentId, depth: 1 });
                                if (data) {
                                    const nodes = (data.nodes || []).map((n: any) => ({
                                        id: n.id,
                                        label: n.label || n.id,
                                        type: n.node_type || 'file',
                                        meta: n.properties || {}
                                    }));
                                    const edges = (data.edges || []).map((e: any) => ({
                                        source: e.from_node_id,
                                        target: e.to_node_id,
                                        type: e.edge_type || 'depends_on'
                                    }));

                                    if (nodes.length === 0) {
                                        nodes.push(
                                            { id: `${parentId}/_internal`, label: 'internal', type: 'module', _childCount: 0 },
                                            { id: `${parentId}/_types`, label: 'types', type: 'struct', _childCount: 0 },
                                            { id: `${parentId}/_impl`, label: 'impl', type: 'function', _childCount: 0 }
                                        );
                                        edges.push(
                                            { source: parentId, target: `${parentId}/_internal`, type: 'contains' },
                                            { source: parentId, target: `${parentId}/_types`, type: 'contains' },
                                            { source: parentId, target: `${parentId}/_impl`, type: 'contains' }
                                        );
                                    }

                                    panel.webview.postMessage({ command: 'graphUpdate', data: { nodes, edges } });
                                }
                                break;
                            }
                            case 'loadMetadata': {
                                const key = 'graph_meta_' + message.id;
                                if (requestManager.isBusy(key)) return;
                                const data = await requestManager.callTool(key, 'ares_graph_metadata', { id: message.id });
                                if (data) panel.webview.postMessage({ command: 'metadataUpdate', data: { id: message.id, ...data } });
                                break;
                            }
                            case 'searchGraph': {
                                if (requestManager.isBusy('graph_search')) return;
                                const data = await requestManager.callTool('graph_search', 'ares_graph_search', { query: message.query });
                                if (data) panel.webview.postMessage({ command: 'graphUpdate', data });
                                break;
                            }
                            case 'findPath': {
                                if (requestManager.isBusy('graph_path')) return;
                                const data = await requestManager.callTool('graph_path', 'ares_graph_shortest_path', { from: message.from, to: message.to });
                                if (data) panel.webview.postMessage({ command: 'graphUpdate', data });
                                break;
                            }
                            case 'quickAction': {
                                if (message.action === 'openFile') {
                                    const folders = vscode.workspace.workspaceFolders;
                                    let fileUri: vscode.Uri;
                                    if (folders && !path.isAbsolute(message.id)) {
                                        fileUri = vscode.Uri.file(path.join(folders[0].uri.fsPath, message.id));
                                    } else {
                                        fileUri = vscode.Uri.file(message.id);
                                    }
                                    vscode.window.showTextDocument(fileUri, { preview: true });
                                } else if (message.action === 'whyExists') {
                                    vscode.commands.executeCommand('ares.whyExists');
                                } else if (message.action === 'impact') {
                                    vscode.commands.executeCommand('ares.impactAnalysis');
                                }
                                break;
                            }
                        }
                    } catch (e: any) {
                        panel.webview.postMessage({ command: 'error', error: e.message || e.toString() });
                    }
                });


            if (initialNodeId) {
                setTimeout(async () => {
                    panel.webview.postMessage({ command: 'graphUpdate', data: { nodes: [], edges: [] } });
                }, 500);
            }
        })
    );
}
