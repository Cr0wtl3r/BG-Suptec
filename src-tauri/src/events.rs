use tauri::Emitter;

/// Tauri event names used to stream activation logs to the frontend,
/// replacing the legacy Wails `EventsEmit`/`EventsOn` string-based
/// channel. Centralized as constants (rather than inlined string
/// literals scattered across commands) so a typo can't silently desync
/// a backend emit from a frontend listener.
pub const LOG_ATIVACAO_WINDOWS: &str = "log:ativacao:windows";
pub const ATIVACAO_WINDOWS_FINALIZADO: &str = "ativacao:windows:finalizado";
pub const LOG_ATIVACAO_OFFICE: &str = "log:ativacao:office";
pub const ATIVACAO_OFFICE_FINALIZADO: &str = "ativacao:office:finalizado";
pub const LOG_AJUSTAR_HORA_FORMATACAO: &str = "log:ajustar:hora:formatacao";

/// Emits a single log line on `event_name`. Mirrors the legacy
/// `emitLogRunner`'s fire-and-forget semantics: if the window has already
/// closed, the failure is reported to stderr and otherwise ignored — a
/// dropped log line shouldn't abort an in-flight activation.
pub fn emit_log(window: &tauri::Window, event_name: &str, message: &str) {
    if let Err(e) = window.emit(event_name, message) {
        eprintln!("falha ao emitir evento {event_name}: {e}");
    }
}

/// Emits a boolean completion signal on `event_name` once an activation
/// flow (Windows, Office, ...) finishes — reused across activation
/// features since they all signal completion the same way.
pub fn emit_finalizado(window: &tauri::Window, event_name: &str, sucesso: bool) {
    if let Err(e) = window.emit(event_name, sucesso) {
        eprintln!("falha ao emitir evento {event_name}: {e}");
    }
}
