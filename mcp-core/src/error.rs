//! Error types for the MCP protocol implementation.
//! 
//! This module defines error types that correspond to the JSON-RPC 2.0 error codes
//! used by the Model Context Protocol, as well as custom error types for MCP-specific
//! error conditions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias for MCP operations.
pub type McpResult<T> = Result<T, McpError>;

/// MCP-specific error types following the JSON-RPC 2.0 error specification.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum McpError {
    /// JSON-RPC 2.0 parse error (-32700)
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    /// JSON-RPC 2.0 invalid request (-32600)
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    
    /// JSON-RPC 2.0 method not found (-32601)
    #[error("Method not found: {method}")]
    MethodNotFound { method: String },
    
    /// JSON-RPC 2.0 invalid params (-32602)
    #[error("Invalid params: {message}")]
    InvalidParams { message: String },
    
    /// JSON-RPC 2.0 internal error (-32603)
    #[error("Internal error: {message}")]
    InternalError { message: String },
    
    /// MCP-specific tool not found error
    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },
    
    /// MCP-specific tool execution error
    #[error("Tool execution failed: {tool_name} - {message}")]
    ToolExecutionError { tool_name: String, message: String },
    
    /// Transport-level error
    #[error("Transport error: {message}")]
    TransportError { message: String },
    
    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}

impl McpError {
    /// Convert the MCP error to a JSON-RPC 2.0 error code.
    pub fn to_json_rpc_code(&self) -> i32 {
        match self {
            McpError::ParseError { .. } => -32700,
            McpError::InvalidRequest { .. } => -32600,
            McpError::MethodNotFound { .. } => -32601,
            McpError::InvalidParams { .. } => -32602,
            McpError::InternalError { .. } => -32603,
            McpError::ToolNotFound { .. } => -32000, // Server-defined error
            McpError::ToolExecutionError { .. } => -32001, // Server-defined error
            McpError::TransportError { .. } => -32002, // Server-defined error
            McpError::SerializationError { .. } => -32003, // Server-defined error
        }
    }
    
    /// Create a new parse error.
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError { message: message.into() }
    }
    
    /// Create a new invalid request error.
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest { message: message.into() }
    }
    
    /// Create a new method not found error.
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::MethodNotFound { method: method.into() }
    }
    
    /// Create a new invalid params error.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::InvalidParams { message: message.into() }
    }
    
    /// Create a new internal error.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError { message: message.into() }
    }
    
    /// Create a new tool not found error.
    pub fn tool_not_found(tool_name: impl Into<String>) -> Self {
        Self::ToolNotFound { tool_name: tool_name.into() }
    }
    
    /// Create a new tool execution error.
    pub fn tool_execution_error(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ToolExecutionError { 
            tool_name: tool_name.into(), 
            message: message.into() 
        }
    }
    
    /// Create a new transport error.
    pub fn transport_error(message: impl Into<String>) -> Self {
        Self::TransportError { message: message.into() }
    }
    
    /// Create a new serialization error.
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError { message: message.into() }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::serialization_error(err.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::transport_error(err.to_string())
    }
}
