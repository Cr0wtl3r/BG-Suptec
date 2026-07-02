use std::ffi::CStr;
use std::net::Ipv4Addr;
use std::{ptr, slice};

use windows_sys::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NO_DATA, ERROR_SUCCESS};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_SKIP_ANYCAST,
    GAA_FLAG_SKIP_MULTICAST, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211, IP_ADAPTER_ADDRESSES_LH,
    IP_ADAPTER_DNS_SERVER_ADDRESS_XP, IP_ADAPTER_GATEWAY_ADDRESS_LH, IP_ADAPTER_UNICAST_ADDRESS_LH,
};
use windows_sys::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_UNSPEC, SOCKADDR, SOCKADDR_IN};

use crate::ports::{NetworkInfo, NetworkReader};

const INITIAL_ADAPTER_BUFFER_SIZE: u32 = 15_000;
const ADAPTER_QUERY_FLAGS: u32 =
    GAA_FLAG_INCLUDE_GATEWAYS | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdapterSnapshot {
    interface_name: String,
    mac_address: String,
    ipv4_address: String,
    prefix_length: u8,
    default_gateway: Option<String>,
    dns_servers: Vec<String>,
    metric: u32,
    oper_status_up: bool,
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

fn select_active_adapter(adapters: Vec<AdapterSnapshot>) -> Option<NetworkInfo> {
    adapters
        .into_iter()
        .filter(|adapter| {
            adapter.oper_status_up
                && !adapter.ipv4_address.is_empty()
                && adapter
                    .default_gateway
                    .as_ref()
                    .map(|gateway| !gateway.is_empty())
                    .unwrap_or(false)
        })
        .min_by_key(|adapter| adapter.metric)
        .map(network_info_from_snapshot)
}

fn network_info_from_snapshot(snapshot: AdapterSnapshot) -> NetworkInfo {
    let dns_primary = snapshot.dns_servers.first().cloned();
    let dns_secondary = snapshot.dns_servers.get(1).cloned();

    NetworkInfo {
        interface_name: snapshot.interface_name,
        mac_address: snapshot.mac_address,
        ip_address: snapshot.ipv4_address,
        subnet_mask: prefix_to_subnet_mask(snapshot.prefix_length)
            .unwrap_or_else(|| "Inválida".to_string()),
        default_gateway: snapshot.default_gateway.unwrap_or_default(),
        dns_primary,
        dns_secondary,
    }
}

fn read_adapter_snapshots() -> Result<Vec<AdapterSnapshot>, u32> {
    let mut buffer_size = INITIAL_ADAPTER_BUFFER_SIZE;

    loop {
        let mut buffer = vec![0usize; words_for_bytes(buffer_size)];
        let mut actual_size = buffer_size;

        // GetAdaptersAddresses writes a linked list into caller-provided memory.
        // The Vec<usize> allocation gives the buffer pointer-aligned storage.
        let result = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC as u32,
                ADAPTER_QUERY_FLAGS,
                ptr::null(),
                buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>(),
                &mut actual_size,
            )
        };

        match result {
            ERROR_SUCCESS => {
                let head = buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
                return Ok(adapter_list_to_snapshots(head));
            }
            ERROR_BUFFER_OVERFLOW => {
                buffer_size = if actual_size > buffer_size {
                    actual_size
                } else {
                    buffer_size.saturating_mul(2)
                };
            }
            ERROR_NO_DATA => return Ok(Vec::new()),
            code => return Err(code),
        }
    }
}

fn words_for_bytes(bytes: u32) -> usize {
    let word_size = std::mem::size_of::<usize>();
    (bytes as usize).div_ceil(word_size)
}

fn adapter_list_to_snapshots(head: *mut IP_ADAPTER_ADDRESSES_LH) -> Vec<AdapterSnapshot> {
    let mut snapshots = Vec::new();
    let mut current = head;

    while !current.is_null() {
        // `current` points inside the buffer owned by `read_adapter_snapshots`
        // and remains valid for the duration of this traversal.
        let adapter = unsafe { &*current };
        if let Some(snapshot) = adapter_to_snapshot(adapter) {
            snapshots.push(snapshot);
        }
        current = adapter.Next;
    }

    snapshots
}

fn adapter_to_snapshot(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<AdapterSnapshot> {
    if !is_supported_physical_adapter(adapter.IfType) {
        return None;
    }

    let (ipv4_address, prefix_length) = first_unicast_ipv4(adapter.FirstUnicastAddress)?;

    Some(AdapterSnapshot {
        interface_name: adapter_display_name(adapter),
        mac_address: format_mac_address(adapter),
        ipv4_address,
        prefix_length,
        default_gateway: first_gateway_ipv4(adapter.FirstGatewayAddress),
        dns_servers: dns_server_ipv4s(adapter.FirstDnsServerAddress),
        metric: adapter.Ipv4Metric,
        oper_status_up: adapter.OperStatus == IfOperStatusUp,
    })
}

fn is_supported_physical_adapter(if_type: u32) -> bool {
    matches!(if_type, IF_TYPE_ETHERNET_CSMACD | IF_TYPE_IEEE80211)
}

fn first_unicast_ipv4(mut current: *mut IP_ADAPTER_UNICAST_ADDRESS_LH) -> Option<(String, u8)> {
    while !current.is_null() {
        let unicast = unsafe { &*current };
        if let Some(ipv4) = socket_address_to_ipv4(unicast.Address.lpSockaddr) {
            return Some((ipv4, unicast.OnLinkPrefixLength));
        }
        current = unicast.Next;
    }

    None
}

fn first_gateway_ipv4(mut current: *mut IP_ADAPTER_GATEWAY_ADDRESS_LH) -> Option<String> {
    while !current.is_null() {
        let gateway = unsafe { &*current };
        if let Some(ipv4) = socket_address_to_ipv4(gateway.Address.lpSockaddr) {
            return Some(ipv4);
        }
        current = gateway.Next;
    }

    None
}

fn dns_server_ipv4s(mut current: *mut IP_ADAPTER_DNS_SERVER_ADDRESS_XP) -> Vec<String> {
    let mut dns_servers = Vec::new();

    while !current.is_null() {
        let dns_server = unsafe { &*current };
        if let Some(ipv4) = socket_address_to_ipv4(dns_server.Address.lpSockaddr) {
            dns_servers.push(ipv4);
        }
        current = dns_server.Next;
    }

    dns_servers
}

fn socket_address_to_ipv4(sockaddr: *const SOCKADDR) -> Option<String> {
    if sockaddr.is_null() {
        return None;
    }

    let family = unsafe { (*sockaddr).sa_family };
    if family != AF_INET {
        return None;
    }

    let socket_address = unsafe { &*(sockaddr.cast::<SOCKADDR_IN>()) };
    let address = unsafe { socket_address.sin_addr.S_un.S_addr };
    Some(Ipv4Addr::from(address.to_ne_bytes()).to_string())
}

fn adapter_display_name(adapter: &IP_ADAPTER_ADDRESSES_LH) -> String {
    wide_ptr_to_string(adapter.FriendlyName)
        .or_else(|| wide_ptr_to_string(adapter.Description))
        .or_else(|| pstr_to_string(adapter.AdapterName))
        .unwrap_or_else(|| "Adaptador de rede".to_string())
}

fn wide_ptr_to_string(ptr: *const u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    let mut len = 0usize;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    if len == 0 {
        None
    } else {
        let value = unsafe { slice::from_raw_parts(ptr, len) };
        Some(String::from_utf16_lossy(value))
    }
}

fn pstr_to_string(ptr: *const u8) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(ptr.cast()) }
        .to_string_lossy()
        .into_owned();

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn format_mac_address(adapter: &IP_ADAPTER_ADDRESSES_LH) -> String {
    let len = adapter
        .PhysicalAddressLength
        .min(adapter.PhysicalAddress.len() as u32) as usize;

    adapter.PhysicalAddress[..len]
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

/// Discovers the active network interface's configuration through the
/// native Windows IP Helper API, avoiding PowerShell startup and JSON parsing.
pub struct NativeNetworkReader;

impl NetworkReader for NativeNetworkReader {
    async fn active_interface_info(&self) -> Option<NetworkInfo> {
        let adapters = read_adapter_snapshots().ok()?;
        select_active_adapter(adapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_to_subnet_mask_converts_common_prefixes() {
        assert_eq!(prefix_to_subnet_mask(24).as_deref(), Some("255.255.255.0"));
        assert_eq!(prefix_to_subnet_mask(16).as_deref(), Some("255.255.0.0"));
        assert_eq!(
            prefix_to_subnet_mask(32).as_deref(),
            Some("255.255.255.255")
        );
        assert_eq!(prefix_to_subnet_mask(0).as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn prefix_to_subnet_mask_rejects_out_of_range_prefix() {
        assert_eq!(prefix_to_subnet_mask(33), None);
    }

    #[test]
    fn select_active_adapter_prefers_up_ipv4_gateway_with_lowest_metric() {
        let selected = select_active_adapter(vec![
            AdapterSnapshot {
                interface_name: "Ethernet 2".to_string(),
                mac_address: "AA:BB:CC:DD:EE:01".to_string(),
                ipv4_address: "10.0.0.20".to_string(),
                prefix_length: 24,
                default_gateway: Some("10.0.0.1".to_string()),
                dns_servers: vec!["8.8.8.8".to_string()],
                metric: 50,
                oper_status_up: true,
            },
            AdapterSnapshot {
                interface_name: "Ethernet".to_string(),
                mac_address: "AA:BB:CC:DD:EE:02".to_string(),
                ipv4_address: "192.168.1.30".to_string(),
                prefix_length: 24,
                default_gateway: Some("192.168.1.1".to_string()),
                dns_servers: vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()],
                metric: 10,
                oper_status_up: true,
            },
        ])
        .unwrap();

        assert_eq!(selected.interface_name, "Ethernet");
        assert_eq!(selected.default_gateway, "192.168.1.1");
        assert_eq!(selected.dns_primary.as_deref(), Some("1.1.1.1"));
        assert_eq!(selected.dns_secondary.as_deref(), Some("1.0.0.1"));
    }

    #[test]
    fn select_active_adapter_ignores_down_or_gatewayless_adapters() {
        let selected = select_active_adapter(vec![
            AdapterSnapshot {
                interface_name: "Wi-Fi".to_string(),
                mac_address: "AA:BB:CC:DD:EE:01".to_string(),
                ipv4_address: "10.0.0.20".to_string(),
                prefix_length: 24,
                default_gateway: Some("10.0.0.1".to_string()),
                dns_servers: vec![],
                metric: 5,
                oper_status_up: false,
            },
            AdapterSnapshot {
                interface_name: "Loopback".to_string(),
                mac_address: String::new(),
                ipv4_address: "127.0.0.1".to_string(),
                prefix_length: 8,
                default_gateway: None,
                dns_servers: vec![],
                metric: 1,
                oper_status_up: true,
            },
            AdapterSnapshot {
                interface_name: "Ethernet".to_string(),
                mac_address: "AA:BB:CC:DD:EE:02".to_string(),
                ipv4_address: "192.168.1.30".to_string(),
                prefix_length: 24,
                default_gateway: Some("192.168.1.1".to_string()),
                dns_servers: vec![],
                metric: 20,
                oper_status_up: true,
            },
        ])
        .unwrap();

        assert_eq!(selected.interface_name, "Ethernet");
        assert_eq!(selected.ip_address, "192.168.1.30");
    }

    #[test]
    fn select_active_adapter_returns_none_without_usable_gateway() {
        let selected = select_active_adapter(vec![AdapterSnapshot {
            interface_name: "Ethernet".to_string(),
            mac_address: "AA:BB:CC:DD:EE:02".to_string(),
            ipv4_address: "192.168.1.30".to_string(),
            prefix_length: 24,
            default_gateway: None,
            dns_servers: vec![],
            metric: 20,
            oper_status_up: true,
        }]);

        assert!(selected.is_none());
    }
}
