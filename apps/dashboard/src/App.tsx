import { Routes, Route } from 'react-router-dom';
import { Sidebar } from './components/Sidebar';
import { Overview } from './pages/Overview';
import { MemoryExplorer } from './pages/MemoryExplorer';
import { ArchitectureExplorer } from './pages/ArchitectureExplorer';
import { DecisionTimeline } from './pages/DecisionTimeline';
import { PlannerExplorer } from './pages/PlannerExplorer';

function App() {
  return (
    <div className="flex min-h-screen bg-ares-bg">
      <Sidebar />
      <Routes>
        <Route path="/" element={<Overview />} />
        <Route path="/memory" element={<MemoryExplorer />} />
        <Route path="/architecture" element={<ArchitectureExplorer />} />
        <Route path="/decisions" element={<DecisionTimeline />} />
        <Route path="/planner" element={<PlannerExplorer />} />
      </Routes>
    </div>
  );
}

export default App;
