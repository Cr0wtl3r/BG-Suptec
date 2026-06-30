use std::collections::HashMap;

use tauri_plugin_dialog::DialogExt;

use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::domain::security::firewall::{
    block_program_in_firewall, check_firewall_status, list_executables, list_installed_programs,
    unblock_program_in_firewall, ProgramaInfo,
};
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

/// Blocks inbound/outbound network access for each given executable path
/// via Windows Firewall rules — loops the domain's single-path
/// `block_program_in_firewall` over the array from the frontend, mirroring
/// legacy `BloquearProgramasFirewall`'s batch signature at the command
/// boundary. Fails fast on the first path that errors.
#[tauri::command]
pub async fn bloquear_programas_firewall(caminhos: Vec<String>) -> Result<(), String> {
    let runner = WinProcessRunner;
    for caminho in caminhos {
        block_program_in_firewall(&runner, &caminho).await?;
    }
    Ok(())
}

/// Removes the firewall block rule for each given executable path —
/// mirrors legacy `DesbloquearProgramasFirewall`'s batch signature at the
/// command boundary. Fails fast on the first path that errors.
#[tauri::command]
pub async fn desbloquear_programas_firewall(caminhos: Vec<String>) -> Result<(), String> {
    let runner = WinProcessRunner;
    for caminho in caminhos {
        unblock_program_in_firewall(&runner, &caminho).await?;
    }
    Ok(())
}

/// Checks whether each given executable path currently has a firewall
/// block rule — mirrors legacy `VerificarStatusFirewall`.
#[tauri::command]
pub async fn verificar_status_firewall(
    caminhos: Vec<String>,
) -> Result<HashMap<String, bool>, String> {
    let runner = WinProcessRunner;
    Ok(check_firewall_status(&runner, &caminhos).await)
}

/// Lists installed programs (name + install location) from the registry's
/// `Uninstall` keys — mirrors legacy `ObterProgramasInstalados`.
#[tauri::command]
pub fn obter_programas_instalados() -> Result<Vec<ProgramaInfo>, String> {
    Ok(list_installed_programs(&WinRegistryReader))
}

/// Recursively lists `.exe` files under `caminho` — mirrors legacy
/// `ListarExecutaveis`.
#[tauri::command]
pub fn listar_executaveis(caminho: String) -> Result<Vec<String>, String> {
    Ok(list_executables(&caminho))
}

/// Opens a native file-picker dialog filtered to `*.exe`, titled
/// "Selecionar Executável" — mirrors legacy `SelecionarArquivoExe`'s
/// dialog title/filter exactly. Returns `None` if the user cancels, rather
/// than legacy's error-on-cancel `"user cancelled"` string the frontend had
/// to match against.
#[tauri::command]
pub async fn selecionar_arquivo_exe(window: tauri::Window) -> Result<Option<String>, String> {
    let file_path = window
        .dialog()
        .file()
        .set_title("Selecionar Executável")
        .add_filter("Executáveis (*.exe)", &["exe"])
        .blocking_pick_file();

    Ok(file_path.map(|p| p.to_string()))
}
