use std::io;
use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
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

    fn write_local_machine_string(
        &self,
        path: &str,
        name: &str,
        value: &str,
    ) -> Result<(), String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm
            .create_subkey(path)
            .map_err(|e| format!("falha ao abrir/criar a chave {path}: {e}"))?;
        key.set_value(name, &value)
            .map_err(|e| format!("falha ao escrever o valor {name} em {path}: {e}"))
    }

    fn write_current_user_dword(&self, path: &str, name: &str, value: u32) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(path)
            .map_err(|e| format!("falha ao abrir/criar a chave HKCU\\{path}: {e}"))?;
        key.set_value(name, &value)
            .map_err(|e| format!("falha ao escrever o valor HKCU\\{path}\\{name}: {e}"))
    }

    fn write_current_user_string(&self, path: &str, name: &str, value: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(path)
            .map_err(|e| format!("falha ao abrir/criar a chave HKCU\\{path}: {e}"))?;
        key.set_value(name, &value)
            .map_err(|e| format!("falha ao escrever o valor HKCU\\{path}\\{name}: {e}"))
    }

    fn delete_local_machine_value(&self, path: &str, name: &str) -> Result<(), String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = match hklm.open_subkey_with_flags(path, winreg::enums::KEY_SET_VALUE) {
            Ok(key) => key,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("falha ao abrir HKLM\\{path}: {e}")),
        };
        match key.delete_value(name) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("falha ao remover HKLM\\{path}\\{name}: {e}")),
        }
    }

    fn delete_current_user_value(&self, path: &str, name: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = match hkcu.open_subkey_with_flags(path, winreg::enums::KEY_SET_VALUE) {
            Ok(key) => key,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("falha ao abrir HKCU\\{path}: {e}")),
        };
        match key.delete_value(name) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("falha ao remover HKCU\\{path}\\{name}: {e}")),
        }
    }

    fn delete_local_machine_tree(&self, path: &str) -> Result<(), String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        match hklm.delete_subkey_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("falha ao remover HKLM\\{path}: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_windows_product_name_from_real_registry() {
        let reader = WinRegistryReader;
        let value = reader
            .read_local_machine_string(
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
                "ProductName",
            )
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
