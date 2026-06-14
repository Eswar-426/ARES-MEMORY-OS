import { 
  BrainCircuit, 
  LayoutDashboard,
  Database,
  Network,
  Clock,
  Settings,
  ShieldCheck,
  Compass
} from 'lucide-react';
import { NavLink } from 'react-router-dom';

const NavItem = ({ icon, label, to }: any) => (
  <NavLink 
    to={to} 
    className={({ isActive }) => 
      `flex items-center gap-3 px-4 py-3 rounded-lg cursor-pointer transition-colors ${
        isActive 
          ? 'bg-ares-accent/20 text-ares-accent' 
          : 'text-ares-muted hover:bg-white/5 hover:text-ares-text'
      }`
    }
  >
    {icon}
    <span className="font-medium">{label}</span>
  </NavLink>
);

export const Sidebar = () => (
  <aside className="w-64 border-r border-ares-border bg-ares-panel p-6 flex flex-col gap-6 h-screen sticky top-0">
    <div className="flex items-center gap-3 text-ares-text font-bold text-xl tracking-wide">
      <BrainCircuit className="text-ares-accent w-8 h-8" />
      ARES OS
    </div>
    <nav className="flex flex-col gap-2 mt-8">
      <NavItem icon={<LayoutDashboard />} label="Dashboard" to="/" />
      <NavItem icon={<Database />} label="Memory Explorer" to="/memory" />
      <NavItem icon={<Network />} label="Architecture" to="/architecture" />
      <NavItem icon={<Clock />} label="Decisions" to="/decisions" />
      <NavItem icon={<Compass />} label="Autonomous Planner" to="/planner" />
      <div className="h-4"></div>
      <NavItem icon={<ShieldCheck />} label="Provider Health" to="/health" />
      <NavItem icon={<Settings />} label="Settings" to="/settings" />
    </nav>
  </aside>
);
