use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::domain::activation::office::OfficeVersionConfig;

/// GVLK keys and KMS server for activating Windows, keyed by edition
/// (`pro`, `education`, `enterprise`, `home`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WindowsKmsConfig {
    pub kms_server: String,
    pub keys: HashMap<String, String>,
}

/// Per-edition Office activation config (`2016`/`2021`/`2024`), keyed by
/// edition string.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OfficeKmsConfig {
    pub versions: HashMap<String, OfficeVersionConfig>,
}

/// Root of `kms.json` — externalized so activation keys/servers can be
/// edited (e.g. swapped for a corporate KMS host) without recompiling.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KmsConfig {
    pub windows: WindowsKmsConfig,
    pub office: OfficeKmsConfig,
}

/// Loads and parses `kms.json` from `path` at runtime.
pub fn load_kms_config(path: &Path) -> Result<KmsConfig, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("falha ao ler {}: {e}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|e| format!("falha ao interpretar {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_kms_json(name: &str, contents: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "bg-suptec-test-kms-{}-{}.json",
            std::process::id(),
            name
        ));
        std::fs::write(&path, contents).expect("should write temp kms.json");
        path
    }

    #[test]
    fn load_kms_config_reads_windows_server_and_keys() {
        let path = write_temp_kms_json(
            "basic",
            r#"{
                "windows": {
                    "kms_server": "kms.msguides.com",
                    "keys": { "pro": "W269N-WFGWX-YVC9B-4J6C9-T83GX" }
                },
                "office": { "versions": {} }
            }"#,
        );

        let config = load_kms_config(&path).expect("should load valid kms.json");

        std::fs::remove_file(&path).ok();

        assert_eq!(config.windows.kms_server, "kms.msguides.com");
        assert_eq!(
            config.windows.keys.get("pro"),
            Some(&"W269N-WFGWX-YVC9B-4J6C9-T83GX".to_string())
        );
    }

    #[test]
    fn load_kms_config_reflects_edits_without_recompiling() {
        let path = write_temp_kms_json(
            "edited",
            r#"{
                "windows": {
                    "kms_server": "kms.servidor-customizado.local",
                    "keys": { "pro": "CHAVE-EDITADA-MANUALMENTE" }
                },
                "office": { "versions": {} }
            }"#,
        );

        let config = load_kms_config(&path).expect("should load edited kms.json");

        std::fs::remove_file(&path).ok();

        assert_eq!(config.windows.kms_server, "kms.servidor-customizado.local");
        assert_eq!(
            config.windows.keys.get("pro"),
            Some(&"CHAVE-EDITADA-MANUALMENTE".to_string())
        );
    }

    #[test]
    fn load_kms_config_reads_office_version_configs() {
        let path = write_temp_kms_json(
            "office-versions",
            r#"{
                "windows": {
                    "kms_server": "kms.msguides.com",
                    "keys": { "pro": "W269N-WFGWX-YVC9B-4J6C9-T83GX" }
                },
                "office": {
                    "versions": {
                        "2016": {
                            "prod_key": "XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99",
                            "unpkeys": ["BTDRB", "KHGM9", "CPQVG"],
                            "license_patterns": ["proplusvl_kms.*\\.xrm-ms"],
                            "kms_servers": ["23.226.136.46", "kms9.msguides.com"]
                        }
                    }
                }
            }"#,
        );

        let config = load_kms_config(&path).expect("should load kms.json with office section");

        std::fs::remove_file(&path).ok();

        let office_2016 = config
            .office
            .versions
            .get("2016")
            .expect("should have a 2016 office version config");
        assert_eq!(office_2016.prod_key, "XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99");
        assert_eq!(
            office_2016.unpkeys,
            vec!["BTDRB".to_string(), "KHGM9".to_string(), "CPQVG".to_string()]
        );
        assert_eq!(
            office_2016.kms_servers,
            vec!["23.226.136.46".to_string(), "kms9.msguides.com".to_string()]
        );
    }

    #[test]
    fn load_kms_config_errors_when_file_missing() {
        let path = std::env::temp_dir().join("bg-suptec-test-kms-missing.json");
        std::fs::remove_file(&path).ok();

        assert!(load_kms_config(&path).is_err());
    }

    #[test]
    fn load_kms_config_errors_on_malformed_json() {
        let path = write_temp_kms_json("malformed", "{ isso nao e json valido");

        let result = load_kms_config(&path);

        std::fs::remove_file(&path).ok();

        assert!(result.is_err());
    }
}
