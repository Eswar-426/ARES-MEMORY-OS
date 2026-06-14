import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  BrainCircuit, 
  ShieldCheck, 
  ServerCrash, 
  GitCommit, 
  LayoutDashboard,
  Database,
  Settings,
  Bell,
  Loader2
} from 'lucide-react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip as RechartsTooltip } from 'recharts';

interface TelemetryReport {
  id: string;
  timestamp: string;
  source: string;
  continuity_score: number;
  provider_health: Record<string, any>;
  fallback_events: any[];
  dynamic_chains: Record<string, any[]>;
}

const providerHealthData = [
  { name: 'Healthy', value: 3, color: '#10b981' },
  { name: 'Rate Limited', value: 1, color: '#f59e0b' },
  { name: 'Disabled (404)', value: 1, color: '#ef4444' },
];

const fallbackEvents = [
  { time: '10:42:01', event: 'Gemini (403 Forbidden)', type: 'error' },
  { time: '10:42:02', event: 'Fallback to Llama-3.3-70b', type: 'success' },
  { time: '10:45:15', event: 'Simulated 40% Context Loss', type: 'warning' },
  { time: '10:45:16', event: 'ARES rebuilt context graph from disk (787 bytes)', type: 'success' },
];

const Sidebar = () => (
  <aside className="w-64 border-r border-ares-border bg-ares-panel p-6 flex flex-col gap-6 h-screen sticky top-0">
    <div className="flex items-center gap-3 text-ares-text font-bold text-xl tracking-wide">
      <BrainCircuit className="text-ares-accent w-8 h-8" />
      ARES OS
    </div>
    <nav className="flex flex-col gap-2 mt-8">
      <NavItem icon={<LayoutDashboard />} label="Dashboard" active />
      <NavItem icon={<Database />} label="Memory Explorer" />
      <NavItem icon={<ShieldCheck />} label="Provider Health" />
      <NavItem icon={<Settings />} label="Settings" />
    </nav>
  </aside>
);

const NavItem = ({ icon, label, active = false }: any) => (
  <div className={`flex items-center gap-3 px-4 py-3 rounded-lg cursor-pointer transition-colors ${active ? 'bg-ares-accent/20 text-ares-accent' : 'text-ares-muted hover:bg-white/5 hover:text-ares-text'}`}>
    {icon}
    <span className="font-medium">{label}</span>
  </div>
);

function App() {
  const [telemetry, setTelemetry] = useState<TelemetryReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchTelemetry = async () => {
      try {
        const res = await fetch('http://localhost:3000/api/v1/telemetry/latest');
        if (!res.ok) {
          if (res.status === 404) {
            setError("No telemetry data yet. Run the benchmark engine.");
          } else {
            setError("Failed to fetch telemetry.");
          }
          setLoading(false);
          return;
        }
        const data = await res.json();
        setTelemetry(data);
        setError(null);
      } catch (e) {
        setError("Cannot connect to ARES API.");
      } finally {
        setLoading(false);
      }
    };

    fetchTelemetry();
    const interval = setInterval(fetchTelemetry, 5000);
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="flex min-h-screen bg-ares-bg items-center justify-center flex-col gap-4">
        <Loader2 className="w-10 h-10 text-ares-accent animate-spin" />
        <span className="text-ares-muted">Connecting to ARES Core...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-screen bg-ares-bg">
        <Sidebar />
        <main className="flex-1 flex flex-col overflow-hidden">
          <TopBar isLive={false} />
          <div className="flex-1 flex items-center justify-center p-8">
            <div className="glass-panel p-8 rounded-2xl flex flex-col items-center gap-4 max-w-md text-center">
              <ServerCrash className="w-12 h-12 text-ares-danger" />
              <h2 className="text-xl font-bold text-ares-text">Telemetry Offline</h2>
              <p className="text-ares-muted">{error}</p>
            </div>
          </div>
        </main>
      </div>
    );
  }

  if (!telemetry) return null;

  // Compute stats for charts
  let healthy = 0;
  let rate_limited = 0;
  let disabled = 0;
  for (const [_, record] of Object.entries(telemetry.provider_health || {})) {
    if (record.state === 'Healthy') healthy++;
    else if (record.state === 'RateLimited') rate_limited++;
    else if (record.state === 'Disabled') disabled++;
  }
  
  const liveHealthData = [
    { name: 'Healthy', value: healthy, color: '#10b981' },
    { name: 'Rate Limited', value: rate_limited, color: '#f59e0b' },
    { name: 'Disabled', value: disabled, color: '#ef4444' },
  ].filter(d => d.value > 0);

  return (
    <div className="flex min-h-screen bg-ares-bg">
      <Sidebar />
      <main className="flex-1 flex flex-col overflow-hidden">
        <TopBar isLive={true} />
        <div className="p-8 overflow-y-auto">
          <div className="max-w-7xl mx-auto space-y-8">
            
            {/* Top Row */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
              <div className="lg:col-span-2">
                <ContinuityPulse score={telemetry.continuity_score} />
              </div>
              <div className="lg:col-span-1">
                <ProviderHealth data={liveHealthData} />
              </div>
            </div>

            {/* Bottom Row */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <FallbackFeed events={telemetry.fallback_events || []} />
              
              <div className="glass-panel rounded-2xl p-6 glow-border">
                <h2 className="text-lg font-semibold text-ares-muted flex items-center gap-2 mb-4">
                  <GitCommit className="w-5 h-5" /> Dynamic Chains Assembled
                </h2>
                <div className="space-y-4">
                  <ChainItem role="Architecture" models={telemetry.dynamic_chains?.architecture || []} />
                  <ChainItem role="Feature" models={telemetry.dynamic_chains?.feature || []} />
                  <ChainItem role="Debug" models={telemetry.dynamic_chains?.debug || []} />
                </div>
              </div>
            </div>

          </div>
        </div>
      </main>
    </div>
  );
}

const ContinuityPulse = ({ score }: { score: number }) => (
  <div className="glass-panel rounded-2xl p-6 glow-border flex flex-col gap-4">
    <div className="flex items-center justify-between">
      <h2 className="text-lg font-semibold text-ares-muted flex items-center gap-2">
        <Activity className="w-5 h-5" /> ARES Continuity
      </h2>
    </div>
    <div className="flex items-end gap-4">
      <span className="text-6xl font-black text-ares-accent">{score.toFixed(2)}%</span>
      <span className="text-lg text-ares-muted mb-2 border border-ares-border px-2 py-1 rounded-md bg-white/5">
        Live Benchmark
      </span>
    </div>
  </div>
);

const ProviderHealth = ({ data }: { data: any[] }) => (
  <div className="glass-panel rounded-2xl p-6 glow-border flex flex-col h-full">
    <h2 className="text-lg font-semibold text-ares-muted flex items-center gap-2 mb-4">
      <ShieldCheck className="w-5 h-5" /> Provider Registry
    </h2>
    <div className="flex-1 min-h-[200px]">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={60}
            outerRadius={80}
            paddingAngle={5}
            dataKey="value"
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          <RechartsTooltip 
            contentStyle={{ backgroundColor: 'var(--color-ares-bg)', borderColor: 'var(--color-ares-border)', borderRadius: '8px' }}
            itemStyle={{ color: 'var(--color-ares-text)' }}
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
    <div className="flex flex-col gap-2 mt-4">
      {data.map(d => (
        <div key={d.name} className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full" style={{ backgroundColor: d.color }}></span>
            <span className="text-ares-text">{d.name}</span>
          </div>
          <span className="font-medium text-ares-muted">{d.value}</span>
        </div>
      ))}
    </div>
  </div>
);

const FallbackFeed = ({ events }: { events: any[] }) => (
  <div className="glass-panel rounded-2xl p-6 glow-border flex flex-col h-full">
    <h2 className="text-lg font-semibold text-ares-muted flex items-center gap-2 mb-6">
      <ServerCrash className="w-5 h-5" /> Execution Log
    </h2>
    <div className="flex flex-col gap-0 relative">
      {events.length === 0 ? (
         <span className="text-ares-muted italic">No fallback events recorded in this run.</span>
      ) : (
        <>
          <div className="absolute left-[11px] top-2 bottom-2 w-[2px] bg-ares-border"></div>
          {events.map((evt, i) => (
            <div key={i} className="flex gap-4 pb-6 relative">
              <div className={`w-6 h-6 rounded-full border-4 border-ares-bg flex items-center justify-center relative z-10 
                ${evt.type === 'success' ? 'bg-ares-accent' : evt.type === 'error' ? 'bg-ares-danger' : 'bg-ares-warning'}`}>
              </div>
              <div className="flex flex-col pt-0.5">
                <span className="text-xs font-mono text-ares-muted mb-1">{evt.time}</span>
                <span className="text-sm text-ares-text">{evt.event}</span>
              </div>
            </div>
          ))}
        </>
      )}
    </div>
  </div>
);

const TopBar = ({ isLive }: { isLive: boolean }) => (
  <header className="h-20 border-b border-ares-border flex items-center justify-between px-8 glass-panel sticky top-0 z-10">
    <div className="flex flex-col">
      <h1 className="text-2xl font-bold text-ares-text">Continuity Validation Dashboard</h1>
      <span className="text-sm text-ares-muted">Real-time benchmark telemetry and provider health</span>
    </div>
    <div className="flex items-center gap-4">
      {isLive ? (
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-ares-accent/10 border border-ares-accent/20">
          <span className="w-2 h-2 rounded-full bg-ares-accent animate-pulse"></span>
          <span className="text-sm font-medium text-ares-accent">Live Telemetry</span>
        </div>
      ) : (
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-ares-danger/10 border border-ares-danger/20">
          <span className="w-2 h-2 rounded-full bg-ares-danger"></span>
          <span className="text-sm font-medium text-ares-danger">Offline</span>
        </div>
      )}
      <button className="p-2 rounded-full hover:bg-white/5 text-ares-muted transition-colors">
        <Bell className="w-5 h-5" />
      </button>
    </div>
  </header>
);

const ChainItem = ({ role, models }: any) => (
  <div className="p-4 rounded-xl border border-ares-border bg-white/5 flex flex-col gap-2 hover:bg-white/10 transition-colors">
    <div className="text-sm font-medium text-ares-accent uppercase tracking-wider">{role}</div>
    <div className="flex flex-wrap gap-2">
      {models.map((m: any, i: number) => (
        <span key={i} className="text-xs px-2 py-1 rounded bg-black/40 text-ares-text border border-white/10 font-mono">
          {i + 1}. {m.provider}/{m.id}
        </span>
      ))}
    </div>
  </div>
);

export default App;
