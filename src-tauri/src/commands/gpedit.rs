use crate::adapters::process::WinProcessRunner;
use crate::domain::system::gpedit::enable_gpedit;
use crate::events::{self, LOG_ATIVAR_GPEDIT};

fn packages_dir() -> String {
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    format!(r"{system_root}\servicing\Packages")
}

fn list_dir(dir: &str) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub async fn ativar_gpedit(window: tauri::Window) -> Result<usize, String> {
    enable_gpedit(&packages_dir(), &WinProcessRunner, list_dir, |msg| {
        events::emit_log(&window, LOG_ATIVAR_GPEDIT, msg)
    })
    .await
}
