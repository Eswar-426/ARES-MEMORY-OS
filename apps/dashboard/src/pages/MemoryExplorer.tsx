import { useState, useEffect, useCallback } from 'react';
import { TopBar } from '../components/TopBar';
import { ReactFlow, MiniMap, Controls, Background, useNodesState, useEdgesState, addEdge } from '@xyflow/react';
import type { Node, Edge } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { Loader2 } from 'lucide-react';

export function MemoryExplorer() {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [loading, setLoading] = useState(true);

  // In a real implementation we would fetch from /api/v1/memory/graph
  useEffect(() => {
    // For now we simulate the load
    setTimeout(() => {
      setNodes([
        { id: '1', position: { x: 250, y: 5 }, data: { label: 'ARES System' } },
        { id: '2', position: { x: 100, y: 100 }, data: { label: 'Memory Core' } },
        { id: '3', position: { x: 400, y: 100 }, data: { label: 'Scanner' } },
      ]);
      setEdges([
        { id: 'e1-2', source: '1', target: '2', animated: true },
        { id: 'e1-3', source: '1', target: '3' },
      ]);
      setLoading(false);
    }, 500);
  }, []);

  const onConnect = useCallback((params: any) => setEdges((eds) => addEdge(params, eds)), [setEdges]);

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <TopBar title="Memory Explorer" subtitle="Knowledge Graph Visualization" />
      <div className="flex-1 w-full h-full relative p-4">
        {loading ? (
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
