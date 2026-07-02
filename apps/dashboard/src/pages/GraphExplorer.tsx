import { useState, useEffect, useRef } from 'react';
import cytoscape from 'cytoscape';
import dagre from 'cytoscape-dagre';
import { TopBar } from '../components/TopBar';
import { Loader2, RefreshCw } from 'lucide-react';

cytoscape.use(dagre);

export function GraphExplorer() {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<cytoscape.Core | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedNode, setSelectedNode] = useState<any | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  // Initialize cytoscape
  useEffect(() => {
    if (!containerRef.current) return;
    
    cyRef.current = cytoscape({
      container: containerRef.current,
      style: [
        {
          selector: 'node',
          style: {
            'background-color': '#3b82f6',
            'label': 'data(label)',
            'color': '#fff',
            'text-valign': 'center',
            'text-halign': 'center',
            'font-size': '12px',
            'width': '60px',
            'height': '60px',
          }
        },
        {
          selector: 'edge',
          style: {
            'width': 2,
            'line-color': '#4b5563',
            'target-arrow-color': '#4b5563',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier'
          }
        }
      ],
      layout: {
        name: 'dagre'
      }
    });

    cyRef.current.on('tap', 'node', (evt) => {
      const node = evt.target;
      setSelectedNode(node.data());
      // Handle lazy loading (expand neighbors)
      // fetchNeighbors(node.id());
    });

    return () => {
      if (cyRef.current) {
        cyRef.current.destroy();
        cyRef.current = null;
      }
    };
  }, []);

  const loadRootNode = async () => {
    setLoading(true);
    // In a real implementation, this would fetch from the MCP or REST API
    // ares_graph_root -> { nodes, edges }
    setTimeout(() => {
      if (!cyRef.current) return;
      cyRef.current.elements().remove();
      cyRef.current.add([
        { group: 'nodes', data: { id: 'root', label: 'Repository Root' } }
      ]);
      cyRef.current.layout({ name: 'dagre' }).run();
      cyRef.current.fit();
      setLoading(false);
      setRefreshing(false);
    }, 600);
  };

  useEffect(() => {
    loadRootNode();
  }, []);

  const handleRefresh = () => {
    setRefreshing(true);
    loadRootNode();
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden relative">
      <TopBar 
        title="Graph Explorer" 
        subtitle="Explore repository intelligence with lazy loading" 
      />
      
      <div className="flex justify-between items-center px-8 pt-4 pb-2 border-b border-ares-border bg-ares-panel">
        <div className="text-sm text-ares-muted">
          Click nodes to lazy-load dependencies and inspect details.
        </div>
        <button 
          onClick={handleRefresh}
          disabled={refreshing}
          className="flex items-center gap-2 px-4 py-2 bg-ares-accent text-white rounded-lg hover:bg-ares-accent/80 transition-colors disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
          {refreshing ? 'Loading...' : 'Reset Graph'}
        </button>
      </div>

      <div className="flex-1 w-full h-full relative">
        {loading && !refreshing && (
          <div className="absolute inset-0 z-10 flex items-center justify-center bg-ares-bg/50 backdrop-blur-sm">
            <Loader2 className="w-10 h-10 text-ares-accent animate-spin" />
          </div>
        )}
        <div ref={containerRef} className="w-full h-full" />
        
        {/* Node Inspector Drawer */}
        {selectedNode && (
          <div className="absolute top-0 right-0 h-full w-80 bg-ares-panel border-l border-ares-border shadow-2xl p-6 overflow-y-auto z-20 animate-fade">
            <div className="flex justify-between items-center mb-6">
              <h2 className="text-lg font-bold text-ares-text">Node Inspector</h2>
              <button onClick={() => setSelectedNode(null)} className="text-ares-muted hover:text-ares-text">✕</button>
            </div>
            
            <div className="flex flex-col gap-4">
              <div>
                <div className="text-xs text-ares-muted uppercase tracking-wider mb-1">ID</div>
                <div className="font-mono text-sm bg-black/20 p-2 rounded break-all">{selectedNode.id}</div>
              </div>
              
              <div>
                <div className="text-xs text-ares-muted uppercase tracking-wider mb-1">Label</div>
                <div className="text-sm">{selectedNode.label}</div>
              </div>
              
              <div className="pt-4 border-t border-ares-border">
                <button className="w-full py-2 bg-white/5 hover:bg-white/10 rounded border border-white/10 text-sm transition-colors cursor-pointer text-ares-text font-medium">
                  Why Exists
                </button>
                <button className="w-full py-2 bg-white/5 hover:bg-white/10 rounded border border-white/10 text-sm mt-2 transition-colors cursor-pointer text-ares-text font-medium">
                  Impact Analysis
                </button>
                <button className="w-full py-2 bg-white/5 hover:bg-white/10 rounded border border-white/10 text-sm mt-2 transition-colors cursor-pointer text-ares-text font-medium">
                  Traceability
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
