/// Abstraction over invoking an arbitrary external program (no shell
/// intermediary), so domain orchestration logic that isn't a System32 WSH
/// script — e.g. `taskkill`, or `ospp.vbs` which lives under the Office
/// install directory rather than `%SystemRoot%\System32` like
/// `CscriptRunner`'s targets — can be unit tested with a fake recording
/// calls instead of touching real processes. `cwd` lets a caller match
/// flows that require a specific working directory (e.g. `ospp.vbs /act`
/// needs to run from the Office install directory); `None` means the
/// current process directory.
pub trait ProcessRunner {
    async fn run(&self, program: &str, args: &[&str], cwd: Option<&str>) -> Result<String, String>;
}
