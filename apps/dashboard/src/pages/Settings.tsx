import { TopBar } from '../components/TopBar';
import { Settings as SettingsIcon } from 'lucide-react';

export function Settings() {
  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <TopBar title="Settings" subtitle="System Configuration" />
      <div className="flex-1 w-full h-full relative p-8">
        <div className="glass-panel p-6 rounded-2xl glow-border">
          <div className="flex items-center gap-3 mb-4 text-ares-text">
            <SettingsIcon className="w-6 h-6 text-ares-accent" />
            <h2 className="text-xl font-bold">System Settings</h2>
          </div>
          <p className="text-ares-muted">
            Configuration options for ARES OS will be added here.
          </p>
        </div>
      </div>
    </div>
  );
}
