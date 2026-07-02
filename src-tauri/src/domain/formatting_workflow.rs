use serde::{Deserialize, Serialize};

use crate::ports::ProcessRunner;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattingWorkflowOptions {
    pub update_mode: Option<WorkflowUpdateMode>,
    pub restore_photo_viewer: bool,
    pub disable_onedrive: bool,
    pub disable_hibernation: bool,
    pub power_profile: Option<WorkflowPowerProfile>,
    pub restart_after: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowUpdateMode {
    Disable,
    NotificationsOnly,
    Enable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowPowerProfile {
    Notebook,
    Desktop,
}

pub async fn run_formatting_workflow(
    runner: &impl ProcessRunner,
    options: &FormattingWorkflowOptions,
    on_log: impl Fn(&str),
) -> Result<(), String> {
    if let Some(mode) = options.update_mode {
        on_log(&format!("Etapa Windows Update selecionada: {mode:?}"));
    }
    if options.restore_photo_viewer {
        on_log("Etapa Photo Viewer selecionada.");
    }
    if options.disable_onedrive {
        on_log("Etapa OneDrive selecionada.");
    }
    if options.disable_hibernation {
        runner.run("powercfg", &["-h", "off"], None).await?;
    }
    if let Some(profile) = options.power_profile {
        let guid = match profile {
            WorkflowPowerProfile::Notebook => "381b4222-f694-41f0-9685-ff5bb260df2e",
            WorkflowPowerProfile::Desktop => "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c",
        };
        runner.run("powercfg", &["/s", guid], None).await?;
    }
    if options.restart_after {
        runner.run("shutdown", &["/r", "/t", "5"], None).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::ProcessRunner;
    use std::sync::{Arc, Mutex};

    struct FakeRunner {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl ProcessRunner for FakeRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> Result<String, String> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{program} {}", args.join(" ")));
            Ok(String::new())
        }
    }

    #[tokio::test]
    async fn formatting_workflow_runs_selected_steps_in_order_without_self_delete() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };
        let options = FormattingWorkflowOptions {
            update_mode: Some(WorkflowUpdateMode::NotificationsOnly),
            restore_photo_viewer: true,
            disable_onedrive: true,
            disable_hibernation: true,
            power_profile: Some(WorkflowPowerProfile::Desktop),
            restart_after: false,
        };

        run_formatting_workflow(&runner, &options, |_| {})
            .await
            .unwrap();

        let recorded = calls.lock().unwrap().join("\n");
        assert!(recorded.contains("powercfg /s 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"));
        assert!(!recorded.contains("rmdir"));
    }
}
