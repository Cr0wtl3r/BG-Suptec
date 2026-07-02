use crate::adapters::process::WinProcessRunner;
use crate::audit;
use crate::domain::disk::{
    change_windows_edition, convert_mbr_to_gpt, get_windows_editions, WindowsEditionInfo,
};
use crate::events::{self, LOG_DISCO};

#[tauri::command]
pub async fn converter_mbr_para_gpt(window: tauri::Window) -> Result<(), String> {
    let resultado = convert_mbr_to_gpt(&WinProcessRunner, |msg| {
        events::emit_log(&window, LOG_DISCO, msg)
    })
    .await;
    audit::record("converter_mbr_para_gpt", "", &audit::outcome(&resultado));
    resultado
}

#[tauri::command]
pub async fn obter_edicoes_windows() -> Result<WindowsEditionInfo, String> {
    get_windows_editions(&WinProcessRunner).await
}

#[tauri::command]
pub async fn mudar_edicao_windows(
    window: tauri::Window,
    edicao: String,
    chave: String,
) -> Result<String, String> {
    let resultado = change_windows_edition(&WinProcessRunner, &edicao, &chave, |msg| {
        events::emit_log(&window, LOG_DISCO, msg)
    })
    .await;
    audit::record(
        "mudar_edicao_windows",
        &format!("edicao={edicao},chave_fornecida={}", !chave.is_empty()),
        &audit::outcome(&resultado),
    );
    resultado
}
