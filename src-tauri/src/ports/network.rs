/// Parsed network configuration of the active physical interface
/// (the one with a default gateway), already resolved from raw adapter
/// output — no string/regex parsing belongs outside the adapter that
/// produces this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub mac_address: String,
    pub ip_address: String,
    pub subnet_mask: String,
    pub default_gateway: String,
    pub dns_primary: Option<String>,
    pub dns_secondary: Option<String>,
}

/// Abstraction over discovering the active network interface's
/// configuration, so domain logic can be unit tested without spawning
/// real PowerShell processes.
pub trait NetworkReader {
    /// Returns the configuration of the active physical interface
    /// (the one with a default gateway), or `None` if none is found.
    async fn active_interface_info(&self) -> Option<NetworkInfo>;
}
