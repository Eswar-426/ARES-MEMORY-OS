import { TopBar } from '../components/TopBar';
import { ShieldCheck } from 'lucide-react';

export function ProviderHealth() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <TopBar title="Provider Health" subtitle="Model API Status" />
      <div className="flex-1 w-full h-full relative p-8">
        <div className="glass-panel p-6 rounded-2xl glow-border">
          <div className="flex items-center gap-3 mb-4 text-ares-text">
            <ShieldCheck className="w-6 h-6 text-emerald-400" />
            <h2 className="text-xl font-bold">API Providers</h2>
          </div>
          <p className="text-ares-muted mb-6">
            Provider health checks and latency metrics will be displayed here.
          </p>
          <div className="p-4 border border-ares-border bg-black/20 rounded-xl">
            <div className="flex justify-between items-center">
              <span className="font-semibold text-ares-text">MockPlannerProvider</span>
              <span className="px-3 py-1 bg-emerald-500/20 text-emerald-400 text-xs rounded-full border border-emerald-500/30">Healthy</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
