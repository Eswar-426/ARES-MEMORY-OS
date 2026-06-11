use ares_intelligence::telemetry::metrics::MetricsService;
use ares_intelligence::telemetry::tracing::TracingService;

#[test]
fn test_tracing_start_span_generates_uuid() {
    let tracing = TracingService::default();
    let span_id = tracing.start_span("test_operation");

    // Ensure it generates a valid UUID v7
    let parsed = uuid::Uuid::parse_str(&span_id);
    assert!(parsed.is_ok());
}

#[test]
fn test_tracing_start_span_unique_ids() {
    let tracing = TracingService::default();
    let span1 = tracing.start_span("op1");
    let span2 = tracing.start_span("op2");
    assert_ne!(span1, span2);
}

#[test]
fn test_tracing_end_span_does_not_panic_on_invalid_id() {
    let tracing = TracingService::default();
    // It should safely handle an invalid or untracked ID without panic
    tracing.end_span("invalid-span-id");
}

#[test]
fn test_tracing_end_span_does_not_panic_on_valid_id() {
    let tracing = TracingService::default();
    let span_id = tracing.start_span("test");
    tracing.end_span(&span_id);
}

#[test]
fn test_metrics_record_counter_basic() {
    let metrics = MetricsService::default();
    // Verify it doesn't panic
    metrics.record_counter("total_requests", 1);
}

#[test]
fn test_metrics_record_counter_large_value() {
    let metrics = MetricsService::default();
    metrics.record_counter("bytes_processed", 1_000_000_000);
}

#[test]
fn test_metrics_record_counter_zero_value() {
    let metrics = MetricsService::default();
    metrics.record_counter("events", 0);
}

#[test]
fn test_metrics_record_counter_empty_name() {
    let metrics = MetricsService::default();
    // We should be resilient to empty metric names
    metrics.record_counter("", 1);
}

#[test]
fn test_tracing_concurrent_span_creation() {
    use std::sync::Arc;
    use std::thread;

    let tracing = Arc::new(TracingService::default());
    let mut handles = vec![];

    for _ in 0..10 {
        let t = Arc::clone(&tracing);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let _span = t.start_span("concurrent_op");
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_metrics_concurrent_recording() {
    use std::sync::Arc;
    use std::thread;

    let metrics = Arc::new(MetricsService::default());
    let mut handles = vec![];

    for _ in 0..10 {
        let m = Arc::clone(&metrics);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                m.record_counter("concurrent_metric", 1);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
