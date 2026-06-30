/// Checks whether a KMS server is reachable on a given TCP port before an
/// activation flow spends time on a full `/sethst`+`/act` round trip.
/// Injected so domain orchestration (Office's KMS server fallback loop)
/// stays unit testable without opening real sockets.
pub trait TcpHealthChecker {
    async fn is_reachable(&self, host: &str, port: u16) -> bool;
}
