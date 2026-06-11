use ares_intelligence::context::manager::ContextManager;

#[test]
fn test_context_estimation() {
    let cm = ContextManager::new(1000);
    // Simple heuristic: length / 4
    let text = "a".repeat(400);
    assert_eq!(cm.estimate_tokens(&text), 100);
}

#[test]
fn test_context_overflow_protection() {
    // 100 tokens max ~ 400 chars max
    let cm = ContextManager::new(100);
    let sys = "a".repeat(40); // 10 tokens
    let prompt = "b".repeat(80); // 20 tokens

    // We have 100 - 10 - 20 = 70 tokens left for memories.
    // Each memory is 40 tokens (160 chars)
    let m1 = "c".repeat(160); // 40 tokens -> fits
    let m2 = "d".repeat(160); // 40 tokens -> overflows, should be excluded

    let res = cm
        .build_context(&prompt, &[m1.clone(), m2.clone()], &sys)
        .unwrap();

    assert!(res.contains(&sys));
    assert!(res.contains(&prompt));
    assert!(res.contains(&m1));
    assert!(!res.contains(&m2));
}

#[test]
fn test_context_prompt_exceeds_max() {
    let cm = ContextManager::new(10); // 40 chars max
    let sys = "s".repeat(20); // 5 tokens
    let prompt = "p".repeat(24); // 6 tokens -> Total 11 tokens > 10 tokens

    let res = cm.build_context(&prompt, &[], &sys);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "Prompt and system instructions exceed maximum context window"
    );
}

#[test]
fn test_context_100kb_stress() {
    // 100 KB context ~ 25,000 tokens
    let cm = ContextManager::new(30_000);
    let sys = "s".repeat(1000);
    let prompt = "p".repeat(1000);

    // Create 100 memories of 1 KB each -> 100 KB total
    let memories: Vec<String> = (0..100)
        .map(|i| format!("Memory {}: {}", i, "m".repeat(1000)))
        .collect();

    let res = cm.build_context(&prompt, &memories, &sys).unwrap();
    // Ensure all 100 memories fit
    assert!(res.contains("Memory 99:"));
    assert!(res.len() > 100_000); // Verify it produced a large payload
}

#[test]
fn test_context_1mb_stress() {
    // 1 MB context ~ 250,000 tokens
    let cm = ContextManager::new(300_000);
    let sys = "System Context";
    let prompt = "User Prompt";

    // Create 1000 memories of 1 KB each -> 1 MB total
    let memories: Vec<String> = (0..1000)
        .map(|i| format!("Memory {}: {}", i, "m".repeat(1000)))
        .collect();

    let res = cm.build_context(prompt, &memories, sys).unwrap();
    // Ensure the last memory fits
    assert!(res.contains("Memory 999:"));
    assert!(res.len() > 1_000_000); // Verify it produced a 1 MB payload
}

#[test]
fn test_context_empty_memory() {
    let cm = ContextManager::new(1000);
    let res = cm.build_context("prompt", &[], "sys").unwrap();
    assert!(res.contains("prompt"));
    assert!(res.contains("sys"));
    // It should just contain the formatting
    assert!(res.contains("=== Relevant Context ==="));
}

#[test]
fn test_context_duplicate_deduplication() {
    // Note: The original ContextManager doesn't actually deduplicate,
    // it just truncates. The user prompt specified "duplicates".
    // I should test that duplicates are either included or handled appropriately by the manager logic.
    // If we want deduplication, we should enhance the ContextManager.

    let cm = ContextManager::new(1000);
    let m1 = "duplicate memory".to_string();
    let m2 = "duplicate memory".to_string();

    // In our current implementation, it doesn't deduplicate at this layer.
    // But let's verify behavior. It just appends both.
    let res = cm.build_context("prompt", &[m1, m2], "sys").unwrap();
    let matches: Vec<_> = res.match_indices("duplicate memory").collect();
    // Wait, we SHOULD enhance ContextManager to deduplicate as required by "duplicates" edge case?
    // Let's expect the manager to deduplicate.
    assert_eq!(
        matches.len(),
        1,
        "Context manager should deduplicate memories"
    );
}

#[test]
fn test_context_high_fragmentation() {
    let cm = ContextManager::new(1000);
    let sys = "sys";
    let prompt = "prompt";

    // Create 10,000 tiny unique memories
    let memories: Vec<String> = (0..10000).map(|i| format!("x{:04}", i)).collect();

    // The manager should fit up to the token limit and neatly truncate.
    let res = cm.build_context(prompt, &memories, sys).unwrap();

    // Each unique string is "x0000" (5 chars). estimate_tokens("x0000") = max(5/4, 1) = 1 token.
    // 1000 tokens remaining. So we should fit ~1000 memories.
    let count_x = res.match_indices("\nx").count();
    assert!(
        count_x > 100 && count_x < 2000,
        "Should handle fragmentation securely"
    );
}
