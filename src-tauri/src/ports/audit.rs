/// Appends a single pre-formatted line to the audit trail. Implementations
/// own where/how that line is persisted (file, rotated file, etc.) — the
/// caller in `audit::log_action` only knows it can hand off a line.
pub trait AuditWriter {
    fn append_line(&self, line: &str) -> Result<(), String>;
}
