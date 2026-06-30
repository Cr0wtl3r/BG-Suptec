use crate::adapters::registry::WinRegistryReader;
use crate::domain::personalization::restore_photo_viewer;
use crate::events::{self, LOG_RESTAURAR_PHOTOVIEWER, RESTAURAR_PHOTOVIEWER_FINALIZADO};

/// Restores "Windows Photo Viewer" as an available "Open with" option for
/// common image file types, streaming each progress line to the frontend
/// via `LOG_RESTAURAR_PHOTOVIEWER` and signaling completion via
/// `RESTAURAR_PHOTOVIEWER_FINALIZADO`. Like the underlying domain flow,
/// individual registry-write failures are logged, not propagated, so this
/// always returns `Ok` — the explicit completion event (rather than a
/// promise that silently never resolves) is what keeps the frontend from
/// hanging, the correctness concern the plan calls out for this slice.
#[tauri::command]
pub fn restaurar_photo_viewer(window: tauri::Window) -> Result<(), String> {
    restore_photo_viewer(&WinRegistryReader, |msg| {
        events::emit_log(&window, LOG_RESTAURAR_PHOTOVIEWER, msg)
    });

    events::emit_finalizado(&window, RESTAURAR_PHOTOVIEWER_FINALIZADO, true);

    Ok(())
}
