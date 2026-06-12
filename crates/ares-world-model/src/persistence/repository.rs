use crate::forecast::models::{ForecastDeviation, OutcomePrediction, StrategyRanking};
use crate::risk::models::RiskReport;
use crate::scenario::models::Scenario;
use crate::simulation::models::SimulationResult;
use crate::state::models::WorldState;
use ares_core::AresError;
use rusqlite::params;

/// Repository for persisting and retrieving world model entities.
pub struct WorldModelRepository;

impl Default for WorldModelRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldModelRepository {
    pub fn new() -> Self {
        Self
    }

    // ── World States ─────────────────────────────────────────────

    pub fn save_world_state(
        &self,
        conn: &rusqlite::Connection,
        state: &WorldState,
    ) -> Result<(), AresError> {
        let goals_json = serde_json::to_string(&state.goals).map_err(AresError::db)?;
        let resources_json = serde_json::to_string(&state.resources).map_err(AresError::db)?;
        let agents_json = serde_json::to_string(&state.active_agents).map_err(AresError::db)?;
        let constraints_json = serde_json::to_string(&state.constraints).map_err(AresError::db)?;

        conn.execute(
            "INSERT INTO world_states (id, goals_json, resources_json, agents_json, constraints_json, snapshot_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                state.id.as_str(),
                goals_json,
                resources_json,
                agents_json,
                constraints_json,
                state.snapshot_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_world_state(
        &self,
        conn: &rusqlite::Connection,
        id: &str,
    ) -> Result<Option<WorldState>, AresError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, goals_json, resources_json, agents_json, constraints_json, snapshot_at
                 FROM world_states WHERE id = ?1",
            )
            .map_err(AresError::db)?;

        let result = stmt
            .query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let goals_json: String = row.get(1)?;
                let resources_json: String = row.get(2)?;
                let agents_json: String = row.get(3)?;
                let constraints_json: String = row.get(4)?;
                let snapshot_ts: i64 = row.get(5)?;

                Ok((
                    id_str,
                    goals_json,
                    resources_json,
                    agents_json,
                    constraints_json,
                    snapshot_ts,
                ))
            })
            .optional()
            .map_err(AresError::db)?;

        match result {
            Some((
                id_str,
                goals_json,
                resources_json,
                agents_json,
                constraints_json,
                snapshot_ts,
            )) => {
                let state = WorldState {
                    id: ares_core::WorldStateId::from(id_str),
                    goals: serde_json::from_str(&goals_json).unwrap_or_default(),
                    resources: serde_json::from_str(&resources_json).unwrap_or_default(),
                    active_agents: serde_json::from_str(&agents_json).unwrap_or_default(),
                    constraints: serde_json::from_str(&constraints_json).unwrap_or_default(),
                    snapshot_at: chrono::DateTime::from_timestamp(snapshot_ts, 0)
                        .unwrap_or_else(chrono::Utc::now),
                };
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    // ── Scenarios ────────────────────────────────────────────────

    pub fn save_scenario(
        &self,
        conn: &rusqlite::Connection,
        scenario: &Scenario,
        world_state_id: Option<&str>,
    ) -> Result<(), AresError> {
        let agents_json =
            serde_json::to_string(&scenario.agent_assignments).map_err(AresError::db)?;
        let steps_json = serde_json::to_string(&scenario.steps).map_err(AresError::db)?;

        conn.execute(
            "INSERT INTO scenarios (id, goal_id, scenario_type, description, estimated_cost,
             estimated_duration_secs, estimated_quality, agent_assignments_json, steps_json,
             world_state_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                scenario.id.as_str(),
                scenario.goal_id,
                scenario.scenario_type.as_str(),
                scenario.description,
                scenario.estimated_cost,
                scenario.estimated_duration_secs,
                scenario.estimated_quality,
                agents_json,
                steps_json,
                world_state_id,
                scenario.created_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn list_scenarios_for_goal(
        &self,
        conn: &rusqlite::Connection,
        goal_id: &str,
    ) -> Result<Vec<Scenario>, AresError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, goal_id, scenario_type, description, estimated_cost,
                 estimated_duration_secs, estimated_quality, agent_assignments_json,
                 steps_json, created_at
                 FROM scenarios WHERE goal_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(AresError::db)?;

        let scenarios = stmt
            .query_map(params![goal_id], |row| {
                let id_str: String = row.get(0)?;
                let goal_id: String = row.get(1)?;
                let scenario_type_str: String = row.get(2)?;
                let description: String = row.get(3)?;
                let estimated_cost: f64 = row.get(4)?;
                let estimated_duration_secs: f64 = row.get(5)?;
                let estimated_quality: f64 = row.get(6)?;
                let agents_json: String = row.get(7)?;
                let steps_json: String = row.get(8)?;
                let created_ts: i64 = row.get(9)?;

                Ok(Scenario {
                    id: ares_core::ScenarioId::from(id_str),
                    goal_id,
                    scenario_type: crate::scenario::models::ScenarioType::from_str_val(
                        &scenario_type_str,
                    ),
                    description,
                    estimated_cost,
                    estimated_duration_secs,
                    estimated_quality,
                    agent_assignments: serde_json::from_str(&agents_json).unwrap_or_default(),
                    steps: serde_json::from_str(&steps_json).unwrap_or_default(),
                    created_at: chrono::DateTime::from_timestamp(created_ts, 0)
                        .unwrap_or_else(chrono::Utc::now),
                })
            })
            .map_err(AresError::db)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(scenarios)
    }

    // ── Simulations ──────────────────────────────────────────────

    pub fn save_simulation(
        &self,
        conn: &rusqlite::Connection,
        simulation: &SimulationResult,
    ) -> Result<(), AresError> {
        conn.execute(
            "INSERT INTO simulations (id, scenario_id, task_duration_secs, total_cost,
             success_probability, agent_utilization, memory_usage_estimate, risk_score,
             simulated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                simulation.id.as_str(),
                simulation.scenario_id.as_str(),
                simulation.task_duration_secs,
                simulation.total_cost,
                simulation.success_probability,
                simulation.agent_utilization,
                simulation.memory_usage_estimate,
                simulation.risk_score,
                simulation.simulated_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    // ── Risk Reports ─────────────────────────────────────────────

    pub fn save_risk_report(
        &self,
        conn: &rusqlite::Connection,
        report: &RiskReport,
    ) -> Result<(), AresError> {
        let factors_json = serde_json::to_string(&report.risk_factors).map_err(AresError::db)?;
        let mitigations_json = serde_json::to_string(&report.mitigations).map_err(AresError::db)?;

        conn.execute(
            "INSERT INTO risk_reports (id, scenario_id, overall_risk, failure_probability,
             budget_overrun_probability, resource_exhaustion_risk, dependency_risk,
             execution_risk, risk_factors_json, mitigations_json, analyzed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                report.id.as_str(),
                report.scenario_id.as_str(),
                report.overall_risk.as_str(),
                report.failure_probability,
                report.budget_overrun_probability,
                report.resource_exhaustion_risk,
                report.dependency_risk,
                report.execution_risk,
                factors_json,
                mitigations_json,
                report.analyzed_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    // ── Predictions ──────────────────────────────────────────────

    pub fn save_prediction(
        &self,
        conn: &rusqlite::Connection,
        prediction: &OutcomePrediction,
    ) -> Result<(), AresError> {
        let confidence_reasons_json =
            serde_json::to_string(&prediction.confidence_reasons).map_err(AresError::db)?;

        conn.execute(
            "INSERT INTO predictions (id, goal_id, scenario_id, success_probability,
             estimated_cost, estimated_duration_secs, confidence, confidence_reasons_json,
             similar_mission_count, prediction_method, predicted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                prediction.id.as_str(),
                prediction.goal_id,
                prediction.scenario_id.as_ref().map(|s| s.as_str()),
                prediction.success_probability,
                prediction.estimated_cost,
                prediction.estimated_duration_secs,
                prediction.confidence,
                confidence_reasons_json,
                prediction.similar_mission_count,
                prediction.prediction_method.as_str(),
                prediction.predicted_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    // ── Forecast History ─────────────────────────────────────────

    pub fn save_forecast_deviation(
        &self,
        conn: &rusqlite::Connection,
        deviation: &ForecastDeviation,
    ) -> Result<(), AresError> {
        let id = ares_core::new_id();
        conn.execute(
            "INSERT INTO forecast_history (id, prediction_id, predicted_cost, actual_cost,
             predicted_duration_secs, actual_duration_secs, predicted_success, actual_success,
             deviation_score, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                deviation.prediction_id.as_str(),
                deviation.predicted_cost,
                deviation.actual_cost,
                deviation.predicted_duration_secs,
                deviation.actual_duration_secs,
                deviation.predicted_success,
                deviation.actual_success as i32,
                deviation.deviation_score,
                deviation.recorded_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn list_forecast_history(
        &self,
        conn: &rusqlite::Connection,
        limit: u32,
    ) -> Result<Vec<ForecastDeviation>, AresError> {
        let mut stmt = conn
            .prepare(
                "SELECT prediction_id, predicted_cost, actual_cost, predicted_duration_secs,
                 actual_duration_secs, predicted_success, actual_success, deviation_score,
                 recorded_at
                 FROM forecast_history ORDER BY recorded_at DESC LIMIT ?1",
            )
            .map_err(AresError::db)?;

        let deviations = stmt
            .query_map(params![limit], |row| {
                let prediction_id_str: String = row.get(0)?;
                let predicted_cost: f64 = row.get(1)?;
                let actual_cost: f64 = row.get(2)?;
                let predicted_duration_secs: f64 = row.get(3)?;
                let actual_duration_secs: f64 = row.get(4)?;
                let predicted_success: f64 = row.get(5)?;
                let actual_success_int: i32 = row.get(6)?;
                let deviation_score: f64 = row.get(7)?;
                let recorded_ts: i64 = row.get(8)?;

                Ok(ForecastDeviation {
                    prediction_id: ares_core::PredictionId::from(prediction_id_str),
                    predicted_cost,
                    actual_cost,
                    predicted_duration_secs,
                    actual_duration_secs,
                    predicted_success,
                    actual_success: actual_success_int != 0,
                    deviation_score,
                    recorded_at: chrono::DateTime::from_timestamp(recorded_ts, 0)
                        .unwrap_or_else(chrono::Utc::now),
                })
            })
            .map_err(AresError::db)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(deviations)
    }

    // ── Strategy Rankings ────────────────────────────────────────

    pub fn save_strategy_ranking(
        &self,
        conn: &rusqlite::Connection,
        goal_id: &str,
        ranking: &StrategyRanking,
    ) -> Result<(), AresError> {
        let id = ares_core::new_id();
        conn.execute(
            "INSERT INTO strategy_rankings (id, goal_id, scenario_id, rank, composite_score,
             speed_score, quality_score, cost_score, risk_score, success_score, explanation)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                goal_id,
                ranking.scenario_id.as_str(),
                ranking.rank,
                ranking.composite_score,
                ranking.speed_score,
                ranking.quality_score,
                ranking.cost_score,
                ranking.risk_score,
                ranking.success_score,
                ranking.explanation,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn list_rankings_for_goal(
        &self,
        conn: &rusqlite::Connection,
        goal_id: &str,
    ) -> Result<Vec<StrategyRanking>, AresError> {
        let mut stmt = conn
            .prepare(
                "SELECT scenario_id, rank, composite_score, speed_score, quality_score,
                 cost_score, risk_score, success_score, explanation
                 FROM strategy_rankings WHERE goal_id = ?1 ORDER BY rank ASC",
            )
            .map_err(AresError::db)?;

        let rankings = stmt
            .query_map(params![goal_id], |row| {
                let scenario_id_str: String = row.get(0)?;
                Ok(StrategyRanking {
                    scenario_id: ares_core::ScenarioId::from(scenario_id_str),
                    rank: row.get(1)?,
                    composite_score: row.get(2)?,
                    speed_score: row.get(3)?,
                    quality_score: row.get(4)?,
                    cost_score: row.get(5)?,
                    risk_score: row.get(6)?,
                    success_score: row.get(7)?,
                    explanation: row.get(8)?,
                })
            })
            .map_err(AresError::db)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rankings)
    }
}

/// Extension trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
