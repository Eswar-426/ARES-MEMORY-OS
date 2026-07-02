import { Routes, Route } from 'react-router-dom';
import { Sidebar } from './components/Sidebar';
import { Overview } from './pages/Overview';
import { MemoryExplorer } from './pages/MemoryExplorer';
import { ArchitectureExplorer } from './pages/ArchitectureExplorer';
import { GraphExplorer } from './pages/GraphExplorer';
import { DecisionTimeline } from './pages/DecisionTimeline';
import { ContextInjection } from './pages/ContextInjection';
import { PlannerExplorer } from './pages/PlannerExplorer';
import { ProviderHealth } from './pages/ProviderHealth';
import { Settings } from './pages/Settings';

function App() {
  return (
    <div className="flex min-h-screen bg-ares-bg">
      <Sidebar />
      <Routes>
        <Route path="/" element={<Overview />} />
        <Route path="/memory" element={<MemoryExplorer />} />
        <Route path="/architecture" element={<ArchitectureExplorer />} />
        <Route path="/graph" element={<GraphExplorer />} />
        <Route path="/decisions" element={<DecisionTimeline />} />
        <Route path="/context" element={<ContextInjection />} />
        <Route path="/planner" element={<PlannerExplorer />} />
        <Route path="/health" element={<ProviderHealth />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </div>
  );
}

export default App;
