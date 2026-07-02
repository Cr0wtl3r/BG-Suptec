use crate::ports::ProcessRunner;

pub mod cleanup;

/// Flushes the local DNS resolver cache via `ipconfig /flushdns`. Mirrors
/// legacy `LimpaCacheDNS.svelte`'s `ExecutarComando("ipconfig", ["/flushdns"])`.
pub async fn clear_dns_cache(runner: &impl ProcessRunner) -> Result<String, String> {
    runner.run("ipconfig", &["/flushdns"], None).await
}

/// Disables Windows hibernation and removes `hiberfil.sys` via
/// `powercfg /h off`. Mirrors legacy `DesativaHibernacao.svelte`'s
/// `ExecutarComando("powercfg", ["-h", "off"])` (using `/h`, the form
/// called for by the refactor checklist — both prefixes are accepted by
/// `powercfg`).
pub async fn disable_hibernation(runner: &impl ProcessRunner) -> Result<String, String> {
    runner.run("powercfg", &["/h", "off"], None).await
}

/// Stops the Print Spooler service, deletes any stuck `.SHD`/`.SPL` job
/// files left behind in `spool_dir`, then restarts the service — resolves
/// jammed/stuck print queues. Extends legacy `LimpaSpoolImpressao.svelte`
/// (which only stopped/restarted the service) with the file cleanup the
/// plan calls for; `list_dir`/`delete_file` are injected so this stays unit
/// testable without touching the real filesystem (same pattern as
/// `domain::activation::office::install_licenses`). A file that fails to
/// delete (e.g. still locked) is logged as a warning rather than aborting
/// the whole operation — the rest of the cleanup should still proceed.
pub async fn clear_print_spool(
    spool_dir: &str,
    runner: &impl ProcessRunner,
    list_dir: impl Fn(&str) -> Vec<String>,
    delete_file: impl Fn(&str) -> Result<(), String>,
    on_log: impl Fn(&str),
) -> Result<usize, String> {
    on_log("Parando o serviço Spooler...");
    runner.run("net", &["stop", "spooler"], None).await?;

    let mut deleted = 0usize;
    for file_name in list_dir(spool_dir) {
        let lower = file_name.to_ascii_lowercase();
        if lower.ends_with(".shd") || lower.ends_with(".spl") {
            let full_path = format!(r"{spool_dir}\{file_name}");
            on_log(&format!("Excluindo arquivo de spool travado: {file_name}"));
            match delete_file(&full_path) {
                Ok(()) => deleted += 1,
                Err(e) => on_log(&format!("AVISO: não foi possível excluir {full_path}: {e}")),
            }
        }
    }

    on_log("Reiniciando o serviço Spooler...");
    runner.run("net", &["start", "spooler"], None).await?;

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use crate::ports::ProcessRunner;
    use std::sync::Mutex;

    struct FakeProcessRunner {
        calls: Mutex<Vec<(String, Vec<String>, Option<String>)>>,
    }

    impl FakeProcessRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
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
            Ok(String::new())
        }
    }

    #[tokio::test]
    async fn clear_dns_cache_runs_ipconfig_flushdns() {
        let runner = FakeProcessRunner::new();

        super::clear_dns_cache(&runner).await.unwrap();

        assert_eq!(
            runner.calls_for("ipconfig"),
            vec![vec!["/flushdns".to_string()]]
        );
    }

    #[tokio::test]
    async fn disable_hibernation_runs_powercfg_h_off() {
        let runner = FakeProcessRunner::new();

        super::disable_hibernation(&runner).await.unwrap();

        assert_eq!(
            runner.calls_for("powercfg"),
            vec![vec!["/h".to_string(), "off".to_string()]]
        );
    }

    /// Records every operation (process calls and file deletions) in a
    /// single shared, ordered log via `Arc<Mutex<..>>` so a test can assert
    /// the spooler is stopped *before* files are deleted and restarted
    /// *after* — something two separately-tracked fakes couldn't prove.
    struct OrderedFakeProcessRunner {
        ops: std::sync::Arc<Mutex<Vec<String>>>,
    }

    impl ProcessRunner for OrderedFakeProcessRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> Result<String, String> {
            self.ops
                .lock()
                .unwrap()
                .push(format!("{program} {}", args.join(" ")));
            Ok(String::new())
        }
    }

    const SPOOL_DIR: &str = r"C:\Windows\System32\spool\PRINTERS";

    #[tokio::test]
    async fn clear_print_spool_stops_deletes_shd_and_spl_files_in_order_and_restarts_spooler() {
        let ops = std::sync::Arc::new(Mutex::new(Vec::new()));
        let runner = OrderedFakeProcessRunner { ops: ops.clone() };
        let delete_ops = ops.clone();

        let files = vec![
            "PRN1.SHD".to_string(),
            "PRN1.SPL".to_string(),
            "PRN2.shd".to_string(),
            "readme.txt".to_string(),
        ];

        let deleted = super::clear_print_spool(
            SPOOL_DIR,
            &runner,
            |_| files.clone(),
            move |path: &str| {
                delete_ops.lock().unwrap().push(format!("delete {path}"));
                Ok(())
            },
            |_| {},
        )
        .await
        .unwrap();

        assert_eq!(deleted, 3);
        assert_eq!(
            ops.lock().unwrap().clone(),
            vec![
                "net stop spooler".to_string(),
                format!(r"delete {SPOOL_DIR}\PRN1.SHD"),
                format!(r"delete {SPOOL_DIR}\PRN1.SPL"),
                format!(r"delete {SPOOL_DIR}\PRN2.shd"),
                "net start spooler".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn clear_print_spool_ignores_files_with_other_extensions() {
        let ops = std::sync::Arc::new(Mutex::new(Vec::new()));
        let runner = OrderedFakeProcessRunner { ops: ops.clone() };
        let delete_ops = ops.clone();

        let files = vec!["readme.txt".to_string(), "driver.dll".to_string()];

        super::clear_print_spool(
            SPOOL_DIR,
            &runner,
            |_| files.clone(),
            move |path: &str| {
                delete_ops.lock().unwrap().push(format!("delete {path}"));
                Ok(())
            },
            |_| {},
        )
        .await
        .unwrap();

        let recorded = ops.lock().unwrap().clone();
        assert!(!recorded.iter().any(|op| op.starts_with("delete")));
    }
}
