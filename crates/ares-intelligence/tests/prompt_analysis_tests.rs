use ares_intelligence::analysis::prompt::PromptAnalyzer;
use ares_intelligence::models::capability::{ModelCapability, TaskType};

#[test]
fn test_analyze_empty_prompt_rejection() {
    let analyzer = PromptAnalyzer::new();
    let res = analyzer.analyze_task("");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Prompt cannot be empty");

    let res = analyzer.analyze_task("   ");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "Prompt cannot be empty");
}

#[test]
fn test_analyze_too_long_prompt_rejection() {
    let analyzer = PromptAnalyzer::new();
    let long_prompt = "a".repeat(100_001);
    let res = analyzer.analyze_task(&long_prompt);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Prompt exceeds maximum length"
    );
}

#[test]
fn test_analyze_task_type_coding() {
    let analyzer = PromptAnalyzer::new();
    assert_eq!(
        analyzer
            .analyze_task("Can you write a fn main() {}?")
            .unwrap(),
        TaskType::Coding
    );
    assert_eq!(
        analyzer
            .analyze_task("Please define a struct User {")
            .unwrap(),
        TaskType::Coding
    );
    assert_eq!(
        analyzer
            .analyze_task("I need you to write code for this")
            .unwrap(),
        TaskType::Coding
    );
}

#[test]
fn test_analyze_task_type_reasoning() {
    let analyzer = PromptAnalyzer::new();
    assert_eq!(
        analyzer
            .analyze_task("Explain how quantum physics works")
            .unwrap(),
        TaskType::Reasoning
    );
    assert_eq!(
        analyzer.analyze_task("Analyze this trend").unwrap(),
        TaskType::Reasoning
    );
    assert_eq!(
        analyzer.analyze_task("Why is the sky blue?").unwrap(),
        TaskType::Reasoning
    );
}

#[test]
fn test_analyze_task_type_summarization() {
    let analyzer = PromptAnalyzer::new();
    assert_eq!(
        analyzer
            .analyze_task("Please summarize the following article")
            .unwrap(),
        TaskType::Summarization
    );
    assert_eq!(
        analyzer.analyze_task("Give me a TLDR of this").unwrap(),
        TaskType::Summarization
    );
    assert_eq!(
        analyzer
            .analyze_task("Briefly tell me what happened")
            .unwrap(),
        TaskType::Summarization
    );
}

#[test]
fn test_analyze_task_type_data_extraction() {
    let analyzer = PromptAnalyzer::new();
    assert_eq!(
        analyzer
            .analyze_task("Extract the names from this text")
            .unwrap(),
        TaskType::DataExtraction
    );
    assert_eq!(
        analyzer.analyze_task("parse JSON data here").unwrap(),
        TaskType::DataExtraction
    );
}

#[test]
fn test_analyze_task_type_default() {
    let analyzer = PromptAnalyzer::new();
    // Default fallback is Reasoning
    assert_eq!(
        analyzer.analyze_task("Hello there!").unwrap(),
        TaskType::Reasoning
    );
}

#[test]
fn test_extract_capabilities_empty_prompt() {
    let analyzer = PromptAnalyzer::new();
    let res = analyzer.extract_capabilities("");
    assert!(res.is_err());
}

#[test]
fn test_extract_capabilities_tool_use() {
    let analyzer = PromptAnalyzer::new();
    let caps = analyzer
        .extract_capabilities("Search the web for news")
        .unwrap();
    assert!(caps.contains(&ModelCapability::ToolUse));

    let caps2 = analyzer
        .extract_capabilities("Look up the weather")
        .unwrap();
    assert!(caps2.contains(&ModelCapability::ToolUse));
}

#[test]
fn test_extract_capabilities_coding() {
    let analyzer = PromptAnalyzer::new();
    let caps = analyzer
        .extract_capabilities("Write a rust script")
        .unwrap();
    assert!(caps.contains(&ModelCapability::Coding));
}

#[test]
fn test_extract_capabilities_vision() {
    let analyzer = PromptAnalyzer::new();
    let caps = analyzer
        .extract_capabilities("What is in this image?")
        .unwrap();
    assert!(caps.contains(&ModelCapability::Vision));

    let caps2 = analyzer
        .extract_capabilities("Draw a picture of a cat")
        .unwrap();
    assert!(caps2.contains(&ModelCapability::Vision));
}

#[test]
fn test_extract_capabilities_multiple() {
    let analyzer = PromptAnalyzer::new();
    let caps = analyzer
        .extract_capabilities("Search google for an image of a function graph")
        .unwrap();
    // Contains: search -> ToolUse, image -> Vision, function -> Coding
    assert!(caps.contains(&ModelCapability::ToolUse));
    assert!(caps.contains(&ModelCapability::Vision));
    assert!(caps.contains(&ModelCapability::Coding));
}
