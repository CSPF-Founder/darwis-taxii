//! TAXII 2.x entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Account;

/// TAXII 2.x datetime format with 6-digit microsecond precision.
pub const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6fZ";

/// Format datetime for TAXII 2.x response.
pub fn taxii2_datetimeformat(dt: &DateTime<Utc>) -> String {
    dt.format(DATETIME_FORMAT).to_string()
}

/// TAXII 2.x API Root entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRoot {
    /// API Root ID.
    pub id: String,

    /// Whether this is the default API root.
    pub default: bool,

    /// Human readable title.
    pub title: String,

    /// Human readable description.
    pub description: Option<String>,

    /// Whether this is publicly readable.
    pub is_public: bool,
}

/// TAXII 2.x Collection entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection ID (UUID).
    pub id: String,

    /// API Root ID this collection belongs to.
    pub api_root_id: String,

    /// Human readable title.
    pub title: String,

    /// Human readable description.
    pub description: Option<String>,

    /// Human readable alias.
    pub alias: Option<String>,

    /// Whether this is publicly readable.
    pub is_public: bool,

    /// Whether this is publicly writable.
    pub is_public_write: bool,
}

impl Collection {
    /// Determine if account is allowed to read from this collection.
    ///
    /// Permissions are keyed by collection UUID (normalized at CLI sync time).
    pub fn can_read(&self, account: Option<&Account>) -> bool {
        if self.is_public {
            return true;
        }

        if let Some(acct) = account {
            if acct.is_admin {
                return true;
            }

            if let Some(perm) = acct.permissions.get(&self.id) {
                return perm.can_read();
            }
        }

        false
    }

    /// Determine if account is allowed to write to this collection.
    ///
    /// Permissions are keyed by collection UUID (normalized at CLI sync time).
    pub fn can_write(&self, account: Option<&Account>) -> bool {
        if self.is_public_write {
            return true;
        }

        if let Some(acct) = account {
            if acct.is_admin {
                return true;
            }

            if let Some(perm) = acct.permissions.get(&self.id) {
                return perm.can_write();
            }
        }

        false
    }
}

/// TAXII 2.x STIX Object entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STIXObject {
    /// STIX object ID.
    pub id: String,

    /// Collection ID.
    pub collection_id: String,

    /// STIX object type.
    #[serde(rename = "type")]
    pub stix_type: String,

    /// STIX spec version.
    pub spec_version: String,

    /// Date added to collection.
    pub date_added: DateTime<Utc>,

    /// Object version (from modified field).
    pub version: DateTime<Utc>,

    /// The payload of this object.
    pub serialized_data: serde_json::Value,
}

impl STIXObject {
    /// Convert to type-safe StixObject.
    ///
    /// Reconstructs the full JSON object by merging the common fields
    /// back into the serialized_data, then parses as a StixObject.
    pub fn to_typed(&self) -> stix2::Result<stix2::StixObject> {
        // Reconstruct full JSON
        let mut full_json = self.serialized_data.clone();
        if let Some(obj) = full_json.as_object_mut() {
            obj.insert("id".to_string(), serde_json::json!(self.id));
            obj.insert("type".to_string(), serde_json::json!(self.stix_type));
            obj.insert(
                "spec_version".to_string(),
                serde_json::json!(self.spec_version),
            );
        }

        stix2::parse(&full_json.to_string())
    }

    /// Create from type-safe StixObject.
    ///
    /// Extracts common fields (id, type, spec_version) from the object
    /// and stores the remaining data in serialized_data.
    pub fn from_typed(obj: &stix2::StixObject, collection_id: String) -> Self {
        let json = serde_json::to_value(obj).unwrap_or_default();

        // Extract common fields
        let id = obj.id().to_string();
        let stix_type = obj.type_name().to_string();
        let spec_version = "2.1".to_string();

        // Get modified timestamp for version
        let version = obj.modified().unwrap_or_else(chrono::Utc::now);

        // Remove common fields from serialized_data
        let mut data = json;
        if let Some(map) = data.as_object_mut() {
            map.remove("id");
            map.remove("type");
            map.remove("spec_version");
        }

        Self {
            id,
            collection_id,
            stix_type,
            spec_version,
            date_added: chrono::Utc::now(),
            version,
            serialized_data: data,
        }
    }

    /// Reconstruct full STIX JSON for API responses.
    ///
    /// Merges the common fields back into serialized_data.
    pub fn to_full_json(&self) -> serde_json::Value {
        let mut full_json = self.serialized_data.clone();
        if let Some(obj) = full_json.as_object_mut() {
            obj.insert("id".to_string(), serde_json::json!(self.id));
            obj.insert("type".to_string(), serde_json::json!(self.stix_type));
            obj.insert(
                "spec_version".to_string(),
                serde_json::json!(self.spec_version),
            );
        }
        full_json
    }
}

/// TAXII 2.x Manifest Record entity.
///
/// A cut-down version of STIXObject for efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRecord {
    /// STIX object ID.
    pub id: String,

    /// Date added.
    pub date_added: DateTime<Utc>,

    /// Object version.
    pub version: DateTime<Utc>,

    /// STIX spec version.
    pub spec_version: String,
}

/// TAXII 2.x Version Record entity.
///
/// A cut-down version of STIXObject for efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    /// Date added.
    pub date_added: DateTime<Utc>,

    /// Object version.
    pub version: DateTime<Utc>,
}

/// TAXII 2.x Job Detail entity (part of status resource).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetail {
    /// Job detail ID.
    pub id: String,

    /// Parent job ID.
    pub job_id: String,

    /// STIX object ID.
    pub stix_id: String,

    /// Object version.
    pub version: DateTime<Utc>,

    /// Status message.
    pub message: String,

    /// Status (success, failure, pending).
    pub status: String,
}

impl JobDetail {
    /// Turn this object into a TAXII 2.x dict.
    pub fn as_taxii2_dict(&self) -> serde_json::Value {
        let mut response = serde_json::json!({
            "id": self.stix_id,
            "version": taxii2_datetimeformat(&self.version)
        });

        if !self.message.is_empty() {
            response["message"] = serde_json::Value::String(self.message.clone());
        }

        response
    }
}

/// Job details grouped by status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobDetails {
    pub success: Vec<JobDetail>,
    pub failure: Vec<JobDetail>,
    pub pending: Vec<JobDetail>,
}

/// TAXII 2.x Job entity (status resource).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Job ID.
    pub id: String,

    /// API Root ID.
    pub api_root_id: String,

    /// Job status.
    pub status: String,

    /// Request timestamp.
    pub request_timestamp: DateTime<Utc>,

    /// Completion timestamp.
    pub completed_timestamp: Option<DateTime<Utc>>,

    /// Total count.
    pub total_count: i32,

    /// Success count.
    pub success_count: i32,

    /// Failure count.
    pub failure_count: i32,

    /// Pending count.
    pub pending_count: i32,

    /// Job details.
    pub details: JobDetails,
}

impl Job {
    /// Turn this object into a TAXII 2.x dict.
    pub fn as_taxii2_dict(&self) -> serde_json::Value {
        let mut response = serde_json::json!({
            "id": self.id,
            "status": self.status,
            "request_timestamp": taxii2_datetimeformat(&self.request_timestamp),
            "total_count": self.total_count,
            "success_count": self.success_count,
            "failure_count": self.failure_count,
            "pending_count": self.pending_count,
        });

        if !self.details.success.is_empty() {
            response["successes"] = serde_json::Value::Array(
                self.details
                    .success
                    .iter()
                    .map(|d| d.as_taxii2_dict())
                    .collect(),
            );
        }

        if !self.details.failure.is_empty() {
            response["failures"] = serde_json::Value::Array(
                self.details
                    .failure
                    .iter()
                    .map(|d| d.as_taxii2_dict())
                    .collect(),
            );
        }

        if !self.details.pending.is_empty() {
            response["pendings"] = serde_json::Value::Array(
                self.details
                    .pending
                    .iter()
                    .map(|d| d.as_taxii2_dict())
                    .collect(),
            );
        }

        response
    }
}
