//! Performance and stress tests for the p4-mcp server in mock mode

use p4_mcp::mcp::*;
use serde_json::json;
use std::env;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Test helper to set up mock mode
fn setup_mock_mode() {
    env::set_var("P4_MOCK_MODE", "1");
}

/// Test helper to create a call tool message
fn create_call_tool_message(id: &str, tool_name: &str, arguments: serde_json::Value) -> MCPMessage {
    serde_json::from_value(json!({
        "method": "tools/call",
        "id": id,
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    }))
    .unwrap()
}

#[tokio::test]
async fn test_high_volume_message_processing() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message_count = 1000;
    let start_time = Instant::now();

    for i in 0..message_count {
        let message = create_call_tool_message(
            &format!("perf-test-{}", i),
            "p4_status",
            json!({"path": format!("//depot/test/{}/...", i)}),
        );

        let response = server.handle_message(message).await.unwrap().unwrap();

        match response {
            MCPResponse::CallToolResult { id, .. } => {
                assert_eq!(id, format!("perf-test-{}", i));
            }
            _ => panic!("Expected CallToolResult"),
        }
    }

    let duration = start_time.elapsed();
    let messages_per_second = message_count as f64 / duration.as_secs_f64();

    println!(
        "Processed {} messages in {:?} ({:.2} messages/second)",
        message_count, duration, messages_per_second
    );

    // Should be able to process at least 100 messages per second in mock mode
    assert!(
        messages_per_second > 100.0,
        "Performance too slow: {} messages/second",
        messages_per_second
    );
}

#[tokio::test]
async fn test_large_file_lists_performance() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test with increasingly large file lists
    let file_counts = vec![10, 100, 1000, 5000];

    for file_count in file_counts {
        let files: Vec<String> = (0..file_count)
            .map(|i| format!("large_test_file_{}.cpp", i))
            .collect();

        let start_time = Instant::now();

        let message = create_call_tool_message(
            &format!("large-files-{}", file_count),
            "p4_edit",
            json!({"files": files}),
        );

        let response = server.handle_message(message).await.unwrap().unwrap();
        let duration = start_time.elapsed();

        match response {
            MCPResponse::CallToolResult { id, result } => {
                assert_eq!(id, format!("large-files-{}", file_count));
                if let ToolContent::Text { text } = &result.content[0] {
                    assert!(text.contains(&format!("{} file(s) opened for edit", file_count)));
                }
            }
            _ => panic!("Expected CallToolResult"),
        }

        println!(
            "Processed {} files in {:?} ({:.2} files/second)",
            file_count,
            duration,
            file_count as f64 / duration.as_secs_f64()
        );

        // Should handle large file lists within reasonable time (< 100ms for 5000 files)
        assert!(
            duration < Duration::from_millis(100),
            "Too slow for {} files: {:?}",
            file_count,
            duration
        );
    }
}

#[tokio::test]
async fn test_concurrent_message_processing() {
    setup_mock_mode();

    let concurrent_requests = 50;
    let mut handles = Vec::new();

    let start_time = Instant::now();

    for i in 0..concurrent_requests {
        let handle = tokio::spawn(async move {
            let mut server = MCPServer::new();
            let message = create_call_tool_message(
                &format!("concurrent-{}", i),
                "p4_status",
                json!({"path": format!("//depot/concurrent/{}/...", i)}),
            );

            server.handle_message(message).await.unwrap().unwrap()
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut responses = Vec::new();
    for handle in handles {
        let response = handle.await.unwrap();
        responses.push(response);
    }

    let duration = start_time.elapsed();

    assert_eq!(responses.len(), concurrent_requests);

    // Verify all responses are correct
    for (i, response) in responses.iter().enumerate() {
        match response {
            MCPResponse::CallToolResult { id, .. } => {
                assert_eq!(*id, format!("concurrent-{}", i));
            }
            _ => panic!("Expected CallToolResult"),
        }
    }

    println!(
        "Processed {} concurrent requests in {:?}",
        concurrent_requests, duration
    );

    // Should complete concurrent requests within reasonable time
    assert!(
        duration < Duration::from_secs(5),
        "Concurrent processing too slow: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_memory_usage_with_large_responses() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test that large responses don't cause memory issues
    for i in 0..100 {
        let message = create_call_tool_message(
            &format!("memory-test-{}", i),
            "p4_changes",
            json!({"max": 100, "path": format!("//depot/memory-test/{}/...", i)}),
        );

        let response = server.handle_message(message).await.unwrap().unwrap();

        match response {
            MCPResponse::CallToolResult { id, result } => {
                assert_eq!(id, format!("memory-test-{}", i));
                if let ToolContent::Text { text } = &result.content[0] {
                    // Verify we get the expected mock response
                    assert!(text.contains("Mock P4 Changes"));
                }
            }
            _ => panic!("Expected CallToolResult"),
        }

        // Force garbage collection periodically (in a real scenario)
        if i % 10 == 0 {
            tokio::task::yield_now().await;
        }
    }
}

#[tokio::test]
async fn test_response_time_consistency() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let iterations = 100;
    let mut response_times = Vec::new();

    for i in 0..iterations {
        let start_time = Instant::now();

        let message = create_call_tool_message(
            &format!("timing-test-{}", i),
            "p4_status",
            json!({"path": "//depot/timing-test/..."}),
        );

        let _response = server.handle_message(message).await.unwrap().unwrap();
        let duration = start_time.elapsed();
        response_times.push(duration);
    }

    // Calculate statistics
    let total_time: Duration = response_times.iter().sum();
    let average_time = total_time / iterations as u32;

    let min_time = *response_times.iter().min().unwrap();
    let max_time = *response_times.iter().max().unwrap();

    println!("Response time statistics over {} iterations:", iterations);
    println!("  Average: {:?}", average_time);
    println!("  Min: {:?}", min_time);
    println!("  Max: {:?}", max_time);

    // Mock responses should be very fast and consistent
    assert!(
        average_time < Duration::from_millis(10),
        "Average response time too slow: {:?}",
        average_time
    );
    assert!(
        max_time < Duration::from_millis(50),
        "Max response time too slow: {:?}",
        max_time
    );

    // Check for reasonable consistency (max should not be more than 10x average)
    let ratio = max_time.as_nanos() as f64 / average_time.as_nanos() as f64;
    assert!(
        ratio < 10.0,
        "Response times too inconsistent. Max/Avg ratio: {:.2}",
        ratio
    );
}

#[tokio::test]
async fn test_timeout_handling() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test that operations complete well within timeout
    let message = create_call_tool_message(
        "timeout-test",
        "p4_sync",
        json!({"path": "//depot/timeout-test/...", "force": true}),
    );

    let result = timeout(Duration::from_secs(1), server.handle_message(message)).await;

    assert!(result.is_ok(), "Operation should complete within timeout");

    let response = result.unwrap().unwrap().unwrap();
    match response {
        MCPResponse::CallToolResult { id, .. } => {
            assert_eq!(id, "timeout-test");
        }
        _ => panic!("Expected CallToolResult"),
    }
}

#[tokio::test]
async fn test_all_tools_performance() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let tools_and_args = vec![
        ("p4_status", json!({"path": "//depot/perf/..."})),
        (
            "p4_sync",
            json!({"path": "//depot/perf/...", "force": false}),
        ),
        ("p4_edit", json!({"files": ["file1.cpp", "file2.h"]})),
        ("p4_add", json!({"files": ["new_file.cpp"]})),
        (
            "p4_submit",
            json!({"description": "Performance test submit"}),
        ),
        ("p4_revert", json!({"files": ["file1.cpp"]})),
        ("p4_opened", json!({})),
        ("p4_changes", json!({"max": 10})),
    ];

    let start_time = Instant::now();

    for (i, (tool_name, args)) in tools_and_args.iter().enumerate() {
        let message =
            create_call_tool_message(&format!("all-tools-{}", i), tool_name, args.clone());

        let response = server.handle_message(message).await.unwrap().unwrap();

        match response {
            MCPResponse::CallToolResult { id, .. } => {
                assert_eq!(id, format!("all-tools-{}", i));
            }
            _ => panic!("Expected CallToolResult for tool: {}", tool_name),
        }
    }

    let duration = start_time.elapsed();
    let tools_per_second = tools_and_args.len() as f64 / duration.as_secs_f64();

    println!(
        "Processed all {} tools in {:?} ({:.2} tools/second)",
        tools_and_args.len(),
        duration,
        tools_per_second
    );

    // All tools should execute quickly in mock mode
    assert!(
        duration < Duration::from_millis(100),
        "All tools execution too slow: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_mixed_workload_performance() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let operations = vec![
        // Mix of different operations with varying complexity
        (
            "initialize",
            json!({"protocolVersion": "2024-11-05", "capabilities": {"roots": {"listChanged": false}}, "clientInfo": {"name": "perf-test", "version": "1.0"}}),
        ),
        ("tools/list", json!({})),
        (
            "tools/call",
            json!({"name": "p4_status", "arguments": {"path": "//depot/mixed/..."}}),
        ),
        (
            "tools/call",
            json!({"name": "p4_edit", "arguments": {"files": (0..10).map(|i| format!("file{}.cpp", i)).collect::<Vec<_>>()}}),
        ),
        (
            "tools/call",
            json!({"name": "p4_changes", "arguments": {"max": 50}}),
        ),
        ("ping", json!({})),
    ];

    let start_time = Instant::now();

    for (i, (method, params)) in operations.iter().enumerate() {
        let message = if *method == "tools/call" {
            serde_json::from_value(json!({
                "method": method,
                "id": format!("mixed-{}", i),
                "params": params
            }))
            .unwrap()
        } else if *method == "initialize" {
            serde_json::from_value(json!({
                "method": method,
                "id": format!("mixed-{}", i),
                "params": params
            }))
            .unwrap()
        } else {
            serde_json::from_value(json!({
                "method": method,
                "id": format!("mixed-{}", i)
            }))
            .unwrap()
        };

        let response = server.handle_message(message).await.unwrap().unwrap();

        // Verify we got a response with correct ID
        let response_id = match &response {
            MCPResponse::InitializeResult { id, .. } => id,
            MCPResponse::ListToolsResult { id, .. } => id,
            MCPResponse::CallToolResult { id, .. } => id,
            MCPResponse::Pong { id } => id,
            MCPResponse::Error { id, .. } => id,
        };

        assert_eq!(response_id, &format!("mixed-{}", i));
    }

    let duration = start_time.elapsed();
    let ops_per_second = operations.len() as f64 / duration.as_secs_f64();

    println!(
        "Processed mixed workload of {} operations in {:?} ({:.2} ops/second)",
        operations.len(),
        duration,
        ops_per_second
    );

    // Mixed workload should still be fast
    assert!(
        duration < Duration::from_millis(200),
        "Mixed workload too slow: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_stress_test_rapid_fire() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let rapid_fire_count = 500;
    let start_time = Instant::now();

    // Rapid fire requests without waiting
    let mut tasks = Vec::new();

    for i in 0..rapid_fire_count {
        let message = create_call_tool_message(
            &format!("rapid-{}", i),
            if i % 2 == 0 { "p4_status" } else { "p4_opened" },
            json!({}),
        );

        // Clone server state for each task (in real usage, each connection would have its own server)
        tasks.push(tokio::spawn(async move {
            let mut local_server = MCPServer::new();
            local_server.handle_message(message).await
        }));
    }

    // Wait for all to complete
    for task in tasks {
        let result = task.await.unwrap().unwrap().unwrap();
        match result {
            MCPResponse::CallToolResult { .. } => {
                // Success
            }
            _ => panic!("Expected CallToolResult"),
        }
    }

    let duration = start_time.elapsed();
    let requests_per_second = rapid_fire_count as f64 / duration.as_secs_f64();

    println!(
        "Completed {} rapid-fire requests in {:?} ({:.2} requests/second)",
        rapid_fire_count, duration, requests_per_second
    );

    // Should handle rapid fire requests efficiently
    assert!(
        duration < Duration::from_secs(10),
        "Rapid fire test too slow: {:?}",
        duration
    );
}
