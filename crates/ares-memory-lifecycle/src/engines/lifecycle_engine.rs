use super::{
    ArchivalEngine, DecayEngine, FreshnessEngine, RevalidationEngine, SupersessionEngine,
    TrustEngine,
};
use crate::models::{LifecycleReport, LifecycleState, SupersessionRecord};

pub struct LifecycleEngine {
    pub freshness_engine: FreshnessEngine,
    pub trust_engine: TrustEngine,
    pub decay_engine: DecayEngine,
    pub revalidation_engine: RevalidationEngine,
    pub supersession_engine: SupersessionEngine,
    pub archival_engine: ArchivalEngine,
}

pub struct LifecycleInput {
    pub artifact_id: String,
    pub days_since_last_validation: i64,
    pub baseline_change_frequency_days: i64,
    pub evidence_count: usize,
    pub manual_approvals: usize,
    pub revalidation_successes: usize,
    pub contradiction_signals: usize,
    pub is_orphaned: bool,
    pub is_unused: bool,
    pub supersession_records: Vec<SupersessionRecord>,
    pub days_since_superseded: Option<i64>,
    pub revalidation_attempted: bool,
    pub revalidation_successful: bool,
}

impl LifecycleEngine {
    pub fn new(
        stale_multiplier: i64,
        decay_multiplier: i64,
        minimum_evidence: usize,
        superseded_after_days: i64,
    ) -> Self {
        Self {
            freshness_engine: FreshnessEngine::new(stale_multiplier, decay_multiplier),
            trust_engine: TrustEngine::new(minimum_evidence),
            decay_engine: DecayEngine::new(),
            revalidation_engine: RevalidationEngine::new(),
            supersession_engine: SupersessionEngine::new(),
            archival_engine: ArchivalEngine::new(superseded_after_days),
        }
    }

    pub fn evaluate(&self, input: LifecycleInput) -> LifecycleReport {
        let freshness = self.freshness_engine.evaluate_freshness(
            input.days_since_last_validation,
            input.baseline_change_frequency_days,
        );
        let mut state = self.freshness_engine.determine_state(&freshness);

        let trust = self.trust_engine.evaluate_trust(
            input.evidence_count,
            input.manual_approvals,
            input.revalidation_successes,
            input.contradiction_signals,
        );

        let is_decaying =
            self.decay_engine
                .detect_decay(&state, &freshness, input.is_orphaned, input.is_unused);
        if is_decaying {
            state = LifecycleState::Decaying;
        }

        if input.revalidation_attempted {
            state = self
                .revalidation_engine
                .attempt_revalidation(&state, input.revalidation_successful);
        }

        let best_supersession = self
            .supersession_engine
            .evaluate_supersession(&input.supersession_records);
        state = self
            .supersession_engine
            .apply_supersession(&state, &best_supersession);

        state = self.archival_engine.determine_archival(
            &state,
            input.days_since_superseded,
            input.is_unused,
        );

        let is_archivable = matches!(state, LifecycleState::Archived);

        LifecycleReport {
            artifact_id: input.artifact_id,
            current_state: state,
            freshness,
            trust,
            supersession: best_supersession,
            is_archivable,
        }
    }
}
