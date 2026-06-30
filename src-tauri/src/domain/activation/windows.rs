use std::collections::HashMap;

use crate::ports::CscriptRunner;

/// Activates Windows via `slmgr.vbs`: installs the GVLK key for `versao`
/// (looked up in `keys`, sourced from the externalized `kms.json`), points
/// it at `kms_server`, then attempts activation. `on_log` receives each
/// progress line as it happens, so callers can stream it to the UI.
/// Returns `true` only if the final `/ato` activation attempt succeeds —
/// mirrors the legacy behavior where `/ipk`/`/skms` failures are logged as
/// warnings but don't abort the flow (e.g. `/skms` can be a no-op retry).
pub async fn activate(
    versao: &str,
    keys: &HashMap<String, String>,
    kms_server: &str,
    runner: &impl CscriptRunner,
    on_log: impl Fn(&str),
) -> bool {
    on_log(&format!("Iniciando ativação para Windows {versao}..."));

    let Some(key) = keys.get(versao) else {
        on_log("ERRO: Versão do Windows inválida.");
        on_log("--- FALHA NA ATIVAÇÃO ---");
        return false;
    };

    run_and_log(
        runner,
        &on_log,
        "Instalando chave do produto (GVLK)...",
        &["/ipk", key],
    )
    .await;

    run_and_log(
        runner,
        &on_log,
        &format!("Definindo servidor KMS: {kms_server}..."),
        &["/skms", kms_server],
    )
    .await;

    on_log("--> Tentando ativar...");
    let result = runner.run("slmgr.vbs", &["/ato"]).await;
    let success = result.is_ok();
    log_output(&on_log, &result);

    on_log(if success {
        "--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---"
    } else {
        "--- FALHA NA ATIVAÇÃO ---"
    });

    success
}

/// Runs one `slmgr.vbs` step, logging its outcome but never aborting the
/// flow on error — matches legacy `runCommandAndLog` semantics where a
/// non-fatal step error ("pode ser normal") is just surfaced as a warning.
async fn run_and_log(
    runner: &impl CscriptRunner,
    on_log: &impl Fn(&str),
    log_msg: &str,
    args: &[&str],
) {
    on_log(&format!("--> {log_msg}"));
    let result = runner.run("slmgr.vbs", args).await;
    log_output(on_log, &result);
}

fn log_output(on_log: &impl Fn(&str), result: &Result<String, String>) {
    match result {
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
    use super::*;
    use std::sync::Mutex;

    struct FakeCscriptRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        fail_ato: bool,
    }

    impl FakeCscriptRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fail_ato: false,
            }
        }

        fn failing_ato() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fail_ato: true,
            }
        }
    }

    impl CscriptRunner for FakeCscriptRunner {
        async fn run(&self, script_path: &str, args: &[&str]) -> Result<String, String> {
            self.calls.lock().unwrap().push((
                script_path.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
            ));

            if args.first() == Some(&"/ato") {
                return if self.fail_ato {
                    Err("0x8007232B Falha ao ativar.".to_string())
                } else {
                    Ok("O produto foi ativado com sucesso.".to_string())
                };
            }
            Ok(String::new())
        }
    }

    fn sample_keys() -> HashMap<String, String> {
        let mut keys = HashMap::new();
        keys.insert(
            "pro".to_string(),
            "W269N-WFGWX-YVC9B-4J6C9-T83GX".to_string(),
        );
        keys
    }

    #[tokio::test]
    async fn activate_installs_the_correct_gvlk_key_via_slmgr_ipk() {
        let runner = FakeCscriptRunner::new();

        activate("pro", &sample_keys(), "kms.msguides.com", &runner, |_| {}).await;

        let calls = runner.calls.lock().unwrap();
        assert!(
            calls.iter().any(|(script, args)| script == "slmgr.vbs"
                && args.as_slice() == ["/ipk", "W269N-WFGWX-YVC9B-4J6C9-T83GX"]),
            "esperava chamada cscript slmgr.vbs /ipk W269N-WFGWX-YVC9B-4J6C9-T83GX, calls={calls:?}"
        );
    }

    #[tokio::test]
    async fn activate_sets_the_kms_server_via_slmgr_skms() {
        let runner = FakeCscriptRunner::new();

        activate("pro", &sample_keys(), "kms.msguides.com", &runner, |_| {}).await;

        let calls = runner.calls.lock().unwrap();
        assert!(calls
            .iter()
            .any(|(script, args)| script == "slmgr.vbs"
                && args.as_slice() == ["/skms", "kms.msguides.com"]));
    }

    #[tokio::test]
    async fn activate_attempts_activation_via_slmgr_ato_and_returns_true_on_success() {
        let runner = FakeCscriptRunner::new();

        let success = activate("pro", &sample_keys(), "kms.msguides.com", &runner, |_| {}).await;

        assert!(success);
        let calls = runner.calls.lock().unwrap();
        assert!(calls
            .iter()
            .any(|(script, args)| script == "slmgr.vbs" && args.as_slice() == ["/ato"]));
    }

    #[tokio::test]
    async fn activate_returns_false_when_ato_fails() {
        let runner = FakeCscriptRunner::failing_ato();

        let success = activate("pro", &sample_keys(), "kms.msguides.com", &runner, |_| {}).await;

        assert!(!success);
    }

    #[tokio::test]
    async fn activate_returns_false_for_unknown_windows_edition_without_running_any_command() {
        let runner = FakeCscriptRunner::new();

        let success = activate(
            "ultimate",
            &sample_keys(),
            "kms.msguides.com",
            &runner,
            |_| {},
        )
        .await;

        assert!(!success);
        assert!(
            runner.calls.lock().unwrap().is_empty(),
            "não deve tentar nenhum comando para edição inválida"
        );
    }

    #[tokio::test]
    async fn activate_streams_progress_messages_via_log_callback() {
        let runner = FakeCscriptRunner::new();
        let logs: Mutex<Vec<String>> = Mutex::new(Vec::new());

        activate("pro", &sample_keys(), "kms.msguides.com", &runner, |msg| {
            logs.lock().unwrap().push(msg.to_string());
        })
        .await;

        let logs = logs.lock().unwrap();
        assert!(logs
            .iter()
            .any(|l| l.contains("Iniciando ativação para Windows pro")));
        assert!(logs
            .iter()
            .any(|l| l.contains("ATIVAÇÃO CONCLUÍDA COM SUCESSO")));
    }
}
