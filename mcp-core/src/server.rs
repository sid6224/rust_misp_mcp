//! MCP server implementation.
//!
//! This module provides the main `Server` struct that coordinates all MCP
//! functionality, including protocol handling, tool management, and transport
//! communication. It implements the complete MCP server lifecycle from
//! initialization through tool execution.

use crate::error::{McpError, McpResult};
use crate::protocol::{
    CallToolParams, Implementation, InitializeParams, InitializeResult,
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsParams, ListToolsResult,
    ServerCapabilities, ToolsCapability,
};
use crate::registry::{Tool, ToolRegistry};
use crate::transport::{StdioTransport, Transport};
use serde_json::Value;
use tracing::{debug, error, info, warn};

/// MCP server state tracking.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    /// Server is created but not yet initialized.
    Created,
    /// Server has been initialized and is ready to handle requests.
    Initialized,
    /// Server is shutting down.
    Shutdown,
}

/// Main MCP server implementation.
/// 
/// The server handles the complete MCP protocol lifecycle:
/// 1. Initialization handshake with the client
/// 2. Tool registration and management
/// 3. Request processing and response generation
/// 4. Error handling and logging
/// 
/// # Example
/// 
/// ```rust,no_run
/// use mcp_core::{Server, Tool, ToolInput, ToolResult};
/// 
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut server = Server::new("my-server", "1.0.0");
///     
///     server.add_tool(Tool::new(
///         "greet",
///         "Greets a user by name",
///         |input: ToolInput| async move {
///             let name: String = input.get_argument("name")?;
///             Ok(ToolResult::text(format!("Hello, {}!", name)))
///         }
///     ));
///     
///     server.run_stdio().await
/// }
/// ```
pub struct Server {
    /// Server implementation information.
    server_info: Implementation,
    /// Current server state.
    state: ServerState,
    /// Tool registry for managing available tools.
    tool_registry: ToolRegistry,
    /// Server capabilities advertised to clients.
    capabilities: ServerCapabilities,
}

impl Server {
    /// Create a new MCP server with the given name and version.
    /// 
    /// The server starts in the `Created` state and must be initialized
    /// before it can process tool requests.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        let server_info = Implementation {
            name: name.into(),
            version: version.into(),
        };
        
        info!("Creating MCP server: {} v{}", server_info.name, server_info.version);
        
        Self {
            server_info,
            state: ServerState::Created,
            tool_registry: ToolRegistry::new(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability::default()),
                ..Default::default()
            },
        }
    }
    
    /// Add a tool to the server.
    /// 
    /// Tools can be added before or after initialization. If added after
    /// initialization, clients may need to be notified of the tool list
    /// change (if they support the `listChanged` capability).
    pub fn add_tool(&mut self, tool: Tool) {
        self.tool_registry.register(tool);
    }
    
    /// Get the current server state.
    pub fn state(&self) -> ServerState {
        self.state.clone()
    }
    
    /// Get the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.tool_registry.len()
    }
    
    /// Run the server using stdio transport.
    /// 
    /// This is the most common way to run an MCP server, reading JSON-RPC
    /// messages from stdin and writing responses to stdout.
    pub async fn run_stdio(&mut self) -> McpResult<()> {
        let mut transport = StdioTransport::new();
        self.run_with_transport(&mut transport).await
    }
    
    /// Run the server with a custom transport.
    /// 
    /// This allows for using alternative transport mechanisms such as
    /// named pipes, sockets, or testing harnesses.
    pub async fn run_with_transport(&mut self, transport: &mut dyn Transport) -> McpResult<()> {
        info!("Starting MCP server: {} v{}", self.server_info.name, self.server_info.version);
        
        loop {
            match self.handle_next_request(transport).await {
                Ok(should_continue) => {
                    if !should_continue {
                        info!("Server shutting down");
                        break;
                    }
                }
                Err(e) => {
                    // Check if this is a normal client disconnection (EOF)
                    if let McpError::TransportError { message } = &e {
                        if message.contains("EOF reached") {
                            info!("Client disconnected");
                            break;
                        }
                    }
                    
                    error!("Error handling request: {}", e);
                    // Continue processing other requests unless it's a transport error
                    if matches!(e, McpError::TransportError { .. }) {
                        error!("Transport error, shutting down server");
                        break;
                    }
                }
            }
        }
        
        self.state = ServerState::Shutdown;
        transport.close().await?;
        Ok(())
    }
    
    /// Handle the next request from the transport.
    /// 
    /// Returns `Ok(true)` if the server should continue processing requests,
    /// or `Ok(false)` if the server should shut down.
    async fn handle_next_request(&mut self, transport: &mut dyn Transport) -> McpResult<bool> {
        let request = transport.read_message().await?;
        debug!("Processing request: method={}, id={:?}", request.method, request.id);
        
        let response = match self.process_request(request.clone()).await {
            Ok(response) => response,
            Err(e) => {
                warn!("Request processing failed: {}", e);
                self.create_error_response(request.id, e)
            }
        };
        
        transport.write_response(response).await?;
        Ok(true)
    }
    
    /// Process a JSON-RPC request and generate a response.
    async fn process_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            _ => {
                Err(McpError::method_not_found(&request.method))
            }
        }
    }
    
    /// Handle the initialize request.
    async fn handle_initialize(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if self.state != ServerState::Created {
            return Err(McpError::invalid_request("Server already initialized"));
        }
        
        let params: InitializeParams = match request.params {
            Some(params) => serde_json::from_value(params)?,
            None => return Err(McpError::invalid_params("Missing initialization parameters")),
        };
        
        info!("Initializing with client: {} v{}", 
              params.client_info.name, params.client_info.version);
        debug!("Protocol version: {}", params.protocol_version);
        
        // Validate protocol version (simplified - accept any version for now)
        if params.protocol_version.is_empty() {
            return Err(McpError::invalid_params("Protocol version is required"));
        }
        
        self.state = ServerState::Initialized;
        
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(), // Latest MCP protocol version
            server_info: self.server_info.clone(),
            capabilities: self.capabilities.clone(),
        };
        
        JsonRpcResponse::success(request.id, result).map_err(McpError::from)
    }
    
    /// Handle the tools/list request.
    async fn handle_list_tools(&self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if self.state != ServerState::Initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }
        
        // Parse params (should be empty for tools/list)
        let _params: ListToolsParams = match request.params {
            Some(params) => serde_json::from_value(params)?,
            None => ListToolsParams::default(),
        };
        
        let tools = self.tool_registry.list_tools();
        info!("Listing {} available tools", tools.len());
        debug!("Tools: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>());
        
        let result = ListToolsResult { tools };
        JsonRpcResponse::success(request.id, result).map_err(McpError::from)
    }
    
    /// Handle the tools/call request.
    async fn handle_call_tool(&self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if self.state != ServerState::Initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }
        
        let params: CallToolParams = match request.params {
            Some(params) => serde_json::from_value(params)?,
            None => return Err(McpError::invalid_params("Missing tool call parameters")),
        };
        
        info!("Calling tool: {}", params.name);
        debug!("Tool arguments: {:?}", params.arguments);
        
        let arguments = params.arguments.unwrap_or_default();
        let tool_result = self.tool_registry.execute_tool(&params.name, arguments).await?;
        let call_result = tool_result.into_call_result();
        
        JsonRpcResponse::success(request.id, call_result).map_err(McpError::from)
    }
    
    /// Create an error response for a failed request.
    fn create_error_response(&self, request_id: Option<Value>, error: McpError) -> JsonRpcResponse {
        let json_rpc_error = JsonRpcError::new(error.to_json_rpc_code(), error.to_string());
        JsonRpcResponse::error(request_id, json_rpc_error)
    }
}
