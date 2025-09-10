//! Transport layer implementations for MCP communication.
//!
//! This module provides different transport mechanisms for MCP servers,
//! including stdio (standard input/output) and named pipes. All transports
//! implement the `Transport` trait for consistent message handling.

use crate::error::{McpError, McpResult};
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

/// Trait for MCP transport implementations.
/// 
/// A transport handles the low-level communication between the MCP server
/// and client, including message serialization, deserialization, and
/// delivery over the chosen communication channel.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Read the next JSON-RPC message from the transport.
    async fn read_message(&mut self) -> McpResult<JsonRpcRequest>;
    
    /// Write a JSON-RPC response to the transport.
    async fn write_response(&mut self, response: JsonRpcResponse) -> McpResult<()>;
    
    /// Close the transport and clean up resources.
    async fn close(&mut self) -> McpResult<()>;
}

/// Stdio transport implementation using standard input and output.
/// 
/// This transport reads JSON-RPC messages from stdin and writes responses
/// to stdout. Each message is expected to be on a single line (JSON Lines format).
pub struct StdioTransport {
    stdin_reader: TokioBufReader<tokio::io::Stdin>,
    stdout: tokio::io::Stdout,
}

impl StdioTransport {
    /// Create a new stdio transport.
    pub fn new() -> Self {
        info!("Initializing stdio transport");
        Self {
            stdin_reader: TokioBufReader::new(tokio::io::stdin()),
            stdout: tokio::io::stdout(),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transport for StdioTransport {
    async fn read_message(&mut self) -> McpResult<JsonRpcRequest> {
        let mut line = String::new();
        
        match self.stdin_reader.read_line(&mut line).await {
            Ok(0) => {
                debug!("EOF reached on stdin");
                return Err(McpError::transport_error("EOF reached"));
            }
            Ok(_) => {
                trace!("Read line from stdin: {}", line.trim());
            }
            Err(e) => {
                error!("Failed to read from stdin: {}", e);
                return Err(McpError::transport_error(format!("Failed to read from stdin: {}", e)));
            }
        }
        
        // Trim whitespace and parse JSON
        let line = line.trim();
        if line.is_empty() {
            warn!("Received empty line, skipping");
            return self.read_message().await; // Recursively try again
        }
        
        match serde_json::from_str::<JsonRpcRequest>(line) {
            Ok(request) => {
                debug!("Parsed JSON-RPC request: method={}, id={:?}", request.method, request.id);
                Ok(request)
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request from line '{}': {}", line, e);
                Err(McpError::parse_error(format!("Invalid JSON-RPC request: {}", e)))
            }
        }
    }
    
    async fn write_response(&mut self, response: JsonRpcResponse) -> McpResult<()> {
        let json = match serde_json::to_string(&response) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize response: {}", e);
                return Err(McpError::serialization_error(format!("Failed to serialize response: {}", e)));
            }
        };
        
        debug!("Writing JSON-RPC response: id={:?}, error={:?}", response.id, response.error.is_some());
        trace!("Response JSON: {}", json);
        
        // Write JSON + newline to stdout
        match self.stdout.write_all(format!("{}\n", json).as_bytes()).await {
            Ok(_) => {
                if let Err(e) = self.stdout.flush().await {
                    error!("Failed to flush stdout: {}", e);
                    return Err(McpError::transport_error(format!("Failed to flush stdout: {}", e)));
                }
                trace!("Successfully wrote response to stdout");
                Ok(())
            }
            Err(e) => {
                error!("Failed to write to stdout: {}", e);
                Err(McpError::transport_error(format!("Failed to write to stdout: {}", e)))
            }
        }
    }
    
    async fn close(&mut self) -> McpResult<()> {
        info!("Closing stdio transport");
        if let Err(e) = self.stdout.flush().await {
            warn!("Failed to flush stdout on close: {}", e);
        }
        Ok(())
    }
}

/// Channel-based transport for testing and custom implementations.
/// 
/// This transport uses Tokio channels for communication, making it useful
/// for testing and scenarios where you need to control message flow
/// programmatically.
pub struct ChannelTransport {
    request_receiver: mpsc::UnboundedReceiver<JsonRpcRequest>,
    response_sender: mpsc::UnboundedSender<JsonRpcResponse>,
}

impl ChannelTransport {
    /// Create a new channel transport.
    /// 
    /// Returns the transport and the sender/receiver pair for controlling
    /// the message flow from the other side.
    pub fn new() -> (
        Self,
        mpsc::UnboundedSender<JsonRpcRequest>,
        mpsc::UnboundedReceiver<JsonRpcResponse>,
    ) {
        let (request_sender, request_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();
        
        let transport = Self {
            request_receiver,
            response_sender,
        };
        
        (transport, request_sender, response_receiver)
    }
}

#[async_trait::async_trait]
impl Transport for ChannelTransport {
    async fn read_message(&mut self) -> McpResult<JsonRpcRequest> {
        match self.request_receiver.recv().await {
            Some(request) => {
                debug!("Received request via channel: method={}", request.method);
                Ok(request)
            }
            None => {
                debug!("Request channel closed");
                Err(McpError::transport_error("Request channel closed"))
            }
        }
    }
    
    async fn write_response(&mut self, response: JsonRpcResponse) -> McpResult<()> {
        match self.response_sender.send(response) {
            Ok(_) => {
                trace!("Sent response via channel");
                Ok(())
            }
            Err(_) => {
                error!("Response channel closed");
                Err(McpError::transport_error("Response channel closed"))
            }
        }
    }
    
    async fn close(&mut self) -> McpResult<()> {
        debug!("Closing channel transport");
        // Channels will be closed when dropped
        Ok(())
    }
}
