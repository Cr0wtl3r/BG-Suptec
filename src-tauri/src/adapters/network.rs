use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::ports::{NetworkInfo, NetworkReader};

use super::powershell;

/// Finds the active physical network adapter (the one with a default
/// gateway) and emits its configuration as compact JSON. Static script,
/// no interpolated input — safe to run as-is via `powershell::run_script`.
const ACTIVE_INTERFACE_SCRIPT: &str = r#"
$netAdapter = Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' } | ForEach-Object {
    $ipConfig = Get-NetIPConfiguration -InterfaceAlias $_.Name
    if ($ipConfig.IPv4DefaultGateway) {
        return $_
    }
} | Select-Object -First 1

if ($netAdapter) {
    $ipConfig = Get-NetIPConfiguration -InterfaceAlias $netAdapter.Name
    $dnsServers = (Get-DnsClientServerAddress -InterfaceAlias $netAdapter.Name -AddressFamily IPv4).ServerAddresses

    @{
        InterfaceAlias = $netAdapter.Name
        MacAddress = $netAdapter.MacAddress
        IPv4Address = $ipConfig.IPv4Address.IPAddress
        PrefixLength = $ipConfig.IPv4Address.PrefixLength
        DefaultGateway = $ipConfig.IPv4DefaultGateway.NextHop
        DNSServers = $dnsServers
    } | ConvertTo-Json -Compress -Depth 3
}
"#;

/// Raw shape of the PowerShell `ConvertTo-Json` output. Fields that may
/// come back as either a bare scalar or a single-element array (a known
/// `ConvertTo-Json` quirk for PowerShell array properties) are typed as
/// `JsonValue` and normalized by `first_string`/`all_strings` below,
/// instead of the regex-based extraction the legacy code used.
#[derive(Debug, Deserialize)]
struct RawNetworkInfo {
    #[serde(rename = "InterfaceAlias")]
    interface_alias: String,
    #[serde(rename = "MacAddress")]
    mac_address: String,
    #[serde(rename = "IPv4Address", default)]
    ipv4_address: Option<JsonValue>,
    #[serde(rename = "PrefixLength", default)]
    prefix_length: Option<JsonValue>,
    #[serde(rename = "DefaultGateway", default)]
    default_gateway: Option<JsonValue>,
    #[serde(rename = "DNSServers", default)]
    dns_servers: Option<JsonValue>,
}

fn first_string(value: &Option<JsonValue>) -> Option<String> {
    match value {
        Some(JsonValue::String(s)) => Some(s.clone()),
        Some(JsonValue::Number(n)) => Some(n.to_string()),
        Some(JsonValue::Array(items)) => items.first().and_then(|v| match v {
            JsonValue::String(s) => Some(s.clone()),
            JsonValue::Number(n) => Some(n.to_string()),
            _ => None,
        }),
        _ => None,
    }
}

fn all_strings(value: &Option<JsonValue>) -> Vec<String> {
    match value {
        Some(JsonValue::String(s)) => vec![s.clone()],
        Some(JsonValue::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

fn prefix_to_subnet_mask(prefix: u8) -> Option<String> {
    if prefix > 32 {
        return None;
    }
    let mask: u32 = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    let octets = mask.to_be_bytes();
    Some(format!(
        "{}.{}.{}.{}",
        octets[0], octets[1], octets[2], octets[3]
    ))
}

/// Parses the script's JSON output into a `NetworkInfo`. Returns `Ok(None)`
/// when there's simply no active interface (empty output, the expected
/// case on a machine with no connected gateway), and `Err` only when the
/// output is non-empty but not valid/expected JSON.
fn parse_active_interface_info(output: &str) -> Result<Option<NetworkInfo>, String> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let raw: RawNetworkInfo = serde_json::from_str(trimmed)
        .map_err(|e| format!("falha ao interpretar JSON de informações de rede: {e}"))?;

    let dns_servers = all_strings(&raw.dns_servers);
    let prefix_length = first_string(&raw.prefix_length).and_then(|s| s.parse::<u8>().ok());

    Ok(Some(NetworkInfo {
        interface_name: raw.interface_alias,
        mac_address: raw.mac_address.replace('-', ":"),
        ip_address: first_string(&raw.ipv4_address).unwrap_or_default(),
        subnet_mask: prefix_length
            .and_then(prefix_to_subnet_mask)
            .unwrap_or_else(|| "Inválida".to_string()),
        default_gateway: first_string(&raw.default_gateway).unwrap_or_default(),
        dns_primary: dns_servers.first().cloned(),
        dns_secondary: dns_servers.get(1).cloned(),
    }))
}

/// Discovers the active network interface's configuration by running a
/// static `Get-NetAdapter`/`Get-NetIPConfiguration` PowerShell script and
/// parsing its JSON output with `serde_json`.
pub struct PowerShellNetworkReader;

impl NetworkReader for PowerShellNetworkReader {
    async fn active_interface_info(&self) -> Option<NetworkInfo> {
        let output = powershell::run_script(ACTIVE_INTERFACE_SCRIPT).await.ok()?;
        parse_active_interface_info(&output).ok().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_output_with_two_dns_servers() {
        let json = r#"{"InterfaceAlias":"Ethernet","MacAddress":"AA-BB-CC-DD-EE-FF","IPv4Address":"192.168.1.50","PrefixLength":24,"DefaultGateway":"192.168.1.1","DNSServers":["8.8.8.8","8.8.4.4"]}"#;

        let info = parse_active_interface_info(json).unwrap().unwrap();

        assert_eq!(info.interface_name, "Ethernet");
        assert_eq!(info.mac_address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(info.ip_address, "192.168.1.50");
        assert_eq!(info.subnet_mask, "255.255.255.0");
        assert_eq!(info.default_gateway, "192.168.1.1");
        assert_eq!(info.dns_primary.as_deref(), Some("8.8.8.8"));
        assert_eq!(info.dns_secondary.as_deref(), Some("8.8.4.4"));
    }

    #[test]
    fn parses_output_with_a_single_dns_server_collapsed_to_a_scalar() {
        // PowerShell's ConvertTo-Json collapses a 1-element array property
        // into a bare scalar — this is exactly the shape that broke the
        // legacy regex-based parser and motivated switching to serde_json.
        let json = r#"{"InterfaceAlias":"Wi-Fi","MacAddress":"00-11-22-33-44-55","IPv4Address":"10.0.0.5","PrefixLength":24,"DefaultGateway":"10.0.0.1","DNSServers":"1.1.1.1"}"#;

        let info = parse_active_interface_info(json).unwrap().unwrap();

        assert_eq!(info.dns_primary.as_deref(), Some("1.1.1.1"));
        assert_eq!(info.dns_secondary, None);
    }

    #[test]
    fn parses_output_with_escaped_quotes_in_interface_name() {
        // A regex-based extractor (the legacy approach) is fragile against
        // escaped quotes inside a JSON string value; serde_json is not.
        let json = r#"{"InterfaceAlias":"Placa \"Especial\"","MacAddress":"00-11-22-33-44-55","IPv4Address":"10.0.0.5","PrefixLength":24,"DefaultGateway":"10.0.0.1","DNSServers":null}"#;

        let info = parse_active_interface_info(json).unwrap().unwrap();

        assert_eq!(info.interface_name, "Placa \"Especial\"");
        assert_eq!(info.dns_primary, None);
        assert_eq!(info.dns_secondary, None);
    }

    #[test]
    fn returns_none_for_empty_output_when_no_active_interface() {
        let info = parse_active_interface_info("   ").unwrap();
        assert!(info.is_none());
    }

    #[test]
    fn returns_err_for_malformed_json() {
        let result = parse_active_interface_info("{not json");
        assert!(result.is_err());
    }

    #[test]
    fn prefix_to_subnet_mask_converts_common_prefixes() {
        assert_eq!(prefix_to_subnet_mask(24).as_deref(), Some("255.255.255.0"));
        assert_eq!(prefix_to_subnet_mask(16).as_deref(), Some("255.255.0.0"));
        assert_eq!(prefix_to_subnet_mask(32).as_deref(), Some("255.255.255.255"));
        assert_eq!(prefix_to_subnet_mask(0).as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn prefix_to_subnet_mask_rejects_out_of_range_prefix() {
        assert_eq!(prefix_to_subnet_mask(33), None);
    }
}
