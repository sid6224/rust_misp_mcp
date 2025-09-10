// types.rs
// Schema-driven MISP API types for get_users endpoint
// Based on official MISP API schema with response validation adjustments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

/// Custom deserializer for boolean fields that can be empty strings
/// Handles API inconsistency where some endpoints return "" instead of boolean values
fn deserialize_bool_or_empty_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Unexpected, Visitor};
    
    struct BoolOrStringVisitor;
    
    impl<'de> Visitor<'de> for BoolOrStringVisitor {
        type Value = Option<bool>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean, empty string, or boolean as string")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "" => Ok(None),
                "true" => Ok(Some(true)),
                "false" => Ok(Some(false)),
                "0" => Ok(Some(false)),
                "1" => Ok(Some(true)),
                _ => Err(de::Error::invalid_value(
                    Unexpected::Str(value),
                    &self,
                )),
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(BoolOrStringVisitor)
}

// User object based on official schema with clarifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID - string (UserId) <= 10 characters ^\d+$
    pub id: String,
    /// Organisation ID - string (OrganisationId) <= 10 characters ^\d+$
    #[serde(rename = "org_id")]
    pub org_id: String,
    /// Server ID - string (ServerId) <= 10 characters ^\d+$ (optional in get_user_by_id)
    #[serde(rename = "server_id")]
    pub server_id: Option<String>,
    /// Email address - string <email>
    pub email: String,
    /// Auto alert setting - boolean
    pub autoalert: bool,
    /// API auth key - string 40 characters Nullable (can be missing from response)
    #[serde(rename = "authkey")]
    pub authkey: Option<String>,
    /// Invited by user ID - string (UserId) <= 10 characters ^\d+$
    #[serde(rename = "invited_by")]
    pub invited_by: String,
    /// GPG key - string Nullable
    #[serde(rename = "gpgkey")]
    pub gpgkey: Option<String>,
    /// Certificate public - string Nullable
    #[serde(rename = "certif_public")]
    pub certif_public: Option<String>,
    /// NIDS SID - string <= 10 characters ^\d+$
    #[serde(rename = "nids_sid")]
    pub nids_sid: String,
    /// Terms accepted - boolean
    pub termsaccepted: bool,
    /// News read timestamp - string (Timestamp) ^\d+$ Default: "0"
    #[serde(rename = "newsread")]
    pub newsread: String,
    /// Role ID - string (RoleId) <= 10 characters ^\d+$
    #[serde(rename = "role_id")]
    pub role_id: String,
    /// Password change required - boolean (schema says string enum but response is boolean)
    #[serde(rename = "change_pw")]
    pub change_pw: bool,
    /// Contact alert - boolean
    pub contactalert: bool,
    /// Disabled - boolean
    pub disabled: bool,
    /// Expiration - string <date-time> Nullable
    pub expiration: Option<String>,
    /// Current login timestamp - string (Timestamp) ^\d+$ Default: "0"
    #[serde(rename = "current_login")]
    pub current_login: String,
    /// Last login timestamp - string (Timestamp) ^\d+$ Default: "0"
    #[serde(rename = "last_login")]
    pub last_login: String,
    /// Force logout - boolean
    #[serde(rename = "force_logout")]
    pub force_logout: bool,
    /// Date created - string (Timestamp) ^\d+$ Default: "0" (but can be null in response)
    #[serde(rename = "date_created")]
    pub date_created: Option<String>,
    /// Date modified - string (Timestamp) ^\d+$ Default: "0"
    #[serde(rename = "date_modified")]
    pub date_modified: String,
    /// Last API access - Extra field from response (not in schema)
    #[serde(rename = "last_api_access")]
    pub last_api_access: Option<String>,
    /// Last password change - Extra field from response (not in schema)
    #[serde(rename = "last_pw_change")]
    pub last_pw_change: Option<String>,
    /// Password - masked field (get_user_by_id only)
    pub password: Option<String>,
    /// Subject - external auth field (get_user_by_id only)
    pub sub: Option<String>,
    /// External auth required - boolean (get_user_by_id only)
    #[serde(rename = "external_auth_required")]
    pub external_auth_required: Option<bool>,
    /// External auth key - string (get_user_by_id only)
    #[serde(rename = "external_auth_key")]
    pub external_auth_key: Option<String>,
    /// Daily notification setting - boolean (get_user_by_id only)
    #[serde(rename = "notification_daily")]
    pub notification_daily: Option<bool>,
    /// Weekly notification setting - boolean (get_user_by_id only)
    #[serde(rename = "notification_weekly")]
    pub notification_weekly: Option<bool>,
    /// Monthly notification setting - boolean (get_user_by_id only)
    #[serde(rename = "notification_monthly")]
    pub notification_monthly: Option<bool>,
    /// TOTP secret - masked field (get_user_by_id only)
    pub totp: Option<String>,
    /// HOTP counter - integer (get_user_by_id only)
    #[serde(rename = "hotp_counter")]
    pub hotp_counter: Option<String>,
    /// TOTP is set - string (get_user_by_id only)
    #[serde(rename = "totp_is_set")]
    pub totp_is_set: Option<String>,
    /// Organization admins - array (get_user_by_id only)
    #[serde(rename = "orgAdmins")]
    pub org_admins: Option<Vec<serde_json::Value>>,
}

/// Response type for GET /taxonomies endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyEntry {
    #[serde(rename = "Taxonomy")]
    pub taxonomy: Taxonomy,
    pub total_count: i32,
    pub current_count: i32,
}

/// Response type for GET /taxonomies endpoint
pub type GetTaxonomiesResponse = Vec<TaxonomyEntry>;

// Role object based on official schema with all permissions optional
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role ID - string (RoleId) <= 10 characters ^\d+$
    pub id: String,
    /// Role name - string (RoleName) <= 255 characters
    pub name: String,
    /// Permission to add - boolean (optional in responses)
    #[serde(rename = "perm_add")]
    pub perm_add: Option<bool>,
    /// Permission to modify - boolean (optional in responses)
    #[serde(rename = "perm_modify")]
    pub perm_modify: Option<bool>,
    /// Permission to modify organization - boolean (optional in responses)
    #[serde(rename = "perm_modify_org")]
    pub perm_modify_org: Option<bool>,
    /// Permission to publish - boolean (optional in responses)
    #[serde(rename = "perm_publish")]
    pub perm_publish: Option<bool>,
    /// Permission to delegate - boolean (optional in responses)
    #[serde(rename = "perm_delegate")]
    pub perm_delegate: Option<bool>,
    /// Permission to sync - boolean (optional in responses)
    #[serde(rename = "perm_sync")]
    pub perm_sync: Option<bool>,
    /// Permission admin - boolean (optional in responses)
    #[serde(rename = "perm_admin")]
    pub perm_admin: Option<bool>,
    /// Permission audit - boolean (optional in responses)
    #[serde(rename = "perm_audit")]
    pub perm_audit: Option<bool>,
    /// Permission auth - boolean (optional in responses)
    #[serde(rename = "perm_auth")]
    pub perm_auth: Option<bool>,
    /// Permission site admin - boolean (optional in responses)
    #[serde(rename = "perm_site_admin")]
    pub perm_site_admin: Option<bool>,
    /// Permission regexp access - boolean (optional in responses)
    #[serde(rename = "perm_regexp_access")]
    pub perm_regexp_access: Option<bool>,
    /// Permission tagger - boolean (optional in responses)
    #[serde(rename = "perm_tagger")]
    pub perm_tagger: Option<bool>,
    /// Permission template - boolean (optional in responses)
    #[serde(rename = "perm_template")]
    pub perm_template: Option<bool>,
    /// Permission sharing group - boolean (optional in responses)
    #[serde(rename = "perm_sharing_group")]
    pub perm_sharing_group: Option<bool>,
    /// Permission tag editor - boolean (optional in responses)
    #[serde(rename = "perm_tag_editor")]
    pub perm_tag_editor: Option<bool>,
    /// Permission sighting - boolean (optional in responses)
    #[serde(rename = "perm_sighting")]
    pub perm_sighting: Option<bool>,
    /// Permission object template - boolean (optional in responses)
    #[serde(rename = "perm_object_template")]
    pub perm_object_template: Option<bool>,
    /// Permission publish ZMQ - boolean (optional in responses)
    #[serde(rename = "perm_publish_zmq")]
    pub perm_publish_zmq: Option<bool>,
    /// Permission publish Kafka - boolean (optional in responses)
    #[serde(rename = "perm_publish_kafka")]
    pub perm_publish_kafka: Option<bool>,
    /// Permission decaying - boolean (optional in responses)
    #[serde(rename = "perm_decaying")]
    pub perm_decaying: Option<bool>,
    /// Permission galaxy editor - boolean (optional in responses)
    #[serde(rename = "perm_galaxy_editor")]
    pub perm_galaxy_editor: Option<bool>,
    /// Default role - boolean (optional in responses)
    #[serde(rename = "default_role")]
    pub default_role: Option<bool>,
    /// Memory limit - string Nullable ^\d+$|^$ (missing from response)
    #[serde(rename = "memory_limit")]
    pub memory_limit: Option<String>,
    /// Max execution time - string Nullable ^\d+$|^$ (missing from response)
    #[serde(rename = "max_execution_time")]
    pub max_execution_time: Option<String>,
    /// Restricted to site admin - boolean (missing from response)
    #[serde(rename = "restricted_to_site_admin")]
    pub restricted_to_site_admin: Option<bool>,
    /// Enforce rate limit - boolean (missing from response)
    #[serde(rename = "enforce_rate_limit")]
    pub enforce_rate_limit: Option<bool>,
    /// Rate limit count - string ^\d+$ (missing from response)
    #[serde(rename = "rate_limit_count")]
    pub rate_limit_count: Option<String>,
    /// Permission - string ^\d+$ (missing from response)
    pub permission: Option<String>,
    /// Permission description - string (missing from response)
    #[serde(rename = "permission_description")]
    pub permission_description: Option<String>,
    /// Created timestamp - string (get_user_by_id only)
    pub created: Option<String>,
    /// Modified timestamp - string (get_user_by_id only)
    pub modified: Option<String>,
    /// Permission warninglist - boolean (get_user_by_id only)
    #[serde(rename = "perm_warninglist")]
    pub perm_warninglist: Option<bool>,
    /// Permission view feed correlations - boolean (get_user_by_id only)
    #[serde(rename = "perm_view_feed_correlations")]
    pub perm_view_feed_correlations: Option<bool>,
    /// Permission analyst data - boolean (get_user_by_id only)
    #[serde(rename = "perm_analyst_data")]
    pub perm_analyst_data: Option<bool>,
    /// Permission skip OTP - boolean (get_user_by_id only)
    #[serde(rename = "perm_skip_otp")]
    pub perm_skip_otp: Option<bool>,
    /// Permission server sign - boolean (get_user_by_id only)
    #[serde(rename = "perm_server_sign")]
    pub perm_server_sign: Option<bool>,
    /// Permission sync internal - boolean (get_user_by_id only)
    #[serde(rename = "perm_sync_internal")]
    pub perm_sync_internal: Option<bool>,
    /// Permission sync authoritative - boolean (get_user_by_id only)
    #[serde(rename = "perm_sync_authoritative")]
    pub perm_sync_authoritative: Option<bool>,
}

// Organisation object based on official schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organisation {
    /// Organisation ID - string (OrganisationId) <= 10 characters ^\d+$
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Organisation name - string (OrganisationName) <= 255 characters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Date created - string (get_user_by_id only)
    #[serde(rename = "date_created")]
    pub date_created: Option<String>,
    /// Date modified - string (get_user_by_id only)
    #[serde(rename = "date_modified")]
    pub date_modified: Option<String>,
    /// Description - string (get_user_by_id only)
    pub description: Option<String>,
    /// Type - string (get_user_by_id only)
    #[serde(rename = "type")]
    pub org_type: Option<String>,
    /// Nationality - string (get_user_by_id only)
    pub nationality: Option<String>,
    /// Sector - string (get_user_by_id only)
    pub sector: Option<String>,
    /// Created by - string (get_user_by_id only)
    #[serde(rename = "created_by")]
    pub created_by: Option<String>,
    /// UUID - string (get_user_by_id only)
    pub uuid: Option<String>,
    /// Contacts - string (get_user_by_id only)
    pub contacts: Option<String>,
    /// Local - boolean (get_user_by_id only) - API sometimes returns empty string instead of boolean
    #[serde(deserialize_with = "deserialize_bool_or_empty_string", default)]
    pub local: Option<bool>,
    /// Restricted to domain - array (get_user_by_id only)
    #[serde(rename = "restricted_to_domain")]
    pub restricted_to_domain: Option<Vec<String>>,
    /// Landing page - string (get_user_by_id only)
    pub landingpage: Option<String>,
    /// User count - string (get_organisations endpoint)
    #[serde(rename = "user_count")]
    pub user_count: Option<String>,
    /// Created by email - string (get_organisations endpoint)
    #[serde(rename = "created_by_email")]
    pub created_by_email: Option<String>,
}

// Server object from API response (not in schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server ID - optional
    pub id: Option<String>,
    /// Server name - optional
    pub name: Option<String>,
    /// Server URL - optional (get_user_by_id only)
    pub url: Option<String>,
    /// Push rules - optional (get_user_by_id only)
    #[serde(rename = "push_rules")]
    pub push_rules: Option<String>,
}

// Container for a single user entry in get_users response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntry {
    /// User object
    #[serde(rename = "User")]
    pub user: User,
    /// Role object
    #[serde(rename = "Role")]
    pub role: Role,
    /// Organisation object
    #[serde(rename = "Organisation")]
    pub organisation: Organisation,
    /// Server object (extra from response, not in schema)
    #[serde(rename = "Server")]
    pub server: Option<ServerInfo>,
}

/// Response type for GET /admin/users endpoint
pub type GetUsersResponse = Vec<UserEntry>;

// UserSetting types for get_user_by_id endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserSetting {
    /// Empty array case
    Array(Vec<serde_json::Value>),
    /// Object case with user settings
    Object(UserSettingObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingObject {
    /// Publish alert filter - complex object
    #[serde(rename = "publish_alert_filter")]
    pub publish_alert_filter: Option<Vec<serde_json::Value>>,
    /// Dashboard access - boolean
    #[serde(rename = "dashboard_access")]
    pub dashboard_access: Option<bool>,
    /// Dashboard widgets - array
    pub dashboard: Option<Vec<DashboardWidget>>,
    /// Homepage settings - object
    pub homepage: Option<Homepage>,
    /// Default REST search parameters - object
    #[serde(rename = "default_restsearch_parameters")]
    pub default_restsearch_parameters: Option<Vec<serde_json::Value>>,
    /// Tag numerical value override - object
    #[serde(rename = "tag_numerical_value_override")]
    pub tag_numerical_value_override: Option<Vec<serde_json::Value>>,
    /// Event index hide columns - array
    #[serde(rename = "event_index_hide_columns")]
    pub event_index_hide_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    /// Widget type
    pub widget: String,
    /// Widget position
    pub position: WidgetPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    /// X coordinate
    pub x: String,
    /// Y coordinate  
    pub y: String,
    /// Width
    pub width: String,
    /// Height
    pub height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Homepage {
    /// Homepage path
    pub path: String,
}

/// Response type for GET /admin/users/view/{id} endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserByIdResponse {
    // Flat fields at root level (optional as per user requirement)
    /// User ID - string (optional at root)
    pub id: Option<String>,
    /// Organisation ID - string (optional at root)
    #[serde(rename = "org_id")]
    pub org_id: Option<String>,
    /// Server ID - string (optional at root)
    #[serde(rename = "server_id")]
    pub server_id: Option<String>,
    /// Email address - string (optional at root)
    pub email: Option<String>,
    /// Auto alert setting - boolean (optional at root)
    pub autoalert: Option<bool>,
    /// API auth key - string (optional at root)
    pub authkey: Option<String>,
    /// Invited by user ID - string (optional at root)
    #[serde(rename = "invited_by")]
    pub invited_by: Option<String>,
    /// GPG key - string (optional at root)
    pub gpgkey: Option<String>,
    /// Certificate public - string (optional at root)
    #[serde(rename = "certif_public")]
    pub certif_public: Option<String>,
    /// NIDS SID - string (optional at root)
    #[serde(rename = "nids_sid")]
    pub nids_sid: Option<String>,
    /// Terms accepted - boolean (optional at root)
    pub termsaccepted: Option<bool>,
    /// News read timestamp - string (optional at root)
    pub newsread: Option<String>,
    /// Role ID - string (optional at root)
    #[serde(rename = "role_id")]
    pub role_id: Option<String>,
    /// Password change required - string enum (optional at root)
    #[serde(rename = "change_pw")]
    pub change_pw: Option<String>,
    /// Contact alert - boolean (optional at root)
    pub contactalert: Option<bool>,
    /// Disabled - boolean (optional at root)
    pub disabled: Option<bool>,
    /// Expiration - string (optional at root)
    pub expiration: Option<String>,
    /// Current login timestamp - string (optional at root)
    #[serde(rename = "current_login")]
    pub current_login: Option<String>,
    /// Last login timestamp - string (optional at root)
    #[serde(rename = "last_login")]
    pub last_login: Option<String>,
    /// Force logout - boolean (optional at root)
    #[serde(rename = "force_logout")]
    pub force_logout: Option<bool>,
    /// Date created - string (optional at root)
    #[serde(rename = "date_created")]
    pub date_created: Option<String>,
    /// Date modified - string (optional at root)
    #[serde(rename = "date_modified")]
    pub date_modified: Option<String>,
    
    // Nested objects (always present)
    /// User object
    #[serde(rename = "User")]
    pub user: User,
    /// Role object
    #[serde(rename = "Role")]
    pub role: Role,
    /// User settings - can be array or object
    #[serde(rename = "UserSetting")]
    pub user_setting: UserSetting,
    /// Organisation object
    #[serde(rename = "Organisation")]
    pub organisation: Organisation,
    /// Server object
    #[serde(rename = "Server")]
    pub server: Option<ServerInfo>,
}

// =============================================================================
// Galaxy Types for get_galaxies endpoint
// =============================================================================

/// Galaxy object based on official schema with extra fields from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Galaxy {
    /// Galaxy ID - string (GalaxyId) <= 10 characters ^\d+$
    pub id: String,
    /// Galaxy UUID - string <uuid>
    pub uuid: String,
    /// Galaxy name - string (GalaxyName) <= 255 characters
    pub name: String,
    /// Galaxy type - string (GalaxyType) <= 255 characters
    #[serde(rename = "type")]
    pub galaxy_type: String,
    /// Galaxy description - string (GalaxyDescription) <= 65535 characters
    pub description: String,
    /// Galaxy version - string (GalaxyVersion) <= 255 characters
    pub version: String,
    /// Icon - string (Icon) <= 255 characters (optional - not present in all endpoints)
    pub icon: Option<String>,
    /// Namespace - string (GalaxyNamespace) <= 255 characters
    pub namespace: String,
    /// Kill chain order - object Nullable (dynamic structure)
    pub kill_chain_order: Option<serde_json::Value>,
    
    // Extra fields found in API response (all optional)
    /// Whether galaxy is enabled
    pub enabled: Option<bool>,
    /// Whether galaxy is local only
    pub local_only: Option<bool>,
    /// Whether galaxy is default
    pub default: Option<bool>,
    /// Organisation ID - string
    pub org_id: Option<String>,
    /// Organisation Creator ID - string
    pub orgc_id: Option<String>,
    /// Creation timestamp
    pub created: Option<String>,
    /// Modification timestamp
    pub modified: Option<String>,
    /// Distribution level
    pub distribution: Option<String>,
}

/// Galaxy entry wrapper for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyEntry {
    /// Galaxy object
    #[serde(rename = "Galaxy")]
    pub galaxy: Galaxy,
}

/// Response type for get_galaxies endpoint
pub type GetGalaxiesResponse = Vec<GalaxyEntry>;

// =============================================================================
// Search Galaxies Types for POST /galaxies endpoint
// =============================================================================

/// Request payload for POST /galaxies (search galaxies) endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchGalaxiesRequest {
    /// Search value - string filter for galaxy matching
    pub value: String,
}

/// Response type for POST /galaxies (search galaxies) endpoint
/// Reuses existing GalaxyEntry structure - identical response format to get_galaxies
pub type SearchGalaxiesResponse = GetGalaxiesResponse;

// =============================================================================
// Galaxy By ID Types for get_galaxy_by_id endpoint
// =============================================================================

/// Type aliases for Organisation reuse in get_galaxy_by_id
pub type Org = Organisation;
pub type Orgc = Organisation;

/// Tag object for galaxy clusters - Extended version with all API response fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag ID - string
    pub id: Option<String>,
    /// Tag name - string
    pub name: Option<String>,
    /// Tag colour - string (hex color)
    pub colour: Option<String>,
    /// Whether tag is exportable - boolean
    #[serde(deserialize_with = "deserialize_bool_or_empty_string")]
    pub exportable: Option<bool>,
    /// Whether tag is user ID - boolean (API sometimes returns string values like "0")
    #[serde(deserialize_with = "deserialize_bool_or_empty_string")]
    pub user_id: Option<bool>,
    /// Whether tag hides tag - boolean (can be null/empty in API response)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string")]
    pub hide_tag: Option<bool>,
    /// Numerical value - integer (can be null/empty in API response)
    pub numerical_value: Option<String>,
    /// Whether tag is favourite - boolean (can be null/empty in API response)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string", default)]
    pub is_favourite: Option<bool>,
    /// Whether tag is custom galaxy - boolean (can be null/empty in API response)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string", default)]
    pub is_custom_galaxy: Option<bool>,
    /// Whether tag is galaxy - boolean (can be null/empty in API response)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string", default)]
    pub is_galaxy: Option<bool>,
    /// Local only flag - boolean (can be null/empty in API response)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string", default)]
    pub local_only: Option<bool>,
    /// Organisation ID - string (can be null/empty in API response)
    pub org_id: Option<String>,
    /// Count - integer (can be null/empty in API response)
    pub count: Option<i32>,
    /// Attribute count - integer (can be null/empty in API response)
    pub attribute_count: Option<i32>,
    /// Favourite flag - boolean (get_tags endpoint specific field)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string", default)]
    pub favourite: Option<bool>,
    /// Inherited value - integer (from schema, optional as not present in all responses)
    pub inherited: Option<i32>,
}

/// Galaxy element object - Each galaxy element represents a single attribute key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyElement {
    /// Element ID - string (GalaxyElementId) <= 10 characters ^\d+$
    pub id: String,
    /// Galaxy cluster ID - string (GalaxyClusterId) <= 10 characters ^\d+$
    #[serde(rename = "galaxy_cluster_id")]
    pub galaxy_cluster_id: String,
    /// Element key - string (GalaxyElementKey) <= 255 characters
    pub key: String,
    /// Element value - string (GalaxyElementValue) <= 65535 characters
    pub value: String,
}

/// Galaxy cluster relation object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyClusterRelation {
    /// Relation ID - string
    pub id: String,
    /// Source galaxy cluster ID - string
    #[serde(rename = "galaxy_cluster_id")]
    pub galaxy_cluster_id: String,
    /// Source galaxy cluster UUID - string
    #[serde(rename = "galaxy_cluster_uuid")]
    pub galaxy_cluster_uuid: String,
    /// Target galaxy cluster UUID - string
    #[serde(rename = "referenced_galaxy_cluster_uuid")]
    pub referenced_galaxy_cluster_uuid: String,
}

/// Targeting cluster relation object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingClusterRelation {
    /// Relation ID - string
    pub id: String,
    /// Source galaxy cluster ID - string
    #[serde(rename = "galaxy_cluster_id")]
    pub galaxy_cluster_id: String,
    /// Referenced galaxy cluster ID - string  
    #[serde(rename = "referenced_galaxy_cluster_id")]
    pub referenced_galaxy_cluster_id: String,
    /// Target galaxy cluster UUID - string
    #[serde(rename = "referenced_galaxy_cluster_uuid")]
    pub referenced_galaxy_cluster_uuid: String,
    /// Reference type - string
    #[serde(rename = "referenced_galaxy_cluster_type")]
    pub referenced_galaxy_cluster_type: String,
    /// Source galaxy cluster UUID - string
    #[serde(rename = "galaxy_cluster_uuid")]
    pub galaxy_cluster_uuid: String,
    /// Distribution level - string
    pub distribution: String,
    /// Sharing group ID - string (nullable)
    #[serde(rename = "sharing_group_id")]
    pub sharing_group_id: Option<String>,
    /// Whether it's default - boolean
    pub default: bool,
    /// Tag array - array of tags
    #[serde(rename = "Tag")]
    pub tag: Option<Vec<Tag>>,
}

/// Galaxy cluster object with all nested arrays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyCluster {
    /// Cluster ID - string
    pub id: String,
    /// Cluster UUID - string
    pub uuid: String,
    /// Collection UUID - string
    #[serde(rename = "collection_uuid")]
    pub collection_uuid: String,
    /// Cluster type - string
    #[serde(rename = "type")]
    pub cluster_type: String,
    /// Cluster value - string
    pub value: String,
    /// Tag name - string
    #[serde(rename = "tag_name")]
    pub tag_name: String,
    /// Description - string
    pub description: String,
    /// Galaxy ID - string
    #[serde(rename = "galaxy_id")]
    pub galaxy_id: String,
    /// Source - string
    pub source: String,
    /// Authors - array of strings
    pub authors: Vec<String>,
    /// Version - string
    pub version: String,
    /// Distribution - string
    pub distribution: String,
    /// Sharing group ID - string (nullable)
    #[serde(rename = "sharing_group_id")]
    pub sharing_group_id: Option<String>,
    /// Organisation ID - string
    #[serde(rename = "org_id")]
    pub org_id: String,
    /// Organisation creator ID - string
    #[serde(rename = "orgc_id")]
    pub orgc_id: String,
    /// Cluster extends UUID - string (nullable)
    #[serde(rename = "extends_uuid")]
    pub extends_uuid: Option<String>,
    /// Cluster extends version - string
    #[serde(rename = "extends_version")]
    pub extends_version: String,
    /// Published - boolean
    pub published: bool,
    /// Deleted - boolean
    pub deleted: bool,
    /// Locked - boolean (found in get_galaxy_clusters response)
    pub locked: Option<bool>,
    /// Default - boolean (found in get_galaxy_clusters response) 
    pub default: Option<bool>,
    
    // Nested arrays (optional - not present in all endpoints)
    /// Galaxy elements array
    #[serde(rename = "GalaxyElement")]
    pub galaxy_element: Option<Vec<GalaxyElement>>,
    /// Galaxy cluster relations array
    #[serde(rename = "GalaxyClusterRelation")]
    pub galaxy_cluster_relation: Option<Vec<GalaxyClusterRelation>>,
    /// Targeting cluster relations array (optional - may be omitted by API)
    #[serde(rename = "TargetingClusterRelation")]
    pub targeting_cluster_relation: Option<Vec<TargetingClusterRelation>>,
    /// Relationship inbound - array (extra field from API response, not in schema)
    #[serde(rename = "RelationshipInbound")]
    pub relationship_inbound: Option<Vec<serde_json::Value>>,
}

/// Response type for GET /galaxies/view/{id} endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGalaxyByIdResponse {
    /// Galaxy object
    #[serde(rename = "Galaxy")]
    pub galaxy: Galaxy,
    /// Organisation object
    #[serde(rename = "Org")]
    pub org: Org,
    /// Organisation creator object
    #[serde(rename = "Orgc")]
    pub orgc: Orgc,
    /// Galaxy clusters array
    #[serde(rename = "GalaxyCluster")]
    pub galaxy_cluster: Vec<GalaxyCluster>,
}

// =============================================================================
// Galaxy Clusters Types for get_galaxy_clusters endpoint  
// =============================================================================

/// Galaxy cluster entry for get_galaxy_clusters endpoint response
/// Contains both GalaxyCluster and Galaxy objects per entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyClusterEntry {
    /// Galaxy cluster object (simplified version without nested arrays)
    #[serde(rename = "GalaxyCluster")]
    pub galaxy_cluster: GalaxyCluster,
    /// Galaxy object associated with this cluster
    #[serde(rename = "Galaxy")]
    pub galaxy: Galaxy,
}

/// Response type for GET /galaxy_clusters/index/{galaxyId} endpoint
/// Returns array of galaxy cluster entries, each containing both cluster and galaxy data
pub type GetGalaxyClustersResponse = Vec<GalaxyClusterEntry>;

// =============================================================================
// Galaxy Cluster By ID Types for get_galaxy_cluster_by_id endpoint  
// =============================================================================

/// Response type for GET /galaxy_clusters/view/{galaxyClusterId} endpoint
/// Returns detailed galaxy cluster with associated tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGalaxyClusterByIdResponse {
    /// Galaxy cluster object with full details
    #[serde(rename = "GalaxyCluster")]
    pub galaxy_cluster: GalaxyCluster,
    /// Tag object associated with this cluster
    #[serde(rename = "Tag", default)]
    pub tag: Option<Tag>,
}

// =============================================================================
// Search Galaxy Clusters Types for search_galaxy_clusters endpoint  
// =============================================================================

/// Request payload for POST /galaxy_clusters/index/{galaxyId} (search galaxy clusters) endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchGalaxyClustersRequest {
    /// Search context - enum
    pub context: String, // Could be enum but keeping as String for flexibility
    /// Search term - string filter for cluster matching
    pub searchall: String,
}

/// Response type for POST /galaxy_clusters/index/{galaxyId} (search galaxy clusters) endpoint
/// Returns filtered array of galaxy clusters matching search criteria
pub type SearchGalaxyClustersResponse = Vec<GalaxyClusterEntry>;

// =============================================================================
// Organisations Types for get_organisations endpoint  
// =============================================================================

/// Organisation entry wrapper for get_organisations API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganisationEntry {
    /// Organisation object
    #[serde(rename = "Organisation")]
    pub organisation: Organisation,
}

/// Response type for GET /organisations endpoint
pub type GetOrganisationsResponse = Vec<OrganisationEntry>;

// =============================================================================
// Tags Types for get_tags endpoint  
// =============================================================================

/// Wrapper for the GET /tags endpoint response
/// MISP returns tags wrapped in a "Tag" key: {"Tag": [array_of_tags]}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTagsResponseWrapper {
    #[serde(rename = "Tag")]
    pub tag: Vec<Tag>,
}

/// Response type for GET /tags endpoint
/// Returns array of Tag objects wrapped in a "Tag" key
pub type GetTagsResponse = GetTagsResponseWrapper;

// =============================================================================
// Search Tags Types for search_tags endpoint  
// =============================================================================

/// Search tags response entry - combines Tag with optional Taxonomy and TaxonomyPredicate
/// Handles MISP's variable response structure where entries can contain different combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTagEntry {
    /// Tag object (always present)
    #[serde(rename = "Tag")]
    pub tag: Tag,
    /// Taxonomy object (optional - present when tag is associated with a taxonomy)
    #[serde(rename = "Taxonomy")]
    pub taxonomy: Option<Taxonomy>,
    /// Taxonomy predicate object (optional - present when tag has a predicate)
    #[serde(rename = "TaxonomyPredicate")]
    pub taxonomy_predicate: Option<TaxonomyPredicate>,
}

/// Taxonomy object for search_tags endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taxonomy {
    /// Taxonomy ID - string
    pub id: String,
    /// Taxonomy namespace - string
    pub namespace: String,
    /// Taxonomy description - string
    pub description: String,
    /// Taxonomy version - string
    pub version: Option<String>,
    /// Whether taxonomy is enabled - boolean
    pub enabled: Option<bool>,
    /// Whether taxonomy is exclusive - boolean
    pub exclusive: Option<bool>,
    /// Whether taxonomy is required - boolean
    pub required: Option<bool>,
    /// Highlighted terms - can be boolean or string (API inconsistency)
    #[serde(deserialize_with = "deserialize_bool_or_empty_string")]
    pub highlighted: Option<bool>,
}
/// Entry type for get_taxonomy_by_id endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyEntryById {
    pub tag: String,
    pub expanded: String,
    pub description: String,
    pub exclusive_predicate: bool,
    pub existing_tag: bool,
}

/// Extended entry type for get_taxonomy_extended_with_tags endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyExtendedEntryById {
    pub tag: String,
    pub expanded: String,
    pub description: String,
    pub exclusive_predicate: bool,
    pub existing_tag: bool,
    pub events: i32,
    pub attributes: i32,
    // Optional user fields for future-proofing
    pub org_id: Option<String>,
    pub server_id: Option<String>,
    pub email: Option<String>,
    pub autoalert: Option<bool>,
    pub authkey: Option<String>,
    pub invited_by: Option<String>,
    pub gpgkey: Option<String>,
    pub certif_public: Option<String>,
    pub nids_sid: Option<String>,
    pub termsaccepted: Option<bool>,
    pub newsread: Option<String>,
    pub role_id: Option<String>,
    pub change_pw: Option<String>,
    pub contactalert: Option<bool>,
    pub disabled: Option<bool>,
    pub expiration: Option<String>,
    pub current_login: Option<String>,
    pub last_login: Option<String>,
    pub force_logout: Option<bool>,
    pub date_created: Option<String>,
    pub date_modified: Option<String>,
}

/// Response type for GET /taxonomies/view/{id} endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaxonomyByIdResponse {
    #[serde(rename = "Taxonomy")]
    pub taxonomy: Taxonomy,
    #[serde(rename = "entries")]
    pub entries: Vec<TaxonomyEntryById>,
}

/// Response type for GET /taxonomies/taxonomy_tags/{id} endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaxonomyExtendedWithTagsResponse {
    #[serde(rename = "Taxonomy")]
    pub taxonomy: Taxonomy,
    #[serde(rename = "entries")]
    pub entries: Vec<TaxonomyExtendedEntryById>,
}

/// Taxonomy predicate object for search_tags endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyPredicate {
    /// Predicate ID - string
    pub id: String,
    /// Taxonomy ID - string
    #[serde(rename = "taxonomy_id")]
    pub taxonomy_id: String,
    /// Predicate value - string
    pub value: String,
    /// Expanded value - string (optional)
    pub expanded: Option<String>,
    /// Colour - string (optional hex color)
    pub colour: Option<String>,
    /// Description - string (optional)
    pub description: Option<String>,
    /// Whether predicate is exclusive - boolean (optional)
    pub exclusive: Option<bool>,
    /// Numerical value - integer (optional)
    pub numerical_value: Option<i32>,
}

/// Response type for GET /tags/search/{tagSearchTerm} endpoint
/// Returns array of search tag entries with variable structure
pub type SearchTagsResponse = Vec<SearchTagEntry>;


/// Sighting object for MISP get_sightings_by_EventId endpoint
/// All fields are optional to handle incomplete or partial API responses
/// Schema-driven, future-proof, and robust for evolving MISP API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sighting {
    /// Sighting ID - string (SightingId) <= 10 characters ^\d+$
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Sighting UUID - string <uuid>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Event ID associated with sighting - string (EventId)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Attribute ID associated with sighting - string (AttributeId)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribute_id: Option<String>,
    /// Organisation ID - string (OrganisationId)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,
    /// Date of sighting - string (Timestamp)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_sighting: Option<String>,
    /// Source of sighting - string (free text)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Sighting type - string (free text, e.g., "0", "1", "false", "true")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    /// Organisation object (nested) - optional, future-proof
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "Organisation")]
    pub organisation: Option<Organisation>,
}


/// Wrapper for get_sightings_by_EventId response
/// Contains an optional vector of Sighting objects
/// Designed for compatibility with MISP API and robust deserialization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetSightingsResponse {
    /// Array of sightings (can be missing or null in API response)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sightings: Option<Vec<Sighting>>,
}


// =============================================================================
// Noticelists Types for GET /noticelists endpoint
// =============================================================================

/// Top-level response for GET /noticelists endpoint
pub type NoticelistsResponse = Vec<NoticelistContainer>;

/// Container for each noticelist object
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NoticelistContainer {
    /// The actual noticelist data
    #[serde(rename = "Noticelist")]
    pub noticelist: Noticelist,
}

/// Represents a single Noticelist and its metadata.
/// Inclusive of both official schema and current actual response fields.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Noticelist {
    /// Unique identifier for the noticelist (<= 10 chars, digits)
    pub id: String,
    /// Name of the noticelist
    pub name: String,
    /// Type of the noticelist (e.g., "cidr", "hostname", "substring", "string", "regex")
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    /// Description of the noticelist
    #[serde(default)]
    pub description: Option<String>,
    /// Version string (digits)
    #[serde(default)]
    pub version: Option<String>,
    /// Whether the noticelist is enabled
    pub enabled: bool,
    /// Number of entries in the noticelist (digits, optional)
    #[serde(default)]
    pub warninglist_entry_count: Option<String>,
    /// Valid attribute types for this noticelist (comma-separated string, optional)
    #[serde(default)]
    pub valid_attributes: Option<String>,
    /// Array of NoticelistEntry objects (optional, may be missing in some responses)
    #[serde(default, rename = "NoticelistEntry")]
    pub noticelist_entry: Option<Vec<NoticelistEntry>>,
    // --- Fields present in actual response, retained for compatibility ---
    /// Expanded name (full description, optional)
    #[serde(default)]
    pub expanded_name: Option<String>,
    /// Reference URLs (array of strings, optional)
    #[serde(default, rename = "ref")]
    pub ref_: Option<Vec<String>>,
    /// Geographical area (array of strings, optional)
    #[serde(default)]
    pub geographical_area: Option<Vec<String>>,
}

// -----------------------------------------------------------------------------
// Wrapper for GET /noticelists/view/{noticelistId} endpoint response
// -----------------------------------------------------------------------------
/// Top-level response for GET /noticelists/view/{noticelistId}
/// Contains a single Noticelist object under the "Noticelist" key.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NoticelistByIdResponse {
    /// The actual Noticelist object returned by MISP
    #[serde(rename = "Noticelist")]
    pub noticelist: Noticelist,
}

/// Represents a single entry in a NoticelistEntry.
/// Contains entry metadata and nested data object.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NoticelistEntry {
    /// Entry ID (digits, optional)
    #[serde(default)]
    pub id: Option<String>,
    /// Associated noticelist ID (<= 10 chars, digits, optional)
    #[serde(default)]
    pub noticelist_id: Option<String>,
    /// Nested data object containing entry details (optional)
    #[serde(default)]
    pub data: Option<NoticelistEntryData>,
}

/// Nested data object for a NoticelistEntry.
/// Contains scope, field, value, tags, and message.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NoticelistEntryData {
    /// Scope of the entry (array of strings, optional)
    #[serde(default)]
    pub scope: Option<Vec<String>>,
    /// Field(s) associated with the entry (array of strings, optional)
    #[serde(default)]
    pub field: Option<Vec<String>>,
    /// Value(s) for the entry (array of strings, optional)
    #[serde(default)]
    pub value: Option<Vec<String>>,
    /// Tags associated with the entry (array of strings, optional)
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// Message object containing localized strings (optional)
    #[serde(default)]
    pub message: Option<NoticelistEntryMessage>,
}

/// Message object for NoticelistEntryData.
/// Contains localized message strings.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NoticelistEntryMessage {
    /// English message (optional)
    #[serde(default)]
    pub en: Option<String>,
}

// =============================================================================
// Warninglists Types for GET /warninglists endpoint
// =============================================================================

/// Top-level response for GET /warninglists endpoint
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WarninglistsResponse {
    /// List of warninglist containers returned by MISP
    #[serde(rename = "Warninglists")]
    pub warninglists: Vec<WarninglistContainer>,
}

/// Container for each warninglist object
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WarninglistContainer {
    /// The actual warninglist data
    #[serde(rename = "Warninglist")]
    pub warninglist: Warninglist,
}

/// Represents a single warninglist and its metadata
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Warninglist {
    /// Unique identifier for the warninglist
    pub id: String,
    /// Name of the warninglist
    pub name: String,
    /// Type of the warninglist (e.g., "cidr", "string", "hostname", etc.)
    #[serde(rename = "type")]
    pub type_: String,
    /// Description of the warninglist
    pub description: String,
    /// Version string (may be a date or integer)
    pub version: String,
    /// Whether the warninglist is enabled
    pub enabled: bool,
    /// Default flag (optional, may be missing in some responses)
    #[serde(default)]
    pub default: Option<bool>,
    /// Category of the warninglist (optional)
    #[serde(default)]
    pub category: Option<String>,
    /// Number of entries in the warninglist (optional, as string)
    #[serde(default)]
    pub warninglist_entry_count: Option<String>,
    /// Valid attribute types for this warninglist (optional, comma-separated string)
    #[serde(default)]
    pub valid_attributes: Option<String>,
    #[serde(rename = "WarninglistEntry", skip_serializing_if = "Option::is_none")]
    pub warninglist_entry: Option<Vec<WarninglistEntry>>,
    // List of attribute types that this warninglist applies to.
    // Uses API key "WarninglistType" for correct mapping.
    #[serde(rename = "WarninglistType", skip_serializing_if = "Option::is_none")]
    pub warninglist_type: Option<Vec<WarninglistType>>,

}

/// Placeholder for future warninglist entry objects
/// Not present in current /warninglists response, but included for schema compatibility
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WarninglistEntry {
    /// Entry ID (optional, future use)
    #[serde(default)]
    pub id: Option<String>,
    /// Entry value (optional, future use)
    #[serde(default)]
    pub value: Option<String>,
    /// Associated warninglist ID (optional, future use)
    #[serde(default)]
    pub warninglist_id: Option<String>,
    /// Comment or note associated with the warninglist entry (optional)
    #[serde(default)]
    pub comment: Option<String>,
}

// Describes the valid attribute types for a warninglist, such as "ip-src" or "domain|ip".
// Used to indicate which types of data the warninglist applies to.
// All fields are optional for robust deserialization.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WarninglistType {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub warninglist_id: Option<String>,
}

/// Request payload for POST /warninglists (search warninglists) endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWarninglistRequest {
    /// Search value - string filter for warninglist matching
    pub value: String,
}

/// EventReport entry as returned by /eventReports/index.
/// This struct matches the live API response and is future-proofed for optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReportEntry {
    #[serde(rename = "EventReport")]
    pub event_report: EventReport,
}

/// Main EventReport object, strictly as per schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReport {
    pub id: String,
    pub uuid: String,
    pub event_id: String,
    pub name: Option<String>,
    pub content: Option<String>,
    pub distribution: Option<String>,
    pub sharing_group_id: Option<String>,
    pub timestamp: Option<String>,
    pub deleted: Option<bool>,
    #[serde(rename = "Event", default)]
    pub event: Option<EventReportEvent>,
    #[serde(rename = "SharingGroup", default)]
    pub sharing_group: Option<EventReportSharingGroup>,
    #[serde(rename = "RelationshipInbound", default)]
    pub relationship_inbound: Option<Vec<EventReportRelationshipInbound>>,
}

/// Event metadata, always present in response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReportEvent {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub orgc_id: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub info: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(rename = "Org", default)]
    pub org: Option<OrgInfoMinimal>,
    #[serde(rename = "Orgc", default)]
    pub orgc: Option<OrgInfoMinimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInfoMinimal {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
}

/// SharingGroup info, always present, fields nullable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReportSharingGroup {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub uuid: Option<String>,
}

/// RelationshipInbound, always present as array (usually empty).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReportRelationshipInbound {
    // No fields currently observed; placeholder for future compatibility.
}

/// Wrapper for GET /collections/view/{collection_id} response.
/// The API returns: { "Collection": { ... } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCollectionByIdResponse {
    #[serde(rename = "Collection", default)]
    pub collection: Collection,
}

/// Represents a Collection object as returned by the MISP API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collection {
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(rename = "org_id")]
    pub org_id: Option<String>,
    #[serde(rename = "orgc_id")]
    pub orgc_id: Option<String>,
    #[serde(rename = "uuid")]
    pub uuid: Option<String>,
    #[serde(rename = "created")]
    pub created: Option<String>,
    #[serde(rename = "modified")]
    pub modified: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>, // e.g. "campaign", "intrusion_set", etc.
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "description")]
    pub description: Option<String>,
    #[serde(rename = "distribution")]
    pub distribution: Option<String>, // "0" to "5"
    #[serde(rename = "sharing_group_id")]
    pub sharing_group_id: Option<String>,
    #[serde(rename = "Org", default)]
    pub org: Option<OrgInfoMinimal>,
    #[serde(rename = "Orgc", default)]
    pub orgc: Option<OrgInfoMinimal>,
    #[serde(rename = "CollectionElement", default)]
    pub collection_element: Option<Vec<CollectionElement>>,
}

/// Represents a CollectionElement object within a Collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionElement {
    #[serde(rename = "id")]
    pub id: Option<String>,
    #[serde(rename = "uuid")]
    pub uuid: Option<String>,
    #[serde(rename = "collection_id")]
    pub collection_id: Option<String>,
    #[serde(rename = "element_uuid")]
    pub element_uuid: Option<String>,
    #[serde(rename = "element_type")]
    pub element_type: Option<String>, // "Event" or "GalaxyCluster"
    #[serde(rename = "description")]
    pub description: Option<String>,
}

/// Request body for POST /collections/index/{filter}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFilterBody {
    #[serde(rename = "Collection.uuid", skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(rename = "Collection.type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(rename = "Collection.name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Accepts both string and integer for distribution field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnalystDistribution {
    String(String),
    //Int(i32),
}

/// AnalystNote object for analyst data endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystNote {
    pub note: Option<String>,
    pub language: Option<String>,
    pub note_type_name: Option<String>, // Always "Note"
    pub uuid: Option<String>,
    pub object_uuid: Option<String>,
    pub object_type: Option<String>,
    pub authors: Option<String>, // May be a single string or comma-separated
    pub org_uuid: Option<String>,
    pub orgc_uuid: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub distribution: Option<AnalystDistribution>,
    pub sharing_group_id: Option<String>,
    pub locked: Option<bool>,
}

/// AnalystOpinion object for analyst data endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystOpinion {
    pub comment: Option<String>,
    pub opinion: Option<i32>,
    pub note_type_name: Option<String>, // Always "Opinion"
    pub uuid: Option<String>,
    pub object_uuid: Option<String>,
    pub object_type: Option<String>,
    pub authors: Option<String>,
    pub org_uuid: Option<String>,
    pub orgc_uuid: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub distribution: Option<AnalystDistribution>,
    pub sharing_group_id: Option<String>,
    pub locked: Option<bool>,
}

/// AnalystRelationship object for analyst data endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystRelationship {
    pub related_object_uuid: Option<String>,
    pub related_object_type: Option<String>,
    pub relationship_type: Option<String>,
    pub note_type_name: Option<String>, // Always "Relationship"
    pub uuid: Option<String>,
    pub object_uuid: Option<String>,
    pub object_type: Option<String>,
    pub authors: Option<String>,
    pub org_uuid: Option<String>,
    pub orgc_uuid: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub distribution: Option<AnalystDistribution>,
    pub sharing_group_id: Option<String>,
    pub locked: Option<bool>,
}

/// Top-level enum for analyst data array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnalystData {
    Note(AnalystNote),
    Opinion(AnalystOpinion),
    Relationship(AnalystRelationship),
}

/// Attribute object for /attributes endpoint (schema + observed data + future compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute ID - string (AttributeId)
    pub id: String,
    /// Event ID - string (EventId)
    pub event_id: String,
    /// Object ID - string (ObjectId)
    pub object_id: String,
    /// Object relation - string (NullableObjectRelation)
    pub object_relation: Option<String>,
    /// Category - string (AttributeCategory)
    pub category: String,
    /// Type - string (AttributeType)
    #[serde(rename = "type")]
    pub attribute_type: String,
    /// Value - string (AttributeValue)
    pub value: String,
    /// Value1 - string (present in data, not schema)
    pub value1: Option<String>,
    /// Value2 - string (present in data, not schema)
    pub value2: Option<String>,
    /// To IDS - boolean (ToIDS)
    pub to_ids: bool,
    /// UUID - string (UUID)
    pub uuid: String,
    /// Timestamp - string (NullableTimestamp)
    pub timestamp: Option<String>,
    /// Distribution - string (DistributionLevelId)
    pub distribution: String,
    /// Sharing group ID - string (SharingGroupId)
    pub sharing_group_id: Option<String>,
    /// Comment - string (AttributeComment)
    pub comment: Option<String>,
    /// Deleted - boolean (SoftDeletedFlag)
    pub deleted: bool,
    /// Disable correlation - boolean (DisableCorrelationFlag)
    pub disable_correlation: bool,
    /// First seen - string (nullable)
    pub first_seen: Option<String>,
    /// Last seen - string (nullable)
    pub last_seen: Option<String>,
    /// Event UUID - string (present in /attributes/view response, not always in schema)
    pub event_uuid: Option<String>,
    /// Tag array (complex type, optional, for future compatibility)
    #[serde(rename = "Tag")]
    pub tag: Option<Vec<Tag>>,
    /// Galaxy array (complex type, optional, for future compatibility)
    #[serde(rename = "Galaxy")]
    pub galaxy: Option<Vec<Galaxy>>,
    /// Base64 representation of the attachment (AttributeAttachment)
    pub data: Option<String>,
    /// Array of decay score entries
    pub decay_score: Option<Vec<DecayScoreEntry>>,
    /// Embedded Event object (optional, as per schema)
    #[serde(rename = "Event")]
    pub event: Option<Event>,
    /// Embedded Object(s) (optional, as per schema)
    #[serde(rename = "Object")]
    pub object: Option<Object>,
    /// AttributeTag array (optional, for future compatibility)
    #[serde(rename = "AttributeTag", default)]
    pub attribute_tag: Option<Vec<AttributeTag>>,    
}

/// Tag object for attributes (as seen in AttributeTag array)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeTag {
    // Define fields as per actual API response for AttributeTag.
    // If empty, keep as an empty struct for now, and expand as needed.
}

/// Entry for decay_score array in Attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayScoreEntry {
    pub score: f64,
    pub base_score: f64,
    pub decayed: bool,
    /// Decaying model for this decay score entry
    pub decaying_model: DecayingModelEnum,
}

/// DecayingModel can be either minimal or full
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecayingModelEnum {
    Minimal(DecayingModel),
    Full(FullDecayingModel),
}

/// Minimal DecayingModel (id and name only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayingModel {
    pub id: String,
    pub name: String,
}

/// Wrapper for single attribute response from /attributes/view/{attributeId}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeWrapper {
    /// The attribute object, under the "Attribute" key
    #[serde(rename = "Attribute")]
    pub attribute: Attribute,
}

/// Response type for /attributes/attributeStatistics/{context}/{percentage}
/// Maps category/type names to count or percentage strings.
pub type AttributeStatisticsResponse = HashMap<String, String>;




/// Wrapper for /attributes/describeTypes response (top-level "result" key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeTypesWrapper {
    pub result: DescribeTypesResult,
}

/// Main result object for /attributes/describeTypes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeTypesResult {
    /// Maps attribute type to its sane defaults (category, to_ids)
    pub sane_defaults: HashMap<String, SaneDefault>,
    /// List of all available attribute types
    pub types: Vec<String>,
    /// List of all available attribute categories
    pub categories: Vec<String>,
    /// Maps category name to list of attribute types in that category
    pub category_type_mappings: HashMap<String, Vec<String>>,
}

/// Sane default settings for an attribute type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaneDefault {
    /// Default category for this attribute type
    pub default_category: String,
    /// Whether this type is flagged for IDS (0 or 1)
    pub to_ids: u8,
}

/// Request struct for /attributes/restSearch (all fields from official schema, all Option<T>)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeRestSearchRequest {
    /// Page number (>= 1)
    pub page: Option<u32>,
    /// Maximum number of results (0 means maximum allowed)
    pub limit: Option<u32>,
    /// Attribute value filter
    pub value: Option<String>,
    /// Attribute value1 filter
    pub value1: Option<String>,
    /// Attribute value2 filter
    pub value2: Option<String>,
    /// Attribute type (see official enum)
    #[serde(rename = "type")]
    pub attribute_type: Option<String>,
    /// Attribute category (see official enum)
    pub category: Option<String>,
    /// Organisation ID or name
    pub org: Option<String>,
    /// Tags filter
    pub tags: Option<Vec<String>>,
    /// Start date/time filter
    pub from: Option<String>,
    /// End date/time filter
    pub to: Option<String>,
    /// Events published within the last x amount of time (int or string)
    pub last: Option<serde_json::Value>,
    /// Event ID filter
    pub eventid: Option<String>,
    /// Include base64 attachments
    #[serde(rename = "withAttachments")]
    pub with_attachments: Option<bool>,
    /// Attribute UUID filter
    pub uuid: Option<String>,
    /// Publish timestamp filter
    pub publish_timestamp: Option<String>,
    /// Published flag
    pub published: Option<bool>,
    /// Attribute timestamp filter
    pub timestamp: Option<String>,
    /// Attribute timestamp filter (alternative)
    pub attribute_timestamp: Option<String>,
    /// Enforce warninglist
    #[serde(rename = "enforceWarninglist")]
    pub enforce_warninglist: Option<bool>,
    /// To IDS flag
    pub to_ids: Option<bool>,
    /// Include soft-deleted attributes
    pub deleted: Option<bool>,
    /// Event timestamp filter
    pub event_timestamp: Option<String>,
    /// Threat level ID (see official enum)
    pub threat_level_id: Option<String>,
    /// Event info filter
    pub eventinfo: Option<String>,
    /// Sharing group IDs
    pub sharinggroup: Option<Vec<String>>,
    /// Decaying model name
    #[serde(rename = "decayingModel")]
    pub decaying_model: Option<String>,
    /// Decaying model score override
    pub score: Option<String>,
    /// First seen filter
    pub first_seen: Option<String>,
    /// Last seen filter
    pub last_seen: Option<String>,
    /// Include event UUIDs in response
    #[serde(rename = "includeEventUuid")]
    pub include_event_uuid: Option<bool>,
    /// Include event tags in response
    #[serde(rename = "includeEventTags")]
    pub include_event_tags: Option<bool>,
    /// Include proposals in response
    #[serde(rename = "includeProposals")]
    pub include_proposals: Option<bool>,
    /// List of requested attribute properties (for CSV export)
    pub requested_attributes: Option<Vec<String>>,
    /// Include event context fields (for CSV export)
    #[serde(rename = "includeContext")]
    pub include_context: Option<bool>,
    /// Remove header in CSV export
    pub headerless: Option<bool>,
    /// Include warninglist hits
    #[serde(rename = "includeWarninglistHits")]
    pub include_warninglist_hits: Option<bool>,
    /// Attack galaxy filter
    #[serde(rename = "attackGalaxy")]
    pub attack_galaxy: Option<String>,
    /// Object relation filter
    pub object_relation: Option<String>,
    /// Include sightings in response
    #[serde(rename = "includeSightings")]
    pub include_sightings: Option<bool>,
    /// Include correlations in response
    #[serde(rename = "includeCorrelations")]
    pub include_correlations: Option<bool>,
    /// Model overrides for decaying model
    #[serde(rename = "modelOverrides")]
    pub model_overrides: Option<ModelOverridesRestSearchFilter>,
    /// Include decaying score in response
    #[serde(rename = "includeDecayScore")]
    pub include_decay_score: Option<bool>,
    /// Include full model information in response
    #[serde(rename = "includeFullModel")]
    pub include_full_model: Option<bool>,
    /// Exclude decayed elements
    #[serde(rename = "excludeDecayed")]
    pub exclude_decayed: Option<bool>,
    /// Response format (see official enum)
    #[serde(rename = "returnFormat")]
    pub return_format: Option<String>,
}

/// ModelOverridesRestSearchFilter object for decaying model overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOverridesRestSearchFilter {
    /// Lifetime override
    pub lifetime: Option<f64>,
    /// Decay speed override
    pub decay_speed: Option<f64>,
    /// Threshold override
    pub threshold: Option<f64>,
    /// Default base score override
    pub default_base_score: Option<f64>,
    /// Base score config (map of string to float)
    pub base_score_config: Option<std::collections::HashMap<String, f64>>,
}

    // =============================================================================
    // Types for /attributes/restSearch response (strict, schema-driven)
    // =============================================================================


/// Wrapper for the /attributes/restSearch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeListResponse {
    pub response: AttributeListResponseInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeListResponseInner {
    #[serde(rename = "Attribute")]
    pub attribute: Vec<Attribute>,
}

    /// DecayScore for an attribute
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DecayScore {
        /// Decay score value
        pub score: f64,
        /// Model name
        pub model: String,
    }

    /// Parameters for decaying models
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DecayingModelParameters {
        /// Lifetime (float)
        pub lifetime: f64,
        /// Decay speed (float)
        pub decay_speed: f64,
        /// Threshold (float)
        pub threshold: f64,
        /// Default base score (float)
        pub default_base_score: f64,
        /// Arbitrary config object, may be any JSON structure
        pub base_score_config: Value,
    }


    /// FullDecayingModel for an attribute
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FullDecayingModel {
        /// Numeric string, <= 10 chars
        pub id: String,
        /// UUID string
        pub uuid: String,
        /// Name, <= 255 chars
        pub name: String,
        /// Description, <= 65535 chars
        pub description: String,
        pub parameters: DecayingModelParameters,
        pub attribute_types: Vec<AttributeType>,
        /// Organisation ID, numeric string <= 10 chars
        pub org_id: String,
        pub enabled: bool,
        pub all_orgs: bool,
        #[serde(rename = "ref")]
        pub r#ref: Vec<String>,
        /// Should always be "Polynomial"
        pub formula: String,
        pub version: String,
        pub default: bool,
        #[serde(rename = "isEditable")]
        pub is_editable: bool,
    }

    /// All possible attribute types as per MISP schema (stringly-typed for flexibility)
    pub type AttributeType = String;

/// Event structure for related events
/// Event object as per official MISP schema for /attributes/restSearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID - string (EventId) <= 10 characters ^\d+$
    pub id: String,
    /// Event info - string (EventInfo) <= 65535 characters
    pub info: String,
    /// Event UUID - string <uuid> (UUID)
    pub uuid: Option<String>,
    /// Distribution level - string (DistributionLevelId) "0"-"5"
    pub distribution: Option<String>,
    /// Organisation ID - string (OrganisationId) <= 10 characters ^\d+$
    #[serde(rename = "org_id")]
    pub org_id: Option<String>,
    /// Organisation creator ID - string (OrganisationId) <= 10 characters ^\d+$
    #[serde(rename = "orgc_id")]
    pub orgc_id: Option<String>,
    /// Event date - string
    pub date: Option<String>,
    /// Published flag - boolean (PublishedFlag)
    pub published: Option<bool>,
    /// Analysis level - string (AnalysisLevelId) "0"-"2"
    pub analysis: Option<String>,
    /// Attribute count - string (EventAttributeCount) ^\\d+$
    #[serde(rename = "attribute_count")]
    pub attribute_count: Option<String>,
    /// Timestamp - string (NullableTimestamp) Nullable ^\\d+$|^$
    pub timestamp: Option<String>,
    /// Sharing group ID - string (SharingGroupId) <= 10 characters Nullable ^\\d+$|^$
    #[serde(rename = "sharing_group_id")]
    pub sharing_group_id: Option<String>,
    /// Proposal email lock - boolean (EventProposalEmailLock)
    #[serde(rename = "proposal_email_lock")]
    pub proposal_email_lock: Option<bool>,
    /// Locked flag - boolean (IsLocked)
    pub locked: Option<bool>,
    /// Threat level ID - string (ThreatLevelId) "1"-"4"
    #[serde(rename = "threat_level_id")]
    pub threat_level_id: Option<String>,
    /// Publish timestamp - string (Timestamp) ^\\d+$, default "0"
    #[serde(rename = "publish_timestamp")]
    pub publish_timestamp: Option<String>,
    /// Sighting timestamp - string (Timestamp) ^\\d+$, default "0"
    #[serde(rename = "sighting_timestamp")]
    pub sighting_timestamp: Option<String>,
    /// Disable correlation flag - boolean (DisableCorrelationFlag)
    #[serde(rename = "disable_correlation")]
    pub disable_correlation: Option<bool>,
    /// Extends UUID - string (ExtendsUUID) <= 36 characters Nullable
    #[serde(rename = "extends_uuid")]
    pub extends_uuid: Option<String>,
    /// Event creator email - string <email>
    #[serde(rename = "event_creator_email")]
    pub event_creator_email: Option<String>,
    /// Organisation object (optional, from API response)
    #[serde(rename = "Org", default)]
    pub org: Option<Organisation>,
    /// Organisation creator object (optional, from API response)
    #[serde(rename = "Orgc", default)]
    pub orgc: Option<Organisation>,
    /// User ID (optional, from API response)
    #[serde(default)]
    pub user_id: Option<String>,
    /// Threat level object (optional, from API response)
    #[serde(rename = "ThreatLevel", default)]
    pub threat_level: Option<ThreatLevel>,
    /// Feed array (optional, from API response)
    /// Changed from Option<Feed> to Option<Vec<Feed>> to match API response
    #[serde(rename = "Feed", default)]
    pub feed: Option<Vec<Feed>>,
    /// Attribute array (from API response)
    #[serde(rename = "Attribute", default)]
    pub attribute: Vec<Attribute>,
    /// ShadowAttribute array (from API response)
    #[serde(rename = "ShadowAttribute", default)]
    pub shadow_attribute: Vec<Attribute>,
    /// RelatedEvent array (from API response)
    #[serde(rename = "RelatedEvent", default)]
    pub related_event: Vec<RelatedEvent>,
    /// Galaxy array (from API response)
    #[serde(rename = "Galaxy", default)]
    pub galaxy: Vec<Galaxy>,
    /// Object array (from API response)
    #[serde(rename = "Object", default)]
    pub object: Vec<Object>,
    /// EventReport array (from API response)
    #[serde(rename = "EventReport", default)]
    pub event_report: Vec<EventReport>,
    /// Tag array (from API response)
    #[serde(rename = "Tag", default)]
    pub tag: Vec<Tag>,
    /// Protected flag (optional, from API response)
    /// Added to match API response field "protected"
    pub protected: Option<bool>,
    ///orgc_uuid found in restSearch response for Event
    pub orgc_uuid: Option<String>,
    ///for future compatibility - CryptographicKey array (from API response)
    #[serde(rename = "CryptographicKey", default)]
    pub cryptographic_key: Vec<CryptographicKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CryptographicKey {
    // No fields observed in sample, but add fields if schema is known in future.
}

// Wrapper for GET /events/view/{{eventId}} endpoint
// The API returns: { "Event": { ... } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEventByIdResponse {
    #[serde(rename = "Event")]
    pub event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatLevel {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Object structure for related objects
/// Object object as per official MISP schema for /attributes/restSearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Object ID - string (ObjectId) <= 10 characters ^\d+$
    pub id: String,
    /// Object name - string (ObjectName) <= 131071 characters
    pub name: String,
    /// Meta category - string (ObjectMetaCategory)
    #[serde(rename = "meta-category")]
    pub meta_category: Option<String>,
    /// Description - string (ObjectDescription)
    pub description: Option<String>,
    /// Template UUID - string <uuid> (UUID)
    #[serde(rename = "template_uuid")]
    pub template_uuid: Option<String>,
    /// Template version - string (ObjectTemplateVersion) ^\d+$
    #[serde(rename = "template_version")]
    pub template_version: Option<String>,
    /// Event ID - string (EventId) <= 10 characters ^\d+$
    #[serde(rename = "event_id")]
    pub event_id: Option<String>,
    /// Object UUID - string <uuid> (UUID)
    pub uuid: Option<String>,
    /// Timestamp - string (Timestamp) ^\d+$, default "0"
    pub timestamp: Option<String>,
    /// Distribution level - string (DistributionLevelId) "0"-"5"
    pub distribution: Option<String>,
    /// Sharing group ID - string (SharingGroupId) <= 10 characters Nullable ^\d+$|^$
    #[serde(rename = "sharing_group_id")]
    pub sharing_group_id: Option<String>,
    /// Comment - string
    pub comment: Option<String>,
    /// Deleted flag - boolean
    pub deleted: Option<bool>,
    /// First seen - string (NullableMicroTimestamp) Nullable ^\d+$|^$, default null
    #[serde(rename = "first_seen")]
    pub first_seen: Option<String>,
    /// Last seen - string (NullableMicroTimestamp) Nullable ^\d+$|^$, default null
    #[serde(rename = "last_seen")]
    pub last_seen: Option<String>,
    /// Array of Attribute objects (recursive)
    #[serde(rename = "Attribute")]
    pub attributes: Option<Vec<Attribute>>,
    /// Event Object from official schema (optional)
    #[serde(rename = "Event", default)]
    pub event: Option<Event>,
}



/// Feed object for GET /events 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    /// Feed ID - string (FeedId) <= 10 characters ^\d+$
    pub id: String,
    /// Feed name - string (FeedName) <= 255 characters
    pub name: String,
    /// Feed provider - string (FeedProvider)
    pub provider: String,
    /// Feed URL - string (FeedUrl)
    pub url: String,
    /// Feed rules - stringified JSON filter rules (nullable)
    pub rules: Option<String>,
    /// Feed enabled flag - boolean
    pub enabled: Option<bool>,
    /// Distribution level - string (DistributionLevelId)
    pub distribution: Option<String>,
    /// Sharing group ID - string (nullable)
    pub sharing_group_id: Option<String>,
    /// Tag ID - string (TagId)
    pub tag_id: Option<String>,
    /// Default flag - boolean
    pub default: Option<bool>,
    /// Source format - string (FeedSourceFormat)
    pub source_format: Option<String>,
    /// Fixed event flag - boolean
    pub fixed_event: Option<bool>,
    /// Delta merge flag - boolean
    pub delta_merge: Option<bool>,
    /// Event ID - string (EventId)
    pub event_id: Option<String>,
    /// Publish flag - boolean
    pub publish: Option<bool>,
    /// Override IDS flag - boolean
    pub override_ids: Option<bool>,
    /// Feed settings - string (nullable)
    pub settings: Option<String>,
    /// Input source - string (FeedInputSource)
    pub input_source: Option<String>,
    /// Delete local file flag - boolean
    pub delete_local_file: Option<bool>,
    /// Lookup visible flag - boolean
    pub lookup_visible: Option<bool>,
    /// Headers - string (nullable)
    pub headers: Option<String>,
    /// Caching enabled flag - boolean
    pub caching_enabled: Option<bool>,
    /// Force to IDS flag - boolean
    pub force_to_ids: Option<bool>,
    /// Organisation creator ID - string
    pub orgc_id: Option <String>,
    /// Cache timestamp - string or boolean or null
    #[serde(default)]
    pub cache_timestamp: Option<CacheTimestamp>,
}

/// Helper enum for cache_timestamp (string or bool or null)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CacheTimestamp {
    String(String),
    Bool(bool),
    Null,
}

/// RelatedEvent object for GET /events (recursive Event reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEvent {
    #[serde(rename = "Event")]
    pub event: Box<Event>,
}


/// Request body for POST /events/index (Event search/filter)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventIndexRequest {
    /// Page number (>= 1)
    pub page: Option<u32>,
    /// Maximum number of results to return (>= 0, 0 means maximum)
    pub limit: Option<u32>,
    /// Field to sort by
    pub sort: Option<String>,
    /// Sort direction ("asc" or "desc")
    pub direction: Option<String>,
    /// Return minimal event objects (default: false)
    pub minimal: Option<bool>,
    /// Filter events by attribute value
    pub attribute: Option<String>,
    /// Filter by event ID (string, <= 10 digits)
    #[serde(rename = "eventid")]
    pub event_id: Option<String>,
    /// Event creation date >= (YYYY-MM-DD)
    #[serde(rename = "datefrom")]
    pub date_from: Option<String>,
    /// Event creation date <= (YYYY-MM-DD)
    #[serde(rename = "dateuntil")]
    pub date_until: Option<String>,
    /// Filter by creator organisation name
    pub org: Option<String>,
    /// Filter by event info text
    #[serde(rename = "eventinfo")]
    pub event_info: Option<String>,
    /// Filter by single tag name (<= 255 chars)
    pub tag: Option<String>,
    /// Filter by any of a list of tag names
    pub tags: Option<Vec<String>>,
    /// Distribution level ("0"-"5")
    pub distribution: Option<String>,
    /// Sharing group ID (<= 10 digits)
    #[serde(rename = "sharinggroup")]
    pub sharing_group: Option<String>,
    /// Analysis level ("0"-"2")
    pub analysis: Option<String>,
    /// Threat level ("1"-"4")
    #[serde(rename = "threatlevel")]
    pub threat_level: Option<String>,
    /// Only events extending another (true/false)
    pub extending: Option<bool>,
    /// Only events extended by another (true/false)
    pub extended: Option<bool>,
    /// Filter by creator user email
    pub email: Option<String>,
    /// Filter by presence of attribute proposals ("0" or "1")
    #[serde(rename = "hasproposal")]
    pub has_proposal: Option<String>,
    /// Event timestamp >=
    pub timestamp: Option<String>,
    /// Event publish timestamp >=
    pub publish_timestamp: Option<String>,
    /// Filter by date (YYYY-MM-DD), newer than
    #[serde(rename = "searchDatefrom")]
    pub search_date_from: Option<String>,
    /// Filter by date (YYYY-MM-DD), older than
    #[serde(rename = "searchDateuntil")]
    pub search_date_until: Option<String>,
}

/// Request body for POST /events/restSearch (filtered and paginated event search)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventsRestSearchRequest {
    /// Page number (>= 1)
    pub page: Option<u32>,
    /// Maximum number of results to return (>= 0, 0 means maximum)
    pub limit: Option<u32>,
    /// Attribute value to search for (<= 131071 characters)
    pub value: Option<String>,
    /// Attribute type (<= 100 characters, see API docs for enum)
    #[serde(rename = "type")]
    pub attr_type: Option<String>,
    /// Attribute category (<= 255 characters, see API docs for enum)
    pub category: Option<String>,
    /// Organisation ID or name
    pub org: Option<String>,
    /// List of tag names to filter (nullable)
    pub tags: Option<Vec<String>>,
    /// List of event tag names to filter (nullable)
    pub event_tags: Option<Vec<String>>,
    /// Search all fields (event descriptions, attribute values, comments)
    #[serde(rename = "searchall")]
    pub searchall: Option<String>,
    /// Date/time filter: from (nullable, e.g. "7d", timestamp, range)
    pub from: Option<String>,
    /// Date/time filter: to (nullable, e.g. "7d", timestamp, range)
    pub to: Option<String>,
    /// Events published within the last x amount of time (nullable, int or string)
    pub last: Option<serde_json::Value>,
    /// Filter by event ID (<= 10 digits)
    #[serde(rename = "eventid")]
    pub event_id: Option<String>,
    /// Extends response with base64 attachments if present (default: false)
    #[serde(rename = "withAttachments")]
    pub with_attachments: Option<bool>,
    /// Sharing group IDs (nullable, single or list)
    #[serde(rename = "sharinggroup")]
    pub sharing_group: Option<Vec<String>>,
    /// Only return metadata (nullable)
    pub metadata: Option<bool>,
    /// Filter by event UUID
    pub uuid: Option<String>,
    /// Event publish timestamp (default: "0")
    pub publish_timestamp: Option<String>,
    /// Event timestamp (default: "0")
    pub timestamp: Option<String>,
    /// Only published events (default: false)
    pub published: Option<bool>,
    /// Enforce warninglist (nullable)
    #[serde(rename = "enforceWarninglist")]
    pub enforce_warninglist: Option<bool>,
    /// Only return sharing group ID
    #[serde(rename = "sgReferenceOnly")]
    pub sg_reference_only: Option<bool>,
    /// List of requested attributes for CSV export
    pub requested_attributes: Option<Vec<String>>,
    /// Add event context fields in CSV export (nullable)
    #[serde(rename = "includeContext")]
    pub include_context: Option<bool>,
    /// Remove header in CSV export (nullable)
    pub headerless: Option<bool>,
    /// Include warninglist hits in export (nullable)
    #[serde(rename = "includeWarninglistHits")]
    pub include_warninglist_hits: Option<bool>,
    /// Attack galaxy filter (nullable)
    #[serde(rename = "attackGalaxy")]
    pub attack_galaxy: Option<String>,
    /// Only attributes with to_ids=true (default: true)
    pub to_ids: Option<bool>,
    /// Include soft-deleted attributes (default: false)
    pub deleted: Option<bool>,
    /// Exclude local tags from export (nullable)
    #[serde(rename = "excludeLocalTags")]
    pub exclude_local_tags: Option<bool>,
    /// Date filter (nullable, e.g. "7d", timestamp, range)
    pub date: Option<String>,
    /// Extend response with Sightings DB results (nullable)
    #[serde(rename = "includeSightingdb")]
    pub include_sightingdb: Option<bool>,
    /// Filter by tag name (<= 255 characters)
    pub tag: Option<String>,
    /// Filter by attribute object relation value (nullable)
    pub object_relation: Option<String>,
    /// Threat level ID ("1"-"4")
    pub threat_level_id: Option<String>,
    /// Only events extending another (see docs)
    pub extending: Option<bool>,
    /// Only events extended by another (see docs)
    pub extended: Option<bool>,
    /// Response format (see API docs for enum)
    #[serde(rename = "returnFormat")]
    pub return_format: Option<String>,
}

/// Response wrapper for POST /events/restSearch.
/// The API returns: { "response": [ { "Event": { ... } }, ... ] }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsRestSearchResponse {
    pub response: Vec<EventWrapper>,
}

/// Helper struct for the array of { "Event": { ... } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWrapper {
    #[serde(rename = "Event")]
    pub event: Event,
}

/// Request payload for POST /objects/restsearch endpoint
/// Official schema: https://www.misp-project.org/documentation/
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectsRestSearchRequest {
    /// Page number (>= 1)
    pub page: Option<u32>,
    /// Maximum number of results (0 means maximum allowed)
    pub limit: Option<u32>,
    /// Quick filter: match any tag names, event descriptions, attribute values or comments
    #[serde(rename = "quickFilter")]
    pub quick_filter: Option<String>,
    /// Search all: match any tag names, event descriptions, attribute values or comments
    pub searchall: Option<String>,
    /// Timestamp filter (as string, e.g. "0")
    pub timestamp: Option<String>,
    /// Object name filter
    #[serde(rename = "object_name")]
    pub object_name: Option<String>,
    /// Object template UUID filter
    #[serde(rename = "object_template_uuid")]
    pub object_template_uuid: Option<String>,
    /// Object template version filter
    #[serde(rename = "object_template_version")]
    pub object_template_version: Option<String>,
    /// Event ID filter
    pub eventid: Option<String>,
    /// Event info filter
    pub eventinfo: Option<String>,
    /// Ignore to_ids and published flags (if true, matches both true and false)
    pub ignore: Option<bool>,
    /// From date/time filter (string or null)
    pub from: Option<String>,
    /// To date/time filter (string or null)
    pub to: Option<String>,
    /// Date filter (string or null)
    pub date: Option<String>,
    /// Tags filter (array of strings)
    pub tags: Option<Vec<String>>,
    /// Last filter (integer or string)
    pub last: Option<serde_json::Value>,
    /// Event timestamp filter (as string)
    pub event_timestamp: Option<String>,
    /// Publish timestamp filter (as string)
    pub publish_timestamp: Option<String>,
    /// Organisation ID or name
    pub org: Option<String>,
    /// Object UUID filter
    pub uuid: Option<String>,
    /// Attribute value filter
    pub value: Option<String>,
    /// Attribute type filter (see MISP attribute types)
    #[serde(rename = "type")]
    pub attribute_type: Option<String>,
    /// Attribute category filter
    pub category: Option<String>,
    /// Object relation filter (string or null)
    pub object_relation: Option<String>,
    /// Attribute timestamp filter (as string)
    pub attribute_timestamp: Option<String>,
    /// First seen filter (string or null)
    pub first_seen: Option<String>,
    /// Last seen filter (string or null)
    pub last_seen: Option<String>,
    /// Comment filter
    pub comment: Option<String>,
    /// To IDS flag filter
    pub to_ids: Option<bool>,
    /// Published flag filter
    pub published: Option<bool>,
    /// Deleted flag filter
    pub deleted: Option<bool>,
    /// With attachments flag
    #[serde(rename = "withAttachments")]
    pub with_attachments: Option<bool>,
    /// Enforce warninglist flag
    #[serde(rename = "enforceWarninglist")]
    pub enforce_warninglist: Option<bool>,
    /// Include all tags flag
    #[serde(rename = "includeAllTags")]
    pub include_all_tags: Option<bool>,
    /// Include event UUID flag
    #[serde(rename = "includeEventUuid")]
    pub include_event_uuid: Option<bool>,
    /// Include event UUID flag (alternative spelling)
    #[serde(rename = "include_event_uuid")]
    pub include_event_uuid_alt: Option<bool>,
    /// Include event tags flag
    #[serde(rename = "includeEventTags")]
    pub include_event_tags: Option<bool>,
    /// Include proposals flag
    #[serde(rename = "includeProposals")]
    pub include_proposals: Option<bool>,
    /// Include warninglist hits flag
    #[serde(rename = "includeWarninglistHits")]
    pub include_warninglist_hits: Option<bool>,
    /// Include context flag
    #[serde(rename = "includeContext")]
    pub include_context: Option<bool>,
    /// Include sightings flag
    #[serde(rename = "includeSightings")]
    pub include_sightings: Option<bool>,
    /// Include sightingdb flag
    #[serde(rename = "includeSightingdb")]
    pub include_sightingdb: Option<bool>,
    /// Include correlations flag
    #[serde(rename = "includeCorrelations")]
    pub include_correlations: Option<bool>,
    /// Include decay score flag
    #[serde(rename = "includeDecayScore")]
    pub include_decay_score: Option<bool>,
    /// Include full model flag
    #[serde(rename = "includeFullModel")]
    pub include_full_model: Option<bool>,
    /// Allow proposal blocking flag
    pub allow_proposal_blocking: Option<bool>,
    /// Metadata only flag
    pub metadata: Option<bool>,
    /// Attack galaxy filter
    #[serde(rename = "attackGalaxy")]
    pub attack_galaxy: Option<String>,
    /// Exclude decayed elements flag
    #[serde(rename = "excludeDecayed")]
    pub exclude_decayed: Option<bool>,
    /// Decaying model filter
    #[serde(rename = "decayingModel")]
    pub decaying_model: Option<String>,
    /// Model overrides for decaying model
    #[serde(rename = "modelOverrides")]
    pub model_overrides: Option<ModelOverridesRestSearchFilter>,
    /// Decaying model score override
    pub score: Option<String>,
    /// Return format (should be "json")
    #[serde(rename = "returnFormat")]
    pub return_format: Option<String>,
}
