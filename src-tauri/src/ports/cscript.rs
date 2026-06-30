/// Abstraction over invoking Windows Script Host scripts via `cscript`
/// (e.g. `slmgr.vbs`, `ospp.vbs`), so activation domain logic can be unit
/// tested with a fake recording calls instead of touching the real
/// licensing subsystem.
pub trait CscriptRunner {
    /// Runs `cscript <script_path> <args...>` and returns captured stdout.
    async fn run(&self, script_path: &str, args: &[&str]) -> Result<String, String>;
}
