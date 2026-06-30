use crate::adapters::process::WinProcessRunner;
use crate::domain::system::keyboard::{self, KeyboardLayout};

#[tauri::command]
pub fn obter_layouts_teclado() -> Vec<KeyboardLayout> {
    keyboard::get_available_layouts()
}

#[tauri::command]
pub async fn obter_layout_ativo() -> Result<String, String> {
    keyboard::get_active_layout(&WinProcessRunner).await
}

#[tauri::command]
pub async fn alterar_layout_teclado(tag_idioma: String) -> Result<String, String> {
    keyboard::change_keyboard_layout(&tag_idioma, &WinProcessRunner).await
}
