use crate::adapters::process::WinProcessRunner;
use crate::domain::system::power;

#[tauri::command]
pub async fn agendar_desligamento(segundos: u32) -> Result<String, String> {
    power::schedule_shutdown(segundos, &WinProcessRunner).await
}

#[tauri::command]
pub async fn cancelar_desligamento() -> Result<String, String> {
    power::cancel_shutdown(&WinProcessRunner).await
}
