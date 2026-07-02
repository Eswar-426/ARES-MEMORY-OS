use crate::Store;
use ares_core::{AresError, NodeType};

pub struct GraphStats {
    pub nodes: usize,
    pub edges: usize,
    pub files: usize,
    pub directories: usize,
    pub commits: usize,
    pub authors: usize,
}

pub struct IntegrityStats {
    pub missing_sources: usize,
    pub missing_targets: usize,
    pub orphans: usize,
}

pub struct CoverageStats {
    pub adrs: usize,
    pub requirements: usize,
    pub architecture_docs: usize,
    pub decisions: usize,
}

pub struct RepositoryStats {
    pub files: usize,
    pub functions: usize,
    pub directories: usize,
    pub modules: usize,
}

impl Store {
    pub fn overview_graph_stats(&self) -> Result<GraphStats, AresError> {
        let conn = self.get_conn()?;
        let nodes = conn
            .query_row("SELECT COUNT(*) FROM graph_nodes", [], |r| r.get(0))
            .unwrap_or(0);
        let edges = conn
            .query_row("SELECT COUNT(*) FROM graph_edges", [], |r| r.get(0))
            .unwrap_or(0);
        let files = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::File.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let directories = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Folder.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let commits = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Commit.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let authors = conn
            .query_row(
                &format!(
                    "SELECT COUNT(DISTINCT properties) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Person.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(1);

        Ok(GraphStats {
            nodes,
            edges,
            files,
            directories,
            commits,
            authors,
        })
    }

    pub fn overview_integrity_stats(&self) -> Result<IntegrityStats, AresError> {
        let conn = self.get_conn()?;
        let missing_sources: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.from_node_id = n.id WHERE n.id IS NULL", [], |r| r.get(0)).unwrap_or(0);
        let missing_targets: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.to_node_id = n.id WHERE n.id IS NULL", [], |r| r.get(0)).unwrap_or(0);
        let orphans: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE id NOT IN (SELECT from_node_id FROM graph_edges UNION SELECT to_node_id FROM graph_edges)", [], |r| r.get(0)).unwrap_or(0);

        Ok(IntegrityStats {
            missing_sources: missing_sources as usize,
            missing_targets: missing_targets as usize,
            orphans: orphans as usize,
        })
    }

    pub fn overview_coverage_stats(&self) -> Result<CoverageStats, AresError> {
        let conn = self.get_conn()?;
        let adrs = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Architecture.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let requirements = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Requirement.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let architecture_docs = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Architecture.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let decisions = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Decision.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        Ok(CoverageStats {
            adrs,
            requirements,
            architecture_docs,
            decisions,
        })
    }

    pub fn overview_repository_stats(&self) -> Result<RepositoryStats, AresError> {
        let conn = self.get_conn()?;
        let files = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::File.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let functions = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Function.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let directories = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Folder.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let modules = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM graph_nodes WHERE node_type = '{}'",
                    NodeType::Module.as_str()
                ),
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        Ok(RepositoryStats {
            files,
            functions,
            directories,
            modules,
        })
    }
}
