use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use crate::ports::{RegistryReader, RegistryWriter};

/// Reads/writes values from the real Windows registry via the `winreg`
/// crate. Implements both `RegistryReader` and `RegistryWriter` — one
/// adapter, two narrowly-scoped port traits.
pub struct WinRegistryReader;

impl RegistryReader for WinRegistryReader {
    fn read_local_machine_string(&self, path: &str, name: &str) -> Option<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hklm.open_subkey(path).ok()?;
        key.get_value(name).ok()
    }
}

impl RegistryWriter for WinRegistryReader {
    fn write_local_machine_dword(&self, path: &str, name: &str, value: u32) -> Result<(), String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm
            .create_subkey(path)
            .map_err(|e| format!("falha ao abrir/criar a chave {path}: {e}"))?;
        key.set_value(name, &value)
            .map_err(|e| format!("falha ao escrever o valor {name} em {path}: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_windows_product_name_from_real_registry() {
        let reader = WinRegistryReader;
        let value = reader
            .read_local_machine_string(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", "ProductName")
            .expect("ProductName should exist on any Windows machine");

        assert!(value.to_lowercase().contains("windows"));
    }

    #[test]
    fn returns_none_for_missing_key() {
        let reader = WinRegistryReader;
        let value = reader.read_local_machine_string(r"SOFTWARE\BG-SupTec\DoesNotExist", "Nope");

        assert!(value.is_none());
    }

    #[test]
    fn returns_none_for_missing_value_in_existing_key() {
        let reader = WinRegistryReader;
        let value = reader.read_local_machine_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "ValorQueNaoExiste",
        );

        assert!(value.is_none());
    }
}
