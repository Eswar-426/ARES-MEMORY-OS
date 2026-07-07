with open('crates/ares-store/src/repositories/gaps.rs', 'r', encoding='utf-8') as f:
    text = f.read()

health_score_struct = """
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub overall: f64,
    pub files_with_decisions_term: f64,
    pub decisions_with_requirements_term: f64,
    pub files_with_owners_term: f64,
    pub fresh_decisions_term: f64,
    
    pub total_files: i64,
    pub files_with_decisions: i64,
    pub total_decisions: i64,
    pub decisions_with_requirements: i64,
    pub files_with_owners: i64,
    pub stale_decisions: i64,
    pub fresh_decisions: i64,
}
"""
idx = text.find('pub struct SqliteGapRepository')
text = text[:idx] + health_score_struct + '\n' + text[idx:]

calc_fn = """
    pub fn calculate_health_score(&self, project_id: &ProjectId) -> Result<HealthScore, AresError> {
        let conn = self.store.get_conn()?;

        let total_files: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND node_type = 'file'",
            params![project_id.as_str()],
            |row| row.get(0),
        ).unwrap_or(0);

        let files_with_decisions: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT n.id) FROM graph_nodes n 
             JOIN graph_edges e ON e.target_id = n.id
             JOIN graph_nodes d ON e.source_id = d.id 
             WHERE n.project_id = ?1 AND n.node_type = 'file' AND d.node_type = 'decision'",
            params![project_id.as_str()],
            |row| row.get(0),
        ).unwrap_or(0);

        let total_decisions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND node_type = 'decision'",
            params![project_id.as_str()],
            |row| row.get(0),
        ).unwrap_or(0);

        let decisions_with_requirements: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT d.id) FROM graph_nodes d
             JOIN graph_edges e ON e.source_id = d.id OR e.target_id = d.id
             JOIN graph_nodes r ON (e.target_id = r.id AND r.node_type = 'requirement') OR (e.source_id = r.id AND r.node_type = 'requirement')
             WHERE d.project_id = ?1 AND d.node_type = 'decision'",
            params![project_id.as_str()],
            |row| row.get(0),
        ).unwrap_or(0);

        let files_with_owners: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT n.id) FROM graph_nodes n
             JOIN graph_edges e ON e.target_id = n.id
             JOIN graph_nodes p ON e.source_id = p.id
             WHERE n.project_id = ?1 AND n.node_type = 'file' AND p.node_type IN ('person', 'team') AND e.edge_type IN ('authored_by', 'contributed_to')",
            params![project_id.as_str()],
            |row| row.get(0),
        ).unwrap_or(0);

        let thirty_days_micros = 30 * 24 * 60 * 60 * 1_000_000_i64;
        let threshold = Self::now_micros() - thirty_days_micros;

        let fresh_decisions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM graph_nodes 
             WHERE project_id = ?1 AND node_type = 'decision' AND created_at > ?2",
            params![project_id.as_str(), threshold],
            |row| row.get(0),
        ).unwrap_or(0);

        let stale_decisions = total_decisions - fresh_decisions;

        let term1 = if total_files == 0 { 1.0 } else { (files_with_decisions as f64) / (total_files as f64) };
        let term2 = if total_decisions == 0 { 1.0 } else { (decisions_with_requirements as f64) / (total_decisions as f64) };
        let term3 = if total_files == 0 { 1.0 } else { (files_with_owners as f64) / (total_files as f64) };
        
        let term4 = if total_decisions == 0 { 
            0.0 
        } else if stale_decisions == 0 { 
            if fresh_decisions > 0 { 1.0 } else { 0.0 } 
        } else { 
            ((fresh_decisions as f64) / (stale_decisions as f64)).min(1.0) 
        };

        let overall = (term1 * 0.4 + term2 * 0.3 + term3 * 0.2 + term4 * 0.1) * 100.0;

        Ok(HealthScore {
            overall,
            files_with_decisions_term: term1,
            decisions_with_requirements_term: term2,
            files_with_owners_term: term3,
            fresh_decisions_term: term4,
            total_files,
            files_with_decisions,
            total_decisions,
            decisions_with_requirements,
            files_with_owners,
            stale_decisions,
            fresh_decisions,
        })
    }
"""
idx2 = text.find('fn now_micros() -> i64')
text = text[:idx2] + calc_fn + '\n    ' + text[idx2:]

with open('crates/ares-store/src/repositories/gaps.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print('Updated gaps.rs')
