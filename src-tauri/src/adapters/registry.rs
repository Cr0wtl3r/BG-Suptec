use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_LOCAL_MACHINE};
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

    /// Lists subkey names under `HKEY_LOCAL_MACHINE\{path}`. Each subkey
    /// handle opened by callers iterating this list (e.g.
    /// `domain::security::firewall::list_installed_programs`) must be a
    /// local variable scoped to a single loop iteration — `winreg`'s
    /// `RegKey` closes its handle in `Drop`, so a normally-scoped loop
    /// closes each handle before the next iteration opens a new one. This
    /// mirrors (and fixes) the legacy Go handle leak in
    /// `ObterProgramasInstalados`, where `defer subKey.Close()` inside the
    /// loop only ran at function return, not per-iteration, holding
    /// hundreds of handles open for the whole scan.
    fn list_local_machine_subkeys(&self, path: &str) -> Vec<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let Ok(key) = hklm.open_subkey(path) else {
            return Vec::new();
        };
        key.enum_keys().filter_map(Result::ok).collect()
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

    fn write_classes_root_string(&self, path: &str, name: &str, value: &str) -> Result<(), String> {
        let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
        let (key, _) = hkcr
            .create_subkey(path)
            .map_err(|e| format!("falha ao abrir/criar a chave {path}: {e}"))?;
        key.set_value(name, &value)
            .map_err(|e| format!("falha ao escrever o valor {name} em {path}: {e}"))
    }

    fn write_local_machine_string(&self, path: &str, name: &str, value: &str) -> Result<(), String> {
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
