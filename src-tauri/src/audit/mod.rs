mod file_logger;

pub use file_logger::FileAuditWriter;

use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ports::AuditWriter;

/// Resolves `%APPDATA%` for the current user — the audit log lives at
/// `%APPDATA%\BG-SupTec\audit-YYYY-MM.log`, the standard per-user data
/// location on Windows.
fn appdata_dir() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Fire-and-forget audit recording for Tauri commands: wires a real
/// `FileAuditWriter` against `%APPDATA%` and writes the line, but never
/// fails the calling command if the audit log itself couldn't be written
/// (logged to stderr instead, mirroring `events::emit_log`'s fire-and-forget
/// semantics) — a logging-channel hiccup shouldn't block the destructive
/// action it's auditing. There are no individual user accounts in
/// BG-SupTec (single shared technician password), so `user` is always
/// `"admin"`.
pub fn record(action: &str, params: &str, result: &str) {
    let writer = FileAuditWriter::new(appdata_dir());
    if let Err(e) = log_action("admin", action, params, result, now_unix(), &writer) {
        eprintln!("AVISO: falha ao gravar audit log ({action}): {e}");
    }
}

/// Renders a command's `Result` as the `result` field for `record`:
/// `"ok"` for success, `"erro: {e}"` otherwise — used at every destructive
/// command's call site so the audit line reflects what actually happened.
pub fn outcome<T, E: std::fmt::Display>(result: &Result<T, E>) -> String {
    match result {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("erro: {e}"),
    }
}

/// Records one auditable action: who did it, what command ran, with which
/// parameters, and what the outcome was. Timestamp is injected as Unix
/// seconds (mirrors the `now_unix` pattern in `domain::system::time`) so the
/// function stays pure and testable without touching the real clock; the
/// actual persistence is delegated to `writer` (`FileAuditWriter` in
/// production, an in-memory fake in tests).
pub fn log_action(
    user: &str,
    action: &str,
    params: &str,
    result: &str,
    now_unix: i64,
    writer: &impl AuditWriter,
) -> Result<(), String> {
    writer.append_line(&format_line(user, action, params, result, now_unix))
}

fn format_line(user: &str, action: &str, params: &str, result: &str, now_unix: i64) -> String {
    let timestamp = DateTime::<Utc>::from_timestamp(now_unix, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| now_unix.to_string());

    format!("{timestamp} user={user} action={action} params={params} result={result}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeAuditWriter {
        lines: Mutex<Vec<String>>,
    }

    impl FakeAuditWriter {
        fn new() -> Self {
            Self {
                lines: Mutex::new(Vec::new()),
            }
        }
    }

    impl AuditWriter for FakeAuditWriter {
        fn append_line(&self, line: &str) -> Result<(), String> {
            self.lines.lock().unwrap().push(line.to_string());
            Ok(())
        }
    }

    fn some_timestamp() -> i64 {
        DateTime::parse_from_rfc3339("2026-06-30T18:40:00Z")
            .unwrap()
            .timestamp()
    }

    #[test]
    fn log_action_writes_a_single_formatted_line_via_the_writer() {
        let writer = FakeAuditWriter::new();

        log_action(
            "admin",
            "alterar_ip",
            "interface=Ethernet,ip=192.168.1.50",
            "ok",
            some_timestamp(),
            &writer,
        )
        .expect("should log");

        let lines = writer.lines.lock().unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("2026-06-30T18:40:00+00:00 "));
        assert!(lines[0].contains("user=admin"));
        assert!(lines[0].contains("action=alterar_ip"));
        assert!(lines[0].contains("params=interface=Ethernet,ip=192.168.1.50"));
        assert!(lines[0].contains("result=ok"));
    }

    #[test]
    fn log_action_writes_to_a_real_file_via_file_audit_writer() {
        let appdata = std::env::temp_dir().join(format!(
            "bg-suptec-test-audit-log-action-{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&appdata).ok();
        let writer = FileAuditWriter::new(appdata.clone());

        log_action(
            "admin",
            "alterar_ip",
            "interface=Ethernet,ip=192.168.1.50",
            "ok",
            some_timestamp(),
            &writer,
        )
        .expect("should log to file");

        let path = file_logger::audit_log_path(&appdata, Utc::now());
        let contents = std::fs::read_to_string(&path).expect("audit log file should exist");

        std::fs::remove_dir_all(&appdata).ok();

        assert!(contents.contains("user=admin action=alterar_ip"));
        assert!(contents.ends_with('\n'));
    }

    #[test]
    fn log_action_propagates_writer_errors() {
        struct FailingWriter;
        impl AuditWriter for FailingWriter {
            fn append_line(&self, _line: &str) -> Result<(), String> {
                Err("disco cheio".to_string())
            }
        }

        let result = log_action("admin", "x", "", "ok", some_timestamp(), &FailingWriter);

        assert_eq!(result, Err("disco cheio".to_string()));
    }
}
