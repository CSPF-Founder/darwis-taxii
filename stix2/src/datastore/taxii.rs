//! TAXII 2.1 DataStore implementation
//!
//! This module provides a client for interacting with TAXII 2.1 servers
//! to retrieve and publish STIX objects.

use reqwest::{Client, StatusCode, header};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::bundle::Bundle;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use crate::datastore::{Filter, FilterOperator};

/// TAXII 2.1 Media Types
pub mod media_types {
    /// TAXII 2.1 content type
    pub const TAXII_21: &str = "application/taxii+json;version=2.1";
    /// STIX 2.1 content type
    pub const STIX_21: &str = "application/stix+json;version=2.1";
}

/// TAXII Discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    /// Title of the TAXII server
    pub title: String,
    /// Description of the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Contact information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    /// Default API root
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// List of API roots
    #[serde(default)]
    pub api_roots: Vec<String>,
}

/// TAXII API Root response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRoot {
    /// Title of the API root
    pub title: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Supported versions
    pub versions: Vec<String>,
    /// Maximum content length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_content_length: Option<u64>,
}

/// TAXII Collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection ID
    pub id: String,
    /// Collection title
    pub title: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Alias for the collection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// Whether the collection can be read
    #[serde(default = "default_true")]
    pub can_read: bool,
    /// Whether the collection can be written to
    #[serde(default)]
    pub can_write: bool,
    /// Media types supported
    #[serde(default)]
    pub media_types: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// TAXII Collections response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collections {
    /// List of collections
    #[serde(default)]
    pub collections: Vec<Collection>,
}

/// TAXII Envelope for objects response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Whether there are more objects available
    #[serde(default)]
    pub more: bool,
    /// Next page URL or token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// The STIX objects
    #[serde(default)]
    pub objects: Vec<serde_json::Value>,
}

/// TAXII Status response for add operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Status ID
    pub id: String,
    /// Status value (pending, complete)
    pub status: String,
    /// Request timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_timestamp: Option<String>,
    /// Total count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u32>,
    /// Success count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_count: Option<u32>,
    /// Failure count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_count: Option<u32>,
    /// Pending count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_count: Option<u32>,
    /// Successes
    #[serde(default)]
    pub successes: Vec<StatusDetail>,
    /// Failures
    #[serde(default)]
    pub failures: Vec<StatusDetail>,
    /// Pending
    #[serde(default)]
    pub pendings: Vec<StatusDetail>,
}

/// Status detail for individual objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDetail {
    /// Object ID
    pub id: String,
    /// Version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// TAXII Manifest entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Object ID
    pub id: String,
    /// Date added
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_added: Option<String>,
    /// Version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Media types
    #[serde(default)]
    pub media_types: Vec<String>,
}

/// TAXII Manifest response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Whether there are more entries
    #[serde(default)]
    pub more: bool,
    /// Next page token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// Manifest entries
    #[serde(default)]
    pub objects: Vec<ManifestEntry>,
}

/// TAXII 2.1 Client
#[derive(Debug)]
pub struct TaxiiClient {
    client: Client,
    server_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl TaxiiClient {
    /// Create a new TAXII client
    pub fn new(server_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .build()
            .map_err(|e| Error::Custom(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            server_url: server_url.into().trim_end_matches('/').to_string(),
            username: None,
            password: None,
        })
    }

    /// Create a new TAXII client with basic authentication
    pub fn with_auth(
        server_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self> {
        let mut client = Self::new(server_url)?;
        client.username = Some(username.into());
        client.password = Some(password.into());
        Ok(client)
    }

    fn build_request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.client.request(method, url);

        req = req.header(header::ACCEPT, media_types::TAXII_21);
        req = req.header(header::CONTENT_TYPE, media_types::TAXII_21);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            req = req.basic_auth(user, Some(pass));
        }

        req
    }

    /// Discover the TAXII server
    pub async fn discover(&self) -> Result<Discovery> {
        let url = format!("{}/taxii2/", self.server_url);
        let response = self
            .build_request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Discovery request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Discovery failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse discovery response: {}", e)))
    }

    /// Get API root information
    pub async fn get_api_root(&self, api_root: &str) -> Result<ApiRoot> {
        let url = format!("{}/{}/", self.server_url, api_root.trim_matches('/'));
        let response = self
            .build_request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("API root request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "API root request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse API root response: {}", e)))
    }

    /// Get collections for an API root
    pub async fn get_collections(&self, api_root: &str) -> Result<Collections> {
        let url = format!(
            "{}/{}/collections/",
            self.server_url,
            api_root.trim_matches('/')
        );
        let response = self
            .build_request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Collections request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Collections request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse collections response: {}", e)))
    }

    /// Get a specific collection
    pub async fn get_collection(&self, api_root: &str, collection_id: &str) -> Result<Collection> {
        let url = format!(
            "{}/{}/collections/{}/",
            self.server_url,
            api_root.trim_matches('/'),
            collection_id
        );
        let response = self
            .build_request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Collection request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Collection request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse collection response: {}", e)))
    }

    /// Get objects from a collection
    pub async fn get_objects(
        &self,
        api_root: &str,
        collection_id: &str,
        params: Option<&ObjectsParams>,
    ) -> Result<Envelope> {
        let mut url = format!(
            "{}/{}/collections/{}/objects/",
            self.server_url,
            api_root.trim_matches('/'),
            collection_id
        );

        if let Some(p) = params {
            let query = p.to_query_string();
            if !query.is_empty() {
                url = format!("{}?{}", url, query);
            }
        }

        let mut req = self.build_request(reqwest::Method::GET, &url);
        req = req.header(header::ACCEPT, media_types::STIX_21);

        let response = req
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Get objects request failed: {}", e)))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Envelope {
                more: false,
                next: None,
                objects: vec![],
            });
        }

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Get objects request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse objects response: {}", e)))
    }

    /// Get a specific object by ID
    pub async fn get_object(
        &self,
        api_root: &str,
        collection_id: &str,
        object_id: &str,
    ) -> Result<Envelope> {
        let url = format!(
            "{}/{}/collections/{}/objects/{}/",
            self.server_url,
            api_root.trim_matches('/'),
            collection_id,
            object_id
        );

        let mut req = self.build_request(reqwest::Method::GET, &url);
        req = req.header(header::ACCEPT, media_types::STIX_21);

        let response = req
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Get object request failed: {}", e)))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Envelope {
                more: false,
                next: None,
                objects: vec![],
            });
        }

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Get object request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse object response: {}", e)))
    }

    /// Add objects to a collection
    pub async fn add_objects(
        &self,
        api_root: &str,
        collection_id: &str,
        objects: Vec<StixObject>,
    ) -> Result<Status> {
        let url = format!(
            "{}/{}/collections/{}/objects/",
            self.server_url,
            api_root.trim_matches('/'),
            collection_id
        );

        let bundle = Bundle::from_objects(objects);

        let mut req = self.build_request(reqwest::Method::POST, &url);
        req = req.header(header::CONTENT_TYPE, media_types::STIX_21);
        req = req.header(header::ACCEPT, media_types::TAXII_21);
        req = req.json(&bundle);

        let response = req
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Add objects request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Add objects request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse status response: {}", e)))
    }

    /// Get the manifest of a collection
    pub async fn get_manifest(
        &self,
        api_root: &str,
        collection_id: &str,
        params: Option<&ObjectsParams>,
    ) -> Result<Manifest> {
        let mut url = format!(
            "{}/{}/collections/{}/manifest/",
            self.server_url,
            api_root.trim_matches('/'),
            collection_id
        );

        if let Some(p) = params {
            let query = p.to_query_string();
            if !query.is_empty() {
                url = format!("{}?{}", url, query);
            }
        }

        let response = self
            .build_request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Manifest request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Manifest request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse manifest response: {}", e)))
    }

    /// Delete an object from a collection
    pub async fn delete_object(
        &self,
        api_root: &str,
        collection_id: &str,
        object_id: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/{}/collections/{}/objects/{}/",
            self.server_url,
            api_root.trim_matches('/'),
            collection_id,
            object_id
        );

        let response = self
            .build_request(reqwest::Method::DELETE, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Delete object request failed: {}", e)))?;

        if !response.status().is_success() && response.status() != StatusCode::NOT_FOUND {
            return Err(Error::Custom(format!(
                "Delete object request failed with status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Get status of an add operation
    pub async fn get_status(&self, api_root: &str, status_id: &str) -> Result<Status> {
        let url = format!(
            "{}/{}/status/{}/",
            self.server_url,
            api_root.trim_matches('/'),
            status_id
        );

        let response = self
            .build_request(reqwest::Method::GET, &url)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("Status request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Custom(format!(
                "Status request failed with status: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Custom(format!("Failed to parse status response: {}", e)))
    }
}

/// Parameters for objects queries
#[derive(Debug, Clone, Default)]
pub struct ObjectsParams {
    /// Filter by added_after timestamp
    pub added_after: Option<String>,
    /// Filter by object ID
    pub id: Option<String>,
    /// Filter by object type
    pub type_: Option<String>,
    /// Filter by version (all, first, last, or specific timestamp)
    pub version: Option<String>,
    /// Limit number of results
    pub limit: Option<u32>,
    /// Next page token
    pub next: Option<String>,
}

impl ObjectsParams {
    /// Create new empty params
    pub fn new() -> Self {
        Self::default()
    }

    /// Set added_after filter
    pub fn added_after(mut self, timestamp: impl Into<String>) -> Self {
        self.added_after = Some(timestamp.into());
        self
    }

    /// Set ID filter
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set type filter
    pub fn type_(mut self, type_: impl Into<String>) -> Self {
        self.type_ = Some(type_.into());
        self
    }

    /// Set version filter
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set next page token
    pub fn next(mut self, next: impl Into<String>) -> Self {
        self.next = Some(next.into());
        self
    }

    /// Convert to query string
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(ref v) = self.added_after {
            params.push(format!("added_after={}", v));
        }
        if let Some(ref v) = self.id {
            params.push(format!("match[id]={}", v));
        }
        if let Some(ref v) = self.type_ {
            params.push(format!("match[type]={}", v));
        }
        if let Some(ref v) = self.version {
            params.push(format!("match[version]={}", v));
        }
        if let Some(v) = self.limit {
            params.push(format!("limit={}", v));
        }
        if let Some(ref v) = self.next {
            params.push(format!("next={}", v));
        }

        params.join("&")
    }
}

/// TAXII Collection DataStore
///
/// Provides DataSource and DataSink implementation for a TAXII collection.
pub struct TaxiiCollectionStore {
    client: TaxiiClient,
    api_root: String,
    collection_id: String,
    collection_info: RwLock<Option<Collection>>,
    items_per_page: u32,
}

impl TaxiiCollectionStore {
    /// Create a new TAXII collection store
    pub fn new(
        client: TaxiiClient,
        api_root: impl Into<String>,
        collection_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            api_root: api_root.into(),
            collection_id: collection_id.into(),
            collection_info: RwLock::new(None),
            items_per_page: 1000,
        }
    }

    /// Set items per page for pagination
    pub fn with_items_per_page(mut self, items: u32) -> Self {
        self.items_per_page = items;
        self
    }

    /// Get collection info (cached)
    pub async fn collection_info(&self) -> Result<Collection> {
        {
            let guard = self.collection_info.read().await;
            if let Some(ref info) = *guard {
                return Ok(info.clone());
            }
        }

        let info = self
            .client
            .get_collection(&self.api_root, &self.collection_id)
            .await?;

        {
            let mut guard = self.collection_info.write().await;
            *guard = Some(info.clone());
        }

        Ok(info)
    }

    /// Check if collection can be read
    pub async fn can_read(&self) -> Result<bool> {
        Ok(self.collection_info().await?.can_read)
    }

    /// Check if collection can be written
    pub async fn can_write(&self) -> Result<bool> {
        Ok(self.collection_info().await?.can_write)
    }

    /// Get an object by ID
    pub async fn get(&self, id: &Identifier) -> Result<Option<StixObject>> {
        let envelope = self
            .client
            .get_object(&self.api_root, &self.collection_id, &id.to_string())
            .await?;

        if envelope.objects.is_empty() {
            return Ok(None);
        }

        let obj: StixObject = serde_json::from_value(envelope.objects[0].clone())
            .map_err(|e| Error::Custom(format!("Failed to parse STIX object: {}", e)))?;

        Ok(Some(obj))
    }

    /// Get all versions of an object
    pub async fn all_versions(&self, id: &Identifier) -> Result<Vec<StixObject>> {
        let params = ObjectsParams::new().id(id.to_string()).version("all");

        let mut all_objects = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut query_params = params.clone();
            if let Some(ref token) = next_token {
                query_params = query_params.next(token.clone());
            }
            query_params = query_params.limit(self.items_per_page);

            let envelope = self
                .client
                .get_objects(&self.api_root, &self.collection_id, Some(&query_params))
                .await?;

            for obj_value in envelope.objects {
                let obj: StixObject = serde_json::from_value(obj_value)
                    .map_err(|e| Error::Custom(format!("Failed to parse STIX object: {}", e)))?;
                all_objects.push(obj);
            }

            if envelope.more && envelope.next.is_some() {
                next_token = envelope.next;
            } else {
                break;
            }
        }

        Ok(all_objects)
    }

    /// Query objects with filters
    pub async fn query(&self, filters: &[Filter]) -> Result<Vec<StixObject>> {
        let mut params = ObjectsParams::new().limit(self.items_per_page);

        // Apply TAXII-supported filters
        for filter in filters {
            match (filter.property.as_str(), &filter.operator) {
                ("type", FilterOperator::Equal) => {
                    if let crate::datastore::FilterValue::String(s) = &filter.value {
                        params = params.type_(s.clone());
                    }
                }
                ("id", FilterOperator::Equal) => {
                    if let crate::datastore::FilterValue::String(s) = &filter.value {
                        params = params.id(s.clone());
                    }
                }
                _ => {}
            }
        }

        let mut all_objects = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut query_params = params.clone();
            if let Some(ref token) = next_token {
                query_params = query_params.next(token.clone());
            }

            let envelope = self
                .client
                .get_objects(&self.api_root, &self.collection_id, Some(&query_params))
                .await?;

            for obj_value in envelope.objects {
                let obj: StixObject = serde_json::from_value(obj_value)
                    .map_err(|e| Error::Custom(format!("Failed to parse STIX object: {}", e)))?;

                // Apply local filters that TAXII doesn't support
                if self.matches_filters(&obj, filters) {
                    all_objects.push(obj);
                }
            }

            if envelope.more && envelope.next.is_some() {
                next_token = envelope.next;
            } else {
                break;
            }
        }

        Ok(all_objects)
    }

    /// Get all objects from the collection
    pub async fn get_all(&self) -> Result<Vec<StixObject>> {
        self.query(&[]).await
    }

    /// Add an object to the collection
    pub async fn add(&self, object: StixObject) -> Result<Status> {
        self.client
            .add_objects(&self.api_root, &self.collection_id, vec![object])
            .await
    }

    /// Add multiple objects to the collection
    pub async fn add_all(&self, objects: Vec<StixObject>) -> Result<Status> {
        self.client
            .add_objects(&self.api_root, &self.collection_id, objects)
            .await
    }

    /// Delete an object from the collection
    pub async fn delete(&self, id: &Identifier) -> Result<()> {
        self.client
            .delete_object(&self.api_root, &self.collection_id, &id.to_string())
            .await
    }

    fn matches_filters(&self, obj: &StixObject, filters: &[Filter]) -> bool {
        for filter in filters {
            match (filter.property.as_str(), &filter.operator, &filter.value) {
                ("type", FilterOperator::Equal, crate::datastore::FilterValue::String(s)) => {
                    if obj.type_name() != s {
                        return false;
                    }
                }
                ("id", FilterOperator::Equal, crate::datastore::FilterValue::String(s)) => {
                    if obj.id().to_string() != *s {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objects_params_query_string() {
        let params = ObjectsParams::new()
            .type_("indicator")
            .limit(100)
            .added_after("2023-01-01T00:00:00Z");

        let query = params.to_query_string();
        assert!(query.contains("match[type]=indicator"));
        assert!(query.contains("limit=100"));
        assert!(query.contains("added_after=2023-01-01T00:00:00Z"));
    }

    #[test]
    fn test_taxii_client_creation() {
        let client = TaxiiClient::new("https://example.com");
        assert!(client.is_ok());
    }

    #[test]
    fn test_taxii_client_with_auth() {
        let client = TaxiiClient::with_auth("https://example.com", "user", "pass");
        assert!(client.is_ok());
    }
}
