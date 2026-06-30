/// Abstraction over reading total physical memory, so domain logic can be
/// unit tested without querying the real OS.
pub trait MemoryReader {
    /// Returns total physical RAM in bytes.
    fn total_physical_bytes(&self) -> u64;
}
