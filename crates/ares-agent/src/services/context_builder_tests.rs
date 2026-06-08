#[cfg(test)]
mod tests {
    use crate::services::context_builder::{
        ContextBudget, ContextCompressionLevel, ReasoningContextBuilder,
    };
    use crate::services::context_intelligence::ContextAnalysis;
    use ares_core::{
        ImportanceLevel, Memory, MemoryId, MemorySource, MemoryStatus, MemoryType, Project,
        ProjectId, ProjectMaturity,
    };

    fn make_project() -> Project {
        Project {
            id: ProjectId::new(),
            name: "Test Project".to_string(),
            description: "Test description".to_string(),
            root_path: "/tmp/test".to_string(),
            primary_language: "rust".to_string(),
            domain: "test".to_string(),
            maturity: ProjectMaturity::Greenfield,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        }
    }

    fn make_memory() -> Memory {
        Memory {
            id: MemoryId::new(),
            project_id: ProjectId::new(),
            memory_type: MemoryType::Feature,
            title: "Test".to_string(),
            content: serde_json::Value::String("A long memory content piece...".to_string()),
            status: MemoryStatus::Active,
            version: 1,
            parent_id: None,
            confidence: 1.0,
            importance: ImportanceLevel::Medium,
            source: MemorySource::Human,
            ai_assisted: false,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        }
    }

    #[test]
    fn test_context_assembly_limits() {
        let builder = ReasoningContextBuilder::new();
        let project = make_project();

        let mut memories = Vec::new();
        for _ in 0..50 {
            memories.push(make_memory());
        }

        let budget = ContextBudget {
            max_total_tokens: 50, // Base text (~12) + 5 memories (~7 each) = ~47 tokens. 6 memories would be ~54.
            compression_level: ContextCompressionLevel::Full,
        };

        let start = std::time::Instant::now();
        let snapshot = builder.build(
            &project,
            "How does auth work?",
            memories,
            vec![],
            ContextAnalysis {
                relevant_memories: vec![],
                related_decisions: vec![],
                contradictions: vec![],
                dependency_chain: vec![],
                reasoning_summary: "".into(),
                confidence: 1.0,
            },
            None,
            budget,
        );
        let duration = start.elapsed();

        println!("Context assembly took: {:?}", duration);
        assert!(duration.as_millis() < 250, "Performance target: < 250 ms");
        assert_eq!(snapshot.memories.len(), 4);
        assert!(snapshot.estimated_tokens > 0);
    }
}
