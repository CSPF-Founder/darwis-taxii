//! Password hashing using scrypt.
//!
//! Format: scrypt:n:r:p$salt$hash
//! Example: scrypt:32768:8:1$K2a4Wu51MKGutWTo$hash

use scrypt::{Params, scrypt};
use subtle::ConstantTimeEq;

/// Default log2(N) for scrypt (N = 32768).
const DEFAULT_LOG_N: u8 = 15;

/// Default r parameter (block size).
const DEFAULT_R: u32 = 8;

/// Default p parameter (parallelization).
const DEFAULT_P: u32 = 1;

/// Output length in bytes.
const OUTPUT_LEN: usize = 64;

/// Salt length in bytes.
const SALT_LENGTH: usize = 16;

/// Generate a password hash.
///
/// Format: scrypt:n:r:p$salt$hash
/// Where n=32768, r=8, p=1
#[expect(clippy::expect_used, reason = "infallible: valid scrypt parameters")]
pub fn generate_password_hash(password: &str) -> String {
    use rand::Rng;

    // Generate random salt
    let mut salt_bytes = [0u8; SALT_LENGTH];
    rand::rng().fill(&mut salt_bytes);

    // Use URL-safe base64 without padding
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let salt = URL_SAFE_NO_PAD.encode(salt_bytes);

    // Create params with known-valid values
    let params =
        Params::new(DEFAULT_LOG_N, DEFAULT_R, DEFAULT_P, OUTPUT_LEN).expect("valid scrypt params");

    // Derive key (64 bytes) - safe to expect because output length matches hash size
    let mut hash = [0u8; 64];
    scrypt(password.as_bytes(), salt.as_bytes(), &params, &mut hash)
        .expect("valid scrypt output length");

    // Format: scrypt:n:r:p$salt$hash
    // n = 2^15 = 32768
    format!("scrypt:32768:8:1${}${}", salt, hex::encode(hash))
}

/// Check a password against a scrypt hash.
pub fn check_password_hash(hash: &str, password: &str) -> bool {
    // Parse format: scrypt:n:r:p$salt$hash
    let parts: Vec<&str> = hash.split('$').collect();
    if parts.len() != 3 {
        return false;
    }

    // Parse scrypt:n:r:p
    let method_parts: Vec<&str> = parts[0].split(':').collect();
    if method_parts.len() != 4 || method_parts[0] != "scrypt" {
        return false;
    }

    let Ok(n) = method_parts[1].parse::<u64>() else {
        return false;
    };
    let Ok(r) = method_parts[2].parse::<u32>() else {
        return false;
    };
    let Ok(p) = method_parts[3].parse::<u32>() else {
        return false;
    };

    // Convert n to log2(n)
    let log_n = (n as f64).log2() as u8;

    let salt = parts[1];
    let expected_hash = parts[2];

    // Detect output length from stored hash (hex-encoded, so len/2 = bytes)
    let output_len = expected_hash.len() / 2;
    if output_len == 0 || output_len > 64 {
        return false;
    }

    // Create params with detected length
    let Ok(params) = Params::new(log_n, r, p, output_len) else {
        return false;
    };

    // Derive key with same length as stored hash
    let mut computed = vec![0u8; output_len];
    if scrypt(password.as_bytes(), salt.as_bytes(), &params, &mut computed).is_err() {
        return false;
    }

    // Constant-time comparison to prevent timing attacks
    let computed_hex = hex::encode(computed);
    computed_hex
        .as_bytes()
        .ct_eq(expected_hash.as_bytes())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_check() {
        let password = "test_password";
        let hash = generate_password_hash(password);

        assert!(hash.starts_with("scrypt:32768:8:1$"));
        assert!(check_password_hash(&hash, password));
        assert!(!check_password_hash(&hash, "wrong_password"));
    }

    #[test]
    fn test_scrypt_format() {
        let hash = "scrypt:32768:8:1$abcd1234$0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let parts: Vec<&str> = hash.split('$').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "scrypt:32768:8:1");
    }

    #[test]
    fn test_known_hash() {
        // Known valid hash for 'testpass123'
        let hash = "scrypt:32768:8:1$N50ZCQSW6AvqH9ra$c1f158d18766f1991f34ac71071c66c2aec70440aadb9286d88086491645fc1da71a2bb4de3cde87bb4eaaf50fb89f537a97176b2541d5f29369f6d7d926754c";
        assert!(check_password_hash(hash, "testpass123"));
        assert!(!check_password_hash(hash, "wrongpassword"));
    }

    #[test]
    fn test_invalid_format() {
        // Wrong method
        assert!(!check_password_hash(
            "pbkdf2:sha256:600000$salt$hash",
            "password"
        ));

        // Wrong number of parts
        assert!(!check_password_hash("scrypt:32768:8:1$salt", "password"));

        // Invalid n value
        assert!(!check_password_hash(
            "scrypt:invalid:8:1$salt$hash",
            "password"
        ));
    }
}
