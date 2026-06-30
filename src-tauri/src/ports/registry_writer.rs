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
    fn write_local_machine_string(&self, path: &str, name: &str, value: &str) -> Result<(), String>;
}
