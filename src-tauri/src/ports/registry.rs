/// Abstraction over reading values from the Windows registry, so domain
/// logic can be unit tested without touching the real registry.
pub trait RegistryReader {
    /// Reads a string value from `HKEY_LOCAL_MACHINE\{path}\{name}`.
    /// Returns `None` if the key, value, or registry itself is unavailable.
    fn read_local_machine_string(&self, path: &str, name: &str) -> Option<String>;

    /// Lists the subkey names directly under `HKEY_LOCAL_MACHINE\{path}`.
    /// Returns an empty `Vec` if the path doesn't exist or can't be
    /// enumerated.
    fn list_local_machine_subkeys(&self, path: &str) -> Vec<String>;
}
