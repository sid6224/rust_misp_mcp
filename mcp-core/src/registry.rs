//! Tool registry and execution system for MCP servers.
//!
//! This module provides the core tool management functionality, including:
//! - Tool definition and registration
//! - Tool input validation and parsing
//! - Tool execution with async support
//! - Result formatting and error handling

use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, ToolContent, ToolDefinition, ToolInputSchema};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Type alias for tool handler functions.
/// 
/// Tool handlers are async functions that take a `ToolInput` and return a `ToolResult`.
/// They are boxed to allow for dynamic dispatch and stored in the registry.
pub type ToolHandler = Arc<
    dyn Fn(ToolInput) -> Pin<Box<dyn Future<Output = McpResult<ToolResult>> + Send>> + Send + Sync
>;

/// Input parameters passed to tool handlers.
#[derive(Debug, Clone)]
pub struct ToolInput {
    /// The name of the tool being invoked.
    pub name: String,
    /// The raw arguments passed to the tool as a JSON object.
    pub arguments: HashMap<String, Value>,
}

/// Result returned by tool handlers.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// The content returned by the tool.
    pub content: Vec<ToolContent>,
    /// Whether this result represents an error condition.
    pub is_error: bool,
}

/// A registered tool with its metadata and handler.
#[derive(Clone)]
pub struct Tool {
    /// The tool definition (name, description, schema).
    pub definition: ToolDefinition,
    /// The handler function for this tool.
    pub handler: ToolHandler,
}

/// Registry for managing MCP tools.
/// 
/// The tool registry maintains a collection of available tools and provides
/// methods for registration, lookup, and execution. It is thread-safe and
/// can be shared across async tasks.
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolInput {
    /// Create a new tool input.
    pub fn new(name: impl Into<String>, arguments: HashMap<String, Value>) -> Self {
        Self {
            name: name.into(),
            arguments,
        }
    }
    
    /// Get a typed argument from the input parameters.
    /// 
    /// This method attempts to deserialize the specified argument into the
    /// requested type. It returns an error if the argument is missing or
    /// cannot be deserialized.
    pub fn get_argument<T>(&self, key: &str) -> McpResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let value = self.arguments
            .get(key)
            .ok_or_else(|| McpError::invalid_params(format!("Missing required argument: {}", key)))?;
        
        serde_json::from_value(value.clone())
            .map_err(|e| McpError::invalid_params(format!("Invalid argument '{}': {}", key, e)))
    }
    
    /// Get an optional typed argument from the input parameters.
    /// 
    /// This method attempts to deserialize the specified argument into the
    /// requested type, returning `None` if the argument is missing.
    pub fn get_optional_argument<T>(&self, key: &str) -> McpResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.arguments.get(key) {
            Some(value) => {
                let result = serde_json::from_value(value.clone())
                    .map_err(|e| McpError::invalid_params(format!("Invalid argument '{}': {}", key, e)))?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
    
    /// Get all arguments as a typed struct.
    /// 
    /// This method attempts to deserialize all arguments into a single struct.
    /// The struct should have field names matching the argument keys.
    pub fn deserialize_arguments<T>(&self) -> McpResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(serde_json::to_value(&self.arguments)?)
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e)))
    }
}

impl ToolResult {
    /// Create a new tool result with content.
    pub fn new(content: Vec<ToolContent>) -> Self {
        Self {
            content,
            is_error: false,
        }
    }
    
    /// Create a successful text result.
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(vec![ToolContent::text(text)])
    }
    
    /// Create a successful image result.
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::new(vec![ToolContent::image(data, mime_type)])
    }
    
    /// Create a successful resource reference result.
    pub fn resource(uri: impl Into<String>) -> Self {
        Self::new(vec![ToolContent::resource(uri)])
    }
    
    /// Create an error result with a text message.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(message)],
            is_error: true,
        }
    }
    
    /// Create an empty successful result.
    pub fn empty() -> Self {
        Self::new(vec![])
    }
    
    /// Convert this result into a `CallToolResult` for the MCP protocol.
    pub fn into_call_result(self) -> CallToolResult {
        CallToolResult {
            content: self.content,
            is_error: if self.is_error { Some(true) } else { None },
        }
    }
}

impl Tool {
    /// Create a new tool with a simple handler function.
    /// 
    /// The handler is a function that takes `ToolInput` and returns a future
    /// that resolves to a `ToolResult`.
    pub fn new<F, Fut>(name: impl Into<String>, description: impl Into<String>, handler: F) -> Self
    where
        F: Fn(ToolInput) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = McpResult<ToolResult>> + Send + 'static,
    {
        let name = name.into();
        let definition = ToolDefinition {
            name: name.clone(),
            description: description.into(),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::new()),
                required: vec![],
                additional_properties: Some(true),
            },
        };
        
        let handler = Arc::new(move |input: ToolInput| {
            Box::pin(handler(input)) as Pin<Box<dyn Future<Output = McpResult<ToolResult>> + Send>>
        });
        
        Self { definition, handler }
    }
    
    /// Create a new tool with a detailed schema.
    /// 
    /// This allows for more precise input validation by specifying the expected
    /// parameter types and requirements in the JSON schema.
    pub fn with_schema<F, Fut>(
        name: impl Into<String>, 
        description: impl Into<String>, 
        input_schema: ToolInputSchema,
        handler: F
    ) -> Self
    where
        F: Fn(ToolInput) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = McpResult<ToolResult>> + Send + 'static,
    {
        let definition = ToolDefinition {
            name: name.into(),
            description: description.into(),
            input_schema,
        };
        
        let handler = Arc::new(move |input: ToolInput| {
            Box::pin(handler(input)) as Pin<Box<dyn Future<Output = McpResult<ToolResult>> + Send>>
        });
        
        Self { definition, handler }
    }
    
    /// Execute the tool with the given input.
    pub async fn execute(&self, input: ToolInput) -> McpResult<ToolResult> {
        debug!("Executing tool '{}' with arguments: {:?}", input.name, input.arguments);
        
        let start_time = std::time::Instant::now();
        let result = (self.handler)(input.clone()).await;
        let duration = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                info!("Tool '{}' completed successfully in {:?}", input.name, duration);
            }
            Err(e) => {
                error!("Tool '{}' failed after {:?}: {}", input.name, duration, e);
            }
        }
        
        result
    }
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a tool in the registry.
    /// 
    /// If a tool with the same name already exists, it will be replaced
    /// and a warning will be logged.
    pub fn register(&mut self, tool: Tool) {
        let name = tool.definition.name.clone();
        
        if self.tools.contains_key(&name) {
            warn!("Replacing existing tool: {}", name);
        }
        
        info!("Registered tool: {} - {}", name, tool.definition.description);
        self.tools.insert(name, tool);
    }
    
    /// Get a list of all registered tool definitions.
    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|tool| tool.definition.clone()).collect()
    }
    
    /// Get a tool by name.
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }
    
    /// Execute a tool by name with the given arguments.
    /// 
    /// This method looks up the tool, creates a `ToolInput`, and executes
    /// the tool's handler. It returns appropriate errors if the tool is
    /// not found or execution fails.
    pub async fn execute_tool(&self, name: &str, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let tool = self.get_tool(name)
            .ok_or_else(|| McpError::tool_not_found(name))?;
        
        let input = ToolInput::new(name, arguments);
        
        match tool.execute(input).await {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Tool execution failed for '{}': {}", name, e);
                Err(McpError::tool_execution_error(name, e.to_string()))
            }
        }
    }
    
    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }
    
    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
    
    /// Get the names of all registered tools.
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}
