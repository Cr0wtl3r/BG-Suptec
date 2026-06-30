use crate::ports::{ProcessRunner, RegistryWriter};

const REG_PATH_WINDOWS_VERSION: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";

/// Configures the Windows Time service to auto-start, syncs with the NTP
/// pool, restarts the service, and stamps `InstallDate` in the registry
/// with `now_unix` — mirrors legacy `AjustarHoraFormatacao`. Every step is
/// non-fatal: a failure is logged as a warning but doesn't stop the rest
/// of the flow, matching legacy `runCommandAndLog` semantics (a command
/// erroring "pode ser normal" never aborted the sequence) — same pattern
/// already used by `domain::activation::windows::activate`'s
/// `run_and_log`. `now_unix` is injected (rather than read via
/// `SystemTime::now()` here) so the flow stays unit testable; the caller
/// supplies the real current time.
///
/// Differs from legacy in one respect: the Go version passed
/// `/manualpeerlist:\"pool.ntp.br\"` with literal embedded quote
/// characters, a leftover of shell-style quoting that has no effect (and
/// arguably a latent bug — those quote characters become part of the
/// configured peer list) once the value is passed as a literal argv
/// element with no shell intermediary, as this and the rest of the
/// codebase already does. Dropped here.
pub async fn adjust_formatting_time(
    runner: &impl ProcessRunner,
    registry: &impl RegistryWriter,
    now_unix: u32,
    on_log: impl Fn(&str),
) {
    on_log("Iniciando ajuste da hora de formatação...");

    run_and_log(
        runner,
        &on_log,
        "Configurando serviço de horário (w32time) para iniciar automaticamente...",
        "sc",
        &["config", "w32time", "start=auto"],
    )
    .await;

    run_and_log(
        runner,
        &on_log,
        "Sincronizando hora com servidor NTP (pool.ntp.br)...",
        "w32tm",
        &[
            "/config",
            "/syncfromflags:manual",
            "/manualpeerlist:pool.ntp.br",
            "/reliable:YES",
            "/update",
        ],
    )
    .await;

    run_and_log(
        runner,
        &on_log,
        "Reiniciando o serviço de horário do Windows...",
        "net",
        &["stop", "w32time"],
    )
    .await;
    run_and_log(runner, &on_log, "", "net", &["start", "w32time"]).await;

    on_log("--> Ajustando InstallDate no registro para o timestamp atual...");
    if let Err(e) = registry.write_local_machine_dword(REG_PATH_WINDOWS_VERSION, "InstallDate", now_unix)
    {
        on_log(&format!(
            "AVISO: não foi possível ajustar o InstallDate (pode ser normal): {e}"
        ));
    }

    on_log("--- AJUSTE DE HORA CONCLUÍDO ---");
}

/// Runs one step, logging its outcome but never aborting the flow on
/// error — matches legacy `runCommandAndLog` semantics.
async fn run_and_log(
    runner: &impl ProcessRunner,
    on_log: &impl Fn(&str),
    log_msg: &str,
    program: &str,
    args: &[&str],
) {
    if !log_msg.is_empty() {
        on_log(&format!("--> {log_msg}"));
    }
    match runner.run(program, args, None).await {
        Ok(output) => {
            let trimmed = output.trim();
            if !trimmed.is_empty() {
                on_log(trimmed);
            }
        }
        Err(e) => on_log(&format!(
            "AVISO: Comando encontrou um erro (pode ser normal): {e}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::ports::{ProcessRunner, RegistryWriter};
    use std::sync::Mutex;

    struct FakeProcessRunner {
        calls: Mutex<Vec<(String, Vec<String>, Option<String>)>>,
        fails_program: Option<&'static str>,
    }

    impl FakeProcessRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fails_program: None,
            }
        }

        fn failing_on(program: &'static str) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fails_program: Some(program),
            }
        }

        fn calls_for(&self, program: &str) -> Vec<Vec<String>> {
            self.calls
                .lock()
                .unwrap()
                .iter()
                .filter(|(p, _, _)| p == program)
                .map(|(_, args, _)| args.clone())
                .collect()
        }
    }

    impl ProcessRunner for FakeProcessRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            cwd: Option<&str>,
        ) -> Result<String, String> {
            self.calls.lock().unwrap().push((
                program.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
                cwd.map(|c| c.to_string()),
            ));
            if self.fails_program == Some(program) {
                return Err("comando falhou".to_string());
            }
            Ok(String::new())
        }
    }

    struct FakeRegistryWriter {
        writes: Mutex<Vec<(String, String, u32)>>,
    }

    impl FakeRegistryWriter {
        fn new() -> Self {
            Self {
                writes: Mutex::new(Vec::new()),
            }
        }
    }

    impl RegistryWriter for FakeRegistryWriter {
        fn write_local_machine_dword(
            &self,
            path: &str,
            name: &str,
            value: u32,
        ) -> Result<(), String> {
            self.writes
                .lock()
                .unwrap()
                .push((path.to_string(), name.to_string(), value));
            Ok(())
        }
    }

    #[tokio::test]
    async fn adjust_formatting_time_configures_w32time_syncs_ntp_and_writes_install_date() {
        let runner = FakeProcessRunner::new();
        let registry = FakeRegistryWriter::new();

        super::adjust_formatting_time(&runner, &registry, 1_700_000_000, |_| {}).await;

        assert_eq!(
            runner.calls_for("sc"),
            vec![vec!["config".to_string(), "w32time".to_string(), "start=auto".to_string()]]
        );
        assert_eq!(
            runner.calls_for("w32tm"),
            vec![vec![
                "/config".to_string(),
                "/syncfromflags:manual".to_string(),
                "/manualpeerlist:pool.ntp.br".to_string(),
                "/reliable:YES".to_string(),
                "/update".to_string(),
            ]]
        );
        assert_eq!(
            runner.calls_for("net"),
            vec![
                vec!["stop".to_string(), "w32time".to_string()],
                vec!["start".to_string(), "w32time".to_string()],
            ]
        );
        assert_eq!(
            registry.writes.lock().unwrap().clone(),
            vec![(
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion".to_string(),
                "InstallDate".to_string(),
                1_700_000_000,
            )]
        );
    }

    #[tokio::test]
    async fn adjust_formatting_time_continues_past_a_failing_step() {
        let runner = FakeProcessRunner::failing_on("sc");
        let registry = FakeRegistryWriter::new();

        super::adjust_formatting_time(&runner, &registry, 1_700_000_000, |_| {}).await;

        // Even though "sc" failed, the rest of the flow (NTP sync, service
        // restart, registry write) still ran — non-fatal-per-step.
        assert_eq!(runner.calls_for("w32tm").len(), 1);
        assert_eq!(runner.calls_for("net").len(), 2);
        assert_eq!(registry.writes.lock().unwrap().len(), 1);
    }
}
