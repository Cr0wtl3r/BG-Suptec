use std::time::{SystemTime, UNIX_EPOCH};

use crate::adapters::process::WinProcessRunner;
use crate::adapters::registry::WinRegistryReader;
use crate::audit;
use crate::domain::formatting_workflow::{
    run_formatting_workflow, FormattingWorkflowOptions, WorkflowUpdateMode,
};
use crate::domain::personalization::restore_photo_viewer;
use crate::domain::security::enable_system_protection;
use crate::domain::system::policies::{
    configure_defender_policy, configure_onedrive_integration, configure_windows_update,
    DefenderPolicy, WindowsUpdateMode,
};
use crate::domain::system::time::adjust_formatting_time;
use crate::events::{self, LOG_FORMATACAO};

#[tauri::command]
pub async fn executar_conclusao_formatacao(
    window: tauri::Window,
    options: FormattingWorkflowOptions,
) -> Result<(), String> {
    if let Some(mode) = options.update_mode {
        let mode = match mode {
            WorkflowUpdateMode::Disable => WindowsUpdateMode::Disable,
            WorkflowUpdateMode::NotificationsOnly => WindowsUpdateMode::NotificationsOnly,
            WorkflowUpdateMode::Enable => WindowsUpdateMode::Enable,
        };
        configure_windows_update(&WinProcessRunner, &WinRegistryReader, mode, |msg| {
            events::emit_log(&window, LOG_FORMATACAO, msg)
        })
        .await;
    }

    adjust_formatting_time(&WinProcessRunner, &WinRegistryReader, now_unix(), |msg| {
        events::emit_log(&window, LOG_FORMATACAO, msg)
    })
    .await;

    if options.restore_photo_viewer {
        restore_photo_viewer(&WinRegistryReader, |msg| {
            events::emit_log(&window, LOG_FORMATACAO, msg)
        });
    }

    if options.disable_onedrive {
        configure_onedrive_integration(&WinRegistryReader, false, |msg| {
            events::emit_log(&window, LOG_FORMATACAO, msg)
        });
    }

    configure_defender_policy(
        &WinProcessRunner,
        &WinRegistryReader,
        DefenderPolicy::Enable,
        |msg| events::emit_log(&window, LOG_FORMATACAO, msg),
    )
    .await;
    enable_system_protection(&WinProcessRunner, |msg| {
        events::emit_log(&window, LOG_FORMATACAO, msg)
    })
    .await;

    let resultado = run_formatting_workflow(&WinProcessRunner, &options, |msg| {
        events::emit_log(&window, LOG_FORMATACAO, msg)
    })
    .await;

    audit::record(
        "executar_conclusao_formatacao",
        &format!(
            "update_mode={:?},photo_viewer={},onedrive={},hibernacao={},restart={}",
            options.update_mode,
            options.restore_photo_viewer,
            options.disable_onedrive,
            options.disable_hibernation,
            options.restart_after
        ),
        &audit::outcome(&resultado),
    );
    resultado
}

fn now_unix() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0)
}
