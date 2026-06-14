import { useState, useEffect } from 'react';
import { TopBar } from '../components/TopBar';
import { Loader2, Calendar, CheckCircle2, AlertTriangle, Info } from 'lucide-react';

interface Decision {
  id: string;
  title: string;
  decision_text: string;
  reason: string;
  status: string;
  created_at: number;
}

export function DecisionTimeline() {
  const [decisions, setDecisions] = useState<Decision[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate API delay, would fetch from /api/v1/memory/decisions
    setTimeout(() => {
      setDecisions([
        {
          id: 'dec_1',
          title: 'Use React Flow for Visualizations',
          decision_text: 'We will use @xyflow/react for rendering the Memory Graph and Architecture Explorer.',
          reason: 'React Flow provides out-of-the-box support for interactive graphs, nodes, and mini-maps which speeds up development.',
          status: 'accepted',
          created_at: Date.now() - 86400000, // 1 day ago
        },
        {
          id: 'dec_2',
          title: 'Paginated APIs for Graph Nodes',
          decision_text: 'All list endpoints will return a paginated envelope.',
          reason: 'To avoid loading massive JSON payloads when the workspace graph grows large.',
          status: 'accepted',
          created_at: Date.now() - 172800000, // 2 days ago
        },
        {
          id: 'dec_3',
          title: 'Separate Stores for Events and Decisions',
          decision_text: 'We proposed separating SQLite tables for events and decisions.',
          reason: 'Decisions have complex nested fields (alternatives, risks) while events are simple JSON blobs.',
          status: 'proposed',
          created_at: Date.now() - 259200000, // 3 days ago
        }
      ]);
      setLoading(false);
    }, 500);
  }, []);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'accepted': return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case 'proposed': return <Info className="w-5 h-5 text-blue-500" />;
      case 'rejected': return <AlertTriangle className="w-5 h-5 text-red-500" />;
      default: return <Info className="w-5 h-5 text-gray-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'accepted': return 'border-green-500/30 bg-green-500/10 text-green-400';
      case 'proposed': return 'border-blue-500/30 bg-blue-500/10 text-blue-400';
      case 'rejected': return 'border-red-500/30 bg-red-500/10 text-red-400';
      default: return 'border-gray-500/30 bg-gray-500/10 text-gray-400';
    }
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <TopBar 
        title="Decision Timeline" 
        subtitle="Chronological history of architectural decisions" 
      />
      <div className="flex-1 p-8 overflow-y-auto">
        <div className="max-w-4xl mx-auto">
          {loading ? (
            <div className="flex items-center justify-center py-20">
              <Loader2 className="w-10 h-10 text-ares-accent animate-spin" />
            </div>
          ) : (
            <div className="relative border-l border-ares-border ml-6 space-y-12">
              {decisions.map((decision) => (
                <div key={decision.id} className="relative pl-8">
                  {/* Timeline dot */}
                  <div className="absolute -left-3 top-1 w-6 h-6 rounded-full bg-ares-panel border-2 border-ares-border flex items-center justify-center">
                    <div className="w-2 h-2 rounded-full bg-ares-accent" />
                  </div>
                  
                  {/* Content card */}
                  <div className="glass-panel p-6 rounded-2xl glow-border">
                    <div className="flex justify-between items-start mb-4">
                      <div className="flex items-center gap-3">
                        {getStatusIcon(decision.status)}
                        <h3 className="text-xl font-bold text-ares-text">{decision.title}</h3>
                      </div>
                      <div className={`px-3 py-1 rounded-full text-xs font-semibold border uppercase tracking-wider ${getStatusColor(decision.status)}`}>
                        {decision.status}
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2 text-sm text-ares-muted mb-6">
                      <Calendar className="w-4 h-4" />
                      {new Date(decision.created_at).toLocaleString()}
                    </div>

                    <div className="space-y-4">
                      <div>
                        <div className="text-sm font-semibold text-ares-muted uppercase tracking-wider mb-1">Decision</div>
                        <div className="text-ares-text bg-white/5 p-4 rounded-xl border border-white/5">{decision.decision_text}</div>
                      </div>
                      <div>
                        <div className="text-sm font-semibold text-ares-muted uppercase tracking-wider mb-1">Reasoning</div>
                        <div className="text-ares-muted">{decision.reason}</div>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
