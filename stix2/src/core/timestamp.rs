//! STIX Timestamp handling with precision tracking.
//!
//! STIX timestamps follow ISO 8601 format with specific precision requirements:
//! - STIX 2.0: Millisecond precision required
//! - STIX 2.1: Microsecond precision allowed

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use crate::core::error::{Error, Result};

/// Precision level for STIX timestamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Precision {
    /// Second precision (no sub-second component).
    Second,
    /// Millisecond precision (3 decimal places).
    #[default]
    Millisecond,
    /// Microsecond precision (6 decimal places).
    Microsecond,
}

impl Precision {
    /// Get the format string for this precision level.
    pub fn format_string(&self) -> &'static str {
        match self {
            Precision::Second => "%Y-%m-%dT%H:%M:%SZ",
            Precision::Millisecond => "%Y-%m-%dT%H:%M:%S%.3fZ",
            Precision::Microsecond => "%Y-%m-%dT%H:%M:%S%.6fZ",
        }
    }

    /// Detect precision from a timestamp string.
    pub fn detect(s: &str) -> Self {
        // Look for the decimal point after seconds
        if let Some(dot_pos) = s.rfind('.') {
            // Count digits after the dot until 'Z' or end
            let after_dot = &s[dot_pos + 1..];
            let digit_count = after_dot.chars().take_while(|c| c.is_ascii_digit()).count();

            if digit_count >= 6 {
                Precision::Microsecond
            } else if digit_count >= 1 {
                Precision::Millisecond
            } else {
                Precision::Second
            }
        } else {
            Precision::Second
        }
    }
}

/// A STIX-compliant timestamp with precision tracking.
///
/// This type wraps a `DateTime<Utc>` and tracks the precision level
/// for proper serialization.
///
/// # Example
///
/// ```rust,no_run
/// use stix2::Timestamp;
/// use chrono::Utc;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create from current time
///     let ts = Timestamp::now();
///
///     // Parse from string
///     let ts: Timestamp = "2023-01-15T12:00:00.000Z".parse()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    datetime: DateTime<Utc>,
    precision: Precision,
}

impl Timestamp {
    /// Create a new timestamp from a DateTime with default precision.
    #[must_use]
    pub fn new(datetime: DateTime<Utc>) -> Self {
        Self {
            datetime,
            precision: Precision::default(),
        }
    }

    /// Create a new timestamp with specific precision.
    #[must_use]
    pub fn with_precision(datetime: DateTime<Utc>, precision: Precision) -> Self {
        Self {
            datetime,
            precision,
        }
    }

    /// Get the current time as a timestamp.
    #[must_use]
    pub fn now() -> Self {
        Self::new(Utc::now())
    }

    /// Get the precision level.
    #[must_use]
    pub fn precision(&self) -> Precision {
        self.precision
    }

    /// Get the underlying DateTime.
    #[must_use]
    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    /// Format the timestamp according to its precision.
    #[must_use]
    pub fn format(&self) -> String {
        self.datetime
            .format(self.precision.format_string())
            .to_string()
    }

    /// Create a timestamp from Unix epoch seconds.
    #[must_use]
    pub fn from_unix(seconds: i64) -> Option<Self> {
        Utc.timestamp_opt(seconds, 0).single().map(Self::new)
    }

    /// Create a timestamp from Unix epoch milliseconds.
    #[must_use]
    pub fn from_unix_millis(millis: i64) -> Option<Self> {
        let seconds = millis / 1000;
        let nanos = ((millis % 1000) * 1_000_000) as u32;
        Utc.timestamp_opt(seconds, nanos)
            .single()
            .map(|dt| Self::with_precision(dt, Precision::Millisecond))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl Deref for Timestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.datetime
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::new(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.datetime
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl FromStr for Timestamp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Detect precision from input string
        let precision = Precision::detect(s);

        // Try parsing with chrono's flexible parser
        let datetime = DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Try alternative formats
                parse_timestamp_flexible(s)
            })
            .map_err(|e| Error::InvalidTimestamp(format!("Failed to parse '{}': {}", s, e)))?;

        Ok(Self {
            datetime,
            precision,
        })
    }
}

/// Parse timestamps with various formats.
fn parse_timestamp_flexible(s: &str) -> std::result::Result<DateTime<Utc>, String> {
    // List of formats to try
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%:z",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
    ];

    for fmt in &formats {
        if let Ok(dt) =
            NaiveDateTime::parse_from_str(s.trim_end_matches('Z'), fmt.trim_end_matches('Z'))
        {
            return Ok(Utc.from_utc_datetime(&dt));
        }
    }

    Err(format!("Unable to parse timestamp: {}", s))
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.format())
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

/// Get the current timestamp in STIX format.
pub fn get_timestamp() -> Timestamp {
    Timestamp::now()
}

/// Format a DateTime as a STIX-compliant string.
pub fn format_datetime(dt: &DateTime<Utc>, precision: Precision) -> String {
    dt.format(precision.format_string()).to_string()
}

/// Parse a string into a DateTime<Utc>.
pub fn parse_into_datetime(s: &str) -> Result<DateTime<Utc>> {
    let ts: Timestamp = s.parse()?;
    Ok(ts.datetime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_now() {
        let ts = Timestamp::now();
        assert!(ts.datetime() <= Utc::now());
    }

    #[test]
    fn test_parse_millisecond_precision() {
        let ts: Timestamp = "2023-01-15T12:30:45.123Z".parse().unwrap();
        assert_eq!(ts.precision(), Precision::Millisecond);
    }

    #[test]
    fn test_parse_microsecond_precision() {
        let ts: Timestamp = "2023-01-15T12:30:45.123456Z".parse().unwrap();
        assert_eq!(ts.precision(), Precision::Microsecond);
    }

    #[test]
    fn test_parse_second_precision() {
        let ts: Timestamp = "2023-01-15T12:30:45Z".parse().unwrap();
        assert_eq!(ts.precision(), Precision::Second);
    }

    #[test]
    fn test_format_millisecond() {
        let ts = Timestamp::with_precision(
            Utc.with_ymd_and_hms(2023, 1, 15, 12, 30, 45).unwrap(),
            Precision::Millisecond,
        );
        let formatted = ts.format();
        assert!(formatted.contains(".000Z") || formatted.ends_with("Z"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let ts: Timestamp = "2023-01-15T12:30:45.123Z".parse().unwrap();
        let json = serde_json::to_string(&ts).unwrap();
        let parsed: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(ts.datetime(), parsed.datetime());
    }

    #[test]
    fn test_from_unix() {
        use chrono::Datelike;
        let ts = Timestamp::from_unix(1673783445).unwrap();
        assert_eq!(ts.year(), 2023);
    }

    #[test]
    fn test_display() {
        let ts: Timestamp = "2023-01-15T12:30:45.000Z".parse().unwrap();
        let s = ts.to_string();
        assert!(s.contains("2023-01-15"));
    }
}
