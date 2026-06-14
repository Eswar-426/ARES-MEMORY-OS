import React, { useState, useEffect } from 'react';
import { 
  Compass, 
  Play, 
  Loader2, 
  CheckCircle, 
  Clock, 
  ArrowRight,
  TrendingUp,
  ListTodo,
  Calendar,
  Layers,
  ChevronRight
} from 'lucide-react';

interface Goal {
  id: string;
  title: string;
  description: string;
  priority: string;
  deadline: string | null;
  created_at: string;
}

interface Plan {
  id: string;
  goal_id: string;
  state: string;
  created_at: string;
  updated_at: string;
}

interface Milestone {
  id: string;
  plan_id: string;
  title: string;
  description: string;
  created_at: string;
}

interface Task {
  id: string;
  milestone_id: string;
  plan_id: string;
  title: string;
  description: string;
  status: string;
  estimated_duration: number | null;
  complexity: string | null;
  execution_order: number;
}

interface TaskDependency {
  task_id: string;
  depends_on_id: string;
}

interface PlanDetails {
  plan: Plan;
  goal: Goal;
  milestones: Milestone[];
  tasks: Task[];
  dependencies: TaskDependency[];
}

export const PlannerExplorer: React.FC = () => {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [selectedPlanId, setSelectedPlanId] = useState<string | null>(null);
  const [planDetails, setPlanDetails] = useState<PlanDetails | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [generating, setGenerating] = useState<boolean>(false);
  const [goalText, setGoalText] = useState<string>('');
  const [priority, setPriority] = useState<string>('Medium');
  const [activeTab, setActiveTab] = useState<'timeline' | 'hierarchy'>('timeline');

  // Fetch plan list
  const fetchPlans = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/v1/plans');
      if (res.ok) {
        const data = await res.json();
        setPlans(data);
        if (data.length > 0 && !selectedPlanId) {
          setSelectedPlanId(data[0].id);
        }
      }
    } catch (e) {
      console.error('Failed to fetch plans', e);
    } finally {
      setLoading(false);
    }
  };

  // Fetch details for a specific plan
  const fetchPlanDetails = async (id: string) => {
    try {
      const res = await fetch(`/api/v1/plans/${id}`);
      if (res.ok) {
        const data = await res.json();
        setPlanDetails(data);
      }
    } catch (e) {
      console.error('Failed to fetch plan details', e);
    }
  };

  useEffect(() => {
    fetchPlans();
  }, []);

  useEffect(() => {
    if (selectedPlanId) {
      fetchPlanDetails(selectedPlanId);
    }
  }, [selectedPlanId]);

  const handleGeneratePlan = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!goalText.trim()) return;

    setGenerating(true);
    try {
      const res = await fetch('/api/v1/plans/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ goal: goalText, priority }),
      });
      if (res.ok) {
        const newPlanDetails: PlanDetails = await res.json();
        setGoalText('');
        // Refresh plans and select the new one
        await fetchPlans();
        setSelectedPlanId(newPlanDetails.plan.id);
        setPlanDetails(newPlanDetails);
      }
    } catch (err) {
      console.error('Failed to generate plan', err);
    } finally {
      setGenerating(false);
    }
  };

  const getPriorityColor = (p: string) => {
    switch (p.toLowerCase()) {
      case 'critical': return 'bg-rose-500/20 text-rose-400 border border-rose-500/30';
      case 'high': return 'bg-amber-500/20 text-amber-400 border border-amber-500/30';
      case 'medium': return 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30';
      default: return 'bg-zinc-500/20 text-zinc-400 border border-zinc-500/30';
    }
  };

  const getComplexityColor = (c: string) => {
    switch (c?.toLowerCase()) {
      case 'high': return 'text-rose-400 border border-rose-500/20 bg-rose-500/5 px-2 py-0.5 rounded text-xs';
      case 'medium': return 'text-amber-400 border border-amber-500/20 bg-amber-500/5 px-2 py-0.5 rounded text-xs';
      default: return 'text-emerald-400 border border-emerald-500/20 bg-emerald-500/5 px-2 py-0.5 rounded text-xs';
    }
  };

  return (
    <main className="flex-1 p-8 overflow-y-auto max-h-screen">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-ares-text flex items-center gap-3">
            <Compass className="text-ares-accent w-9 h-9 animate-pulse" />
            Autonomous Planner Explorer
          </h1>
          <p className="text-ares-muted mt-2">
            Input a high-level goal and ARES will design a structured milestone plan with topological task dependency ordering.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Left Side: Create Plan and Plan History list */}
        <div className="flex flex-col gap-8 lg:col-span-1">
          {/* Create Goal Form */}
          <div className="glass-panel p-6 rounded-2xl glow-border">
            <h2 className="text-lg font-semibold mb-4 text-ares-text flex items-center gap-2">
              <Play className="text-ares-accent w-5 h-5" />
              Generate Development Plan
            </h2>
            <form onSubmit={handleGeneratePlan} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-ares-muted uppercase tracking-wider mb-2">
                  Goal / Objective
                </label>
                <textarea
                  className="w-full bg-black/40 border border-ares-border rounded-xl p-3 text-ares-text placeholder-ares-muted/50 focus:outline-none focus:border-ares-accent/50 focus:ring-1 focus:ring-ares-accent/30 min-h-[100px] resize-none"
                  placeholder="e.g. Add OAuth Authentication to ARES"
                  value={goalText}
                  onChange={(e) => setGoalText(e.target.value)}
                  disabled={generating}
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-ares-muted uppercase tracking-wider mb-2">
                  Priority
                </label>
                <div className="grid grid-cols-4 gap-2">
                  {['Low', 'Medium', 'High', 'Critical'].map((p) => (
                    <button
                      key={p}
                      type="button"
                      onClick={() => setPriority(p)}
                      className={`py-2 px-3 rounded-lg border text-sm font-medium transition-all ${
                        priority === p
                          ? 'border-ares-accent bg-ares-accent/15 text-ares-accent'
                          : 'border-ares-border bg-black/20 text-ares-muted hover:border-white/20'
                      }`}
                    >
                      {p}
                    </button>
                  ))}
                </div>
              </div>

              <button
                type="submit"
                disabled={generating || !goalText.trim()}
                className="w-full bg-ares-accent hover:bg-ares-accent/90 disabled:bg-ares-muted/30 disabled:text-ares-muted/60 disabled:cursor-not-allowed text-black font-semibold py-3 px-4 rounded-xl transition-all flex items-center justify-center gap-2 cursor-pointer shadow-lg shadow-ares-accent/10"
              >
                {generating ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    Analyzing Goal...
                  </>
                ) : (
                  <>
                    <ListTodo className="w-5 h-5" />
                    Generate Plan
                  </>
                )}
              </button>
            </form>
          </div>

          {/* History List */}
          <div className="glass-panel p-6 rounded-2xl flex-1 flex flex-col min-h-[300px]">
            <h2 className="text-lg font-semibold mb-4 text-ares-text flex items-center gap-2">
              <Calendar className="text-ares-accent w-5 h-5" />
              Generated Plans History
            </h2>
            {loading ? (
              <div className="flex-1 flex flex-col items-center justify-center text-ares-muted">
                <Loader2 className="w-8 h-8 animate-spin text-ares-accent mb-2" />
                Loading history...
              </div>
            ) : plans.length === 0 ? (
              <div className="flex-1 flex flex-col items-center justify-center text-ares-muted text-sm text-center px-4">
                No plans generated yet. Enter a goal above to get started.
              </div>
            ) : (
              <div className="space-y-2 overflow-y-auto max-h-[350px] pr-1">
                {plans.map((p) => (
                  <button
                    key={p.id}
                    onClick={() => setSelectedPlanId(p.id)}
                    className={`w-full text-left p-4 rounded-xl border transition-all flex items-center justify-between cursor-pointer ${
                      selectedPlanId === p.id
                        ? 'border-ares-accent bg-ares-accent/5'
                        : 'border-ares-border bg-black/10 hover:border-white/10'
                    }`}
                  >
                    <div className="truncate pr-4 flex-1">
                      <div className="text-sm font-semibold text-ares-text truncate">
                        {p.id}
                      </div>
                      <div className="text-xs text-ares-muted mt-1 flex items-center gap-2">
                        <span>Status:</span>
                        <span className="text-ares-accent font-medium">{p.state}</span>
                      </div>
                    </div>
                    <ChevronRight className={`w-5 h-5 transition-transform ${selectedPlanId === p.id ? 'text-ares-accent translate-x-1' : 'text-ares-muted'}`} />
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Right Side: Active Plan detail view */}
        <div className="lg:col-span-2">
          {planDetails ? (
            <div className="space-y-6">
              {/* Plan Overview Card */}
              <div className="glass-panel p-6 rounded-2xl">
                <div className="flex flex-wrap items-start justify-between gap-4 border-b border-ares-border pb-4 mb-4">
                  <div>
                    <div className="text-xs font-semibold text-ares-accent uppercase tracking-wider">
                      Goal Details
                    </div>
                    <h2 className="text-xl font-bold text-ares-text mt-1">
                      {planDetails.goal.title}
                    </h2>
                    <p className="text-ares-muted text-sm mt-1">
                      {planDetails.goal.description || 'No description provided.'}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-semibold uppercase ${getPriorityColor(planDetails.goal.priority)}`}>
                      {planDetails.goal.priority}
                    </span>
                    <span className="bg-ares-accent/10 text-ares-accent border border-ares-accent/20 px-3 py-1 rounded-full text-xs font-semibold">
                      {planDetails.plan.state}
                    </span>
                  </div>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                  <div className="bg-black/20 p-3 rounded-xl border border-ares-border">
                    <span className="block text-xs text-ares-muted">Plan ID</span>
                    <span className="font-semibold text-ares-text">{planDetails.plan.id}</span>
                  </div>
                  <div className="bg-black/20 p-3 rounded-xl border border-ares-border">
                    <span className="block text-xs text-ares-muted">Milestones</span>
                    <span className="font-semibold text-ares-text">{planDetails.milestones.length}</span>
                  </div>
                  <div className="bg-black/20 p-3 rounded-xl border border-ares-border">
                    <span className="block text-xs text-ares-muted">Total Tasks</span>
                    <span className="font-semibold text-ares-text">{planDetails.tasks.length}</span>
                  </div>
                  <div className="bg-black/20 p-3 rounded-xl border border-ares-border">
                    <span className="block text-xs text-ares-muted">Est. Duration</span>
                    <span className="font-semibold text-ares-text">
                      {planDetails.tasks.reduce((sum, t) => sum + (t.estimated_duration || 0), 0)} min
                    </span>
                  </div>
                </div>
              </div>

              {/* Navigation Tabs */}
              <div className="flex gap-2 border-b border-ares-border pb-1">
                <button
                  onClick={() => setActiveTab('timeline')}
                  className={`pb-3 px-4 font-semibold text-sm transition-all border-b-2 cursor-pointer ${
                    activeTab === 'timeline'
                      ? 'border-ares-accent text-ares-accent'
                      : 'border-transparent text-ares-muted hover:text-ares-text'
                  }`}
                >
                  <span className="flex items-center gap-2">
                    <TrendingUp className="w-4 h-4" />
                    Topological Execution Timeline
                  </span>
                </button>
                <button
                  onClick={() => setActiveTab('hierarchy')}
                  className={`pb-3 px-4 font-semibold text-sm transition-all border-b-2 cursor-pointer ${
                    activeTab === 'hierarchy'
                      ? 'border-ares-accent text-ares-accent'
                      : 'border-transparent text-ares-muted hover:text-ares-text'
                  }`}
                >
                  <span className="flex items-center gap-2">
                    <Layers className="w-4 h-4" />
                    Milestones & Subtasks Hierarchy
                  </span>
                </button>
              </div>

              {/* Tab Content 1: Topological Timeline */}
              {activeTab === 'timeline' && (
                <div className="glass-panel p-6 rounded-2xl space-y-6">
                  <div>
                    <h3 className="text-md font-semibold text-ares-text flex items-center gap-2">
                      <TrendingUp className="text-ares-accent w-5 h-5" />
                      Sequence of Actionable Tasks (Resolved Dependencies)
                    </h3>
                    <p className="text-xs text-ares-muted mt-1">
                      Tasks are sorted topologically. No task begins until all tasks to its left or preceding it are marked complete.
                    </p>
                  </div>

                  <div className="relative pl-8 border-l-2 border-ares-border/55 space-y-8 ml-2 py-2">
                    {planDetails.tasks.map((task, index) => {
                      // Find direct dependencies
                      const directDeps = planDetails.dependencies
                        .filter(d => d.task_id === task.id)
                        .map(d => planDetails.tasks.find(t => t.id === d.depends_on_id)?.title)
                        .filter(Boolean);

                      return (
                        <div key={task.id} className="relative group">
                          {/* Dot Indicator */}
                          <div className="absolute -left-[41px] top-0 bg-zinc-900 border-2 border-ares-accent w-6 h-6 rounded-full flex items-center justify-center font-bold text-xs text-ares-accent shadow-lg shadow-ares-accent/20 transition-transform group-hover:scale-110">
                            {index + 1}
                          </div>

                          <div className="bg-black/35 border border-ares-border hover:border-ares-accent/30 p-5 rounded-2xl transition-all shadow-md">
                            <div className="flex flex-wrap items-center justify-between gap-4 mb-2">
                              <h4 className="font-semibold text-ares-text text-base">
                                {task.title}
                              </h4>
                              <div className="flex items-center gap-2">
                                {task.complexity && (
                                  <span className={getComplexityColor(task.complexity)}>
                                    {task.complexity}
                                  </span>
                                )}
                                {task.estimated_duration && (
                                  <span className="text-xs text-ares-muted flex items-center gap-1">
                                    <Clock className="w-3 h-3" />
                                    {task.estimated_duration}m
                                  </span>
                                )}
                              </div>
                            </div>

                            <p className="text-sm text-ares-muted">
                              {task.description || 'No description provided.'}
                            </p>

                            {/* Dependencies chips */}
                            {directDeps.length > 0 && (
                              <div className="mt-3 pt-3 border-t border-ares-border/40 flex flex-wrap items-center gap-2">
                                <span className="text-xs font-semibold text-ares-accent/80 flex items-center gap-1 uppercase tracking-wider">
                                  <ArrowRight className="w-3 h-3" />
                                  Depends On:
                                </span>
                                {directDeps.map((depTitle, dIdx) => (
                                  <span key={dIdx} className="bg-zinc-800/80 border border-ares-border text-ares-text text-xs px-2 py-0.5 rounded-md flex items-center gap-1">
                                    <CheckCircle className="w-3 h-3 text-ares-accent/70" />
                                    {depTitle}
                                  </span>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Tab Content 2: Hierarchy */}
              {activeTab === 'hierarchy' && (
                <div className="space-y-6">
                  {planDetails.milestones.map((milestone) => {
                    const milestoneTasks = planDetails.tasks.filter(
                      (t) => t.milestone_id === milestone.id
                    );

                    return (
                      <div key={milestone.id} className="glass-panel p-6 rounded-2xl space-y-4">
                        <div className="border-b border-ares-border pb-3">
                          <h3 className="text-lg font-bold text-ares-text flex items-center gap-2">
                            <Layers className="text-ares-accent w-5 h-5" />
                            {milestone.title}
                          </h3>
                          <p className="text-sm text-ares-muted mt-1">
                            {milestone.description || 'No milestone description.'}
                          </p>
                        </div>

                        <div className="space-y-3">
                          {milestoneTasks.map((task) => (
                            <div
                              key={task.id}
                              className="bg-black/20 border border-ares-border/70 p-4 rounded-xl flex items-start justify-between gap-4"
                            >
                              <div className="space-y-1">
                                <h4 className="font-semibold text-ares-text text-sm">
                                  {task.title}
                                </h4>
                                <p className="text-xs text-ares-muted">
                                  {task.description || 'No description provided.'}
                                </p>
                              </div>
                              <div className="flex flex-col items-end gap-1 text-right shrink-0">
                                <div className="flex items-center gap-2">
                                  {task.complexity && (
                                    <span className={getComplexityColor(task.complexity)}>
                                      {task.complexity}
                                    </span>
                                  )}
                                  <span className="text-xs font-semibold text-ares-accent bg-ares-accent/5 px-2 py-0.5 rounded border border-ares-accent/10">
                                    Order: #{task.execution_order}
                                  </span>
                                </div>
                                {task.estimated_duration && (
                                  <span className="text-[11px] text-ares-muted mt-1">
                                    Est: {task.estimated_duration} minutes
                                  </span>
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          ) : (
            <div className="glass-panel rounded-2xl p-12 flex flex-col items-center justify-center text-center min-h-[500px]">
              <div className="bg-ares-accent/10 p-4 rounded-full border border-ares-accent/20 mb-4 animate-bounce">
                <Compass className="text-ares-accent w-10 h-10" />
              </div>
              <h3 className="text-xl font-bold text-ares-text">
                Select a Plan to Inspect
              </h3>
              <p className="text-ares-muted text-sm max-w-sm mt-2">
                Generate a new plan from a development goal on the left or select an existing plan from the history list to view its tasks and dependency timelines.
              </p>
            </div>
          )}
        </div>
      </div>
    </main>
  );
};
