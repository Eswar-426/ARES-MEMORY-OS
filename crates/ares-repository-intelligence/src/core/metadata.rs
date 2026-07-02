use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Execution Metadata — Attached to every engine result and response
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub engine: String,
    pub duration_ms: u64,
    pub cache_hit: bool,
    pub confidence: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub retry_count: u32,
    pub sources_used: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════
// Planner Trace — Event stream driving UI, logs, replay, telemetry
// ═══════════════════════════════════════════════════════════════════

/// Hierarchical event stream representing the DAG of planner execution.
/// One source of truth for VS Code progress, planner visualizer, debug logs,
/// replay, and telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlannerTraceEvent {
    PlanningStarted {
        execution_id: String,
        timestamp_ms: u64,
    },
    IntentDetected {
        intent: String,
        duration_ms: u64,
    },
    CapabilityExpanded {
        added: Vec<String>,
        duration_ms: u64,
    },
    CapabilityOptimized {
        removed: Vec<String>,
        final_count: usize,
        duration_ms: u64,
    },
    PlanBuilt {
        plan_id: String,
        capabilities: Vec<String>,
        duration_ms: u64,
    },
    PlanCacheHit {
        plan_id: String,
    },
    DependencyResolved {
        node_count: usize,
        duration_ms: u64,
    },
    EngineStarted {
        engine_id: String,
        capability: String,
    },
    EngineFinished {
        engine_id: String,
        duration_ms: u64,
        success: bool,
    },
    AggregationStarted,
    AggregationFinished {
        duration_ms: u64,
    },
    ValidationStarted,
    ValidationFinished {
        issues: usize,
        duration_ms: u64,
    },
    KnowledgeStarted,
    KnowledgeFinished {
        duration_ms: u64,
    },
    Completed {
        total_duration_ms: u64,
    },
}

/// The full trace of a planner execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlannerTrace {
    pub events: Vec<PlannerTraceEvent>,
}

impl PlannerTrace {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: PlannerTraceEvent) {
        self.events.push(event);
    }
}
