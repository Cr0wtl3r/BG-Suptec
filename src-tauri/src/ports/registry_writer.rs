/// Abstraction over writing values to the Windows registry, so domain
/// logic can be unit tested without touching the real registry. Kept
/// separate from `RegistryReader` (rather than added to it) so existing
/// read-only fakes don't need a no-op write method just to keep compiling.
pub trait RegistryWriter {
    /// Writes a `REG_DWORD` value to `HKEY_LOCAL_MACHINE\{path}\{name}`,
    /// creating the value if it doesn't already exist.
    fn write_local_machine_dword(&self, path: &str, name: &str, value: u32) -> Result<(), String>;

    /// Writes a `REG_SZ` string value to `HKEY_CLASSES_ROOT\{path}\{name}`,
    /// creating the key if it doesn't already exist. `name = ""` means the
    /// key's default (unnamed) value — the Windows convention every
    /// `shell\open\command` write in `domain::personalization` relies on.
    fn write_classes_root_string(&self, path: &str, name: &str, value: &str) -> Result<(), String>;

    /// Writes a `REG_SZ` string value to `HKEY_LOCAL_MACHINE\{path}\{name}`,
    /// creating the key if it doesn't already exist. Used for
    /// `OpenWithProgids` entries, where `name` is the ProgID and `value` is
    /// an empty string per Windows convention for that key.
    fn write_local_machine_string(&self, path: &str, name: &str, value: &str)
        -> Result<(), String>;

    /// Writes a `REG_DWORD` value to `HKEY_CURRENT_USER\{path}\{name}`.
    fn write_current_user_dword(
        &self,
        _path: &str,
        _name: &str,
        _value: u32,
    ) -> Result<(), String> {
        Err("write_current_user_dword não implementado para este RegistryWriter".to_string())
    }

    /// Writes a `REG_SZ` string value to `HKEY_CURRENT_USER\{path}\{name}`.
    /// `name = ""` writes the default value.
    fn write_current_user_string(
        &self,
        _path: &str,
        _name: &str,
        _value: &str,
    ) -> Result<(), String> {
        Err("write_current_user_string não implementado para este RegistryWriter".to_string())
    }

    /// Deletes a named value from `HKEY_LOCAL_MACHINE\{path}`. Missing values
    /// should be treated as success by concrete implementations.
    fn delete_local_machine_value(&self, _path: &str, _name: &str) -> Result<(), String> {
        Err("delete_local_machine_value não implementado para este RegistryWriter".to_string())
    }

    /// Deletes a named value from `HKEY_CURRENT_USER\{path}`. Missing values
    /// should be treated as success by concrete implementations.
    fn delete_current_user_value(&self, _path: &str, _name: &str) -> Result<(), String> {
        Err("delete_current_user_value não implementado para este RegistryWriter".to_string())
    }

    /// Deletes an entire key tree below `HKEY_LOCAL_MACHINE\{path}`. Missing
    /// keys should be treated as success by concrete implementations.
    fn delete_local_machine_tree(&self, _path: &str) -> Result<(), String> {
        Err("delete_local_machine_tree não implementado para este RegistryWriter".to_string())
    }
}
