//! Query parameters and pagination helpers for TAXII 2.x.

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, NaiveDateTime, Utc};

/// Paginated result for TAXII 2.x queries.
///
/// Generic container for paginated responses, replacing tuple returns
/// like `(Vec<T>, bool, Option<String>)` with named fields.
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    /// The items in this page of results.
    pub items: T,
    /// Whether there are more results available.
    pub more: bool,
    /// Pagination cursor for fetching the next page.
    pub next: Option<String>,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result.
    #[inline]
    pub fn new(items: T, more: bool, next: Option<String>) -> Self {
        Self { items, more, next }
    }

    /// Create an empty result (no items, no more pages).
    pub fn empty() -> Self
    where
        T: Default,
    {
        Self {
            items: T::default(),
            more: false,
            next: None,
        }
    }
}

/// Pagination cursor for keyset pagination.
///
/// Replaces tuple `(DateTime<Utc>, String)` with named fields for clarity.
/// Used to track position in paginated result sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationCursor {
    /// The date when the object was added to the collection.
    pub date_added: DateTime<Utc>,
    /// The object ID for tie-breaking when dates are equal.
    pub object_id: String,
}

impl PaginationCursor {
    /// Create a new pagination cursor.
    #[inline]
    pub fn new(date_added: DateTime<Utc>, object_id: impl Into<String>) -> Self {
        Self {
            date_added,
            object_id: object_id.into(),
        }
    }

    /// Convert to tuple for backward compatibility with existing code.
    #[inline]
    pub fn as_tuple(&self) -> (DateTime<Utc>, &str) {
        (self.date_added, &self.object_id)
    }
}

/// Query parameters for TAXII 2.x object retrieval.
///
/// Groups common filtering parameters to reduce function argument count.
/// Matches TAXII 2.x specification query parameters.
#[derive(Debug, Default, Clone)]
pub struct Taxii2QueryParams<'a> {
    /// Limit number of results
    pub limit: Option<i64>,
    /// Filter to objects added after this timestamp
    pub added_after: Option<DateTime<Utc>>,
    /// Pagination cursor for keyset pagination
    pub next: Option<&'a PaginationCursor>,
    /// Filter by object IDs
    pub match_id: Option<&'a [String]>,
    /// Filter by object types
    pub match_type: Option<&'a [String]>,
    /// Filter by version timestamps
    pub match_version: Option<&'a [String]>,
    /// Filter by STIX spec versions
    pub match_spec_version: Option<&'a [String]>,
}

/// Get value for `next` based on dict instance.
///
/// Accepts NaiveDateTime because database stores timestamps without timezone.
/// We append +00:00 to match datetime.isoformat() output for UTC times.
#[must_use]
pub fn get_next_param(date_added: &NaiveDateTime, id: &str) -> String {
    let data = format!(
        "{}+00:00|{}",
        date_added.format("%Y-%m-%dT%H:%M:%S%.6f"),
        id
    );
    BASE64.encode(data.as_bytes())
}

/// Parse provided `next_param` into a pagination cursor.
///
/// Handles timestamps with timezone offsets (e.g., +00:00, -05:00) and without.
#[must_use]
pub fn parse_next_param(next_param: &str) -> Option<PaginationCursor> {
    let decoded = BASE64.decode(next_param).ok()?;
    let data = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = data.split('|').collect();
    if parts.len() != 2 {
        return None;
    }

    // Try RFC3339 parsing first (handles +00:00, -05:00, Z, etc.)
    let date_added = if let Ok(dt) = DateTime::parse_from_rfc3339(parts[0]) {
        dt.with_timezone(&Utc)
    } else {
        // Fallback for timestamps without timezone (legacy cursors)
        chrono::NaiveDateTime::parse_from_str(parts[0], "%Y-%m-%dT%H:%M:%S%.6f")
            .ok()?
            .and_utc()
    };

    Some(PaginationCursor::new(date_added, parts[1]))
}
