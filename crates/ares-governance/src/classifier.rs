use ares_core::types::node::NodeType;
use std::path::Path;

pub const CLASSIFIER_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactCategory {
    // Reasoning Nodes
    Requirement,
    Decision,
    Architecture,
    Evidence,

    // Implementation Nodes
    Code,
    Test,
    Infrastructure,
    Configuration,

    // Management Nodes
    Ownership,
    RepositoryEvent,
    Snapshot,
    Documentation,

    // Governance Nodes
    Policy,

    // External Nodes
    Generated,
    Vendor,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryEligibility {
    Required,
    Recommended,
    Optional,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassificationConfidence {
    Certain,
    Inferred,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ArtifactClassification {
    pub category: ArtifactCategory,
    pub eligibility: MemoryEligibility,
    pub confidence: ClassificationConfidence,
    pub classifier_version: &'static str,
}

pub struct ArtifactClassifier;

impl ArtifactClassifier {
    pub fn classify(node_type: Option<&NodeType>, path: Option<&str>) -> ArtifactClassification {
        let (category, confidence) = Self::determine_category(node_type, path);
        let eligibility = Self::determine_eligibility(&category);

        ArtifactClassification {
            category,
            eligibility,
            confidence,
            classifier_version: CLASSIFIER_VERSION,
        }
    }

    fn determine_category(
        node_type: Option<&NodeType>,
        path: Option<&str>,
    ) -> (ArtifactCategory, ClassificationConfidence) {
        // Priority 1: Graph Node Type
        if let Some(nt) = node_type {
            match nt {
                NodeType::Requirement => {
                    println!("DEBUG_CLASSIFIER: Matched NodeType::Requirement -> Returning ArtifactCategory::Requirement");
                    return (
                        ArtifactCategory::Requirement,
                        ClassificationConfidence::Certain,
                    );
                }
                NodeType::Decision => {
                    return (
                        ArtifactCategory::Decision,
                        ClassificationConfidence::Certain,
                    )
                }
                NodeType::Assumption | NodeType::Risk | NodeType::Alternative => {
                    return (
                        ArtifactCategory::Decision,
                        ClassificationConfidence::Certain,
                    )
                }
                NodeType::Concept
                | NodeType::Function
                | NodeType::Method
                | NodeType::Class
                | NodeType::Struct
                | NodeType::Enum
                | NodeType::Trait
                | NodeType::Interface
                | NodeType::Tag
                | NodeType::Bug
                | NodeType::Feature => {
                    return (ArtifactCategory::Unknown, ClassificationConfidence::Certain)
                }
                _ => {} // Fall through for File, Module, Folder, Project, etc. to check path
            }
        }

        let path_str = match path {
            Some(p) => p.replace('\\', "/"),
            None => return (ArtifactCategory::Unknown, ClassificationConfidence::Unknown),
        };
        let p = Path::new(&path_str);

        let path_str_lower = path_str.to_lowercase();

        // Priority 2: Repository Location
        if path_str_lower.contains("docs/requirements/") {
            return (
                ArtifactCategory::Requirement,
                ClassificationConfidence::Certain,
            );
        }
        if path_str_lower.contains("docs/decisions/") {
            return (
                ArtifactCategory::Decision,
                ClassificationConfidence::Certain,
            );
        }
        if path_str_lower.contains("docs/architecture/") {
            return (
                ArtifactCategory::Architecture,
                ClassificationConfidence::Certain,
            );
        }
        if path_str_lower.contains("docs/evidence/") {
            return (
                ArtifactCategory::Evidence,
                ClassificationConfidence::Certain,
            );
        }
        if path_str_lower.contains("node_modules/")
            || path_str_lower.contains("vendor/")
            || path_str_lower.contains(".git/")
        {
            return (ArtifactCategory::Vendor, ClassificationConfidence::Certain);
        }
        if path_str_lower.contains("target/")
            || path_str_lower.contains("dist/")
            || path_str_lower.contains("build/")
            || path_str_lower.contains("out/")
        {
            return (
                ArtifactCategory::Generated,
                ClassificationConfidence::Certain,
            );
        }

        // Priority 3: File Role Heuristics
        let file_name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name == "codeowners" {
            return (
                ArtifactCategory::Ownership,
                ClassificationConfidence::Certain,
            );
        }
        if file_name == "ares.yaml" || file_name == "governance.yaml" || file_name == "quality.yaml"
        {
            return (ArtifactCategory::Policy, ClassificationConfidence::Certain);
        }
        if file_name == "package.json"
            || file_name == "cargo.toml"
            || file_name == "cargo.lock"
            || file_name == "yarn.lock"
            || file_name == "package-lock.json"
            || file_name == "tsconfig.json"
        {
            return (
                ArtifactCategory::Configuration,
                ClassificationConfidence::Inferred,
            );
        }
        if file_name.starts_with("dockerfile")
            || file_name == "docker-compose.yml"
            || file_name.ends_with(".tf")
        {
            return (
                ArtifactCategory::Infrastructure,
                ClassificationConfidence::Inferred,
            );
        }
        if file_name == "readme.md"
            || file_name == "changelog.md"
            || file_name == "license"
            || file_name == "license.md"
        {
            return (
                ArtifactCategory::Documentation,
                ClassificationConfidence::Certain,
            );
        }

        if path_str_lower.contains("/tests/")
            || path_str_lower.contains("/__tests__/")
            || file_name.ends_with(".spec.ts")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".test.js")
            || file_name.starts_with("test_")
            || file_name.ends_with("_test.rs")
            || file_name.ends_with("_test.go")
            || file_name.ends_with("test.rs")
        {
            return (ArtifactCategory::Test, ClassificationConfidence::Inferred);
        }

        // Priority 4: Extension
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" | "ts" | "js" | "go" | "java" | "py" | "c" | "cpp" | "cs" | "rb" | "php"
                | "swift" | "kt" => {
                    return (ArtifactCategory::Code, ClassificationConfidence::Inferred);
                }
                "md" | "txt" => {
                    return (
                        ArtifactCategory::Documentation,
                        ClassificationConfidence::Inferred,
                    );
                }
                "yaml" | "yml" | "json" | "toml" | "xml" | "ini" | "conf" => {
                    return (
                        ArtifactCategory::Configuration,
                        ClassificationConfidence::Inferred,
                    );
                }
                "sh" | "bash" | "ps1" | "bat" => {
                    return (
                        ArtifactCategory::Infrastructure,
                        ClassificationConfidence::Inferred,
                    );
                }
                _ => {}
            }
        }

        // Priority 5: Fallback
        (ArtifactCategory::Unknown, ClassificationConfidence::Unknown)
    }

    fn determine_eligibility(category: &ArtifactCategory) -> MemoryEligibility {
        match category {
            ArtifactCategory::Requirement
            | ArtifactCategory::Decision
            | ArtifactCategory::Architecture
            | ArtifactCategory::Evidence
            | ArtifactCategory::Code
            | ArtifactCategory::Infrastructure => MemoryEligibility::Required,

            ArtifactCategory::Test => MemoryEligibility::Recommended,

            ArtifactCategory::Documentation
            | ArtifactCategory::Configuration
            | ArtifactCategory::Ownership
            | ArtifactCategory::Policy
            | ArtifactCategory::RepositoryEvent
            | ArtifactCategory::Snapshot => MemoryEligibility::Optional,

            ArtifactCategory::Generated | ArtifactCategory::Vendor | ArtifactCategory::Unknown => {
                MemoryEligibility::Excluded
            }
        }
    }
}
