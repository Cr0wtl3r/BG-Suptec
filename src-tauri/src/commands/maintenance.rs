use crate::adapters::process::WinProcessRunner;
use crate::domain::maintenance::{clear_dns_cache, clear_print_spool, disable_hibernation};

/// Flushes the local DNS resolver cache. Mirrors legacy
/// `LimpaCacheDNS.svelte`'s `ExecutarComando("ipconfig", ["/flushdns"])`.
#[tauri::command]
pub async fn limpar_cache_dns() -> Result<String, String> {
    clear_dns_cache(&WinProcessRunner).await
}

/// Disables Windows hibernation and removes `hiberfil.sys`. Mirrors legacy
/// `DesativaHibernacao.svelte`.
#[tauri::command]
pub async fn desativar_hibernacao() -> Result<String, String> {
    disable_hibernation(&WinProcessRunner).await
}

/// Resolves the Print Spooler's job queue directory
/// (`%SystemRoot%\System32\spool\PRINTERS`), where stuck `.SHD`/`.SPL` job
/// files accumulate.
fn spool_dir() -> String {
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    format!(r"{system_root}\System32\spool\PRINTERS")
}

/// Stops the Print Spooler, deletes stuck `.SHD`/`.SPL` job files, and
/// restarts it. Returns how many files were removed. Mirrors/extends
/// legacy `LimpaSpoolImpressao.svelte`.
#[tauri::command]
pub async fn limpar_spool_impressao() -> Result<usize, String> {
    clear_print_spool(
        &spool_dir(),
        &WinProcessRunner,
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
        |path| std::fs::remove_file(path).map_err(|e| e.to_string()),
        |_msg| {},
    )
    .await
}
