import { useState, useEffect, useCallback } from 'react';
import { TopBar } from '../components/TopBar';
import { ReactFlow, MiniMap, Controls, Background, useNodesState, useEdgesState, addEdge } from '@xyflow/react';
import type { Node, Edge } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Loader2, RefreshCw } from 'lucide-react';

export function ArchitectureExplorer() {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  // In a real implementation we would fetch from /api/v1/memory/graph or similar and filter by architecture types
  const fetchGraph = async () => {
    setLoading(true);
    // Simulate API delay
    setTimeout(() => {
      setNodes([
        { id: 'app', position: { x: 250, y: 5 }, data: { label: 'ares-app' } },
        { id: 'api', position: { x: 100, y: 100 }, data: { label: 'ares-api' } },
        { id: 'store', position: { x: 400, y: 100 }, data: { label: 'ares-store' } },
        { id: 'core', position: { x: 250, y: 200 }, data: { label: 'ares-core' } },
      ]);
      setEdges([
        { id: 'e-app-api', source: 'app', target: 'api', animated: true },
        { id: 'e-app-store', source: 'app', target: 'store' },
        { id: 'e-api-core', source: 'api', target: 'core' },
        { id: 'e-store-core', source: 'store', target: 'core' },
      ]);
      setLoading(false);
      setRefreshing(false);
    }, 600);
  };

  useEffect(() => {
    fetchGraph();
  }, []);

  const onConnect = useCallback((params: any) => setEdges((eds) => addEdge(params, eds)), [setEdges]);

  const handleRefresh = () => {
    setRefreshing(true);
    fetchGraph();
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <TopBar 
        title="Architecture Explorer" 
        subtitle="Visualize workspace crates, internal dependencies, and API routes" 
      />
      
      <div className="flex justify-between items-center px-8 pt-4 pb-2 border-b border-ares-border bg-ares-panel">
        <div className="text-sm text-ares-muted">
          Nodes are loaded from the knowledge graph. Click refresh to trigger a new scan.
        </div>
        <button 
          onClick={handleRefresh}
          disabled={refreshing}
          className="flex items-center gap-2 px-4 py-2 bg-ares-accent text-white rounded-lg hover:bg-ares-accent/80 transition-colors disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
          {refreshing ? 'Scanning...' : 'Refresh Scan'}
        </button>
      </div>

      <div className="flex-1 w-full h-full relative p-4">
        {loading && !refreshing ? (
          <div className="flex items-center justify-center w-full h-full">
            <Loader2 className="w-10 h-10 text-ares-accent animate-spin" />
          </div>
        ) : (
          <div className="w-full h-full glass-panel rounded-2xl overflow-hidden border border-ares-border">
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              fitView
              colorMode="dark"
            >
              <Controls />
              <MiniMap />
              <Background gap={12} size={1} />
            </ReactFlow>
          </div>
        )}
      </div>
    </div>
  );
}
