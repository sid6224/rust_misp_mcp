//! MISP HTTP client implementation.
//! 
//! This module provides a robust HTTP client for interacting with MISP APIs,
//! including proper error handling, authentication, and response parsing.

use misp_types::*;
//use crate::types::{AttributeRestSearchRequest, AttributeListResponse};
use reqwest::{Client, Response, StatusCode};
use serde_json;
use std::time::Duration;
use tracing::{debug, info, trace, warn, error};

/// Errors that can occur during MISP API operations.
#[derive(Debug, thiserror::Error)]
pub enum MispError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("MISP API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Authentication failed: invalid API key")]
    Authentication,
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Invalid configuration: {message}")]
    Config { message: String },
}

/// HTTP client for MISP API operations.
/// 
/// This client handles authentication, request/response serialization,
/// error handling, and logging for all MISP API interactions.
#[derive(Debug, Clone)]
pub struct MispClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl MispClient {
    /// Create a new MISP client.
    /// 
    /// # Arguments
    /// - `base_url`: MISP server base URL (e.g., "https://misp.local")
    /// - `api_key`: MISP API authentication key
    /// - `verify_tls`: Whether to verify TLS certificates
    /// - `timeout_seconds`: Request timeout in seconds
    pub async fn new(
        base_url: String,
        api_key: String,
        verify_tls: bool,
        timeout_seconds: u64,
    ) -> Result<Self, MispError> {
        // Validate configuration
        if base_url.is_empty() {
            return Err(MispError::Config {
                message: "MISP URL cannot be empty".to_string(),
            });
        }
        
        if api_key.is_empty() {
            return Err(MispError::Config {
                message: "API key cannot be empty".to_string(),
            });
        }
        
        // Build HTTP client with appropriate settings
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .danger_accept_invalid_certs(!verify_tls)
            .user_agent("misp-mcp-server/0.1.0");
        
        if !verify_tls {
            warn!("TLS certificate verification is disabled");
            client_builder = client_builder.danger_accept_invalid_hostnames(true);
        }
        
        let client = client_builder.build()?;
        
        info!("Created MISP client for {}", base_url);
        debug!("Client configuration: verify_tls={}, timeout={}s", verify_tls, timeout_seconds);
        
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }
    
    /// Execute a GET request to a MISP endpoint.
    async fn misp_get<T>(&self, endpoint: &str) -> Result<T, MispError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        debug!("GET {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_key)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Execute a POST request to a MISP endpoint.
    async fn misp_post<T, B>(&self, endpoint: &str, body: &B) -> Result<T, MispError>
    where
        T: for<'de> serde::Deserialize<'de>,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        debug!("POST {}", url);
        
        let json_body = serde_json::to_string(body)?;
        trace!("Request body: {}", json_body);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.api_key)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await?;
        
        self.handle_response(response).await
    }
    
    /// Handle HTTP response and deserialize JSON.
    async fn handle_response<T>(&self, response: Response) -> Result<T, MispError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let status = response.status();
        let url = response.url().to_string();
        
        debug!("Response: {} {}", status, url);
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("HTTP error {}: {}", status, error_text);
            
            return Err(match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => MispError::Authentication,
                StatusCode::NOT_FOUND => MispError::NotFound {
                    resource: url,
                },
                _ => MispError::Api {
                    status: status.as_u16(),
                    message: error_text,
                },
            });
        }
        
        let response_text = response.text().await?;
        trace!("Response body: {}", response_text);
        
        // Try to deserialize the response
        match serde_json::from_str::<T>(&response_text) {
            Ok(data) => {
                debug!("Successfully parsed response");
                Ok(data)
            }
            Err(e) => {
                error!("Failed to parse JSON response: {}", e);
                error!("Response was: {}", response_text);
                Err(MispError::Json(e))
            }
        }
    }
    
    /// Get all users from MISP.
    /// 
    /// Corresponds to: GET /admin/users
    pub async fn get_users(&self) -> Result<GetUsersResponse, MispError> {
        info!("Fetching all users");
        self.misp_get("/admin/users").await
    }
    
    /// Get a specific user by ID from MISP.
    /// 
    /// Corresponds to: GET /admin/users/view/{user_id}
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<GetUserByIdResponse, MispError> {
        info!("Fetching user with ID: {}", user_id);
        let endpoint = format!("/admin/users/view/{}", user_id);
        self.misp_get(&endpoint).await
    }
    
    /// Get all galaxies from MISP.
    /// 
    /// Corresponds to: GET /galaxies
    pub async fn get_galaxies(&self) -> Result<GetGalaxiesResponse, MispError> {
        info!("Fetching all galaxies");
        self.misp_get("/galaxies").await
    }
    
    /// Get a specific galaxy by ID from MISP.
    /// 
    /// Corresponds to: GET /galaxies/view/{galaxy_id}
    /// 
    /// # Arguments
    /// - `galaxy_id`: Galaxy ID (can be numeric ID or UUID)
    pub async fn get_galaxy_by_id(&self, galaxy_id: &str) -> Result<GetGalaxyByIdResponse, MispError> {
        info!("Fetching galaxy with ID: {}", galaxy_id);
        let endpoint = format!("/galaxies/view/{}.json", galaxy_id);
        self.misp_get(&endpoint).await
    }
    
    /// Search galaxies by value filter.
    /// 
    /// Corresponds to: POST /galaxies
    /// 
    /// # Arguments
    /// - `search_value`: Search term to filter galaxies (e.g., "botnet", "apt", "malware")
    pub async fn search_galaxies(&self, search_value: &str) -> Result<SearchGalaxiesResponse, MispError> {
        info!("Searching galaxies with value: {}", search_value);
        
        let request_payload = SearchGalaxiesRequest {
            value: search_value.to_string(),
        };
        
        self.misp_post("/galaxies", &request_payload).await
    }
    
    /// Get galaxy clusters for a specific galaxy.
    /// 
    /// Corresponds to: GET /galaxy_clusters/index/{galaxy_id}
    /// 
    /// # Arguments
    /// - `galaxy_id`: Galaxy ID (can be numeric ID or UUID)
    pub async fn get_galaxy_clusters(&self, galaxy_id: &str) -> Result<GetGalaxyClustersResponse, MispError> {
        info!("Fetching galaxy clusters for galaxy ID: {}", galaxy_id);
        let endpoint = format!("/galaxy_clusters/index/{}.json", galaxy_id);
        self.misp_get(&endpoint).await
    }

    /// Get a specific galaxy cluster by its ID.
    /// 
    /// Returns detailed information about a galaxy cluster including all metadata,
    /// elements, relationships, and associated tag information.
    /// 
    /// # Arguments
    /// - `galaxy_cluster_id`: Galaxy cluster ID (can be numeric ID or UUID)
    pub async fn get_galaxy_cluster_by_id(&self, galaxy_cluster_id: &str) -> Result<GetGalaxyClusterByIdResponse, MispError> {
        info!("Fetching galaxy cluster by ID: {}", galaxy_cluster_id);
        let endpoint = format!("/galaxy_clusters/view/{}.json", galaxy_cluster_id);
        self.misp_get(&endpoint).await
    }

    /// Search galaxy clusters within a specific galaxy using search criteria.
    /// 
    /// Corresponds to: POST /galaxy_clusters/index/{galaxy_id}
    /// 
    /// # Arguments
    /// - `galaxy_id`: Galaxy ID to search within
    /// - `context`: Search context ("all", "default", "org", "deleted")
    /// - `searchall`: Search term to filter clusters
    pub async fn search_galaxy_clusters(
        &self, 
        galaxy_id: &str, 
        context: &str, 
        searchall: &str
    ) -> Result<SearchGalaxyClustersResponse, MispError> {
        info!("Searching galaxy clusters in galaxy ID: {} with context: '{}' and term: '{}'", galaxy_id, context, searchall);
        
        let request_payload = SearchGalaxyClustersRequest {
            context: context.to_string(),
            searchall: searchall.to_string(),
        };
        
        let endpoint = format!("/galaxy_clusters/index/{}", galaxy_id);
        self.misp_post(&endpoint, &request_payload).await
    }

    /// Get all organisations.
    /// 
    /// Corresponds to: GET /organisations
    /// 
    /// Returns a list of all organisations in the MISP instance.
    pub async fn get_organisations(&self) -> Result<GetOrganisationsResponse, MispError> {
        info!("Fetching all organisations");
        self.misp_get("/organisations.json").await
    }

    /// Get all tags.
    /// 
    /// Corresponds to: GET /tags
    /// 
    /// Returns a list of all tags in the MISP instance.
    pub async fn get_tags(&self) -> Result<Vec<Tag>, MispError> {
        info!("Fetching all tags");
        let response: GetTagsResponse = self.misp_get("/tags.json").await?;
        Ok(response.tag)
    }

    /// Get a specific tag by ID.
    /// 
    /// Corresponds to: GET /tags/view/{tag_id}
    /// 
    /// # Arguments
    /// - `tag_id`: Tag ID (numeric string)
    pub async fn get_tag_by_id(&self, tag_id: &str) -> Result<Tag, MispError> {
        info!("Fetching tag with ID: {}", tag_id);
        let endpoint = format!("/tags/view/{}", tag_id);
        self.misp_get(&endpoint).await
    }

    /// Search for tags by search term.
    /// 
    /// Corresponds to: GET /tags/search/{search_term}
    /// 
    /// # Arguments
    /// - `search_term`: Search term to filter tags (should be URL-encoded if needed)
    pub async fn search_tags(&self, search_term: &str) -> Result<SearchTagsResponse, MispError> {
        info!("Searching tags with term: {}", search_term);
        let endpoint = format!("/tags/search/{}", search_term);
        self.misp_get(&endpoint).await
    }
    
    /// Get a specific organisation by ID from MISP.
    /// 
    /// Corresponds to: GET /organisations/view/{organisation_id}
    /// 
    /// # Arguments
    /// - `organisation_id`: Organisation ID (can be numeric ID or UUID)
    pub async fn get_organisation_by_id(&self, organisation_id: &str) -> Result<OrganisationEntry, MispError> {
        info!("Fetching organisation with ID: {}", organisation_id);
        let endpoint = format!("/organisations/view/{}", organisation_id);
        self.misp_get(&endpoint).await
    }

    /// Get all taxonomies from the MISP instance.
    ///
    /// Corresponds to: GET /taxonomies
    pub async fn get_taxonomies(&self) -> Result<GetTaxonomiesResponse, MispError> {
        info!("Fetching all taxonomies");
        self.misp_get("/taxonomies").await
    }

    /// Get a specific taxonomy by ID from the MISP instance.
    ///
    /// Corresponds to: GET /taxonomies/view/{taxonomy_id}
    ///
    /// # Arguments
    /// - `taxonomy_id`: Taxonomy ID (numeric string)
    pub async fn get_taxonomy_by_id(&self, taxonomy_id: &str) -> Result<GetTaxonomyByIdResponse, MispError> {
        info!("Fetching taxonomy with ID: {}", taxonomy_id);
        let endpoint = format!("/taxonomies/view/{}", taxonomy_id);
        self.misp_get(&endpoint).await
    }

    /// Get a taxonomy with its extended tags from the MISP instance.
    /// Corresponds to: GET /taxonomies/taxonomy_tags/{taxonomy_id}
    /// - `taxonomy_id`: Taxonomy ID (numeric string)
    pub async fn get_taxonomy_extended_with_tags(&self, taxonomy_id: &str) -> Result<GetTaxonomyExtendedWithTagsResponse, MispError> {
        info!("Fetching taxonomy extended with tags for ID: {}", taxonomy_id);
        let endpoint = format!("/taxonomies/taxonomy_tags/{}", taxonomy_id);
        self.misp_get(&endpoint).await
    }

    /// Get sightings for a specific event by ID or UUID from MISP.
    ///
    /// Corresponds to: GET /sightings/index/{eventId}
    ///
    /// # Arguments
    /// - `event_id`: Event ID or UUID (string, required)
    pub async fn get_sightings_by_event_id(&self, event_id: &str) -> Result<GetSightingsResponse, MispError> {
        info!("Fetching sightings for event ID/UUID: {}", event_id);
        let endpoint = format!("/sightings/index/{}", event_id);
        self.misp_get(&endpoint).await
    }

    /// Get all warninglists from MISP.
    ///
    /// Corresponds to: GET /warninglists
    pub async fn get_warninglists(&self) -> Result<WarninglistsResponse, MispError> {
        info!("Fetching all warninglists");
        self.misp_get("/warninglists").await
    }

    /// Get a specific warninglist by ID from MISP.
    /// Corresponds to: GET /warninglists/view/{warninglist_id}
    /// Returns a deserialized Warninglist struct with all metadata, entries, and types.
    pub async fn get_warninglist_by_id(&self, warninglist_id: &str) -> Result<Warninglist, MispError> {
        info!("Fetching warninglist with ID: {}", warninglist_id);
        let endpoint = format!("/warninglists/view/{}", warninglist_id);
        // The API returns {"Warninglist": {...}}, so we need to extract the inner object.
        let response: serde_json::Value = self.misp_get(&endpoint).await?;
        let warninglist = serde_json::from_value(response["Warninglist"].clone())
            .map_err(MispError::Json)?;
        Ok(warninglist)
    }

    /// Search warninglists by value (POST /warninglists)
    pub async fn search_warninglists(&self, value: &str) -> Result<WarninglistsResponse, MispError> {
        info!("Searching warninglists with value: {}", value);

        let request_payload = SearchWarninglistRequest {
            value: value.to_string(),
        };

        self.misp_post("/warninglists", &request_payload).await
    }

    /// Get all noticelists from MISP.
    ///
    /// Corresponds to: GET /noticelists
    pub async fn get_noticelists(&self) -> Result<NoticelistsResponse, MispError> {
        info!("Fetching all noticelists");
        self.misp_get("/noticelists").await
    }

    // -----------------------------------------------------------------------------
    // Client method for GET /noticelists/view/{noticelistId}
    // -----------------------------------------------------------------------------
    /// Get a specific noticelist by ID from MISP.
    /// Corresponds to: GET /noticelists/view/{noticelistId}
    /// Returns a deserialized Noticelist object with all metadata and entries.
    pub async fn get_noticelist_by_id(&self, noticelist_id: &str) -> Result<Noticelist, MispError> {
        info!("Fetching noticelist with ID: {}", noticelist_id);
        let endpoint = format!("/noticelists/view/{}", noticelist_id);
        // The API returns {"Noticelist": {...}}, so we need to extract the inner object.
        let response: NoticelistByIdResponse = self.misp_get(&endpoint).await?;
        Ok(response.noticelist)
    }

    /// Fetches all event reports from /eventReports/index endpoint.
    /// Returns a vector of EventReportEntry objects.
    pub async fn get_event_reports(&self) -> Result<Vec<EventReportEntry>, MispError> {
        info!("Fetching all event reports");
        self.misp_get("/eventReports/index").await
    }

    
    /// Fetch a single event report by its ID from /eventReports/view/{eventReportId}.
    /// Returns the full EventReport object with all nested fields.
    pub async fn get_event_report_by_id(&self, event_report_id: &str) -> Result<EventReport, MispError> {
        let endpoint = format!("/eventReports/view/{}", event_report_id);
        // The response is a top-level object with "EventReport" key
        let response: serde_json::Value = self.misp_get(&endpoint).await?;
        let event_report = serde_json::from_value::<EventReport>(response["EventReport"].clone())
            .map_err(MispError::Json)?;
        Ok(event_report)
    }


    /// Get a specific collection by ID from MISP.
    /// Corresponds to: GET /collections/view/{collection_id}
    pub async fn get_collection_by_id(&self, collection_id: &str) -> Result<Collection, MispError> {
        let endpoint = format!("/collections/view/{}", collection_id);
        let response: serde_json::Value = self.misp_get(&endpoint).await?;
        // Assuming the API returns { "Collection": { ... } }
        let collection = serde_json::from_value::<Collection>(response["Collection"].clone())
            .map_err(MispError::Json)?;
        Ok(collection)
    }

    /// Get a list of collections with filtering.
    /// Corresponds to: POST /collections/index/{filter}
    /// - `filter`: "my_collections" or "org_collections"
    /// - `body`: filter fields for the request body
    pub async fn search_collections(&self, filter: &str, body: &CollectionFilterBody) -> Result<Vec<Collection>, MispError> {
        let endpoint = format!("/collections/index/{}", filter);
        self.misp_post(&endpoint, body).await
    }

    // In MispClient impl
    /// List analyst data by type (GET /analystData/index/{analystType})
    pub async fn list_analyst_data(&self, analyst_type: &str) -> Result<Vec<AnalystData>, MispError> {
        let endpoint = format!("/analystData/index/{}", analyst_type);
        self.misp_get(&endpoint).await
    }

    /// Get a single analyst data object by type and ID (GET /analystData/view/{analystType}/{analystDataID})
    pub async fn get_analyst_data_by_id(&self, analyst_type: &str, analyst_data_id: &str) -> Result<AnalystData, MispError> {
        let endpoint = format!("/analystData/view/{}/{}", analyst_type, analyst_data_id);
        self.misp_get(&endpoint).await
    }

    /// Get all attributes (GET /attributes)
    pub async fn list_attributes(&self) -> Result<Vec<Attribute>, MispError> {
        self.misp_get("/attributes").await
    }

    /// Get a single attribute by ID or UUID (GET /attributes/view/{attributeId})
    pub async fn get_attribute_by_id(&self, attribute_id: &str) -> Result<Attribute, MispError> {
        let endpoint = format!("/attributes/view/{}", attribute_id);
        let wrapper: AttributeWrapper = self.misp_get(&endpoint).await?;
        Ok(wrapper.attribute)
    }

    /// Get attribute statistics by context and percentage (GET /attributes/attributeStatistics/{context}/{percentage})
    /// # Arguments
    /// - `context`: "type" or "category"
    /// - `percentage`: 0 for count, 1 for percentage
    pub async fn get_attribute_statistics(&self, context: &str, percentage: u8) -> Result<AttributeStatisticsResponse, MispError> {
        let endpoint = format!("/attributes/attributeStatistics/{}/{}", context, percentage);
        self.misp_get(&endpoint).await
    }

    /// Get list of available attribute types, categories, and sane defaults (GET /attributes/describeTypes)
    pub async fn describe_attribute_types(&self) -> Result<DescribeTypesResult, MispError> {
        let wrapper: DescribeTypesWrapper = self.misp_get("/attributes/describeTypes").await?;
        Ok(wrapper.result)
    }

    /// Search for attributes with filters and pagination.
    /// Mirrors the /attributes/restSearch endpoint.
    pub async fn attributes_rest_search(&self, params: &AttributeRestSearchRequest) -> Result<AttributeListResponse, MispError> {
        self.misp_post("/attributes/restSearch", params).await
    }

    /// Fetch all events from the MISP instance (GET /events).
    /// Returns a vector of Event objects as per schema.
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails or the response cannot be deserialized.
    pub async fn get_events(&self) -> Result<Vec<Event>, anyhow::Error> {
        // Construct the endpoint URL
        let url = format!("{}/events", self.base_url);

        // Send the GET request with required headers
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?; // Return error if not 2xx
        // Deserialize the response body as a Vec<Event>
        let events = response.json::<Vec<Event>>().await?;

        Ok(events)
    }

    /// Get a single event by its ID from MISP.
    ///
    /// Corresponds to: GET /events/view/{eventId}
    /// # Arguments
    /// - `event_id`: Event ID or UUID (string, required)
    /// # Returns
    /// - `GetEventByIdResponse` wrapper (see types.rs)
    pub async fn get_event_by_id(&self, event_id: &str) -> Result<GetEventByIdResponse, MispError> {
        info!("Fetching event with ID: {}", event_id);
        let endpoint = format!("/events/view/{}", event_id);
        self.misp_get(&endpoint).await
    }

    /// Search for events using POST /events/index.
    /// Accepts an EventIndexRequest and returns a vector of Event objects.
    pub async fn search_events(&self, request: &EventIndexRequest) -> Result<Vec<Event>, MispError> {
        info!("Searching events with POST /events/index");
        self.misp_post("/events/index", request).await
    }

    /// Mirrors the /events/restSearch endpoint.
    /// Accepts an EventsRestSearchRequest and returns EventsRestSearchResponse.
    pub async fn events_rest_search(&self, params: &EventsRestSearchRequest) -> Result<EventsRestSearchResponse, MispError> {
        self.misp_post("/events/restSearch", params).await
    }

    /// Fetch a MISP Object by its numeric ID or UUID.
    /// Returns the full Object as defined in types.rs.
    pub async fn get_object_by_id(&self, object_id: &str) -> Result<Object, anyhow::Error> {
        // Build the endpoint URL
        let url = format!("{}/objects/view/{}", self.base_url, object_id);
        // Send GET request with authentication headers
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;

    // Parse the JSON response
    let json: serde_json::Value = response.json().await?;
    // Log the Object field for debugging
    // println!("DEBUG: json[\"Object\"] = {}", json["Object"]);
    // Extract the "Object" field and deserialize
    let object: Object = serde_json::from_value(json["Object"].clone())?;
    Ok(object)
    }

    /// Fetch a filtered and paginated list of objects using /objects/restsearch.
    /// Returns a vector of Object structs as per the official schema.
    pub async fn objects_rest_search(&self, params: &ObjectsRestSearchRequest) -> Result<Vec<Object>, anyhow::Error> {
        let url = format!("{}/objects/restsearch", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.api_key)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(params)
            .send()
            .await?
            .error_for_status()?;

        // The response is expected to be: { "response": [ { "Object": { ... } }, ... ] }
        let json: serde_json::Value = response.json().await?;
        let objects = json["response"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Missing 'response' array in /objects/restsearch response"))?
            .iter()
            .filter_map(|entry| entry.get("Object"))
            .map(|obj| serde_json::from_value(obj.clone()))
            .collect::<Result<Vec<Object>, _>>()?;

        Ok(objects)
    }

}
