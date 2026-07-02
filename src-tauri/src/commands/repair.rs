use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::audit;
use crate::domain::repair::{fix_start_menu_search, run_dism_restore_health, run_sfc_scannow};
use crate::events::{self, LOG_REPARO};

#[tauri::command]
pub async fn corrigir_busca_menu_iniciar(window: tauri::Window) -> Result<(), String> {
    fix_start_menu_search(&WinProcessRunner, &WinRegistryReader, |msg| {
        events::emit_log(&window, LOG_REPARO, msg)
    })
    .await;
    audit::record("corrigir_busca_menu_iniciar", "", "ok");
    Ok(())
}

#[tauri::command]
pub async fn executar_dism_restore_health(window: tauri::Window) -> Result<String, String> {
    let resultado = run_dism_restore_health(&WinProcessRunner, |msg| {
        events::emit_log(&window, LOG_REPARO, msg)
    })
    .await;
    audit::record(
        "executar_dism_restore_health",
        "",
        &audit::outcome(&resultado),
    );
    resultado
}

#[tauri::command]
pub async fn executar_sfc_scannow(window: tauri::Window) -> Result<String, String> {
    let resultado = run_sfc_scannow(&WinProcessRunner, |msg| {
        events::emit_log(&window, LOG_REPARO, msg)
    })
    .await;
    audit::record("executar_sfc_scannow", "", &audit::outcome(&resultado));
    resultado
}
