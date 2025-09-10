//! Unit tests for MCP types and individual components

use p4_mcp::mcp::*;
use p4_mcp::p4::*;
use serde_json::json;
use std::env;

#[test]
fn test_mcp_message_deserialization() {
    // Test Initialize message
    let json_str = r#"{"method": "initialize", "id": "1", "params": {"protocolVersion": "2024-11-05", "capabilities": {"roots": {"listChanged": false}}, "clientInfo": {"name": "test", "version": "1.0"}}}"#;

    let message: MCPMessage = serde_json::from_str(json_str).unwrap();

    match message {
        MCPMessage::Initialize { id, params } => {
            assert_eq!(id, 1);
            assert_eq!(params.protocol_version, "2024-11-05");
            assert_eq!(params.client_info.name, "test");
            assert_eq!(params.client_info.version, "1.0");
        }
        _ => panic!("Expected Initialize message"),
    }
}

#[test]
fn test_list_tools_message_deserialization() {
    let json_str = r#"{"method": "tools/list", "id": "2"}"#;

    let message: MCPMessage = serde_json::from_str(json_str).unwrap();

    match message {
        MCPMessage::ListTools { id } => {
            assert_eq!(id, 2);
        }
        _ => panic!("Expected ListTools message"),
    }
}

#[test]
fn test_call_tool_message_deserialization() {
    let json_str = r#"{"method": "tools/call", "id": "3", "params": {"name": "p4_status", "arguments": {"path": "//depot/main/..."}}}"#;

    let message: MCPMessage = serde_json::from_str(json_str).unwrap();

    match message {
        MCPMessage::CallTool { id, params } => {
            assert_eq!(id, 3);
            assert_eq!(params.name, "p4_status");
            assert_eq!(params.arguments["path"], "//depot/main/...");
        }
        _ => panic!("Expected CallTool message"),
    }
}

#[test]
fn test_ping_message_deserialization() {
    let json_str = r#"{"method": "ping", "id": "ping-1"}"#;

    let message: MCPMessage = serde_json::from_str(json_str).unwrap();

    match message {
        MCPMessage::Ping { id } => {
            assert_eq!(id, 1);
        }
        _ => panic!("Expected Ping message"),
    }
}

#[test]
fn test_list_tools_response_serialization() {
    let tools = vec![
        Tool {
            name: "p4_status".to_string(),
            description: "Get Perforce workspace status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Optional path to check status for"
                    }
                }
            }),
        },
        Tool {
            name: "p4_sync".to_string(),
            description: "Sync files from Perforce depot".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to sync"
                    }
                }
            }),
        },
    ];

    let response = MCPResponse::ListToolsResult {
        id: 2,
        result: ListToolsResult { tools },
    };

    let json_str = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["id"], 2);
    assert_eq!(parsed["result"]["tools"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["result"]["tools"][0]["name"], "p4_status");
    assert_eq!(parsed["result"]["tools"][1]["name"], "p4_sync");
}

#[test]
fn test_call_tool_response_serialization() {
    let response = MCPResponse::CallToolResult {
        id: 3,
        result: CallToolResult {
            content: vec![ToolContent::Text {
                text: "Mock P4 Status result".to_string(),
            }],
        },
    };

    let json_str = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["id"], 3);
    assert_eq!(parsed["result"]["content"][0]["type"], "text");
    assert_eq!(
        parsed["result"]["content"][0]["text"],
        "Mock P4 Status result"
    );
}

#[test]
fn test_error_response_serialization() {
    let response = MCPResponse::Error {
        id: 123,
        error: MCPError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(json!({"details": "Missing required parameter"})),
        },
    };

    let json_str = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["id"], 123);
    assert_eq!(parsed["error"]["code"], -32602);
    assert_eq!(parsed["error"]["message"], "Invalid params");
    assert_eq!(
        parsed["error"]["data"]["details"],
        "Missing required parameter"
    );
}

#[test]
fn test_pong_response_serialization() {
    let response = MCPResponse::Pong { id: 456 };

    let json_str = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["id"], 456);
}

#[test]
fn test_p4_command_to_args() {
    // Test Status command
    let cmd = P4Command::Status {
        path: Some("//depot/main/...".to_string()),
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["opened", "//depot/main/..."]);

    // Test Status command without path
    let cmd = P4Command::Status { path: None };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["opened"]);

    // Test Sync command
    let cmd = P4Command::Sync {
        path: "//depot/main/...".to_string(),
        force: true,
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["sync", "-f", "//depot/main/..."]);

    // Test Edit command
    let cmd = P4Command::Edit {
        files: vec!["file1.cpp".to_string(), "file2.h".to_string()],
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["edit", "file1.cpp", "file2.h"]);

    // Test Add command
    let cmd = P4Command::Add {
        files: vec!["new_file.cpp".to_string()],
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["add", "new_file.cpp"]);

    // Test Submit command with description only
    let cmd = P4Command::Submit {
        description: "Fix bug".to_string(),
        files: None,
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["submit", "-d", "Fix bug"]);

    // Test Submit command with files
    let cmd = P4Command::Submit {
        description: "Fix bug".to_string(),
        files: Some(vec!["file1.cpp".to_string()]),
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["submit", "-d", "Fix bug", "file1.cpp"]);

    // Test Revert command
    let cmd = P4Command::Revert {
        files: vec!["file1.cpp".to_string(), "file2.h".to_string()],
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["revert", "file1.cpp", "file2.h"]);

    // Test Opened command without changelist
    let cmd = P4Command::Opened { changelist: None };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["opened"]);

    // Test Opened command with changelist
    let cmd = P4Command::Opened {
        changelist: Some("12345".to_string()),
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["opened", "-c", "12345"]);

    // Test Changes command
    let cmd = P4Command::Changes {
        max: 10,
        path: Some("//depot/main/...".to_string()),
    };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["changes", "-m", "10", "//depot/main/..."]);

    // Test Changes command without path
    let cmd = P4Command::Changes { max: 5, path: None };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["changes", "-m", "5"]);

    // Test Info command
    let cmd = P4Command::Info;
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["info"]);
}

#[tokio::test]
async fn test_p4_handler_mock_mode() {
    // Set mock mode
    env::set_var("P4_MOCK_MODE", "1");

    let mut handler = P4Handler::new();

    // Test Status command
    let result = handler
        .execute(P4Command::Status {
            path: Some("//depot/test/...".to_string()),
        })
        .await
        .unwrap();

    assert!(result.contains("Mock P4 Status"));
    assert!(result.contains("//depot/test/..."));

    // Test Sync command
    let result = handler
        .execute(P4Command::Sync {
            path: "//depot/main/...".to_string(),
            force: true,
        })
        .await
        .unwrap();

    assert!(result.contains("Mock P4 Sync"));
    assert!(result.contains("(forced)"));
    assert!(result.contains("//depot/main/..."));

    // Test Edit command
    let result = handler
        .execute(P4Command::Edit {
            files: vec!["test.cpp".to_string()],
        })
        .await
        .unwrap();

    assert!(result.contains("Mock P4 Edit"));
    assert!(result.contains("test.cpp"));
    assert!(result.contains("1 file(s) opened for edit"));

    // Test Info command
    let result = handler.execute(P4Command::Info).await.unwrap();

    assert!(result.contains("Mock P4 Info"));
    assert!(result.contains("User name: testuser"));
    assert!(result.contains("Client name: test-client"));
    assert!(result.contains("Server version:"));

    // Clean up
    env::remove_var("P4_MOCK_MODE");
}

#[test]
fn test_server_capabilities_default() {
    let capabilities = ServerCapabilities::default();

    assert!(capabilities.logging.is_none());
    assert!(capabilities.prompts.is_none());
    assert!(capabilities.resources.is_none());
    assert!(capabilities.tools.is_none());
}

#[test]
fn test_tool_content_variants() {
    // Test Text content
    let text_content = ToolContent::Text {
        text: "Sample text content".to_string(),
    };

    let json_str = serde_json::to_string(&text_content).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["type"], "text");
    assert_eq!(parsed["text"], "Sample text content");

    // Test Image content
    let image_content = ToolContent::Image {
        data: "base64-encoded-data".to_string(),
        mime_type: "image/png".to_string(),
    };

    let json_str = serde_json::to_string(&image_content).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(parsed["type"], "image");
    assert_eq!(parsed["data"], "base64-encoded-data");
    assert_eq!(parsed["mimeType"], "image/png");
}

#[test]
fn test_invalid_json_handling() {
    // Test with malformed JSON
    let invalid_json = r#"{"method": "initialize", "id": "1", "params": {"invalid"}"#;

    let result: Result<MCPMessage, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());

    // Test with missing required fields
    let incomplete_json = r#"{"method": "initialize"}"#;

    let result: Result<MCPMessage, _> = serde_json::from_str(incomplete_json);
    assert!(result.is_err());

    // Test with unknown method
    let unknown_method_json = r#"{"method": "unknown", "id": "1"}"#;

    let result: Result<MCPMessage, _> = serde_json::from_str(unknown_method_json);
    assert!(result.is_err());
}

#[test]
fn test_large_data_handling() {
    // Test with large file list
    let large_file_list: Vec<String> = (0..1000).map(|i| format!("file{}.cpp", i)).collect();

    let cmd = P4Command::Edit {
        files: large_file_list.clone(),
    };

    let (_, args) = cmd.to_command_args();
    assert_eq!(args.len(), 1001); // "edit" + 1000 files
    assert_eq!(args[0], "edit");
    assert_eq!(args[1], "file0.cpp");
    assert_eq!(args[1000], "file999.cpp");

    // Test with very long description
    let long_description = "x".repeat(10000);
    let cmd = P4Command::Submit {
        description: long_description.clone(),
        files: None,
    };

    let (_, args) = cmd.to_command_args();
    assert_eq!(args[2], long_description);
}

#[test]
fn test_special_characters_in_paths() {
    // Test with special characters in file paths
    let special_files = vec![
        "file with spaces.cpp".to_string(),
        "file-with-dashes.cpp".to_string(),
        "file_with_underscores.cpp".to_string(),
        "file.with.dots.cpp".to_string(),
        "file@with@symbols.cpp".to_string(),
    ];

    let cmd = P4Command::Add {
        files: special_files.clone(),
    };

    let (_, args) = cmd.to_command_args();
    assert_eq!(args.len(), 6); // "add" + 5 files

    for (i, expected_file) in special_files.iter().enumerate() {
        assert_eq!(args[i + 1], *expected_file);
    }
}

#[test]
fn test_empty_collections() {
    // Test with empty files array
    let cmd = P4Command::Edit { files: vec![] };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["edit"]);

    // Test with empty changelist
    let cmd = P4Command::Opened { changelist: None };
    let (_, args) = cmd.to_command_args();
    assert_eq!(args, vec!["opened"]);
}

#[tokio::test]
async fn test_p4_handler_creation() {
    // Test default creation
    let handler = P4Handler::default();
    // Should not panic and should create a valid handler

    // Test new creation
    let handler = P4Handler::new();
    // Should create the same as default
}

#[test]
fn test_mcp_server_initialization() {
    // Test that MCPServer can be created
    let server = MCPServer::new();
    // Should create server with all expected tools registered
    // The actual tool validation is covered in integration tests
}
