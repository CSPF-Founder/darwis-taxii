//! JSON Canonicalization (RFC 8785)
//!
//! This module provides JSON Canonicalization Scheme (JCS) implementation
//! as defined in RFC 8785 for producing deterministic JSON output.
//!
//! Canonicalization is required for:
//! - Deterministic STIX ID generation
//! - Digital signatures
//! - Object comparison

use indexmap::IndexMap;
use serde_json::{Map, Number, Value};
use std::io::{self, Write};

use crate::core::error::{Error, Result};

/// Canonicalize a JSON value according to RFC 8785.
///
/// # Example
///
/// ```rust
/// use stix2::canonicalization::canonicalize;
/// use serde_json::json;
///
/// let value = json!({"b": 2, "a": 1});
/// let canonical = canonicalize(&value).unwrap();
/// assert_eq!(canonical, r#"{"a":1,"b":2}"#);
/// ```
pub fn canonicalize(value: &Value) -> Result<String> {
    let mut buffer = Vec::new();
    write_canonical(&mut buffer, value)
        .map_err(|e| Error::Custom(format!("Canonicalization error: {}", e)))?;
    String::from_utf8(buffer).map_err(|e| Error::Custom(format!("UTF-8 error: {}", e)))
}

/// Canonicalize a serializable value.
pub fn canonicalize_object<T: serde::Serialize>(obj: &T) -> Result<String> {
    let value = serde_json::to_value(obj)
        .map_err(|e| Error::Custom(format!("Serialization error: {}", e)))?;
    canonicalize(&value)
}

/// Write canonical JSON to a writer.
fn write_canonical<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    match value {
        Value::Null => writer.write_all(b"null"),
        Value::Bool(b) => {
            if *b {
                writer.write_all(b"true")
            } else {
                writer.write_all(b"false")
            }
        }
        Value::Number(n) => write_canonical_number(writer, n),
        Value::String(s) => write_canonical_string(writer, s),
        Value::Array(arr) => {
            writer.write_all(b"[")?;
            let mut first = true;
            for item in arr {
                if !first {
                    writer.write_all(b",")?;
                }
                first = false;
                write_canonical(writer, item)?;
            }
            writer.write_all(b"]")
        }
        Value::Object(obj) => {
            writer.write_all(b"{")?;

            // Sort keys according to RFC 8785 (UTF-16 code unit order)
            let mut sorted_keys: Vec<&String> = obj.keys().collect();
            sorted_keys.sort_by(|a, b| compare_strings_utf16(a, b));

            let mut first = true;
            for key in sorted_keys {
                if !first {
                    writer.write_all(b",")?;
                }
                first = false;
                write_canonical_string(writer, key)?;
                writer.write_all(b":")?;
                if let Some(val) = obj.get(key) {
                    write_canonical(writer, val)?;
                }
            }
            writer.write_all(b"}")
        }
    }
}

/// Compare strings by UTF-16 code unit order (RFC 8785 requirement).
fn compare_strings_utf16(a: &str, b: &str) -> std::cmp::Ordering {
    let a_units: Vec<u16> = a.encode_utf16().collect();
    let b_units: Vec<u16> = b.encode_utf16().collect();
    a_units.cmp(&b_units)
}

/// Write a canonical number according to RFC 8785.
///
/// Numbers must be serialized according to ECMAScript number-to-string rules.
fn write_canonical_number<W: Write>(writer: &mut W, n: &Number) -> io::Result<()> {
    if let Some(i) = n.as_i64() {
        // Integer - write directly
        write!(writer, "{}", i)
    } else if let Some(u) = n.as_u64() {
        // Unsigned integer
        write!(writer, "{}", u)
    } else if let Some(f) = n.as_f64() {
        // Floating point - use ECMAScript formatting
        write_ecmascript_number(writer, f)
    } else {
        // Fallback
        writer.write_all(n.to_string().as_bytes())
    }
}

/// Write a floating point number according to ECMAScript Number.toString() rules.
fn write_ecmascript_number<W: Write>(writer: &mut W, f: f64) -> io::Result<()> {
    if f.is_nan() {
        return writer.write_all(b"null");
    }
    if f.is_infinite() {
        return writer.write_all(b"null");
    }
    if f == 0.0 {
        // Handle -0.0 -> "0"
        return writer.write_all(b"0");
    }

    // Check if it's an integer
    if f.trunc() == f && f.abs() < 1e21 {
        write!(writer, "{}", f as i64)
    } else {
        // Use shortest representation
        let s = format_shortest_float(f);
        writer.write_all(s.as_bytes())
    }
}

/// Format a float to its shortest decimal representation.
fn format_shortest_float(f: f64) -> String {
    // Try different precisions and pick the shortest that round-trips
    let abs_f = f.abs();

    if !(1e-6..1e21).contains(&abs_f) {
        // Use exponential notation
        let s = format!("{:e}", f);
        normalize_exponential(&s)
    } else {
        // Use decimal notation
        for precision in 0..17 {
            let s = format!("{:.prec$}", f, prec = precision);
            if let Ok(parsed) = s.parse::<f64>()
                && parsed == f
            {
                // Remove trailing zeros after decimal point
                return normalize_decimal(&s);
            }
        }
        format!("{}", f)
    }
}

/// Normalize decimal representation (remove trailing zeros).
fn normalize_decimal(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }

    let s = s.trim_end_matches('0');
    s.strip_suffix('.').unwrap_or(s).to_string()
}

/// Normalize exponential notation to ECMAScript format.
fn normalize_exponential(s: &str) -> String {
    // Convert "1.5e-7" to "1.5e-7" (already correct)
    // Convert "1.500000e-7" to "1.5e-7"
    if let Some(pos) = s.find('e') {
        let mantissa = normalize_decimal(&s[..pos]);
        let exponent = &s[pos + 1..];

        // Remove leading + from exponent
        let exp_num: i32 = exponent.parse().unwrap_or(0);
        format!(
            "{}e{}{}",
            mantissa,
            if exp_num < 0 { "-" } else { "+" },
            exp_num.abs()
        )
    } else {
        s.to_string()
    }
}

/// Write a canonical string with proper escaping.
fn write_canonical_string<W: Write>(writer: &mut W, s: &str) -> io::Result<()> {
    writer.write_all(b"\"")?;

    for c in s.chars() {
        match c {
            '"' => writer.write_all(b"\\\"")?,
            '\\' => writer.write_all(b"\\\\")?,
            '\x08' => writer.write_all(b"\\b")?,
            '\x0c' => writer.write_all(b"\\f")?,
            '\n' => writer.write_all(b"\\n")?,
            '\r' => writer.write_all(b"\\r")?,
            '\t' => writer.write_all(b"\\t")?,
            c if c < '\x20' => {
                // Control characters
                write!(writer, "\\u{:04x}", c as u32)?;
            }
            c => {
                // Regular character - write as UTF-8
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                writer.write_all(encoded.as_bytes())?;
            }
        }
    }

    writer.write_all(b"\"")
}

/// Create a deterministic hash of a canonicalized JSON object.
pub fn canonical_hash(value: &Value) -> Result<String> {
    use sha2::{Digest, Sha256};

    let canonical = canonicalize(value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let result = hasher.finalize();

    Ok(hex::encode(result))
}

/// Create a deterministic hash of a serializable object.
pub fn canonical_hash_object<T: serde::Serialize>(obj: &T) -> Result<String> {
    let value = serde_json::to_value(obj)
        .map_err(|e| Error::Custom(format!("Serialization error: {}", e)))?;
    canonical_hash(&value)
}

/// Sort a JSON object's keys recursively.
pub fn sort_object_keys(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let mut sorted: IndexMap<String, Value> = IndexMap::new();
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort_by(|a, b| compare_strings_utf16(a, b));

            for key in keys {
                if let Some(v) = obj.get(key) {
                    sorted.insert(key.clone(), sort_object_keys(v));
                }
            }

            // Convert to serde_json Map
            let map: Map<String, Value> = sorted.into_iter().collect();
            Value::Object(map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sort_object_keys).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonicalize_object_key_order() {
        let value = json!({"b": 2, "a": 1, "c": 3});
        let canonical = canonicalize(&value).unwrap();
        assert_eq!(canonical, r#"{"a":1,"b":2,"c":3}"#);
    }

    #[test]
    fn test_canonicalize_nested_object() {
        let value = json!({"outer": {"b": 2, "a": 1}});
        let canonical = canonicalize(&value).unwrap();
        assert_eq!(canonical, r#"{"outer":{"a":1,"b":2}}"#);
    }

    #[test]
    fn test_canonicalize_string_escaping() {
        let value = json!({"key": "hello\nworld"});
        let canonical = canonicalize(&value).unwrap();
        assert_eq!(canonical, r#"{"key":"hello\nworld"}"#);
    }

    #[test]
    fn test_canonicalize_unicode() {
        let value = json!({"key": "日本語"});
        let canonical = canonicalize(&value).unwrap();
        assert!(canonical.contains("日本語"));
    }

    #[test]
    fn test_canonicalize_numbers() {
        assert_eq!(canonicalize(&json!(0)).unwrap(), "0");
        assert_eq!(canonicalize(&json!(1)).unwrap(), "1");
        assert_eq!(canonicalize(&json!(-1)).unwrap(), "-1");
        assert_eq!(canonicalize(&json!(1.5)).unwrap(), "1.5");
    }

    #[test]
    fn test_canonicalize_array() {
        let value = json!([3, 1, 2]);
        let canonical = canonicalize(&value).unwrap();
        assert_eq!(canonical, "[3,1,2]");
    }

    #[test]
    fn test_canonicalize_null_bool() {
        assert_eq!(canonicalize(&json!(null)).unwrap(), "null");
        assert_eq!(canonicalize(&json!(true)).unwrap(), "true");
        assert_eq!(canonicalize(&json!(false)).unwrap(), "false");
    }

    #[test]
    fn test_utf16_key_ordering() {
        // Test that keys are sorted by UTF-16 code units
        let value = json!({"ä": 1, "a": 2, "z": 3});
        let canonical = canonicalize(&value).unwrap();
        // 'a' < 'z' < 'ä' in UTF-16 order
        assert!(canonical.starts_with(r#"{"a":2"#));
    }

    #[test]
    fn test_canonical_hash() {
        let value = json!({"b": 2, "a": 1});
        let hash = canonical_hash(&value).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 hex string

        // Same content, different order should produce same hash
        let value2 = json!({"a": 1, "b": 2});
        let hash2 = canonical_hash(&value2).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_sort_object_keys() {
        let value = json!({"c": {"y": 1, "x": 2}, "a": 1, "b": [{"z": 1, "a": 2}]});
        let sorted = sort_object_keys(&value);

        // Verify first key is "a"
        if let Value::Object(map) = sorted {
            let keys: Vec<&String> = map.keys().collect();
            assert_eq!(keys[0], "a");
            assert_eq!(keys[1], "b");
            assert_eq!(keys[2], "c");
        } else {
            panic!("Expected object");
        }
    }
}
