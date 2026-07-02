use std::time::{SystemTime, UNIX_EPOCH};

use crate::adapters::memory::WinMemoryReader;
use crate::adapters::network::NativeNetworkReader;
use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::domain::system::{self, time::adjust_formatting_time, SystemInfo};
use crate::events::{self, LOG_AJUSTAR_HORA_FORMATACAO};

fn hostname() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "N/A".to_string())
}

#[tauri::command]
pub async fn obter_informacoes_sistema() -> Result<SystemInfo, String> {
    Ok(system::get_info(
        &hostname(),
        &WinRegistryReader,
        &WinMemoryReader,
        &NativeNetworkReader,
    )
    .await)
}

/// Configures Windows Time/NTP sync and stamps the registry `InstallDate`
/// with the current timestamp, streaming each progress line to the
/// frontend via `LOG_AJUSTAR_HORA_FORMATACAO` — mirrors legacy
/// `AjustarHoraFormatacao`. Like the underlying domain flow, this never
/// fails as a whole (individual step errors are logged, not propagated),
/// so it always returns `Ok`.
#[tauri::command]
pub async fn ajustar_hora_formatacao(window: tauri::Window) -> Result<(), String> {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0);

    adjust_formatting_time(&WinProcessRunner, &WinRegistryReader, now_unix, |msg| {
        events::emit_log(&window, LOG_AJUSTAR_HORA_FORMATACAO, msg)
    })
    .await;

    Ok(())
}
