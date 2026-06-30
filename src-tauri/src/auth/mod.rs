use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Version, Params,
};
mod rate_limiter;
pub use rate_limiter::RateLimiter;

/// Configuration for argon2id hashing.
/// Uses memory-hard parameters resistant to GPU/ASIC attacks.
pub struct AuthConfig {
    /// Memory cost in KiB (64 MiB = 65536)
    pub memory_cost: u32,
    /// Number of iterations
    pub time_cost: u32,
    /// Degree of parallelism
    pub parallelism: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MiB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

/// Result of a password verification attempt.
#[derive(Debug, PartialEq)]
pub enum VerifyResult {
    /// Password matches the stored hash.
    Match,
    /// Password does not match.
    NoMatch,
    /// Hash string is malformed or verification failed due to an error.
    Error(String),
}

/// Generate a new argon2id hash from a plaintext password.
///
/// Returns the PHC-encoded hash string suitable for storage.
pub fn hash_password(password: &str) -> Result<String, String> {
    let config = AuthConfig::default();
    let params = Params::new(
        config.memory_cost,
        config.time_cost,
        config.parallelism,
        None,
    )
    .map_err(|e| format!("Failed to create argon2 params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    Ok(hash.to_string())
}

/// Verify a plaintext password against a stored PHC hash string.
///
/// # Arguments
/// * `password` - The plaintext password to verify
/// * `stored_hash` - The PHC-encoded argon2id hash string
///
/// # Returns
/// * `VerifyResult::Match` if the password matches
/// * `VerifyResult::NoMatch` if it doesn't match
/// * `VerifyResult::Error` if the hash is malformed
pub fn verify_password(password: &str, stored_hash: &str) -> VerifyResult {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(h) => h,
        Err(e) => return VerifyResult::Error(format!("Invalid hash format: {}", e)),
    };

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => VerifyResult::Match,
        Err(_) => VerifyResult::NoMatch,
    }
}

/// Loads a PHC-encoded argon2id hash string from an external file
/// (e.g. `auth.hash` alongside the executable), so the password
/// can be rotated without recompiling the app.
pub fn load_hash_from_file(path: &std::path::Path) -> std::io::Result<String> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password_returns_match_for_correct_password() {
        let password = "minhasenha123";
        let hash = hash_password(password).expect("hashing should succeed");

        assert_eq!(verify_password(password, &hash), VerifyResult::Match);
    }

    #[test]
    fn verify_password_returns_no_match_for_wrong_password() {
        let password = "minhasenha123";
        let hash = hash_password(password).expect("hashing should succeed");

        assert_eq!(
            verify_password("senhaerrada", &hash),
            VerifyResult::NoMatch
        );
    }

    #[test]
    fn verify_password_returns_no_match_for_empty_password() {
        let password = "minhasenha123";
        let hash = hash_password(password).expect("hashing should succeed");

        assert_eq!(verify_password("", &hash), VerifyResult::NoMatch);
    }

    #[test]
    fn verify_password_returns_error_for_invalid_hash() {
        match verify_password("anything", "not-a-valid-hash") {
            VerifyResult::Error(_) => {}
            other => panic!("expected VerifyResult::Error, got {:?}", other),
        }
    }

    #[test]
    fn hash_password_produces_argon2id_hash() {
        let hash = hash_password("testpassword").expect("hashing should succeed");
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn hash_password_different_each_time_due_to_random_salt() {
        let password = "samepassword";
        let hash1 = hash_password(password).expect("hash1 should succeed");
        let hash2 = hash_password(password).expect("hash2 should succeed");

        // Different salts → different hash strings
        assert_ne!(hash1, hash2);

        // But both verify correctly
        assert_eq!(verify_password(password, &hash1), VerifyResult::Match);
        assert_eq!(verify_password(password, &hash2), VerifyResult::Match);
    }

    #[test]
    fn verify_password_handles_special_characters() {
        let password = "p@$$w0rd!#%^&*()_+{}|:<>?";
        let hash = hash_password(password).expect("hashing should succeed");
        assert_eq!(verify_password(password, &hash), VerifyResult::Match);
    }

    #[test]
    fn verify_password_handles_unicode() {
        let password = "senha-com-acentos-çaçoê";
        let hash = hash_password(password).expect("hashing should succeed");
        assert_eq!(verify_password(password, &hash), VerifyResult::Match);
    }

    #[test]
    fn load_hash_from_file_reads_trimmed_contents() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("bg-suptec-test-auth-{}.hash", std::process::id()));
        let hash = hash_password("senha_do_arquivo").expect("hashing should succeed");
        std::fs::write(&path, format!("{hash}\r\n")).expect("should write temp hash file");

        let loaded = load_hash_from_file(&path).expect("should load hash from file");

        std::fs::remove_file(&path).ok();

        assert_eq!(loaded, hash);
        assert_eq!(
            verify_password("senha_do_arquivo", &loaded),
            VerifyResult::Match
        );
    }

    #[test]
    fn load_hash_from_file_errors_when_missing() {
        let path = std::env::temp_dir().join("bg-suptec-test-auth-missing.hash");
        std::fs::remove_file(&path).ok();

        assert!(load_hash_from_file(&path).is_err());
    }
}
