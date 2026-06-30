use std::path::{Path, PathBuf};

use crate::adapters::cscript::WinCscriptRunner;
use crate::adapters::process::WinProcessRunner;
use crate::adapters::tcp_health::TokioTcpHealthChecker;
use crate::config::load_kms_config;
use crate::domain::activation::office::{self, find_office_path};
use crate::domain::activation::windows::activate;
use crate::events::{
    self, ATIVACAO_OFFICE_FINALIZADO, ATIVACAO_WINDOWS_FINALIZADO, LOG_ATIVACAO_OFFICE,
    LOG_ATIVACAO_WINDOWS,
};

/// Resolves `kms.json`, expected alongside the running executable so GVLK
/// keys/KMS server can be edited without recompiling (mirrors
/// `auth_hash_path` in `lib.rs` for `auth.hash`).
fn kms_config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kms.json")
}

/// Activates Windows for `versao` using the GVLK key/KMS server from
/// `kms.json`, streaming each progress line to the frontend via
/// `LOG_ATIVACAO_WINDOWS` and signaling completion via
/// `ATIVACAO_WINDOWS_FINALIZADO` — mirrors legacy `AtivarWindows` from
/// `app.go`.
#[tauri::command]
pub async fn ativar_windows(window: tauri::Window, versao: String) -> Result<bool, String> {
    let config = load_kms_config(&kms_config_path())?;
    let runner = WinCscriptRunner;

    let sucesso = activate(
        &versao,
        &config.windows.keys,
        &config.windows.kms_server,
        &runner,
        |msg| events::emit_log(&window, LOG_ATIVACAO_WINDOWS, msg),
    )
    .await;

    events::emit_finalizado(&window, ATIVACAO_WINDOWS_FINALIZADO, sucesso);

    Ok(sucesso)
}

/// Activates Office for `versao` (`2016`/`2021`/`2024`) using the GVLK
/// key/unpkeys/license patterns/KMS servers from `kms.json`'s `office`
/// section, streaming progress via `LOG_ATIVACAO_OFFICE` and signaling
/// completion via `ATIVACAO_OFFICE_FINALIZADO` — mirrors legacy
/// `AtivarOffice` from `app.go`.
#[tauri::command]
pub async fn ativar_office(window: tauri::Window, versao: String) -> Result<bool, String> {
    let config = load_kms_config(&kms_config_path())?;

    let office_path = match find_office_path(
        std::env::var("ProgramFiles").ok().as_deref(),
        std::env::var("ProgramFiles(x86)").ok().as_deref(),
        |p| Path::new(p).exists(),
    ) {
        Ok(path) => path,
        Err(e) => {
            events::emit_log(&window, LOG_ATIVACAO_OFFICE, &format!("ERRO: {e}"));
            events::emit_finalizado(&window, ATIVACAO_OFFICE_FINALIZADO, false);
            return Ok(false);
        }
    };

    let runner = WinProcessRunner;
    let health_checker = TokioTcpHealthChecker;

    let sucesso = office::activate(
        &versao,
        &office_path,
        &config.office.versions,
        &runner,
        &health_checker,
        |p| Path::new(p).exists(),
        |dir| {
            std::fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .filter_map(|entry| entry.ok())
                        .map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .collect()
                })
                .unwrap_or_default()
        },
        |msg| events::emit_log(&window, LOG_ATIVACAO_OFFICE, msg),
    )
    .await;

    events::emit_finalizado(&window, ATIVACAO_OFFICE_FINALIZADO, sucesso);

    Ok(sucesso)
}
