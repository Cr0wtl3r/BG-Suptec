use serde::Serialize;

use crate::ports::{MemoryReader, NetworkReader, RegistryReader};

pub mod time;

const REG_PATH_WINDOWS_VERSION: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
const REG_PATH_PROCESSOR: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";
const NA: &str = "N/A";

/// Aggregated system information shown in the "Painel de Informações"
/// feature. Field names are serialized in Portuguese to match the
/// existing frontend contract ported from the legacy Svelte app.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SystemInfo {
    #[serde(rename = "nomeComputador")]
    pub nome_computador: String,
    #[serde(rename = "versaoWindows")]
    pub versao_windows: String,
    #[serde(rename = "edicaoWindows")]
    pub edicao_windows: String,
    #[serde(rename = "buildWindows")]
    pub build_windows: String,
    pub processador: String,
    #[serde(rename = "memoriaTotalGB")]
    pub memoria_total_gb: String,
    #[serde(rename = "enderecoMAC")]
    pub endereco_mac: String,
    #[serde(rename = "enderecoIP")]
    pub endereco_ip: String,
    #[serde(rename = "mascaraRede")]
    pub mascara_rede: String,
    #[serde(rename = "gatewayPadrao")]
    pub gateway_padrao: String,
    #[serde(rename = "dnsPrimario")]
    pub dns_primario: String,
    #[serde(rename = "dnsSecundario")]
    pub dns_secundario: String,
    #[serde(rename = "interfaceAtiva")]
    pub interface_ativa: String,
}

/// Collects system information from the injected ports: Windows
/// edition/version/build and CPU name from the registry, total RAM from
/// the OS, and the active network interface's configuration. Pure given
/// its inputs — real OS access lives entirely in the `adapters` that
/// implement these port traits.
pub async fn get_info(
    hostname: &str,
    registry: &impl RegistryReader,
    memory: &impl MemoryReader,
    network: &impl NetworkReader,
) -> SystemInfo {
    let edicao_windows = registry
        .read_local_machine_string(REG_PATH_WINDOWS_VERSION, "ProductName")
        .unwrap_or_default();
    let versao_windows = registry
        .read_local_machine_string(REG_PATH_WINDOWS_VERSION, "DisplayVersion")
        .unwrap_or_default();
    let build_windows = registry
        .read_local_machine_string(REG_PATH_WINDOWS_VERSION, "CurrentBuild")
        .unwrap_or_default();
    let processador = registry
        .read_local_machine_string(REG_PATH_PROCESSOR, "ProcessorNameString")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let total_memory_gb = memory.total_physical_bytes() as f64 / (1024.0 * 1024.0 * 1024.0);
    let memoria_total_gb = format!("{:.2} GB", total_memory_gb);

    let net_info = network.active_interface_info().await;

    SystemInfo {
        nome_computador: hostname.to_string(),
        versao_windows,
        edicao_windows,
        build_windows,
        processador,
        memoria_total_gb,
        endereco_mac: net_info
            .as_ref()
            .map(|n| n.mac_address.clone())
            .unwrap_or_else(|| NA.to_string()),
        endereco_ip: net_info
            .as_ref()
            .map(|n| n.ip_address.clone())
            .unwrap_or_else(|| NA.to_string()),
        mascara_rede: net_info
            .as_ref()
            .map(|n| n.subnet_mask.clone())
            .unwrap_or_else(|| NA.to_string()),
        gateway_padrao: net_info
            .as_ref()
            .map(|n| n.default_gateway.clone())
            .unwrap_or_else(|| NA.to_string()),
        dns_primario: net_info
            .as_ref()
            .and_then(|n| n.dns_primary.clone())
            .unwrap_or_else(|| NA.to_string()),
        dns_secundario: net_info
            .as_ref()
            .and_then(|n| n.dns_secondary.clone())
            .unwrap_or_else(|| NA.to_string()),
        interface_ativa: net_info
            .map(|n| n.interface_name)
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{get_info, SystemInfo};
    use crate::ports::{MemoryReader, NetworkInfo, NetworkReader, RegistryReader};

    struct FakeRegistry;
    impl RegistryReader for FakeRegistry {
        fn read_local_machine_string(&self, path: &str, name: &str) -> Option<String> {
            match (path, name) {
                (
                    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
                    "ProductName",
                ) => Some("Windows 11 Pro".to_string()),
                (
                    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
                    "DisplayVersion",
                ) => Some("23H2".to_string()),
                (
                    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
                    "CurrentBuild",
                ) => Some("22631".to_string()),
                (
                    r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
                    "ProcessorNameString",
                ) => Some("Intel(R) Core(TM) i7-12700K".to_string()),
                _ => None,
            }
        }
    }

    struct FakeMemory;
    impl MemoryReader for FakeMemory {
        fn total_physical_bytes(&self) -> u64 {
            17_179_869_184 // 16 GiB
        }
    }

    struct FakeNetwork;
    impl NetworkReader for FakeNetwork {
        async fn active_interface_info(&self) -> Option<NetworkInfo> {
            Some(NetworkInfo {
                interface_name: "Ethernet".to_string(),
                mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
                ip_address: "192.168.1.50".to_string(),
                subnet_mask: "255.255.255.0".to_string(),
                default_gateway: "192.168.1.1".to_string(),
                dns_primary: Some("8.8.8.8".to_string()),
                dns_secondary: Some("8.8.4.4".to_string()),
            })
        }
    }

    struct FakeNetworkUnavailable;
    impl NetworkReader for FakeNetworkUnavailable {
        async fn active_interface_info(&self) -> Option<NetworkInfo> {
            None
        }
    }

    #[tokio::test]
    async fn get_info_assembles_hostname_ram_windows_cpu_and_network() {
        let info: SystemInfo =
            get_info("DESKTOP-TEST", &FakeRegistry, &FakeMemory, &FakeNetwork).await;

        assert_eq!(info.nome_computador, "DESKTOP-TEST");
        assert_eq!(info.versao_windows, "23H2");
        assert_eq!(info.edicao_windows, "Windows 11 Pro");
        assert_eq!(info.build_windows, "22631");
        assert_eq!(info.processador, "Intel(R) Core(TM) i7-12700K");
        assert_eq!(info.memoria_total_gb, "16.00 GB");
        assert_eq!(info.endereco_mac, "AA:BB:CC:DD:EE:FF");
        assert_eq!(info.endereco_ip, "192.168.1.50");
        assert_eq!(info.mascara_rede, "255.255.255.0");
        assert_eq!(info.gateway_padrao, "192.168.1.1");
        assert_eq!(info.dns_primario, "8.8.8.8");
        assert_eq!(info.dns_secundario, "8.8.4.4");
        assert_eq!(info.interface_ativa, "Ethernet");
    }

    #[tokio::test]
    async fn get_info_falls_back_to_na_when_no_active_network_interface() {
        let info = get_info(
            "DESKTOP-TEST",
            &FakeRegistry,
            &FakeMemory,
            &FakeNetworkUnavailable,
        )
        .await;

        assert_eq!(info.endereco_mac, "N/A");
        assert_eq!(info.endereco_ip, "N/A");
        assert_eq!(info.mascara_rede, "N/A");
        assert_eq!(info.gateway_padrao, "N/A");
        assert_eq!(info.dns_primario, "N/A");
        assert_eq!(info.dns_secundario, "N/A");
        assert_eq!(info.interface_ativa, "");
    }

    #[test]
    fn system_info_serializes_with_legacy_frontend_field_names() {
        let info = SystemInfo {
            nome_computador: "PC".to_string(),
            versao_windows: "23H2".to_string(),
            edicao_windows: "Windows 11 Pro".to_string(),
            build_windows: "22631".to_string(),
            processador: "CPU".to_string(),
            memoria_total_gb: "16.00 GB".to_string(),
            endereco_mac: "AA:BB:CC:DD:EE:FF".to_string(),
            endereco_ip: "192.168.1.50".to_string(),
            mascara_rede: "255.255.255.0".to_string(),
            gateway_padrao: "192.168.1.1".to_string(),
            dns_primario: "8.8.8.8".to_string(),
            dns_secundario: "8.8.4.4".to_string(),
            interface_ativa: "Ethernet".to_string(),
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["nomeComputador"], "PC");
        assert_eq!(json["memoriaTotalGB"], "16.00 GB");
        assert_eq!(json["enderecoMAC"], "AA:BB:CC:DD:EE:FF");
        assert_eq!(json["interfaceAtiva"], "Ethernet");
    }
}
