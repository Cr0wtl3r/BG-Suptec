use crate::ports::ProcessRunner;

/// Windows' `ERROR_NO_SHUTDOWN_IN_PROGRESS` exit code — returned by
/// `shutdown /a` when there is nothing scheduled to cancel. Legacy
/// `ExecutarComando` special-cased this exact code to return a friendly
/// "nothing to cancel" message instead of surfacing it as an error;
/// `cancel_shutdown` mirrors that so the UI doesn't show a false failure.
const ERROR_NO_SHUTDOWN_IN_PROGRESS: &str = "código 1116";

/// Schedules a system shutdown in `seconds` seconds via `shutdown /s /t`.
/// Mirrors legacy `AgendarDesligamento.svelte`'s
/// `ExecutarComando("shutdown", ["/s", "/t", tempoSelecionado])`.
pub async fn schedule_shutdown(seconds: u32, runner: &impl ProcessRunner) -> Result<String, String> {
    runner
        .run("shutdown", &["/s", "/t", &seconds.to_string()], None)
        .await
}

/// Cancels a previously scheduled shutdown via `shutdown /a`. Mirrors
/// legacy's `ExecutarComando("shutdown", ["/a"])`.
pub async fn cancel_shutdown(runner: &impl ProcessRunner) -> Result<String, String> {
    match runner.run("shutdown", &["/a"], None).await {
        Err(e) if e.contains(ERROR_NO_SHUTDOWN_IN_PROGRESS) => {
            Ok("Nenhum desligamento agendado para cancelar.".to_string())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeProcessRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        output: Result<String, String>,
    }

    impl FakeProcessRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                output: Ok(String::new()),
            }
        }

        fn returning(output: Result<String, String>) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                output,
            }
        }
    }

    impl ProcessRunner for FakeProcessRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> Result<String, String> {
            self.calls.lock().unwrap().push((
                program.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
            ));
            self.output.clone()
        }
    }

    #[tokio::test]
    async fn schedule_shutdown_runs_shutdown_s_t_with_seconds() {
        let runner = FakeProcessRunner::new();

        schedule_shutdown(300, &runner).await.unwrap();

        let calls = runner.calls.lock().unwrap();
        assert_eq!(
            calls[0],
            (
                "shutdown".to_string(),
                vec!["/s".to_string(), "/t".to_string(), "300".to_string()]
            )
        );
    }

    #[tokio::test]
    async fn cancel_shutdown_runs_shutdown_a() {
        let runner = FakeProcessRunner::new();

        cancel_shutdown(&runner).await.unwrap();

        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls[0], ("shutdown".to_string(), vec!["/a".to_string()]));
    }

    #[tokio::test]
    async fn cancel_shutdown_treats_no_shutdown_in_progress_as_a_friendly_success() {
        let runner = FakeProcessRunner::returning(Err(
            "shutdown retornou código 1116: nenhum desligamento em andamento".to_string(),
        ));

        let result = cancel_shutdown(&runner).await;

        assert_eq!(result, Ok("Nenhum desligamento agendado para cancelar.".to_string()));
    }

    #[tokio::test]
    async fn cancel_shutdown_propagates_other_errors() {
        let runner = FakeProcessRunner::returning(Err("falha inesperada".to_string()));

        let result = cancel_shutdown(&runner).await;

        assert!(result.is_err());
    }
}
