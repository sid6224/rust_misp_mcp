//! # misp-types
//!
//! Rust type definitions for the MISP (Malware Information Sharing Platform) API.
//! 
//! This library provides strongly-typed Rust structs that correspond to the JSON
//! structures returned by MISP API endpoints. All types are designed to serialize
//! and deserialize correctly with serde, preserving the exact field names and
//! structure expected by MISP.
//!
//! The types are organized into logical groups:
//! - User management: `User`, `Role`, `Organisation`, etc.
//! - Events and threat intelligence: `Event`, `Attribute`, `Object`, etc.
//! - Request/response wrappers for specific endpoints
//!
//! ## Example
//!
//! ```rust
//! use misp_types::{Event, SearchEventsRequest};
//! use serde_json;
//!
//! // Parse a MISP event from JSON
//! let event_json = r#"{"id": "1", "info": "Test event", ...}"#;
//! let event: Event = serde_json::from_str(event_json)?;
//!
//! // Create a search request
//! let mut search_req = SearchEventsRequest::default();
//! search_req.limit = Some(10);
//! search_req.published = Some(true);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Re-export all types from the types module
pub use types::*;

pub mod types;
