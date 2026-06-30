use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::domain::security::{enable_system_protection, fix_network_sharing};
use crate::events::{
    self, ATIVAR_PROTECAO_FINALIZADO, COMPARTILHAMENTO_FINALIZADO, LOG_ATIVAR_PROTECAO,
    LOG_COMPARTILHAMENTO,
};

/// Applies the full network sharing fix (services, firewall, registry,
/// group policy refresh), streaming each progress line to the frontend via
/// `LOG_COMPARTILHAMENTO` and signaling completion via
/// `COMPARTILHAMENTO_FINALIZADO` — mirrors legacy
/// `CorrigirCompartilhamentoWindows` from `app.go`. The frontend is
/// responsible for warning the user about the SMB security tradeoffs
/// (`RequireSecuritySignature`, `limitblankpassworduse`) before invoking
/// this command.
#[tauri::command]
pub async fn corrigir_compartilhamento(window: tauri::Window) -> Result<(), String> {
    let runner = WinProcessRunner;

    fix_network_sharing(&runner, &WinRegistryReader, |msg| {
        events::emit_log(&window, LOG_COMPARTILHAMENTO, msg)
    })
    .await;

    events::emit_finalizado(&window, COMPARTILHAMENTO_FINALIZADO, true);

    Ok(())
}

/// Enables System Restore on drive `C:` and caps its shadow-storage usage
/// at 5%, streaming each progress line to the frontend via
/// `LOG_ATIVAR_PROTECAO` and signaling completion via
/// `ATIVAR_PROTECAO_FINALIZADO`. This is a first implementation, not a
/// legacy port: the legacy Svelte UI called an `AtivarProtecaoSistema()`
/// Wails function that was never defined in `app.go`, so clicking the
/// button there never did anything functional. Like the underlying domain
/// flow, this never fails as a whole (individual step errors are logged,
/// not propagated), so it always returns `Ok`.
#[tauri::command]
pub async fn ativar_protecao_sistema(window: tauri::Window) -> Result<(), String> {
    let runner = WinProcessRunner;

    enable_system_protection(&runner, |msg| {
        events::emit_log(&window, LOG_ATIVAR_PROTECAO, msg)
    })
    .await;

    events::emit_finalizado(&window, ATIVAR_PROTECAO_FINALIZADO, true);

    Ok(())
}
