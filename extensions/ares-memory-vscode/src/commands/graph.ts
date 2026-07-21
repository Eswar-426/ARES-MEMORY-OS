import * as vscode from 'vscode';
import * as path from 'path';
import { RequestManager } from '../requestManager';
import { GraphPanel } from '../graphPanel';

const extractAnswer = (data: any) => data.answer || data;

const transformNodes = (nodes: any[]) => (nodes || []).map((n: any) => ({
    id: n.file_path || n.id || n.label || '',
    label: n.label || n.file_path || 'unknown',
    type: (n.node_type || n.type || 'file').toLowerCase(),
    meta: n.properties || n.meta || {},
    _childCount: n._childCount || n.child_count || 0,
}));

const transformEdges = (edges: any[]) => (edges || []).map((e: any) => ({
    source: e.from || e.source || '',
    target: e.to || e.target || '',
    type: e.type || e.edge_type || 'contains',
}));

// Shortest path returns hops with label-based from/to — map back to node IDs
const transformShortestPath = (answer: any) => {
    const pathNodes = transformNodes(answer.path || []);
    const labelToId = new Map<string, string>();
    pathNodes.forEach((n: any) => {
        if (n.label && n.id) labelToId.set(n.label, n.id);
    });
    const edges = (answer.hops || []).map((h: any) => ({
        source: labelToId.get(h.from) || h.from || '',
        target: labelToId.get(h.to) || h.to || '',
        type: h.via || h.type || 'depends_on',
    }));
    return { nodes: pathNodes, edges };
};

// graph_neighbors returns {neighbors: [{direction, file_path, label, node_type, relationship}]}
// NOT {nodes, edges}. Convert to nodes+edges format.
const transformNeighbors = (answer: any, parentId: string) => {
    const raw = answer.neighbors || [];
    const nodes: any[] = [];
    const edges: any[] = [];
    const seen = new Set<string>();

    for (const n of raw) {
        // Skip null file_paths (person/commit nodes — not visualizable)
        const nodeId = n.file_path;
        if (!nodeId || nodeId === 'null' || seen.has(nodeId)) continue;
        seen.add(nodeId);

        nodes.push({
            id: nodeId,
            label: n.label || nodeId,
            type: (n.node_type || 'file').toLowerCase(),
            meta: {},
            _childCount: 0,
        });

        if (n.direction === 'outgoing') {
            edges.push({ source: parentId, target: nodeId, type: n.relationship || 'contains' });
        } else if (n.direction === 'incoming') {
            edges.push({ source: nodeId, target: parentId, type: n.relationship || 'contains' });
        }
    }

    return { nodes, edges };
};

export function registerGraphCommand(
    context: vscode.ExtensionContext,
    requestManager: RequestManager,
    output: vscode.OutputChannel,
): void {
    context.subscriptions.push(
        vscode.commands.registerCommand('ares.graphExplorer', async () => {
            output.appendLine('\n--- Graph Explorer ---');
            const panel = GraphPanel.show(context);

            panel.webview.onDidReceiveMessage(async (message: any) => {
                try {
                    switch (message.command) {
                        case 'loadRoot': {
                            if (requestManager.isBusy('graph_root')) return;
                            const data = await requestManager.callTool('graph_root', 'ares_graph_root', { name: message.projectId || 'ares_memory_os' });
                            if (data) {
                                const answer = extractAnswer(data);
                                panel.webview.postMessage({
                                    command: 'graphUpdate',
                                    data: { nodes: transformNodes(answer.nodes), edges: transformEdges(answer.edges) }
                                });
                            }
                            break;
                        }
                        case 'loadNeighbors': {
                            const parentId = message.id;
                            const key = 'graph_neighbors_' + parentId;
                            if (requestManager.isBusy(key)) return;
                            const data = await requestManager.callTool(key, 'ares_graph_neighbors', { node_id: parentId, depth: 1 });
                            if (data) {
                                const answer = extractAnswer(data);
                                const { nodes, edges } = transformNeighbors(answer, parentId);
                                panel.webview.postMessage({
                                    command: 'graphUpdate',
                                    data: { nodes, edges }
                                });
                            }
                            break;
                        }
                        case 'loadMetadata': {
                            const key = 'graph_meta_' + message.id;
                            if (requestManager.isBusy(key)) return;
                            // Bug fix: original passed 'id' but tool expects 'file_path'
                            const data = await requestManager.callTool(key, 'ares_graph_metadata', { file_path: message.id });
                            if (data) {
                                const answer = extractAnswer(data);
                                panel.webview.postMessage({ command: 'metadataUpdate', data: { id: message.id, ...answer } });
                            }
                            break;
                        }
                        case 'searchGraph': {
                            if (requestManager.isBusy('graph_search')) return;
                            const data = await requestManager.callTool('graph_search', 'ares_graph_search', { query: message.query });
                            if (data) {
                                const answer = extractAnswer(data);
                                panel.webview.postMessage({
                                    command: 'graphUpdate',
                                    data: { nodes: transformNodes(answer.nodes), edges: transformEdges(answer.edges) }
                                });
                            }
                            break;
                        }
                        case 'findPath': {
                            if (requestManager.isBusy('graph_path')) return;
                            // Bug fix: original passed 'from'/'to' but tool expects 'from_id'/'to_id'
                            const data = await requestManager.callTool('graph_path', 'ares_graph_shortest_path', { from_id: message.from, to_id: message.to });
                            if (data) {
                                const answer = extractAnswer(data);
                                const { nodes, edges } = transformShortestPath(answer);
                                panel.webview.postMessage({ command: 'graphUpdate', data: { nodes, edges } });
                            }
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
        })
    );
}
