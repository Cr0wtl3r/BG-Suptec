use std::path::{Path, PathBuf};

use crate::adapters::process::WinProcessRunner;
use crate::audit;
use crate::domain::office_c2r::{
    add_office_product, change_office_channel, office_update_channels, remove_office_product,
    OfficeC2rInstall, OfficeUpdateChannel,
};
use crate::events::{self, LOG_OFFICE_C2R};

#[tauri::command]
pub fn obter_canais_office() -> Vec<OfficeUpdateChannel> {
    office_update_channels()
}

#[tauri::command]
pub fn detectar_office_c2r() -> Result<OfficeC2rInstall, String> {
    detect_install().ok_or_else(|| "Instalação Click-to-Run do Office não encontrada".to_string())
}

#[tauri::command]
pub async fn alterar_canal_office(
    window: tauri::Window,
    canal_id: String,
) -> Result<String, String> {
    let install = detectar_office_c2r()?;
    let resultado = change_office_channel(&WinProcessRunner, &install, &canal_id, |msg| {
        events::emit_log(&window, LOG_OFFICE_C2R, msg)
    })
    .await;
    audit::record(
        "alterar_canal_office",
        &format!("canal_id={canal_id}"),
        &audit::outcome(&resultado),
    );
    resultado
}

#[tauri::command]
pub async fn adicionar_produto_office(
    window: tauri::Window,
    produto_id: String,
    apps_excluidos: Vec<String>,
) -> Result<String, String> {
    let install = detectar_office_c2r()?;
    let resultado = add_office_product(
        &WinProcessRunner,
        &install,
        &produto_id,
        &apps_excluidos,
        |msg| events::emit_log(&window, LOG_OFFICE_C2R, msg),
    )
    .await;
    audit::record(
        "adicionar_produto_office",
        &format!(
            "produto_id={produto_id},apps_excluidos={}",
            apps_excluidos.join(",")
        ),
        &audit::outcome(&resultado),
    );
    resultado
}

#[tauri::command]
pub async fn remover_produto_office(
    window: tauri::Window,
    produto_id: String,
) -> Result<String, String> {
    let install = detectar_office_c2r()?;
    let resultado = remove_office_product(&WinProcessRunner, &install, &produto_id, |msg| {
        events::emit_log(&window, LOG_OFFICE_C2R, msg)
    })
    .await;
    audit::record(
        "remover_produto_office",
        &format!("produto_id={produto_id}"),
        &audit::outcome(&resultado),
    );
    resultado
}

fn detect_install() -> Option<OfficeC2rInstall> {
    candidate_roots()
        .into_iter()
        .find_map(|root| detect_install_at(&root))
}

fn candidate_roots() -> Vec<PathBuf> {
    ["ProgramFiles", "ProgramFiles(x86)"]
        .into_iter()
        .filter_map(|name| std::env::var_os(name).map(PathBuf::from))
        .map(|dir| dir.join(r"Common Files\Microsoft Shared\ClickToRun"))
        .collect()
}

fn detect_install_at(root: &Path) -> Option<OfficeC2rInstall> {
    let client_exe = root.join("OfficeC2RClient.exe");
    let click_to_run_exe = root.join("OfficeClickToRun.exe");
    if !client_exe.exists() || !click_to_run_exe.exists() {
        return None;
    }

    let root_text = root.to_string_lossy().to_string();
    Some(OfficeC2rInstall {
        client_exe: client_exe.to_string_lossy().to_string(),
        click_to_run_exe: click_to_run_exe.to_string_lossy().to_string(),
        install_root: root_text.clone(),
        platform: if root_text
            .to_ascii_lowercase()
            .contains("program files (x86)")
        {
            "x86".to_string()
        } else {
            "x64".to_string()
        },
        culture: std::env::var("BG_SUPTEC_OFFICE_CULTURE").unwrap_or_else(|_| "pt-br".to_string()),
        version: "desconhecida".to_string(),
        audience_id: "desconhecido".to_string(),
    })
}
