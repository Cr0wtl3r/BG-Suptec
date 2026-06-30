use crate::adapters::process::WinProcessRunner;
use crate::audit;
use crate::domain::system::power;

#[tauri::command]
pub async fn agendar_desligamento(segundos: u32) -> Result<String, String> {
    let resultado = power::schedule_shutdown(segundos, &WinProcessRunner).await;
    audit::record(
        "agendar_desligamento",
        &format!("segundos={segundos}"),
        &audit::outcome(&resultado),
    );
    resultado
}

#[tauri::command]
pub async fn cancelar_desligamento() -> Result<String, String> {
    let resultado = power::cancel_shutdown(&WinProcessRunner).await;
    audit::record("cancelar_desligamento", "", &audit::outcome(&resultado));
    resultado
}
