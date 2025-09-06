//! Integration tests for the p4-mcp server
//! These tests run in mock mode to avoid requiring actual Perforce setup

use p4_mcp::mcp::{MCPMessage, MCPResponse, MCPServer};
use serde_json::json;
use std::env;
use tokio_test;

/// Test helper to set up mock mode
fn setup_mock_mode() {
    env::set_var("P4_MOCK_MODE", "1");
}

/// Test helper to create a basic initialize message
fn create_initialize_message(id: &str) -> MCPMessage {
    serde_json::from_value(json!({
        "method": "initialize",
        "id": id,
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {
                    "listChanged": false
                }
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    }))
    .unwrap()
}

/// Test helper to create a list tools message
fn create_list_tools_message(id: &str) -> MCPMessage {
    serde_json::from_value(json!({
        "method": "tools/list",
        "id": id
    }))
    .unwrap()
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

/// Test helper to create a ping message
fn create_ping_message(id: &str) -> MCPMessage {
    serde_json::from_value(json!({
        "method": "ping",
        "id": id
    }))
    .unwrap()
}

#[tokio::test]
async fn test_initialize_endpoint() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_initialize_message("test-1");
    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::InitializeResult { id, result } => {
            assert_eq!(id, "test-1");
            assert_eq!(result.protocol_version, "2024-11-05");
            assert_eq!(result.server_info.name, "p4-mcp");
            assert_eq!(result.server_info.version, "0.1.0");
            assert!(result.capabilities.tools.is_some());
        }
        _ => panic!("Expected InitializeResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_list_tools_endpoint() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_list_tools_message("test-2");
    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::ListToolsResult { id, result } => {
            assert_eq!(id, "test-2");
            assert_eq!(result.tools.len(), 8); // We expect 8 P4 tools

            let tool_names: Vec<&str> = result.tools.iter().map(|t| t.name.as_str()).collect();
            assert!(tool_names.contains(&"p4_status"));
            assert!(tool_names.contains(&"p4_sync"));
            assert!(tool_names.contains(&"p4_edit"));
            assert!(tool_names.contains(&"p4_add"));
            assert!(tool_names.contains(&"p4_submit"));
            assert!(tool_names.contains(&"p4_revert"));
            assert!(tool_names.contains(&"p4_opened"));
            assert!(tool_names.contains(&"p4_changes"));
        }
        _ => panic!("Expected ListToolsResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_ping_endpoint() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_ping_message("test-ping");
    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::Pong { id } => {
            assert_eq!(id, "test-ping");
        }
        _ => panic!("Expected Pong, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_status_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test with path parameter
    let message = create_call_tool_message(
        "test-status-1",
        "p4_status",
        json!({"path": "//depot/main/..."}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-status-1");
            assert_eq!(result.content.len(), 1);
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Status"));
                assert!(text.contains("//depot/main/..."));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }

    // Test without path parameter
    let message = create_call_tool_message("test-status-2", "p4_status", json!({}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-status-2");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Status"));
                assert!(text.contains("current directory"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_sync_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test sync with path and force
    let message = create_call_tool_message(
        "test-sync-1",
        "p4_sync",
        json!({"path": "//depot/main/...", "force": true}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-sync-1");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Sync"));
                assert!(text.contains("(forced)"));
                assert!(text.contains("//depot/main/..."));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }

    // Test sync without force
    let message = create_call_tool_message(
        "test-sync-2",
        "p4_sync",
        json!({"path": "//depot/test/..."}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-sync-2");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Sync"));
                assert!(!text.contains("(forced)"));
                assert!(text.contains("//depot/test/..."));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_edit_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_call_tool_message(
        "test-edit",
        "p4_edit",
        json!({"files": ["file1.cpp", "file2.h", "file3.txt"]}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-edit");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Edit"));
                assert!(text.contains("file1.cpp"));
                assert!(text.contains("file2.h"));
                assert!(text.contains("file3.txt"));
                assert!(text.contains("3 file(s) opened for edit"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_add_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_call_tool_message(
        "test-add",
        "p4_add",
        json!({"files": ["new_file1.cpp", "new_file2.h"]}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-add");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Add"));
                assert!(text.contains("new_file1.cpp"));
                assert!(text.contains("new_file2.h"));
                assert!(text.contains("2 file(s) opened for add"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_submit_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test submit with description only
    let message = create_call_tool_message(
        "test-submit-1",
        "p4_submit",
        json!({"description": "Fix bug in authentication module"}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-submit-1");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Submit"));
                assert!(text.contains("Fix bug in authentication module"));
                assert!(text.contains("All opened files"));
                assert!(text.contains("Change 12345 submitted"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }

    // Test submit with specific files
    let message = create_call_tool_message(
        "test-submit-2",
        "p4_submit",
        json!({
            "description": "Update documentation",
            "files": ["README.md", "docs/api.md"]
        }),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-submit-2");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Submit"));
                assert!(text.contains("Update documentation"));
                assert!(text.contains("README.md"));
                assert!(text.contains("docs/api.md"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_revert_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_call_tool_message(
        "test-revert",
        "p4_revert",
        json!({"files": ["file1.cpp", "file2.h"]}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-revert");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Revert"));
                assert!(text.contains("file1.cpp"));
                assert!(text.contains("file2.h"));
                assert!(text.contains("2 file(s) reverted"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_opened_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test opened without changelist
    let message = create_call_tool_message("test-opened-1", "p4_opened", json!({}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-opened-1");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Opened"));
                assert!(text.contains("//depot/main/file1.txt"));
                assert!(text.contains("//depot/main/file2.cpp"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }

    // Test opened with specific changelist
    let message =
        create_call_tool_message("test-opened-2", "p4_opened", json!({"changelist": "12346"}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-opened-2");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Opened"));
                assert!(text.contains("in changelist 12346"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_p4_changes_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test changes with default max
    let message = create_call_tool_message("test-changes-1", "p4_changes", json!({}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-changes-1");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Changes"));
                assert!(text.contains("max: 10"));
                assert!(text.contains("Change 12350"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }

    // Test changes with custom max and path
    let message = create_call_tool_message(
        "test-changes-2",
        "p4_changes",
        json!({"max": 5, "path": "//depot/main/..."}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-changes-2");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Changes"));
                assert!(text.contains("max: 5"));
                assert!(text.contains("for path //depot/main/..."));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_unknown_tool_error() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    let message = create_call_tool_message("test-unknown", "unknown_tool", json!({}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::Error { id, error } => {
            assert_eq!(id, "test-unknown");
            assert_eq!(error.code, -32602);
            assert!(error.message.contains("Unknown tool: unknown_tool"));
        }
        _ => panic!("Expected Error response, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_missing_required_parameters() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test p4_edit without required files parameter
    let message = create_call_tool_message("test-missing-files", "p4_edit", json!({}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    // Should handle gracefully with empty files array
    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-missing-files");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Edit"));
                assert!(text.contains("0 file(s) opened for edit"));
            } else {
                panic!("Expected text content");
            }
        }
        _ => panic!("Expected CallToolResult, got: {:?}", response),
    }
}

#[tokio::test]
async fn test_message_serialization_deserialization() {
    // Test that our messages can be properly serialized and deserialized
    let init_msg = create_initialize_message("test-serialize");
    let json_str = serde_json::to_string(&init_msg).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&json_str).unwrap();

    match (init_msg, deserialized) {
        (MCPMessage::Initialize { id: id1, .. }, MCPMessage::Initialize { id: id2, .. }) => {
            assert_eq!(id1, id2);
        }
        _ => panic!("Serialization/deserialization failed"),
    }
}

#[tokio::test]
async fn test_sequential_message_handling() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test handling multiple messages in sequence
    let messages = vec![
        create_initialize_message("seq-1"),
        create_list_tools_message("seq-2"),
        create_call_tool_message("seq-3", "p4_status", json!({})),
        create_ping_message("seq-4"),
    ];

    let mut responses = Vec::new();

    for message in messages {
        let response = server.handle_message(message).await.unwrap().unwrap();
        responses.push(response);
    }

    assert_eq!(responses.len(), 4);

    // Check that responses have correct IDs in order
    match &responses[0] {
        MCPResponse::InitializeResult { id, .. } => assert_eq!(id, "seq-1"),
        _ => panic!("Expected InitializeResult"),
    }

    match &responses[1] {
        MCPResponse::ListToolsResult { id, .. } => assert_eq!(id, "seq-2"),
        _ => panic!("Expected ListToolsResult"),
    }

    match &responses[2] {
        MCPResponse::CallToolResult { id, .. } => assert_eq!(id, "seq-3"),
        _ => panic!("Expected CallToolResult"),
    }

    match &responses[3] {
        MCPResponse::Pong { id } => assert_eq!(id, "seq-4"),
        _ => panic!("Expected Pong"),
    }
}

#[tokio::test]
async fn test_edge_cases_and_boundary_values() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Test p4_changes with max = 0
    let message = create_call_tool_message("test-edge-1", "p4_changes", json!({"max": 0}));

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-edge-1");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Changes"));
                assert!(text.contains("max: 0"));
            }
        }
        _ => panic!("Expected CallToolResult"),
    }

    // Test with very long file names
    let long_filename = "a".repeat(1000);
    let message = create_call_tool_message(
        "test-edge-2",
        "p4_edit",
        json!({"files": [long_filename.clone()]}),
    );

    let response = server.handle_message(message).await.unwrap().unwrap();

    match response {
        MCPResponse::CallToolResult { id, result } => {
            assert_eq!(id, "test-edge-2");
            let content = &result.content[0];
            if let p4_mcp::mcp::ToolContent::Text { text } = content {
                assert!(text.contains(&long_filename));
            }
        }
        _ => panic!("Expected CallToolResult"),
    }
}
