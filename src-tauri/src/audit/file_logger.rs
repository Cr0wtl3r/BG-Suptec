use chrono::{DateTime, Utc};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::ports::AuditWriter;

/// Builds the audit log path for a given moment: `{appdata_dir}/BG-SupTec/audit-YYYY-MM.log`.
/// Embedding the year-month in the filename *is* the rotation — once the
/// real clock crosses into a new month, the next write naturally lands in a
/// fresh file, with nothing to clean up or roll over.
pub fn audit_log_path(appdata_dir: &Path, now: DateTime<Utc>) -> PathBuf {
    appdata_dir
        .join("BG-SupTec")
        .join(format!("audit-{}.log", now.format("%Y-%m")))
}

/// Appends audit lines to `%APPDATA%\BG-SupTec\audit-YYYY-MM.log`, creating
/// the directory and file as needed.
pub struct FileAuditWriter {
    appdata_dir: PathBuf,
}

impl FileAuditWriter {
    pub fn new(appdata_dir: PathBuf) -> Self {
        Self { appdata_dir }
    }
}

impl AuditWriter for FileAuditWriter {
    fn append_line(&self, line: &str) -> Result<(), String> {
        let path = audit_log_path(&self.appdata_dir, Utc::now());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("erro ao criar diretório do audit log: {e}"))?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("erro ao abrir {}: {e}", path.display()))?;

        writeln!(file, "{line}").map_err(|e| format!("erro ao escrever no audit log: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bg-suptec-test-audit-{}-{}",
            std::process::id(),
            name
        ))
    }

    #[test]
    fn audit_log_path_embeds_year_and_month() {
        let appdata = Path::new(r"C:\Users\tecnico\AppData\Roaming");
        let now = DateTime::parse_from_rfc3339("2026-06-30T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let path = audit_log_path(appdata, now);

        assert_eq!(
            path,
            appdata.join("BG-SupTec").join("audit-2026-06.log")
        );
    }

    #[test]
    fn audit_log_path_changes_when_the_month_changes() {
        let appdata = Path::new(r"C:\Users\tecnico\AppData\Roaming");
        let june = DateTime::parse_from_rfc3339("2026-06-30T23:59:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let july = DateTime::parse_from_rfc3339("2026-07-01T00:01:00Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_ne!(audit_log_path(appdata, june), audit_log_path(appdata, july));
    }

    #[test]
    fn append_line_creates_directory_and_file_then_appends() {
        let appdata = unique_temp_dir("append");
        std::fs::remove_dir_all(&appdata).ok();
        let writer = FileAuditWriter::new(appdata.clone());

        writer.append_line("first line").expect("should append");
        writer.append_line("second line").expect("should append");

        let path = audit_log_path(&appdata, Utc::now());
        let contents = std::fs::read_to_string(&path).expect("log file should exist");

        std::fs::remove_dir_all(&appdata).ok();

        assert_eq!(contents, "first line\nsecond line\n");
    }
}
