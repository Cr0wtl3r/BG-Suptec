use std::sync::Mutex;
use tauri::State;

use crate::auth::{self, RateLimiter};

/// Tauri-managed state holding the loaded password hash and the
/// login rate limiter, shared across invocations of the `login` command.
pub struct AuthState {
    password_hash: String,
    rate_limiter: Mutex<RateLimiter>,
}

impl AuthState {
    pub fn new(password_hash: String) -> Self {
        Self {
            password_hash,
            rate_limiter: Mutex::new(RateLimiter::new()),
        }
    }
}

/// Pure login attempt logic, independent of the Tauri runtime so it can be
/// unit tested directly without constructing a Tauri `State`.
fn attempt_login(senha: &str, password_hash: &str, limiter: &mut RateLimiter) -> Result<bool, String> {
    if limiter.is_locked() {
        return Err("Muitas tentativas. Aguarde antes de tentar novamente.".to_string());
    }

    match auth::verify_password(senha, password_hash) {
        auth::VerifyResult::Match => {
            limiter.record_success();
            Ok(true)
        }
        auth::VerifyResult::NoMatch => {
            limiter.record_failure();
            Ok(false)
        }
        auth::VerifyResult::Error(e) => {
            limiter.record_failure();
            Err(e)
        }
    }
}

#[tauri::command]
pub fn login(senha: String, state: State<AuthState>) -> Result<bool, String> {
    let mut limiter = state
        .rate_limiter
        .lock()
        .map_err(|_| "Erro interno de autenticação.".to_string())?;

    attempt_login(&senha, &state.password_hash, &mut limiter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{hash_password, RateLimiter};

    #[test]
    fn attempt_login_succeeds_with_correct_password() {
        let hash = hash_password("correta").unwrap();
        let mut limiter = RateLimiter::new();

        assert_eq!(attempt_login("correta", &hash, &mut limiter), Ok(true));
    }

    #[test]
    fn attempt_login_fails_with_wrong_password() {
        let hash = hash_password("correta").unwrap();
        let mut limiter = RateLimiter::new();

        assert_eq!(attempt_login("errada", &hash, &mut limiter), Ok(false));
    }

    #[test]
    fn attempt_login_blocks_after_rate_limit_triggers() {
        let hash = hash_password("correta").unwrap();
        let mut limiter = RateLimiter::new();

        for _ in 0..5 {
            let _ = attempt_login("errada", &hash, &mut limiter);
        }

        assert!(attempt_login("correta", &hash, &mut limiter).is_err());
    }

    #[test]
    fn auth_state_new_starts_unlocked() {
        let state = AuthState::new("qualquer-hash".to_string());
        let limiter = state.rate_limiter.lock().unwrap();

        assert!(!limiter.is_locked());
    }
}
