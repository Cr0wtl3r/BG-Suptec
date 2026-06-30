use std::time::{Duration, Instant};

/// Number of failed attempts allowed before exponential backoff kicks in.
const FAILURE_THRESHOLD: u32 = 5;

/// Tracks failed login attempts and applies exponential backoff
/// once the failure threshold is reached.
pub struct RateLimiter {
    failed_attempts: u32,
    locked_until: Option<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            failed_attempts: 0,
            locked_until: None,
        }
    }

    /// Records a failed login attempt and returns the backoff duration
    /// applied as a result (zero if still under the failure threshold).
    pub fn record_failure(&mut self) -> Duration {
        self.failed_attempts += 1;
        let backoff = Self::backoff_for(self.failed_attempts);
        self.locked_until = if backoff > Duration::ZERO {
            Some(Instant::now() + backoff)
        } else {
            None
        };
        backoff
    }

    /// Resets the limiter after a successful login.
    pub fn record_success(&mut self) {
        self.failed_attempts = 0;
        self.locked_until = None;
    }

    /// Returns true if currently within an active backoff window.
    pub fn is_locked(&self) -> bool {
        self.locked_until
            .map(|until| Instant::now() < until)
            .unwrap_or(false)
    }

    fn backoff_for(attempts: u32) -> Duration {
        if attempts < FAILURE_THRESHOLD {
            return Duration::ZERO;
        }
        let exponent = (attempts - FAILURE_THRESHOLD).min(10);
        Duration::from_secs(1u64 << exponent)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::RateLimiter;
    use std::time::Duration;

    #[test]
    fn applies_no_backoff_before_threshold() {
        let mut limiter = RateLimiter::new();
        for _ in 0..4 {
            let backoff = limiter.record_failure();
            assert_eq!(backoff, Duration::ZERO);
        }
        assert!(!limiter.is_locked());
    }

    #[test]
    fn fifth_failed_attempt_applies_exponential_backoff() {
        let mut limiter = RateLimiter::new();
        for _ in 0..4 {
            limiter.record_failure();
        }
        let backoff = limiter.record_failure();
        assert_eq!(backoff, Duration::from_secs(1));
        assert!(limiter.is_locked());
    }

    #[test]
    fn backoff_grows_exponentially_with_further_failures() {
        let mut limiter = RateLimiter::new();
        for _ in 0..4 {
            limiter.record_failure();
        }
        let fifth = limiter.record_failure();
        let sixth = limiter.record_failure();
        let seventh = limiter.record_failure();
        assert_eq!(fifth, Duration::from_secs(1));
        assert_eq!(sixth, Duration::from_secs(2));
        assert_eq!(seventh, Duration::from_secs(4));
    }

    #[test]
    fn record_success_resets_failed_attempts_and_lock() {
        let mut limiter = RateLimiter::new();
        for _ in 0..5 {
            limiter.record_failure();
        }
        assert!(limiter.is_locked());

        limiter.record_success();
        assert!(!limiter.is_locked());

        let backoff = limiter.record_failure();
        assert_eq!(backoff, Duration::ZERO);
    }
}
