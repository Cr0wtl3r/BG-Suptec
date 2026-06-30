use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::domain::security::fix_network_sharing;
use crate::events::{self, COMPARTILHAMENTO_FINALIZADO, LOG_COMPARTILHAMENTO};

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
