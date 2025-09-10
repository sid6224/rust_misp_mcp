//! # mcp-core
//!
//! A Rust library implementing the Anthropic Model Context Protocol (MCP).
//! 
//! This library provides a complete implementation of the MCP protocol over JSON-RPC 2.0,
//! supporting both stdio and named pipe transports. It includes:
//! 
//! - Complete MCP protocol message types and serialization
//! - Tool registry and invocation system
//! - Error handling following MCP specification
//! - Logging and tracing integration
//! - Async runtime support with Tokio
//!
//! ## Example
//!
//! ```rust,no_run
//! use mcp_core::{Server, Tool, ToolInput, ToolResult};
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut server = Server::new("my-mcp-server", "1.0.0");
//!     
//!     server.add_tool(Tool::new(
//!         "hello",
//!         "Says hello",
//!         |_input: ToolInput| async move {
//!             Ok(ToolResult::text("Hello, world!"))
//!         }
//!     ));
//!     
//!     server.run_stdio().await
//! }
//! ```

pub mod error;
pub mod protocol;
pub mod registry;
pub mod server;
pub mod transport;

pub use error::{McpError, McpResult};
pub use protocol::*;
pub use registry::{Tool, ToolInput, ToolRegistry, ToolResult};
pub use server::Server;
pub use transport::{StdioTransport, Transport};
