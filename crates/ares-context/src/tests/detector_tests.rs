use crate::query::{IntentDetector, QueryIntent};

#[test]
fn test_detect_explain_intent() {
    let detector = IntentDetector::new();
    assert_eq!(detector.detect("explain how parser works"), QueryIntent::FileExplanation);
    assert_eq!(detector.detect("what does this file do"), QueryIntent::FileExplanation);
}

#[test]
fn test_detect_architecture() {
    let detector = IntentDetector::new();
    assert_eq!(detector.detect("explain architecture of the system"), QueryIntent::ArchitectureQuery);
}

#[test]
fn test_detect_impact() {
    let detector = IntentDetector::new();
    assert_eq!(detector.detect("what is the impact of changing auth.rs"), QueryIntent::ChangeImpact);
}

#[test]
fn test_detect_trace() {
    let detector = IntentDetector::new();
    assert_eq!(detector.detect("trace dependencies for main.rs"), QueryIntent::DependencyTrace);
}
