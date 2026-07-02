use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::audit;
use crate::domain::system::policies::{
    apply_windows11_smb_policies, configure_defender_policy, configure_onedrive_integration,
    configure_windows_update, disable_smartscreen, restore_classic_context_menu, set_power_profile,
    DefenderPolicy, PowerProfile, WindowsUpdateMode,
};
use crate::events::{self, LOG_POLITICAS};

#[tauri::command]
pub async fn configurar_windows_update(
    window: tauri::Window,
    modo: WindowsUpdateMode,
) -> Result<(), String> {
    configure_windows_update(&WinProcessRunner, &WinRegistryReader, modo, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    })
    .await;
    audit::record("configurar_windows_update", &format!("modo={modo:?}"), "ok");
    Ok(())
}

#[tauri::command]
pub async fn configurar_defender(
    window: tauri::Window,
    politica: DefenderPolicy,
) -> Result<(), String> {
    configure_defender_policy(&WinProcessRunner, &WinRegistryReader, politica, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    })
    .await;
    audit::record(
        "configurar_defender",
        &format!("politica={politica:?}"),
        "ok",
    );
    Ok(())
}

#[tauri::command]
pub fn desativar_smartscreen(window: tauri::Window) -> Result<(), String> {
    disable_smartscreen(&WinRegistryReader, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    });
    audit::record("desativar_smartscreen", "", "ok");
    Ok(())
}

#[tauri::command]
pub fn configurar_onedrive(window: tauri::Window, ativar: bool) -> Result<(), String> {
    configure_onedrive_integration(&WinRegistryReader, ativar, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    });
    audit::record("configurar_onedrive", &format!("ativar={ativar}"), "ok");
    Ok(())
}

#[tauri::command]
pub fn aplicar_politicas_windows11(window: tauri::Window) -> Result<(), String> {
    apply_windows11_smb_policies(&WinRegistryReader, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    });
    audit::record(
        "aplicar_politicas_windows11",
        "smb_guest=true,signature=false",
        "ok",
    );
    Ok(())
}

#[tauri::command]
pub fn restaurar_menu_classico_windows11(window: tauri::Window) -> Result<(), String> {
    restore_classic_context_menu(&WinRegistryReader, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    });
    audit::record("restaurar_menu_classico_windows11", "", "ok");
    Ok(())
}

#[tauri::command]
pub async fn aplicar_perfil_energia(
    window: tauri::Window,
    perfil: PowerProfile,
) -> Result<String, String> {
    let resultado = set_power_profile(&WinProcessRunner, perfil, |msg| {
        events::emit_log(&window, LOG_POLITICAS, msg)
    })
    .await;
    audit::record(
        "aplicar_perfil_energia",
        &format!("perfil={perfil:?}"),
        &audit::outcome(&resultado),
    );
    resultado
}
