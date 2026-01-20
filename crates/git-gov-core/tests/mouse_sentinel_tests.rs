use git_gov_core::mouse_sentinel::{MouseSentinel, MouseEvent, KinematicMetrics, MouseSentinelError};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_mouse_sentinel_initialization() {
    let sentinel = MouseSentinel::new(100);
    assert_eq!(sentinel.max_buffer_size, 100);
}

#[test]
fn test_capture_event() {
    let sentinel = MouseSentinel::new(10);
    
    sentinel.capture_event(100.0, 200.0);
     
    let buffer = sentinel.event_buffer.lock().unwrap();
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0].x, 100.0);
    assert_eq!(buffer[0].y, 200.0);
    assert!(buffer[0].timestamp > 0); // Verify timestamp is captured
}

#[test]
fn test_buffer_overflow() {
    let sentinel = MouseSentinel::new(3);
    
    for i in 0..5 {
        sentinel.capture_event(i as f64, i as f64);
    }
    
    let buffer = sentinel.event_buffer.lock().unwrap();
    assert_eq!(buffer.len(), 3);
    assert_eq!(buffer[0].x, 2.0); // Oldest event should be dropped
}

#[test]
fn test_insufficient_data_error() {
    let sentinel = MouseSentinel::new(10);
    
    // Capture only 2 events (need at least 3 for analysis)
    sentinel.capture_event(1.0, 1.0);
    sentinel.capture_event(2.0, 2.0);
    
    let result = sentinel.analyze_events();
    assert!(matches!(result, Err(MouseSentinelError::InsufficientData)));
}

#[test]
fn test_kinematic_metrics_calculation() {
    let sentinel = MouseSentinel::new(10);
    
    // Create a simple straight line movement with enough points and time variation
    for i in 0..10 {
        sentinel.capture_event(i as f64 * 10.0, i as f64 * 10.0);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    let result = sentinel.analyze_events();
    assert!(result.is_ok(), "Should have enough data for analysis");
    
    let metrics = result.unwrap();
    assert!(metrics.ldlj.is_finite());
    assert!(metrics.spec_entropy.is_finite());
    assert!(metrics.path_entropy.is_finite());
    assert!(metrics.throughput.is_finite());
}

#[test]
fn test_curvature_entropy() {
    let sentinel = MouseSentinel::new(10);
    
    // Create a curved movement pattern with enough points and time variation
    for i in 0..15 {
        let angle = i as f64 * 0.1;
        sentinel.capture_event(angle.cos() * 100.0, angle.sin() * 100.0);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    let result = sentinel.analyze_events();
    assert!(result.is_ok(), "Should have enough data for curvature analysis");
    
    let metrics = result.unwrap();
    // Curved path should have higher entropy than straight line
    assert!(metrics.path_entropy > 0.5);
}

#[test]
fn test_throughput_calculation() {
    let sentinel = MouseSentinel::new(10);
    
    // Create a fast movement with enough points and time variation
    for i in 0..10 {
        sentinel.capture_event(i as f64 * 100.0, 0.0);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    let result = sentinel.analyze_events();
    assert!(result.is_ok(), "Should have enough data for throughput calculation");
    
    let metrics = result.unwrap();
    // Fast movement should have higher throughput
    assert!(metrics.throughput > 0.1);
}