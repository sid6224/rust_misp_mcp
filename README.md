# MISP MCP Server - Rust Implementation

A comprehensive Model Context Protocol (MCP) server implementation for MISP (Malware Information Sharing Platform) written in Rust. This implementation provides comprehensive coverage of MISP operations through 39 validated tools, structured as a reusable library architecture.

NOTE: The API endpoints' request/response schema were desgined from the official MISP API documentation version 2.4. The testing with a MISP 2.5.20 local version revealed some changes in request/response schema for some of the endpoints. Efforts have been made to be as future proof and inclusive of those changes as possible. 

## Architecture

This project is organized into three crates:

### `mcp-core` - Reusable MCP Library
- **Purpose**: Complete implementation of the Anthropic Model Context Protocol
- **Features**: 
  - JSON-RPC 2.0 over stdio/pipes transport
  - Tool registry and execution system
  - Protocol message types and serialization
  - Comprehensive error handling and logging
  - Async runtime support with Tokio
- **Reusable**: Can be used for any MCP server implementation

### `misp-types` - MISP Type Definitions
- **Purpose**: Strongly-typed Rust structs for MISP API
- **Features**:
  - Complete type coverage for all MISP entities (Event, Attribute, User, Galaxy, Tag, etc.)
  - Comprehensive request/response structures for 40 API endpoints
  - Serde serialization/deserialization with proper field mappings
  - Preserves exact MISP JSON field names and structure
  - Support for optional fields and polymorphic responses
  - Request wrappers for search operations and filters

### `misp-mcp` - MISP MCP Server Binary  
- **Purpose**: MCP server application for MISP integration
- **Features**:
  - HTTP client for MISP API interaction with comprehensive endpoint coverage
  - 40 tool implementations covering all major MISP operations
  - Advanced search capabilities with REST API filter support
  - Configuration via environment variables or CLI arguments
  - Comprehensive logging and error handling with structured output
  - Type-safe request/response handling for all supported endpoints

## Validated Tools

The server provides comprehensive coverage of MISP operations through 39 validated tools organized by functional area:

### User Management
- `get_users`: Retrieve all users from MISP
- `get_user`: Get a specific user by ID

### Galaxy Management 
- `get_galaxies`: Retrieve all galaxies from MISP
- `get_galaxy`: Get a specific galaxy by ID
- `search_galaxies`: Search galaxies with filters
- `get_galaxy_clusters`: Retrieve all galaxy clusters
- `get_galaxy_cluster_by_id`: Get a specific galaxy cluster by ID
- `search_galaxy_clusters`: Search galaxy clusters with filters

### Organization Management
- `get_organisations`: Retrieve all organizations from MISP
- `get_organisation_by_id`: Get a specific organization by ID

### Tag and Taxonomy Management
- `get_tags`: Retrieve all tags from MISP
- `get_tag_by_id`: Get a specific tag by ID
- `search_tags`: Search tags with filters
- `get_taxonomies`: Retrieve all taxonomies from MISP
- `get_taxonomy_by_id`: Get a specific taxonomy by ID
- `get_taxonomy_extended_with_tags`: Get extended taxonomy data with associated tags

### Sightings
- `get_sightings_by_event_id`: Retrieve sightings for a specific event

### Warning Lists and Notice Lists
- `get_warninglists`: Retrieve all warning lists from MISP
- `get_noticelists`: Retrieve all notice lists from MISP
- `get_warninglist_by_id`: Get a specific warning list by ID
- `get_noticelist_by_id`: Get a specific notice list by ID
- `search_warninglists`: Search warning lists with filters

### Event Reports
- `get_eventreports`: Retrieve all event reports from MISP
- `get_event_report_by_id`: Get a specific event report by ID

### Collections and Analyst Data
- `get_collection_by_id`: Get a specific collection by ID
- `search_collections`: Search collections with filters - not functional in this version
- `list_analyst_data`: List analyst data entries
- `get_analyst_data_by_id`: Get specific analyst data by ID

### Attributes
- `list_attributes`: Retrieve all attributes from MISP
- `get_attribute_by_id`: Get a specific attribute by ID
- `get_attribute_statistics`: Get attribute statistics by context and percentage
- `describe_attribute_types`: Get available attribute types and categories
- `attributes_rest_search`: Advanced attribute search with REST API filters

### Events
- `get_events`: Retrieve all events from MISP
- `get_event_by_id`: Get a specific event by ID
- `search_events`: Search events with complex filters (POST /events/index)
- `events_rest_search`: Search events using the REST API with flexible filters

### Objects
- `get_object`: Get a specific MISP object by ID
- `objects_rest_search`: Advanced object search with REST API filters

All tools support comprehensive parameter validation, error handling, and return strongly-typed responses based on actual MISP API schemas.

## Configuration

Set these environment variables:

```bash
export MISP_URL="https://misp.local"
export MISP_API_KEY="your-api-key-here"
export MISP_VERIFY_TLS="true"  # optional, default: false
export MISP_TIMEOUT="30"       # optional, default: 30 seconds
```

Or use command-line arguments:

```bash
./misp-mcp --misp-url https://misp.local --api-key YOUR_KEY --verify-tls --timeout 30
```

## Building

```bash
cargo build --release
```


## Testing

The server uses stdio transport for MCP communication. You can test it using named pipes:

```bash
# 1. Test attributes_rest_search
echo -e '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"attributes_rest_search","arguments":{"category":"Network activity","type":"ip-src","limit":5}}}' | ./target/release/misp-mcp

# 2. Test events_rest_search  
echo -e '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"events_rest_search","arguments":{"eventinfo":"APT","limit":5}}}' | ./target/release/misp-mcp

# 3. Test objects_rest_search
echo -e '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"objects_rest_search","arguments":{"name":"vulnerability","limit":5}}}' | ./target/release/misp-mcp
```

### Alternative Testing with Simple Client

For simpler testing, you can create a basic client script to interact with the server programmatically.

## Implementation Highlights

- **Comprehensive Coverage**: 40 tools covering all major MISP API endpoints
- **Type Safety**: All MISP data structures are strongly typed with serde for compile-time guarantees
- **Advanced Search**: Support for complex REST API filters and search operations
- **Error Handling**: Comprehensive error types with proper JSON-RPC error codes and detailed error messages
- **Logging**: Structured logging with tracing for debugging and monitoring  
- **Async**: Full async/await support with Tokio runtime for high performance
- **Reusable**: mcp-core library can be used for other MCP server implementations
- **Standards Compliant**: Exactly implements the Anthropic MCP specification
- **Documentation**: Extensive rustdoc comments, examples, and comprehensive type documentation
- **Production Ready**: Robust error handling, timeouts, TLS verification, and configuration management

## Development

The codebase follows Rust best practices:

- **Error Handling**: Uses `Result<T, E>` and `thiserror` for comprehensive error management
- **Async**: Tokio for async runtime, async/await throughout for high performance
- **Serialization**: Serde for JSON handling with proper field mapping and validation
- **Logging**: Structured logging with `tracing` crate for observability
- **CLI**: `clap` for command-line argument parsing with comprehensive help
- **HTTP**: `reqwest` for MISP API communication with timeout and TLS support
- **Type Safety**: Comprehensive type definitions covering all MISP API structures
- **Modularity**: Clean separation between protocol implementation, types, and application logic

## Key Advantages

This Rust implementation offers significant advantages over other MCP server implementations:

- **Performance**: Compiled binary with zero-overhead abstractions and async I/O
- **Memory Safety**: Rust's ownership system prevents common security vulnerabilities
- **Type Safety**: Compile-time verification of all MISP API interactions
- **Reliability**: Comprehensive error handling and graceful degradation
- **Maintainability**: Clear module boundaries and extensive documentation
- **Portability**: Single binary deployment with minimal dependencies

This implementation replaces previous versions with a more robust, type-safe, and production-ready Rust codebase that exactly implements the Anthropic MCP specification while providing comprehensive MISP integration.
