import { Bell } from 'lucide-react';

export const TopBar = ({ title, subtitle, status }: { title: string, subtitle?: string, status?: 'live' | 'offline' }) => (
  <header className="h-20 border-b border-ares-border flex items-center justify-between px-8 glass-panel sticky top-0 z-10">
    <div className="flex flex-col">
      <h1 className="text-2xl font-bold text-ares-text">{title}</h1>
      {subtitle && <span className="text-sm text-ares-muted">{subtitle}</span>}
    </div>
    <div className="flex items-center gap-4">
      {status === 'live' && (
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-ares-accent/10 border border-ares-accent/20">
          <span className="w-2 h-2 rounded-full bg-ares-accent animate-pulse"></span>
          <span className="text-sm font-medium text-ares-accent">Live Telemetry</span>
        </div>
      )}
      {status === 'offline' && (
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
