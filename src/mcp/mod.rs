use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

use crate::p4::P4Command;

pub mod types;

pub use types::*;

pub struct MCPServer {
    tools: HashMap<String, Tool>,
    p4_handler: crate::p4::P4Handler,
}

impl MCPServer {
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // Register P4 tools
        tools.insert(
            "p4_status".to_string(),
            Tool {
                name: "p4_status".to_string(),
                description: "Get Perforce workspace status".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Optional path to check status for"
                        }
                    }
                }),
            },
        );

        tools.insert(
            "p4_sync".to_string(),
            Tool {
                name: "p4_sync".to_string(),
                description: "Sync files from Perforce depot".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to sync (e.g., //depot/main/...)"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force sync (overwrite local changes)"
                        }
                    }
                }),
            },
        );

        tools.insert(
            "p4_edit".to_string(),
            Tool {
                name: "p4_edit".to_string(),
                description: "Open file(s) for edit in Perforce".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Files to open for edit"
                        }
                    },
                    "required": ["files"]
                }),
            },
        );

        tools.insert(
            "p4_add".to_string(),
            Tool {
                name: "p4_add".to_string(),
                description: "Add new file(s) to Perforce".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Files to add"
                        }
                    },
                    "required": ["files"]
                }),
            },
        );

        tools.insert(
            "p4_submit".to_string(),
            Tool {
                name: "p4_submit".to_string(),
                description: "Submit changes to Perforce".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "Change description"
                        },
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Optional specific files to submit"
                        }
                    },
                    "required": ["description"]
                }),
            },
        );

        tools.insert(
            "p4_revert".to_string(),
            Tool {
                name: "p4_revert".to_string(),
                description: "Revert files in Perforce".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "files": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Files to revert"
                        }
                    },
                    "required": ["files"]
                }),
            },
        );

        tools.insert(
            "p4_opened".to_string(),
            Tool {
                name: "p4_opened".to_string(),
                description: "List files opened for edit".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "changelist": {
                            "type": "string",
                            "description": "Optional changelist number"
                        }
                    }
                }),
            },
        );

        tools.insert(
            "p4_changes".to_string(),
            Tool {
                name: "p4_changes".to_string(),
                description: "List recent changes".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "max": {
                            "type": "integer",
                            "description": "Maximum number of changes to return",
                            "default": 10
                        },
                        "path": {
                            "type": "string",
                            "description": "Optional path to filter changes"
                        }
                    }
                }),
            },
        );

        Self {
            tools,
            p4_handler: crate::p4::P4Handler::new(),
        }
    }

    pub async fn handle_message(&mut self, message: MCPMessage) -> Result<Option<MCPResponse>> {
        debug!("Handling message: {:?}", message);

        match message {
            MCPMessage::Initialize { id, params } => {
                info!(
                    "Received initialize request with client info: {:?}",
                    params.client_info
                );

                Ok(Some(MCPResponse::InitializeResult {
                    id,
                    result: InitializeResult {
                        protocol_version: "2024-11-05".to_string(),
                        capabilities: ServerCapabilities {
                            tools: Some(ToolsCapability {
                                list_changed: false,
                            }),
                            ..Default::default()
                        },
                        server_info: ServerInfo {
                            name: "p4-mcp".to_string(),
                            version: "0.1.0".to_string(),
                        },
                    },
                }))
            }

            MCPMessage::ListTools { id } => {
                let tools: Vec<Tool> = self.tools.values().cloned().collect();

                Ok(Some(MCPResponse::ListToolsResult {
                    id,
                    result: ListToolsResult { tools },
                }))
            }

            MCPMessage::CallTool { id, params } => {
                let tool_name = &params.name;

                if !self.tools.contains_key(tool_name) {
                    return Ok(Some(MCPResponse::Error {
                        id,
                        error: MCPError {
                            code: -32602,
                            message: format!("Unknown tool: {}", tool_name),
                            data: None,
                        },
                    }));
                }

                let result = self.execute_tool(tool_name, params.arguments).await?;

                Ok(Some(MCPResponse::CallToolResult {
                    id,
                    result: CallToolResult {
                        content: vec![ToolContent::Text { text: result }],
                    },
                }))
            }

            MCPMessage::Ping { id } => Ok(Some(MCPResponse::Pong { id })),
        }
    }

    async fn execute_tool(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String> {
        debug!("Executing tool: {} with args: {}", tool_name, arguments);

        match tool_name {
            "p4_status" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.p4_handler.execute(P4Command::Status { path }).await
            }

            "p4_sync" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or("...".to_string());
                let force = arguments
                    .get("force")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                self.p4_handler
                    .execute(P4Command::Sync { path, force })
                    .await
            }

            "p4_edit" => {
                let files: Vec<String> = arguments
                    .get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                self.p4_handler.execute(P4Command::Edit { files }).await
            }

            "p4_add" => {
                let files: Vec<String> = arguments
                    .get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                self.p4_handler.execute(P4Command::Add { files }).await
            }

            "p4_submit" => {
                let description = arguments
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let files: Option<Vec<String>> = arguments
                    .get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    });
                self.p4_handler
                    .execute(P4Command::Submit { description, files })
                    .await
            }

            "p4_revert" => {
                let files: Vec<String> = arguments
                    .get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                self.p4_handler.execute(P4Command::Revert { files }).await
            }

            "p4_opened" => {
                let changelist = arguments
                    .get("changelist")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.p4_handler
                    .execute(P4Command::Opened { changelist })
                    .await
            }

            "p4_changes" => {
                let max = arguments.get("max").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.p4_handler
                    .execute(P4Command::Changes { max, path })
                    .await
            }

            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }
}
