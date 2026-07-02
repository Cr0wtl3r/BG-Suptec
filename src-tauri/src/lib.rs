mod adapters;
mod audit;
pub mod auth;
mod commands;
mod config;
mod domain;
mod events;
mod ports;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Resolves the path of `auth.hash`, expected alongside the running
/// executable so the password can be rotated without recompiling.
fn auth_hash_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("auth.hash")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let auth_hash_path = auth_hash_path();
    let password_hash = auth::load_hash_from_file(&auth_hash_path).unwrap_or_else(|e| {
        eprintln!(
            "AVISO: não foi possível carregar {} ({}). O login falhará até que o arquivo seja criado (use generate_hash).",
            auth_hash_path.display(),
            e
        );
        String::new()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::auth::AuthState::new(password_hash))
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::auth::login,
            commands::system_info::obter_informacoes_sistema,
            commands::system_info::ajustar_hora_formatacao,
            commands::network::reiniciar_computador,
            commands::network::alterar_nome_computador,
            commands::network::alterar_ip,
            commands::network::alterar_dns,
            commands::activation::ativar_windows,
            commands::activation::ativar_office,
            commands::maintenance::limpar_cache_dns,
            commands::maintenance::limpar_spool_impressao,
            commands::maintenance::desativar_hibernacao,
            commands::security::corrigir_compartilhamento,
            commands::security::ativar_protecao_sistema,
            commands::security::bloquear_programas_firewall,
            commands::security::desbloquear_programas_firewall,
            commands::security::verificar_status_firewall,
            commands::security::obter_programas_instalados,
            commands::security::listar_executaveis,
            commands::security::selecionar_arquivo_exe,
            commands::personalization::restaurar_photo_viewer,
            commands::keyboard::obter_layouts_teclado,
            commands::keyboard::obter_layout_ativo,
            commands::keyboard::alterar_layout_teclado,
            commands::gpedit::ativar_gpedit,
            commands::power::agendar_desligamento,
            commands::power::cancelar_desligamento,
            commands::policies::configurar_windows_update,
            commands::policies::configurar_defender,
            commands::policies::desativar_smartscreen,
            commands::policies::configurar_onedrive,
            commands::policies::aplicar_politicas_windows11,
            commands::policies::restaurar_menu_classico_windows11,
            commands::policies::aplicar_perfil_energia,
            commands::repair::corrigir_busca_menu_iniciar,
            commands::repair::executar_dism_restore_health,
            commands::repair::executar_sfc_scannow,
            commands::disk::converter_mbr_para_gpt,
            commands::disk::obter_edicoes_windows,
            commands::disk::mudar_edicao_windows,
            commands::cleanup::limpar_temporarios,
            commands::cleanup::limpeza_completa,
            commands::anydesk::resetar_anydesk,
            commands::office_c2r::obter_canais_office,
            commands::office_c2r::detectar_office_c2r,
            commands::office_c2r::alterar_canal_office,
            commands::office_c2r::adicionar_produto_office,
            commands::office_c2r::remover_produto_office,
            commands::formatting::executar_conclusao_formatacao,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
