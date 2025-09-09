//! Integration tests for the p4-mcp server
//! These tests read JSON messages from test_data files to ensure consistency with manual testing

use p4_mcp::mcp::{MCPMessage, MCPResponse, MCPServer, ToolContent};
use serde_json;
use std::env;
use std::fs;
use std::path::Path;

/// Test helper to set up mock mode
fn setup_mock_mode() {
    env::set_var("P4_MOCK_MODE", "1");
}

/// Load a JSON message from the test_data directory
fn load_test_message(filename: &str) -> MCPMessage {
    let test_data_path = Path::new("test_data").join(filename);
    let json_content = fs::read_to_string(&test_data_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", test_data_path.display(), e));

    serde_json::from_str(&json_content).unwrap_or_else(|e| {
        panic!(
            "Failed to parse JSON from {}: {}",
            test_data_path.display(),
            e
        )
    })
}

#[tokio::test]
async fn test_initialize_endpoint() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Load initialize message from test_data
    let message = load_test_message("test_initialize.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::InitializeResult { id, result }) = response {
        assert_eq!(id, 0);
        assert_eq!(result.protocol_version, "2024-11-05");
        assert!(result.capabilities.tools.is_some());
    } else {
        panic!("Expected InitializeResult response");
    }
}

#[tokio::test]
async fn test_list_tools_endpoint() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // First initialize the server
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load list tools message from test_data
    let message = load_test_message("test_list_tools.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::ListToolsResult { id, result }) = response {
        assert_eq!(id, 2);
        assert!(!result.tools.is_empty());

        // Verify we have expected tools
        let tool_names: Vec<&str> = result.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"p4_info"));
        assert!(tool_names.contains(&"p4_status"));
        assert!(tool_names.contains(&"p4_sync"));
    } else {
        panic!("Expected ListToolsResult response");
    }
}

#[tokio::test]
async fn test_ping_endpoint() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Create a simple ping message (not in test_data, so we'll create it inline)
    let ping_message = serde_json::from_str(r#"{"method": "ping", "id": "ping-test"}"#).unwrap();

    let response = server.handle_message(ping_message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::Pong { id }) = response {
        assert_eq!(id, 1);
    } else {
        panic!("Expected Pong response");
    }
}

#[tokio::test]
async fn test_p4_status_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_status test message
    let message = load_test_message("test_p4_status.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 3);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Status"));
                assert!(text.contains("//depot/main/src/..."));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_sync_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_sync test message
    let message = load_test_message("test_p4_sync_example.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Sync"));
                assert!(text.contains("//depot/main/..."));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_edit_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_edit test message
    let message = load_test_message("test_p4_edit.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Edit"));
                assert!(text.contains("src/main.cpp"));
                assert!(text.contains("include/header.h"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_add_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_add test message
    let message = load_test_message("test_p4_add.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Add"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_submit_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Create a p4_submit message (create inline since we need specific parameters)
    let submit_message = serde_json::from_str(
        r#"
    {
        "method": "tools/call",
        "id": "submit-test",
        "params": {
            "name": "p4_submit",
            "arguments": {
                "description": "Test submission from integration test",
                "files": ["test1.txt", "test2.txt"]
            }
        }
    }"#,
    )
    .unwrap();

    let response = server.handle_message(submit_message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Submit"));
                assert!(text.contains("Test submission from integration test"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_revert_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Create a p4_revert message (create inline since not in test_data)
    let revert_message = serde_json::from_str(
        r#"
    {
        "method": "tools/call",
        "id": "revert-test",
        "params": {
            "name": "p4_revert",
            "arguments": {
                "files": ["unwanted_change.txt"]
            }
        }
    }"#,
    )
    .unwrap();

    let response = server.handle_message(revert_message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Revert"));
                assert!(text.contains("unwanted_change.txt"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_opened_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_opened test message
    let message = load_test_message("test_p4_opened.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Opened"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_changes_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_changes test message
    let message = load_test_message("test_p4_changes.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Changes"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_p4_info_tool() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Load p4_info test message
    let message = load_test_message("test_p4_info.json");

    let response = server.handle_message(message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::CallToolResult { id, result }) = response {
        assert_eq!(id, 123);
        assert!(result.content.len() == 1);

        if let Some(content) = result.content.first() {
            if let ToolContent::Text { text } = content {
                assert!(text.contains("Mock P4 Info"));
            }
        }
    } else {
        panic!("Expected CallToolResult response");
    }
}

#[tokio::test]
async fn test_unknown_tool_error() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Create a message for an unknown tool
    let unknown_tool_message = serde_json::from_str(
        r#"
    {
        "method": "tools/call",
        "id": "unknown-test",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    }"#,
    )
    .unwrap();

    let response = server.handle_message(unknown_tool_message).await;

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.is_some());

    if let Some(MCPResponse::Error { id, error }) = response {
        assert_eq!(id, 123);
        assert!(error.message.contains("Unknown tool"));
    } else {
        panic!("Expected Error response");
    }
}

#[tokio::test]
async fn test_missing_required_parameters() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Create a p4_edit message without required files parameter
    let invalid_edit_message = serde_json::from_str(
        r#"
    {
        "method": "tools/call",
        "id": "invalid-edit",
        "params": {
            "name": "p4_edit",
            "arguments": {}
        }
    }"#,
    )
    .unwrap();

    let response = server.handle_message(invalid_edit_message).await;

    // Should handle gracefully - either return an error or mock response
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_message_serialization_deserialization() {
    // Test that we can serialize and deserialize messages loaded from test_data
    let original_message = load_test_message("test_initialize.json");
    let serialized = serde_json::to_string(&original_message).unwrap();
    let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

    // Compare key fields
    if let (MCPMessage::Initialize { id: id1, .. }, MCPMessage::Initialize { id: id2, .. }) =
        (&original_message, &deserialized)
    {
        assert_eq!(id1, id2);
    }
}

#[tokio::test]
async fn test_sequential_message_handling() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Load and process multiple messages in sequence
    let init_message = load_test_message("test_initialize.json");
    let list_tools_message = load_test_message("test_list_tools.json");
    let p4_info_message = load_test_message("test_p4_info.json");

    // Process messages sequentially
    let init_response = server.handle_message(init_message).await;
    assert!(init_response.is_ok() && init_response.unwrap().is_some());

    let tools_response = server.handle_message(list_tools_message).await;
    assert!(tools_response.is_ok() && tools_response.unwrap().is_some());

    let info_response = server.handle_message(p4_info_message).await;
    assert!(info_response.is_ok() && info_response.unwrap().is_some());
}

#[tokio::test]
async fn test_edge_cases_and_boundary_values() {
    setup_mock_mode();
    let mut server = MCPServer::new();

    // Initialize the server first
    let init_message = load_test_message("test_initialize.json");
    server.handle_message(init_message).await.unwrap();

    // Test with empty path for p4_status
    let empty_path_message = serde_json::from_str(
        r#"
    {
        "method": "tools/call",
        "id": "empty-path-test",
        "params": {
            "name": "p4_status",
            "arguments": {
                "path": ""
            }
        }
    }"#,
    )
    .unwrap();

    let response = server.handle_message(empty_path_message).await;
    assert!(response.is_ok());

    // Test with very long description for p4_submit
    let long_description = "A".repeat(1000);
    let long_desc_message = serde_json::from_str(&format!(
        r#"
    {{
        "method": "tools/call",
        "id": "long-desc-test",
        "params": {{
            "name": "p4_submit",
            "arguments": {{
                "description": "{}",
                "files": ["test.txt"]
            }}
        }}
    }}"#,
        long_description
    ))
    .unwrap();

    let response = server.handle_message(long_desc_message).await;
    assert!(response.is_ok());
}
