use ares_orchestrator::runtime::retry::RetryPolicy;

#[test]
fn test_immediate_policy() {
    let policy = RetryPolicy::Immediate { max_retries: 3 };

    assert_eq!(policy.calculate_delay_ms(0), Some(0)); // Attempt 1
    assert_eq!(policy.calculate_delay_ms(1), Some(0)); // Attempt 2
    assert_eq!(policy.calculate_delay_ms(2), Some(0)); // Attempt 3
    assert_eq!(policy.calculate_delay_ms(3), None);    // Exceeded
}

#[test]
fn test_fixed_delay_policy() {
    let policy = RetryPolicy::FixedDelay { max_retries: 3, delay_ms: 5000 };

    assert_eq!(policy.calculate_delay_ms(0), Some(5000));
    assert_eq!(policy.calculate_delay_ms(1), Some(5000));
    assert_eq!(policy.calculate_delay_ms(2), Some(5000));
    assert_eq!(policy.calculate_delay_ms(3), None);
}

#[test]
fn test_exponential_backoff_policy() {
    let policy = RetryPolicy::ExponentialBackoff { max_retries: 3, initial_delay_ms: 1000, multiplier: 2.0 };

    // attempt 0 (first retry): 1000 * 2^0 = 1000
    assert_eq!(policy.calculate_delay_ms(0), Some(1000));
    
    // attempt 1 (second retry): 1000 * 2^1 = 2000
    assert_eq!(policy.calculate_delay_ms(1), Some(2000));
    
    // attempt 2 (third retry): 1000 * 2^2 = 4000
    assert_eq!(policy.calculate_delay_ms(2), Some(4000));
    
    // exceeded
    assert_eq!(policy.calculate_delay_ms(3), None);
}
