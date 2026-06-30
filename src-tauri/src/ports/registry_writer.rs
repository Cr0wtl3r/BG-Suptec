/// Abstraction over writing values to the Windows registry, so domain
/// logic can be unit tested without touching the real registry. Kept
/// separate from `RegistryReader` (rather than added to it) so existing
/// read-only fakes don't need a no-op write method just to keep compiling.
pub trait RegistryWriter {
    /// Writes a `REG_DWORD` value to `HKEY_LOCAL_MACHINE\{path}\{name}`,
    /// creating the value if it doesn't already exist.
    fn write_local_machine_dword(&self, path: &str, name: &str, value: u32) -> Result<(), String>;
}
