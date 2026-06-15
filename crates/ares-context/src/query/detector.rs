use super::intent::QueryIntent;

pub struct IntentDetector;

impl IntentDetector {
    pub fn new() -> Self {
        Self
    }

    /// Uses deterministic heuristics to map a natural language query into a QueryIntent.
    pub fn detect(&self, query: &str) -> QueryIntent {
        let q = query.to_lowercase();

        if q.contains("explain") || q.contains("what does") || q.contains("how does") {
            if q.contains("architecture") {
                return QueryIntent::ArchitectureQuery;
            }
            return QueryIntent::FileExplanation;
        }

        if q.contains("trace") || q.contains("depend") || q.contains("used by") {
            return QueryIntent::DependencyTrace;
        }

        if q.contains("owner") || q.contains("who wrote") || q.contains("who owns") {
            return QueryIntent::ComponentOwner;
        }

        if q.contains("impact") || q.contains("affected") || q.contains("if i change") {
            return QueryIntent::ChangeImpact;
        }

        if q.contains("dead") || q.contains("unused") {
            return QueryIntent::DeadCodeDiscovery;
        }

        if q.contains("entry") || q.contains("start") || q.contains("main") {
            return QueryIntent::EntryPointDiscovery;
        }

        if q.contains("memory") || q.contains("decision") || q.contains("why did we") {
            return QueryIntent::MemoryLookup;
        }

        if q.contains("overview") || q.contains("summar") {
            return QueryIntent::RepositoryOverview;
        }

        if q.contains("find") || q.contains("search") || q.contains("where is") || q.contains("implement") {
            return QueryIntent::ImplementationSearch;
        }

        // Default fallback
        QueryIntent::ImplementationSearch
    }
}
