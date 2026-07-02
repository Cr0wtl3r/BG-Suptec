use std::fs;
use std::path::Path;

use crate::adapters::process::WinProcessRunner;
use crate::audit;
use crate::domain::anydesk::reset_anydesk;
use crate::events::{self, LOG_ANYDESK};

#[tauri::command]
pub async fn resetar_anydesk(window: tauri::Window) -> Result<(), String> {
    let all_users_profile = env_or("ALLUSERSPROFILE", r"C:\ProgramData");
    let app_data = env_or("APPDATA", r"C:\Users\Default\AppData\Roaming");
    let system_conf = format!(r"{all_users_profile}\AnyDesk\system.conf");
    let resultado = reset_anydesk(
        &WinProcessRunner,
        &all_users_profile,
        &app_data,
        delete_file_if_exists,
        || anydesk_id_ready(&system_conf),
        |msg| events::emit_log(&window, LOG_ANYDESK, msg),
    )
    .await;

    audit::record("resetar_anydesk", "", &audit::outcome(&resultado));
    resultado
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn delete_file_if_exists(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).map_err(|e| format!("falha ao excluir {}: {e}", path.display()))
}

fn anydesk_id_ready(path: &str) -> bool {
    fs::read_to_string(path)
        .map(|content| content.contains("ad.anynet.id="))
        .unwrap_or(false)
}
