//! MISP MCP Server
//! 
//! A Model Context Protocol (MCP) server that provides access to MISP
//! (Malware Information Sharing Platform) functionality through a set
//! of well-defined tools.
//! 
//! This server exposes the following validated MISP operations:
//! - get_users: List all users
//! 
//! The server uses the mcp-core library for MCP protocol handling and
//! misp-types for strongly-typed MISP data structures.

use clap::{Arg, Command};
use mcp_core::{Server, Tool, ToolInput, ToolResult};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

mod misp_client;
use misp_client::{MispClient, MispError};
use misp_types::{types::CollectionFilterBody, AttributeRestSearchRequest, EventIndexRequest, EventsRestSearchRequest, ObjectsRestSearchRequest};

/// Application configuration loaded from environment variables and command line.
#[derive(Debug, Clone)]
pub struct Config {
    /// MISP server base URL (e.g., "https://misp.local")
    pub misp_url: String,
    /// MISP API key for authentication
    pub api_key: String,
    /// Whether to verify TLS certificates (default: true)
    pub verify_tls: bool,
    /// Request timeout in seconds (default: 30)
    pub timeout_seconds: u64,
}

impl Config {
    /// Load configuration from command line matches.
    pub fn from_matches(matches: &clap::ArgMatches) -> anyhow::Result<Self> {
        let misp_url = matches.get_one::<String>("misp-url").unwrap().clone();
        let api_key = matches.get_one::<String>("api-key").unwrap().clone();
        let verify_tls = matches.get_flag("verify-tls");
        let timeout_seconds: u64 = matches
            .get_one::<String>("timeout")
            .unwrap()
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid timeout value: {}", e))?;

        Ok(Config {
            misp_url,
            api_key,
            verify_tls,
            timeout_seconds,
        })
    }

    /// Load configuration from environment variables and command line arguments.
    pub fn from_env_and_args() -> anyhow::Result<Self> {
        let matches = Command::new("misp-mcp")
            .version("0.1.0")
            .about("MCP server for MISP integration")
            .arg(
                Arg::new("misp-url")
                    .long("misp-url")
                    .env("MISP_URL")
                    .help("MISP server base URL")
                    .required(true)
                    .value_name("URL")
            )
            .arg(
                Arg::new("api-key")
                    .long("api-key")
                    .env("MISP_API_KEY")
                    .help("MISP API key")
                    .required(true)
                    .value_name("KEY")
            )
            .arg(
                Arg::new("verify-tls")
                    .long("verify-tls")
                    .env("MISP_VERIFY_TLS")
                    .help("Verify TLS certificates")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("timeout")
                    .long("timeout")
                    .env("MISP_TIMEOUT")
                    .help("Request timeout in seconds")
                    .default_value("30")
                    .value_name("SECONDS")
            )
            .arg(
                Arg::new("quiet")
                    .long("quiet")
                    .short('q')
                    .help("Disable logging output (for testing)")
                    .action(clap::ArgAction::SetTrue)
            )
            .get_matches();

        let misp_url = matches.get_one::<String>("misp-url").unwrap().clone();
        let api_key = matches.get_one::<String>("api-key").unwrap().clone();
        let verify_tls = matches.get_flag("verify-tls");
        let timeout_seconds: u64 = matches
            .get_one::<String>("timeout")
            .unwrap()
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid timeout value: {}", e))?;

        Ok(Config {
            misp_url,
            api_key,
            verify_tls,
            timeout_seconds,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments first to check for quiet mode
    let matches = Command::new("misp-mcp")
        .version("0.1.0")
        .about("MCP server for MISP integration")
        .arg(
            Arg::new("misp-url")
                .long("misp-url")
                .env("MISP_URL")
                .help("MISP server base URL")
                .required(true)
                .value_name("URL")
        )
        .arg(
            Arg::new("api-key")
                .long("api-key")
                .env("MISP_API_KEY")
                .help("MISP API key")
                .required(true)
                .value_name("KEY")
        )
        .arg(
            Arg::new("verify-tls")
                .long("verify-tls")
                .env("MISP_VERIFY_TLS")
                .help("Verify TLS certificates")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .env("MISP_TIMEOUT")
                .help("Request timeout in seconds")
                .default_value("30")
                .value_name("SECONDS")
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .short('q')
                .help("Disable logging output (for testing)")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let quiet_mode = matches.get_flag("quiet");

    // Initialize logging only if not in quiet mode
    if !quiet_mode {
        let filter = EnvFilter::from_default_env()
            .add_directive("misp_mcp=info".parse()?)
            .add_directive("mcp_core=warn".parse()?)
            .add_directive("misp_client=info".parse()?);
        
        fmt()
            .with_writer(std::io::stderr)  // Send logs to stderr, keep stdout clean for JSON-RPC
            .with_env_filter(filter)
            .with_target(false)
            .init();

        info!("Starting MISP MCP Server");
    }

    // Load configuration
    let config = Config::from_matches(&matches).map_err(|e| {
        if !quiet_mode {
            error!("Configuration error: {}", e);
        }
        e
    })?;

    if !quiet_mode {
        info!("Loaded configuration: MISP URL = {}, Verify TLS = {}, Timeout = {}s", 
              config.misp_url, config.verify_tls, config.timeout_seconds);
    }

    // Create MISP client
    let misp_client = MispClient::new(
        config.misp_url.clone(),
        config.api_key.clone(),
        config.verify_tls,
        config.timeout_seconds,
    ).await.map_err(|e| {
        error!("Failed to create MISP client: {}", e);
        e
    })?;

    // Create MCP server
    let mut server = Server::new("misp-mcp-server", "0.1.0");

    // Register MISP tools
    register_misp_tools(&mut server, misp_client).await?;

    info!("Registered {} tools", server.tool_count());

    // Run the server
    server.run_stdio().await.map_err(|e| {
        error!("Server error: {}", e);
        anyhow::anyhow!("Server failed: {}", e)
    })?;

    info!("MISP MCP Server shutting down");
    Ok(())
}

/// Register all MISP tools with the MCP server.
async fn register_misp_tools(server: &mut Server, client: MispClient) -> anyhow::Result<()> {
    info!("Registering MISP tools...");

    // Clone client for each tool handler
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_users",
        "Retrieve all users from MISP",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_users().await {
                    Ok(users) => {
                        let json = serde_json::to_string_pretty(&users)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_users failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get users: {}", e)))
                    }
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_user",
        "Retrieve a specific user by ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let user_id: String = input.get_argument("user_id")?;
                
                match client.get_user_by_id(&user_id).await {
                    Ok(user) => {
                        let json = serde_json::to_string_pretty(&user)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_user failed for user_id {}: {}", user_id, e);
                        Ok(ToolResult::error(format!("Failed to get user {}: {}", user_id, e)))
                    }
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_galaxies",
        "Retrieve all galaxies from MISP",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_galaxies().await {
                    Ok(galaxies) => {
                        let json = serde_json::to_string_pretty(&galaxies)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_galaxies failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get galaxies: {}", e)))
                    }
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_galaxy",
        "Retrieve a specific galaxy by ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let galaxy_id: String = input.get_argument("galaxy_id")?;
                
                match client.get_galaxy_by_id(&galaxy_id).await {
                    Ok(galaxy) => {
                        let json = serde_json::to_string_pretty(&galaxy)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_galaxy failed for galaxy_id {}: {}", galaxy_id, e);
                        Ok(ToolResult::error(format!("Failed to get galaxy {}: {}", galaxy_id, e)))
                    }
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "search_galaxies",
        "Search MISP galaxies by value filter",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let value: String = input.get_argument("value")?;
                
                match client.search_galaxies(&value).await {
                    Ok(galaxies) => {
                        let json = serde_json::to_string_pretty(&galaxies)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("search_galaxies failed for value '{}': {}", value, e);
                        Ok(ToolResult::error(format!("Failed to search galaxies with value '{}': {}", value, e)))
                    }
                }
            })
        }
    ));

    // Tool 6: get_galaxy_clusters
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_galaxy_clusters",
        "Get galaxy clusters for a specific galaxy by ID",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let galaxy_id: String = input.get_argument("galaxy_id")?;
                
                match client.get_galaxy_clusters(&galaxy_id).await {
                    Ok(clusters) => {
                        let json = serde_json::to_string_pretty(&clusters)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_galaxy_clusters failed for galaxy_id '{}': {}", galaxy_id, e);
                        Ok(ToolResult::error(format!("Failed to get galaxy clusters for galaxy_id '{}': {}", galaxy_id, e)))
                    }
                }
            })
        }
    ));

    // Tool 7: get_galaxy_cluster_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_galaxy_cluster_by_id",
        "Get detailed information about a specific galaxy cluster by ID",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let galaxy_cluster_id: String = input.get_argument("galaxy_cluster_id")?;
                
                match client.get_galaxy_cluster_by_id(&galaxy_cluster_id).await {
                    Ok(cluster) => {
                        let json = serde_json::to_string_pretty(&cluster)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_galaxy_cluster_by_id failed for galaxy_cluster_id '{}': {}", galaxy_cluster_id, e);
                        Ok(ToolResult::error(format!("Failed to get galaxy cluster for galaxy_cluster_id '{}': {}", galaxy_cluster_id, e)))
                    }
                }
            })
        }
    ));

    // Tool 8: search_galaxy_clusters
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "search_galaxy_clusters",
        "Search galaxy clusters within a specific galaxy using search criteria",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let galaxy_id: String = input.get_argument("galaxy_id")?;
                let context: String = input.get_argument("context")?;
                let searchall: String = input.get_argument("searchall")?;
                
                match client.search_galaxy_clusters(&galaxy_id, &context, &searchall).await {
                    Ok(clusters) => {
                        let json = serde_json::to_string_pretty(&clusters)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("search_galaxy_clusters failed for galaxy_id '{}', context '{}', searchall '{}': {}", galaxy_id, context, searchall, e);
                        Ok(ToolResult::error(format!("Failed to search galaxy clusters: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 9: get_organisations
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_organisations",
        "Get all organisations from the MISP instance",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_organisations().await {
                    Ok(organisations) => {
                        let json = serde_json::to_string_pretty(&organisations)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_organisations failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get organisations: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 11: get_tags
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_tags",
        "Get all tags from the MISP instance",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_tags().await {
                    Ok(tags) => {
                        let json = serde_json::to_string_pretty(&tags)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_tags failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get tags: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 12: get_tag_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_tag_by_id",
        "Get a specific tag by ID from the MISP instance",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let tag_id = input.arguments.get("tag_id")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("tag_id parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("tag_id must be a string".to_string()))?;

                match client.get_tag_by_id(tag_id).await {
                    Ok(tag) => {
                        let json = serde_json::to_string_pretty(&tag)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_tag_by_id failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get tag by ID: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 13: search_tags
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "search_tags",
        "Search for tags by search term in the MISP instance",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let search_term = input.arguments.get("search_term")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("search_term parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("search_term must be a string".to_string()))?;

                match client.search_tags(search_term).await {
                    Ok(search_results) => {
                        let json = serde_json::to_string_pretty(&search_results)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("search_tags failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to search tags: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 14: get_organisation_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_organisation_by_id",
        "Get a specific organisation by its ID from the MISP instance",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let organisation_id: String = input.get_argument("organisation_id")?;

                match client.get_organisation_by_id(&organisation_id).await {
                    Ok(organisation) => {
                        let json = serde_json::to_string_pretty(&organisation)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_organisation_by_id failed for organisation_id {}: {}", organisation_id, e);
                        Ok(ToolResult::error(format!("Failed to get organisation {}: {}", organisation_id, e)))
                    }
                }
            })
        }
    ));

    // Tool 15: get_taxonomies
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_taxonomies",
        "Get all taxonomies from the MISP instance",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_taxonomies().await {
                    Ok(taxonomies) => {
                        let json = serde_json::to_string_pretty(&taxonomies)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_taxonomies failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get taxonomies: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 16: get_taxonomy_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_taxonomy_by_id",
        "Get a specific taxonomy by its ID from the MISP instance",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let taxonomy_id = input.arguments.get("taxonomy_id")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("taxonomy_id parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("taxonomy_id must be a string".to_string()))?;

                match client.get_taxonomy_by_id(taxonomy_id).await {
                    Ok(taxonomy) => {
                        let json = serde_json::to_string_pretty(&taxonomy)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_taxonomy_by_id failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get taxonomy by ID: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 17: get_taxonomy_extended_with_tags
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_taxonomy_extended_with_tags",
        "Get a taxonomy with its extended tags from the MISP instance",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let taxonomy_id = input.arguments.get("taxonomy_id")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("taxonomy_id parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("taxonomy_id must be a string".to_string()))?;

                 match client.get_taxonomy_extended_with_tags(taxonomy_id).await {
                    Ok(taxonomy_ext) => {
                        let json = serde_json::to_string_pretty(&taxonomy_ext)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_taxonomy_extended_with_tags failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get taxonomy extended with tags: {}", e)))
                    }
                }
            })
        }
    ));
    // Tool 18: get_sightings_by_event_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_sightings_by_event_id",
        "Retrieve sightings for a specific event by ID or UUID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                // Extract required event_id argument (string)
                let event_id: String = input.get_argument("event_id")?;
                match client.get_sightings_by_event_id(&event_id).await {
                    Ok(sightings) => {
                        let json = serde_json::to_string_pretty(&sightings)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_sightings_by_event_id failed for event_id '{}': {}", event_id, e);
                        Ok(ToolResult::error(format!("Failed to get sightings for event_id '{}': {}", event_id, e)))
                    }
                }
            })
        }
    ));

    // Tool 19: get_warninglists
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_warninglists",
        "Retrieve all warninglists from MISP",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_warninglists().await {
                    Ok(warninglists) => {
                        let json = serde_json::to_string_pretty(&warninglists)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_warninglists failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get warninglists: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 20: get_noticelists
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_noticelists",
        "Retrieve all noticelists from MISP",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_noticelists().await {
                    Ok(noticelists) => {
                        let json = serde_json::to_string_pretty(&noticelists)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_noticelists failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get noticelists: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 21: get_warninglist_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_warninglist_by_id",
        "Retrieve a specific warninglist by its ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let warninglist_id: String = input.get_argument("warninglist_id")?;
                match client.get_warninglist_by_id(&warninglist_id).await {
                    Ok(warninglist) => {
                        let json = serde_json::to_string_pretty(&warninglist)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_warninglist_by_id failed for warninglist_id {}: {}", warninglist_id, e);
                        Ok(ToolResult::error(format!("Failed to get warninglist {}: {}", warninglist_id, e)))
                    }
                }
            })
        }
    ));

        // Tool 22: get_noticelist_by_id
        let client_clone = client.clone();
        server.add_tool(Tool::new(
            "get_noticelist_by_id",
            "Retrieve a specific noticelist by its ID from MISP",
            move |input: ToolInput| {
                let client = client_clone.clone();
                Box::pin(async move {
                    let noticelist_id: String = input.get_argument("noticelist_id")?;
                    match client.get_noticelist_by_id(&noticelist_id).await {
                        Ok(noticelist) => {
                            let json = serde_json::to_string_pretty(&noticelist)
                                .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                            Ok(ToolResult::text(json))
                        }
                        Err(e) => {
                            error!("get_noticelist_by_id failed for noticelist_id {}: {}", noticelist_id, e);
                            Ok(ToolResult::error(format!("Failed to get noticelist {}: {}", noticelist_id, e)))
                        }
                    }
                })
            }
        ));


    // Tool 23: search_warninglists
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "search_warninglists",
        "Search warninglists by value in MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                // Extract "value" argument, error if missing or not a string
                let value = input.arguments.get("value")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("value parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("value must be a string".to_string()))?;

                match client.search_warninglists(value).await {
                    Ok(warninglists) => {
                        let json = serde_json::to_string_pretty(&warninglists)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("search_warninglists failed for value '{}': {}", value, e);
                        Ok(ToolResult::error(format!("Failed to search warninglists with value '{}': {}", value, e)))
                    }
                }
            })
        }
    ));

    // Tool 24: get_eventreports
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_eventreports",
        "Retrieve all event reports from MISP",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_event_reports().await {
                    Ok(eventreports) => {
                        let json = serde_json::to_string_pretty(&eventreports)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_eventreports failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get event reports: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 25: get_event_report_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_event_report_by_id",
        "Retrieve a single event report by its ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let event_report_id: String = input.get_argument("event_report_id")?;
                match client.get_event_report_by_id(&event_report_id).await {
                    Ok(event_report) => {
                        let json = serde_json::to_string_pretty(&event_report)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_event_report_by_id failed for event_report_id {}: {}", event_report_id, e);
                        Ok(ToolResult::error(format!("Failed to get event report {}: {}", event_report_id, e)))
                    }
                }
            })
        }
    ));

    // Tool 26: get_collection_by_id
    // Register the get_collection_by_id tool for retrieving a single collection by its ID from MISP.
    // This follows the same pattern as get_event_report_by_id for consistency and maintainability.
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_collection_by_id",
        "Retrieve a single collection by its ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let collection_id: String = input.get_argument("collection_id")?;
                match client.get_collection_by_id(&collection_id).await {
                    Ok(collection) => {
                        let json = serde_json::to_string_pretty(&collection)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(MispError::NotFound { .. }) => {
                        // Gracefully handle "not found" by returning an empty JSON object
                        Ok(ToolResult::text("{}".to_string()))
                    }                    
                    Err(e) => {
                        error!("get_collection_by_id failed for collection_id {}: {}", collection_id, e);
                        Ok(ToolResult::error(format!("Failed to get collection {}: {}", collection_id, e)))
                    }
                }
            })
        }
    ));


    // Tool 27: search_collections
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "search_collections",
        "Search for collections with filtering from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let filter = input.arguments.get("filter")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("filter parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("filter must be a string".to_string()))?;

                let uuid = input.arguments.get("uuid").and_then(|v| v.as_str()).map(|s| s.to_string());
                let type_ = input.arguments.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
                let name = input.arguments.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());

                let body = CollectionFilterBody { uuid, type_, name };

                match client.search_collections(filter, &body).await {
                    Ok(collections) => {
                        let json = serde_json::to_string_pretty(&collections)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("search_collections failed for filter '{}': {}", filter, e);
                        Ok(ToolResult::error(format!("Failed to search collections for filter '{}': {}", filter, e)))
                    }
                }
            })
        }
    ));


    // Tool 28: list_analyst_data
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "list_analyst_data",
        "List analyst data of a given type (Note, Opinion, Relationship) from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let analyst_type = input.arguments.get("analyst_type")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("analyst_type parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("analyst_type must be a string".to_string()))?;

                match client.list_analyst_data(analyst_type).await {
                    Ok(data) => {
                        let json = serde_json::to_string_pretty(&data)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("list_analyst_data failed for analyst_type '{}': {}", analyst_type, e);
                        Ok(ToolResult::error(format!("Failed to list analyst data for type '{}': {}", analyst_type, e)))
                    }
                }
            })
        }
    ));

    // Tool 29: get_analyst_data_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_analyst_data_by_id",
        "Get a single analyst data object by type and ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let analyst_type = input.arguments.get("analyst_type")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("analyst_type parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("analyst_type must be a string".to_string()))?;
                let analyst_data_id = input.arguments.get("analyst_data_id")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("analyst_data_id parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("analyst_data_id must be a string".to_string()))?;

                match client.get_analyst_data_by_id(analyst_type, analyst_data_id).await {
                    Ok(data) => {
                        let json = serde_json::to_string_pretty(&data)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_analyst_data_by_id failed for type '{}' and id '{}': {}", analyst_type, analyst_data_id, e);
                        Ok(ToolResult::text("{}".to_string()))
                    }
                }
            })
        }
    ));

    // Tool 30: list_attributes
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "list_attributes",
        "List all attributes in the MISP instance.",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.list_attributes().await {
                    Ok(data) => {
                        let json = serde_json::to_string_pretty(&data)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("list_attributes failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to list attributes: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool 31: get_attribute_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_attribute_by_id",
        "Get a single attribute by its ID or UUID.",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let attribute_id = input.arguments.get("attribute_id")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("attribute_id parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("attribute_id must be a string".to_string()))?;
                match client.get_attribute_by_id(attribute_id).await {
                    Ok(attribute) => {
                        let json = serde_json::to_string_pretty(&attribute)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }   
                    Err(e) => {
                        error!("get_attribute_by_id failed for id '{}': {}", attribute_id, e);
                        Ok(ToolResult::error(format!("Failed to get attribute for id '{}': {}", attribute_id, e)))
                    }
                }
            })
        }
    ));

    // Tool 32: get_attribute_statistics
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_attribute_statistics",
        "Get attribute statistics by context (type/category) and count/percentage.",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let context = input.arguments.get("context")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("context parameter is required".to_string()))?
                    .as_str()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("context must be a string".to_string()))?;
                let percentage = input.arguments.get("percentage")
                    .ok_or_else(|| mcp_core::McpError::invalid_params("percentage parameter is required".to_string()))?
                    .as_u64()
                    .ok_or_else(|| mcp_core::McpError::invalid_params("percentage must be an integer (0 or 1)".to_string()))? as u8;
                match client.get_attribute_statistics(context, percentage).await {
                    Ok(stats) => {
                        let json = serde_json::to_string_pretty(&stats)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_attribute_statistics failed for context '{}' and percentage '{}': {}", context, percentage, e);
                        Ok(ToolResult::error(format!("Failed to get attribute statistics for context '{}' and percentage '{}': {}", context, percentage, e)))
                    }
                }
            })
        }
    ));

    // Tool 33: describe_attribute_types
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "describe_attribute_types",
        "Get list of available attribute types, categories, and sane defaults.",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.describe_attribute_types().await {
                    Ok(result) => {
                        let json = serde_json::to_string_pretty(&result)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("describe_attribute_types failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to describe attribute types: {}", e)))
                    }
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "attributes_rest_search",
        "Search attributes using the /attributes/restSearch endpoint",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let filter_json: String = input.get_argument("filter_json")?;
                // println!("DEBUG: filter_json = {:?}", filter_json);
                // Ensure we always expect a struct, not a sequence
                let filter: AttributeRestSearchRequest = serde_json::from_str(&filter_json)
                    .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                match client.attributes_rest_search(&filter).await {
                    Ok(response) => {
                        let json = serde_json::to_string_pretty(&response)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("attributes_rest_search failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to search attributes: {}", e)))
                    }
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_events",
        "Retrieve all events from MISP",
        move |_input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                match client.get_events().await {
                    Ok(events) => {
                        let json = serde_json::to_string_pretty(&events)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_events failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to get events: {}", e)))
                    }
                }
            })
        }
    ));


    // Tool: get_event_by_id
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_event_by_id",
        "Retrieve a single event by its ID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                // Extract required event_id argument (string)
                let event_id: String = input.get_argument("event_id")?;
                match client.get_event_by_id(&event_id).await {
                    Ok(event) => {
                        let json = serde_json::to_string_pretty(&event)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_event_by_id failed for event_id '{}': {}", event_id, e);
                        Ok(ToolResult::error(format!("Failed to get event for event_id '{}': {}", event_id, e)))
                    }
                }
            })
        }
    ));

    // Tool: search_events
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "search_events",
        "Search for events using POST /events/index with flexible filters",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                // Accepts a single argument: "request_json" (stringified EventIndexRequest)
                let request_json: String = input.get_argument("request_json")?;
                let request: EventIndexRequest = serde_json::from_str(&request_json)
                    .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                match client.search_events(&request).await {
                    Ok(events) => {
                        let json = serde_json::to_string_pretty(&events)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("search_events failed: {}", e);
                        Ok(ToolResult::error(format!("Failed to search events: {}", e)))
                    }
                }
            })
        }
    ));

    // Tool: events_rest_search
    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "events_rest_search",
        "Search events using the /events/restSearch endpoint",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                // Parse input as EventsRestSearchRequest
                let map: serde_json::Map<String, serde_json::Value> = input.arguments.into_iter().collect();
                let params: EventsRestSearchRequest = serde_json::from_value(serde_json::Value::Object(map))?;
                match client.events_rest_search(&params).await {
                    Ok(resp) => {
                        let json = serde_json::to_string_pretty(&resp)?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => Ok(ToolResult::error(format!("Failed to search events: {}", e))),
                }
            })
        }
    ));

    let client_clone = client.clone();
    server.add_tool(Tool::new(
        "get_object",
        "Retrieve a specific object by ID or UUID from MISP",
        move |input: ToolInput| {
            let client = client_clone.clone();
            Box::pin(async move {
                let object_id: String = input.get_argument("object_id")?;
                match client.get_object_by_id(&object_id).await {
                    Ok(object) => {
                        let json = serde_json::to_string_pretty(&object)
                            .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                        Ok(ToolResult::text(json))
                    }
                    Err(e) => {
                        error!("get_object failed for object_id {}: {}", object_id, e);
                        Ok(ToolResult::error(format!("Failed to get object {}: {}", object_id, e)))
                    }
                }
            })
        }
    ));

server.add_tool(Tool::new(
    "objects_rest_search",
    "Get a filtered and paginated list of objects from MISP",
    move |input: ToolInput| {
        let client = client.clone();
        Box::pin(async move {
            // Parse input as ObjectsRestSearchRequest
            let map: serde_json::Map<String, serde_json::Value> = input.arguments.into_iter().collect();
            let params: ObjectsRestSearchRequest = serde_json::from_value(serde_json::Value::Object(map))?;
            match client.objects_rest_search(&params).await {
                Ok(objects) => {
                    let json = serde_json::to_string_pretty(&objects)
                        .map_err(|e| mcp_core::McpError::serialization_error(e.to_string()))?;
                    Ok(ToolResult::text(json))
                }
                Err(e) => {
                    error!("objects_rest_search failed: {}", e);
                    Ok(ToolResult::error(format!("Failed to search objects: {}", e)))
                }
            }
        })
    }
));

    info!("Successfully registered get_users, get_user, get_galaxies, get_galaxy, search_galaxies, get_galaxy_clusters, get_galaxy_cluster_by_id, search_galaxy_clusters, get_organisations, get_tags, get_tag_by_id, search_tags, get_organisation_by_id, get_taxonomies, get_taxonomy_by_id, get_taxonomy_extended_with_tags tools, get_sightings_by_event_id, get_warninglists, get_noticelists, get_warninglist_by_id, get_noticelist_by_id, search_warninglists, get_eventreports, get_event_report_by_id, get_collection_by_id, search_collections, list_analyst_data, get_analyst_data_by_id tools, list_attributes, get_attribute_by_id tools, get_attribute_statistics tools, describe_attribute_types tools, attributes_rest_search tools, get_events, and get_event_by_id, search_events, get_object, objects_rest_search tools");
    Ok(())
}
