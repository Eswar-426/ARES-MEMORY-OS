use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCapability {
    Search,
    CodeExecution,
    Browser,
    KnowledgeGraph,
    Filesystem,
    Embeddings,
}
