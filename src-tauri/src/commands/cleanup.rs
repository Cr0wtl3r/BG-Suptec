use std::fs;
use std::path::Path;

use chrono::Local;

use crate::adapters::process::WinProcessRunner;
use crate::audit;
use crate::domain::maintenance::cleanup::{
    clean_full_pc, clean_temp_files, cleanup_digisat_mongo_logs,
};
use crate::events::{self, LOG_LIMPEZA};

#[tauri::command]
pub async fn limpar_temporarios(window: tauri::Window) -> Result<usize, String> {
    let user_profile = env_or("USERPROFILE", r"C:\Users\Default");
    let windows_dir = env_or("SystemRoot", r"C:\Windows");
    let resultado = clean_temp_files(
        &WinProcessRunner,
        &user_profile,
        &windows_dir,
        delete_cleanup_target,
        |msg| events::emit_log(&window, LOG_LIMPEZA, msg),
    )
    .await;

    audit::record(
        "limpar_temporarios",
        &format!("user_profile={user_profile},windows_dir={windows_dir}"),
        &audit::outcome(&resultado),
    );
    resultado
}

#[tauri::command]
pub async fn limpeza_completa(
    window: tauri::Window,
    excluir_sombras: bool,
) -> Result<usize, String> {
    let user_profile = env_or("USERPROFILE", r"C:\Users\Default");
    let windows_dir = env_or("SystemRoot", r"C:\Windows");
    let mongo_log_dir = r"C:\DigiSat\SuiteG6\MongoDB\log";
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mongo_removed = cleanup_digisat_mongo_logs(
        mongo_log_dir,
        &list_file_names(mongo_log_dir),
        &today,
        delete_file_if_exists,
        |msg| events::emit_log(&window, LOG_LIMPEZA, msg),
    )?;

    let resultado = clean_full_pc(
        &WinProcessRunner,
        &user_profile,
        &windows_dir,
        excluir_sombras,
        delete_cleanup_target,
        |msg| events::emit_log(&window, LOG_LIMPEZA, msg),
    )
    .await
    .map(|targets| targets + mongo_removed);

    audit::record(
        "limpeza_completa",
        &format!("excluir_sombras={excluir_sombras},mongo_logs_removidos={mongo_removed}"),
        &audit::outcome(&resultado),
    );
    resultado
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn list_file_names(dir: &str) -> Vec<String> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn delete_file_if_exists(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).map_err(|e| format!("falha ao excluir {}: {e}", path.display()))
}

fn delete_cleanup_target(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(());
    }

    if path
        .to_string_lossy()
        .to_ascii_lowercase()
        .ends_with(r"\mozilla\firefox\profiles")
    {
        return clean_firefox_profiles(path);
    }

    delete_contents(path)
}

fn clean_firefox_profiles(profiles_root: &Path) -> Result<(), String> {
    for profile in fs::read_dir(profiles_root).map_err(|e| {
        format!(
            "falha ao listar perfis Firefox em {}: {e}",
            profiles_root.display()
        )
    })? {
        let profile = profile.map_err(|e| e.to_string())?.path();
        if !profile.is_dir() {
            continue;
        }
        delete_contents(&profile.join(r"cache2\entries"))?;
        delete_contents(&profile.join("startupCache"))?;
        delete_matching_files(&profile.join("cache2"), |name| {
            name.starts_with("index") || name.ends_with(".log")
        })?;
    }
    Ok(())
}

fn delete_contents(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("falha ao ler {}: {e}", path.display())),
    };

    if metadata.is_file() || metadata.file_type().is_symlink() {
        return fs::remove_file(path)
            .map_err(|e| format!("falha ao excluir {}: {e}", path.display()));
    }

    for entry in
        fs::read_dir(path).map_err(|e| format!("falha ao listar {}: {e}", path.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?.path();
        let metadata = fs::symlink_metadata(&entry)
            .map_err(|e| format!("falha ao ler {}: {e}", entry.display()))?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            fs::remove_dir_all(&entry)
                .map_err(|e| format!("falha ao excluir {}: {e}", entry.display()))?;
        } else {
            fs::remove_file(&entry)
                .map_err(|e| format!("falha ao excluir {}: {e}", entry.display()))?;
        }
    }
    Ok(())
}

fn delete_matching_files(path: &Path, matches: impl Fn(&str) -> bool) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(path).map_err(|e| format!("falha ao listar {}: {e}", path.display()))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if matches(&file_name) {
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)
                    .map_err(|e| format!("falha ao excluir {}: {e}", path.display()))?;
            }
        }
    }
    Ok(())
}
