use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::ports::TcpHealthChecker;

const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

/// Real `TcpHealthChecker` — attempts a TCP connect to `host:port`,
/// bounded by `HEALTH_CHECK_TIMEOUT` so an unreachable/firewalled KMS
/// server fails fast instead of hanging on the OS's own (much longer)
/// connect timeout.
pub struct TokioTcpHealthChecker;

impl TcpHealthChecker for TokioTcpHealthChecker {
    async fn is_reachable(&self, host: &str, port: u16) -> bool {
        matches!(
            timeout(HEALTH_CHECK_TIMEOUT, TcpStream::connect((host, port))).await,
            Ok(Ok(_))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn is_reachable_returns_true_for_an_open_local_port() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        let checker = TokioTcpHealthChecker;
        assert!(checker.is_reachable("127.0.0.1", port).await);
    }

    #[tokio::test]
    async fn is_reachable_returns_false_when_nothing_is_listening() {
        // Bind to get a free port, then immediately drop the listener so
        // nothing is actually accepting connections on it.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let checker = TokioTcpHealthChecker;
        assert!(!checker.is_reachable("127.0.0.1", port).await);
    }
}
