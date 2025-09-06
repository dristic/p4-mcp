//! p4-mcp: A Perforce Model Context Protocol server
//!
//! This library provides a server that implements the Model Context Protocol (MCP)
//! to interact with Perforce version control system. It supports both real Perforce
//! operations and mock mode for testing.

pub mod mcp;
pub mod p4;

pub use mcp::{MCPMessage, MCPResponse, MCPServer};
pub use p4::{P4Command, P4Handler};
