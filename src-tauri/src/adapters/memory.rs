use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

use crate::ports::MemoryReader;

/// Reads total physical RAM via the Win32 `GlobalMemoryStatusEx` API —
/// a direct syscall, not a registry value or parsed command output.
pub struct WinMemoryReader;

impl MemoryReader for WinMemoryReader {
    fn total_physical_bytes(&self) -> u64 {
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailVirtual: 0,
            ullAvailExtendedVirtual: 0,
        };

        // SAFETY: `status` is a correctly sized and zero-initialized
        // MEMORYSTATUSEX with `dwLength` set, as required by the API.
        let ok = unsafe { GlobalMemoryStatusEx(&mut status) };

        if ok == 0 {
            0
        } else {
            status.ullTotalPhys
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_plausible_total_physical_memory() {
        let reader = WinMemoryReader;
        let total = reader.total_physical_bytes();

        // Any real machine running this test has at least 1 GiB of RAM.
        assert!(total > 1024 * 1024 * 1024);
    }
}
