use crate::ports::ProcessRunner;

/// Filename fragments (case-insensitive) identifying the gpedit-related
/// servicing packages — the well-documented community technique for
/// enabling the Group Policy Editor on Windows editions (Home) that don't
/// ship it by default. This feature has no legacy implementation to port:
/// the Go backend never had a matching method, and the Svelte UI called a
/// Wails-bound `AtivarGpedit()` that was never written (a dead reference —
/// confirmed absent from the generated `wailsjs` bindings).
const GPEDIT_PACKAGE_NAME_FRAGMENTS: [&str; 2] = [
    "grouppolicy-clientextensions-package",
    "grouppolicy-clienttools-package",
];

/// Enables `gpedit.msc` by installing every gpedit-related servicing
/// package found under `C:\Windows\servicing\Packages\` via
/// `dism /online /norestart /add-package:<path>`. `packages_dir` and
/// `list_dir` are injected (rather than reading the real filesystem
/// directly) so this stays unit testable — same pattern as
/// `domain::activation::office::install_licenses`. A single package that
/// fails to install is logged as a warning, not fatal — the rest of the
/// matched packages should still be attempted. Returns the count of
/// packages successfully installed; an empty match (no gpedit packages
/// found at all) is treated as an outright error, since installing zero
/// packages can never actually enable gpedit.
pub async fn enable_gpedit(
    packages_dir: &str,
    runner: &impl ProcessRunner,
    list_dir: impl Fn(&str) -> Vec<String>,
    on_log: impl Fn(&str),
) -> Result<usize, String> {
    on_log("INICIANDO ATIVAÇÃO DO EDITOR DE POLÍTICA DE GRUPO (GPEDIT.MSC)...");

    let mum_files: Vec<String> = list_dir(packages_dir)
        .into_iter()
        .filter(|file_name| {
            let lower = file_name.to_ascii_lowercase();
            lower.ends_with(".mum")
                && GPEDIT_PACKAGE_NAME_FRAGMENTS
                    .iter()
                    .any(|fragment| lower.contains(fragment))
        })
        .collect();

    if mum_files.is_empty() {
        on_log("Nenhum pacote do Gpedit encontrado em servicing\\Packages.");
        return Err("nenhum pacote do Gpedit foi encontrado neste sistema".to_string());
    }

    let mut installed = 0usize;
    for file_name in mum_files {
        let full_path = format!(r"{packages_dir}\{file_name}");
        on_log(&format!("--> Instalando pacote: {file_name}"));
        match runner
            .run(
                "dism",
                &["/online", "/norestart", &format!("/add-package:{full_path}")],
                None,
            )
            .await
        {
            Ok(_) => installed += 1,
            Err(e) => on_log(&format!("AVISO: falha ao instalar {file_name}: {e}")),
        }
    }

    on_log("\n--- EDITOR DE POLÍTICA DE GRUPO ATIVADO ---");
    Ok(installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeProcessRunner {
        calls: Mutex<Vec<Vec<String>>>,
        fails_on_substring: Option<&'static str>,
    }

    impl FakeProcessRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fails_on_substring: None,
            }
        }

        fn failing_on(substring: &'static str) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                fails_on_substring: Some(substring),
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
            let mut full = vec![program.to_string()];
            full.extend(args.iter().map(|a| a.to_string()));
            self.calls.lock().unwrap().push(full.clone());
            if let Some(substr) = self.fails_on_substring {
                if full.iter().any(|a| a.contains(substr)) {
                    return Err("falha simulada".to_string());
                }
            }
            Ok(String::new())
        }
    }

    const PACKAGES_DIR: &str = r"C:\Windows\servicing\Packages";

    fn sample_files() -> Vec<String> {
        vec![
            "Microsoft-Windows-GroupPolicy-ClientExtensions-Package~31bf3856ad364e35~amd64~~10.0.19041.1.mum".to_string(),
            "Microsoft-Windows-GroupPolicy-ClientTools-Package~31bf3856ad364e35~amd64~~10.0.19041.1.mum".to_string(),
            "Microsoft-Windows-GroupPolicy-ClientExtensions-Package~31bf3856ad364e35~amd64~~10.0.19041.1.cat".to_string(),
            "Microsoft-Windows-Foundation-Package~31bf3856ad364e35~amd64~~10.0.19041.1.mum".to_string(),
        ]
    }

    #[tokio::test]
    async fn enable_gpedit_installs_only_matching_mum_packages() {
        let runner = FakeProcessRunner::new();
        let files = sample_files();

        let installed = enable_gpedit(PACKAGES_DIR, &runner, |_| files.clone(), |_| {})
            .await
            .unwrap();

        assert_eq!(installed, 2);
        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        for call in calls.iter() {
            assert_eq!(call[0], "dism");
            assert!(call.contains(&"/online".to_string()));
            assert!(call.contains(&"/norestart".to_string()));
            assert!(call.iter().any(|a| a.starts_with("/add-package:")
                && a.ends_with(".mum")
                && a.to_ascii_lowercase().contains("grouppolicy")));
        }
    }

    #[tokio::test]
    async fn enable_gpedit_errors_when_no_packages_found() {
        let runner = FakeProcessRunner::new();

        let result = enable_gpedit(PACKAGES_DIR, &runner, |_| Vec::new(), |_| {}).await;

        assert!(result.is_err());
        assert!(runner.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn enable_gpedit_continues_past_a_failing_package() {
        let runner = FakeProcessRunner::failing_on("ClientExtensions");
        let files = sample_files();

        let installed = enable_gpedit(PACKAGES_DIR, &runner, |_| files.clone(), |_| {})
            .await
            .unwrap();

        // Only the ClientTools package succeeded; ClientExtensions failed
        // but didn't abort the loop.
        assert_eq!(installed, 1);
        assert_eq!(runner.calls.lock().unwrap().len(), 2);
    }
}
