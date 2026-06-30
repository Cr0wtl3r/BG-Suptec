use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::ports::{ProcessRunner, RegistryReader};

/// One installed program surfaced to the frontend's "choose an installed
/// program" dropdown — field names (`nome`/`caminho`) match legacy
/// `ProgramaInfo` so the Tauri command's JSON payload lines up with the
/// React component's types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgramaInfo {
    pub nome: String,
    pub caminho: String,
}

/// The two `Uninstall` registry roots scanned for installed programs —
/// mirrors legacy `ObterProgramasInstalados`'s `keys` slice.
const UNINSTALL_KEYS: [&str; 2] = [
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
];

/// Builds the firewall rule name for a given executable path — exact
/// format mirrors legacy `obterNomeRegra`: `[BG-SupTec] Bloqueio - {exe}`,
/// where `{exe}` is just the file name component.
fn rule_name(caminho_executavel: &str) -> String {
    let file_name = Path::new(caminho_executavel)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| caminho_executavel.to_string());
    format!("[BG-SupTec] Bloqueio - {file_name}")
}

/// Blocks a program's inbound and outbound network access via Windows
/// Firewall rules — mirrors legacy `BloquearProgramasFirewall` (single-path
/// version; the Tauri command layer loops over the array from the
/// frontend). Deletes any pre-existing rule with the same name first
/// (best-effort, result ignored — matches legacy's unchecked delete call),
/// then adds the `dir=in` rule, then the `dir=out` rule. Returns `Err` if
/// either `add` call fails.
pub async fn block_program_in_firewall(
    runner: &impl ProcessRunner,
    caminho_executavel: &str,
) -> Result<(), String> {
    let nome_regra = rule_name(caminho_executavel);

    let _ = runner
        .run(
            "netsh",
            &["advfirewall", "firewall", "delete", "rule", &format!("name={nome_regra}")],
            None,
        )
        .await;

    runner
        .run(
            "netsh",
            &[
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={nome_regra}"),
                "dir=in",
                "action=block",
                &format!("program={caminho_executavel}"),
                "enable=yes",
            ],
            None,
        )
        .await
        .map_err(|e| format!("falha ao bloquear entrada para {nome_regra}: {e}"))?;

    runner
        .run(
            "netsh",
            &[
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={nome_regra}"),
                "dir=out",
                "action=block",
                &format!("program={caminho_executavel}"),
                "enable=yes",
            ],
            None,
        )
        .await
        .map_err(|e| format!("falha ao bloquear saída para {nome_regra}: {e}"))?;

    Ok(())
}

/// Removes the firewall block rule for a program — mirrors legacy
/// `DesbloquearProgramasFirewall` (single-path version). An error
/// containing "No rules match the specified criteria" is treated as
/// success (program was already unblocked); any other error is a real
/// failure.
pub async fn unblock_program_in_firewall(
    runner: &impl ProcessRunner,
    caminho_executavel: &str,
) -> Result<(), String> {
    let nome_regra = rule_name(caminho_executavel);

    let result = runner
        .run(
            "netsh",
            &["advfirewall", "firewall", "delete", "rule", &format!("name={nome_regra}")],
            None,
        )
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) if e.contains("No rules match the specified criteria") => Ok(()),
        Err(e) => Err(format!("falha ao desbloquear {nome_regra}: {e}")),
    }
}

/// Checks whether each given path currently has a block rule in the
/// firewall — mirrors legacy `VerificarStatusFirewall`: for each path, runs
/// `netsh advfirewall firewall show rule name={nome_regra}`; the command
/// succeeding means blocked=true, failing means blocked=false.
pub async fn check_firewall_status(
    runner: &impl ProcessRunner,
    caminhos_executaveis: &[String],
) -> HashMap<String, bool> {
    let mut status = HashMap::new();
    for caminho in caminhos_executaveis {
        let nome_regra = rule_name(caminho);
        let result = runner
            .run(
                "netsh",
                &["advfirewall", "firewall", "show", "rule", &format!("name={nome_regra}")],
                None,
            )
            .await;
        status.insert(caminho.clone(), result.is_ok());
    }
    status
}

/// Lists installed programs that have both a `DisplayName` and
/// `InstallLocation`, scanning both `Uninstall` registry roots and sorted
/// by name — mirrors legacy `ObterProgramasInstalados`. Each subkey is
/// enumerated and read within a single loop iteration so its
/// `RegistryReader`-side handle (in the real `WinRegistryReader`
/// implementation) closes before the next iteration opens a new one — see
/// `adapters::registry::WinRegistryReader::list_local_machine_subkeys` for
/// the handle-lifetime note this fixes relative to the legacy Go version.
pub fn list_installed_programs(registry: &impl RegistryReader) -> Vec<ProgramaInfo> {
    let mut programas = Vec::new();

    for key_path in UNINSTALL_KEYS {
        for sub_key_name in registry.list_local_machine_subkeys(key_path) {
            let sub_key_path = format!("{key_path}\\{sub_key_name}");
            let display_name = registry.read_local_machine_string(&sub_key_path, "DisplayName");
            let install_location =
                registry.read_local_machine_string(&sub_key_path, "InstallLocation");

            if let (Some(nome), Some(caminho)) = (display_name, install_location) {
                if !nome.is_empty() && !caminho.is_empty() {
                    programas.push(ProgramaInfo { nome, caminho });
                }
            }
        }
    }

    programas.sort_by(|a, b| a.nome.cmp(&b.nome));
    programas
}

/// Recursively lists `.exe` files (case-insensitive) under `path` — mirrors
/// legacy `ListarExecutaveis`'s `filepath.Walk` + `strings.HasSuffix(...,
/// ".exe")`. Walks a real filesystem tree via `walkdir`, so this is tested
/// against a real temporary directory rather than mocked.
pub fn list_executables(path: &str) -> Vec<String> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .ends_with(".exe")
        })
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::ports::ProcessRunner;
    use std::sync::{Arc, Mutex};

    /// Records every call's program+args (joined) in a single shared,
    /// ordered log via `Arc<Mutex<..>>` so tests can assert exact
    /// sequencing — same pattern as `domain::security::mod::tests::OrderedFakeProcessRunner`.
    struct OrderedFakeProcessRunner {
        ops: Arc<Mutex<Vec<String>>>,
        /// Returns `Err` when the joined "program args" string contains
        /// this substring (lets tests target one specific call, e.g. the
        /// `dir=out add` call, rather than every call for a program).
        fails_when_contains: Option<&'static str>,
        /// Error message returned for a failing call.
        fail_message: &'static str,
    }

    impl OrderedFakeProcessRunner {
        fn new() -> Self {
            Self {
                ops: Arc::new(Mutex::new(Vec::new())),
                fails_when_contains: None,
                fail_message: "comando falhou",
            }
        }

        fn failing_when_contains(substr: &'static str, message: &'static str) -> Self {
            Self {
                ops: Arc::new(Mutex::new(Vec::new())),
                fails_when_contains: Some(substr),
                fail_message: message,
            }
        }
    }

    impl ProcessRunner for OrderedFakeProcessRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> Result<String, String> {
            let joined = format!("{program} {}", args.join(" "));
            self.ops.lock().unwrap().push(joined.clone());
            if let Some(substr) = self.fails_when_contains {
                if joined.contains(substr) {
                    return Err(self.fail_message.to_string());
                }
            }
            Ok(String::new())
        }
    }

    #[tokio::test]
    async fn block_program_in_firewall_deletes_then_adds_in_and_out_rules() {
        let runner = OrderedFakeProcessRunner::new();
        let ops = runner.ops.clone();

        let result = super::block_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_ok());
        let recorded = ops.lock().unwrap().clone();
        assert_eq!(
            recorded,
            vec![
                r#"netsh advfirewall firewall delete rule name=[BG-SupTec] Bloqueio - foo.exe"#
                    .to_string(),
                r#"netsh advfirewall firewall add rule name=[BG-SupTec] Bloqueio - foo.exe dir=in action=block program=C:\Programs\Foo\foo.exe enable=yes"#
                    .to_string(),
                r#"netsh advfirewall firewall add rule name=[BG-SupTec] Bloqueio - foo.exe dir=out action=block program=C:\Programs\Foo\foo.exe enable=yes"#
                    .to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn block_program_in_firewall_fails_if_add_in_fails() {
        let runner = OrderedFakeProcessRunner::failing_when_contains("dir=in", "falhou dir=in");

        let result = super::block_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn block_program_in_firewall_fails_if_add_out_fails() {
        let runner = OrderedFakeProcessRunner::failing_when_contains("dir=out", "falhou dir=out");

        let result = super::block_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn block_program_in_firewall_ignores_delete_failure() {
        // The pre-existing-rule delete is best-effort (legacy ignores its
        // result entirely) — a failing delete must not stop the adds.
        let runner = OrderedFakeProcessRunner::failing_when_contains("delete", "no rule");

        let result = super::block_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn unblock_program_in_firewall_deletes_rule() {
        let runner = OrderedFakeProcessRunner::new();
        let ops = runner.ops.clone();

        let result = super::unblock_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_ok());
        let recorded = ops.lock().unwrap().clone();
        assert_eq!(
            recorded,
            vec![
                r#"netsh advfirewall firewall delete rule name=[BG-SupTec] Bloqueio - foo.exe"#
                    .to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn unblock_program_in_firewall_treats_no_matching_rule_as_success() {
        let runner = OrderedFakeProcessRunner::failing_when_contains(
            "delete",
            "No rules match the specified criteria.",
        );

        let result = super::unblock_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn unblock_program_in_firewall_fails_on_other_errors() {
        let runner = OrderedFakeProcessRunner::failing_when_contains("delete", "acesso negado");

        let result = super::unblock_program_in_firewall(&runner, r"C:\Programs\Foo\foo.exe").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn check_firewall_status_maps_each_path_to_blocked_bool() {
        // "show rule" succeeds for "foo.exe" (blocked=true), fails for
        // "bar.exe" (blocked=false) — mirrors legacy's `err == nil` check.
        let runner = OrderedFakeProcessRunner::failing_when_contains("bar.exe", "not found");

        let result = super::check_firewall_status(
            &runner,
            &[r"C:\Programs\Foo\foo.exe".to_string(), r"C:\Programs\Bar\bar.exe".to_string()],
        )
        .await;

        assert_eq!(result.get(r"C:\Programs\Foo\foo.exe"), Some(&true));
        assert_eq!(result.get(r"C:\Programs\Bar\bar.exe"), Some(&false));
    }

    #[test]
    fn list_installed_programs_filters_and_sorts_by_name() {
        use crate::ports::RegistryReader;

        struct FakeRegistry;
        impl RegistryReader for FakeRegistry {
            fn read_local_machine_string(&self, path: &str, name: &str) -> Option<String> {
                match (path, name) {
                    (r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Zeta", "DisplayName") => {
                        Some("Zeta App".to_string())
                    }
                    (r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Zeta", "InstallLocation") => {
                        Some(r"C:\Zeta".to_string())
                    }
                    (r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NoLocation", "DisplayName") => {
                        Some("Sem Local".to_string())
                    }
                    (
                        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\NoLocation",
                        "InstallLocation",
                    ) => None,
                    (
                        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Alpha",
                        "DisplayName",
                    ) => Some("Alpha App".to_string()),
                    (
                        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Alpha",
                        "InstallLocation",
                    ) => Some(r"C:\Alpha".to_string()),
                    _ => None,
                }
            }

            fn list_local_machine_subkeys(&self, path: &str) -> Vec<String> {
                match path {
                    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall" => {
                        vec!["Zeta".to_string(), "NoLocation".to_string()]
                    }
                    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall" => {
                        vec!["Alpha".to_string()]
                    }
                    _ => vec![],
                }
            }
        }

        let result = super::list_installed_programs(&FakeRegistry);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].nome, "Alpha App");
        assert_eq!(result[0].caminho, r"C:\Alpha");
        assert_eq!(result[1].nome, "Zeta App");
        assert_eq!(result[1].caminho, r"C:\Zeta");
    }

    #[test]
    fn list_executables_finds_nested_exe_files_case_insensitive() {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "bg-suptec-test-list-executables-{}",
            std::process::id()
        ));
        let sub = base.join("subdir");
        std::fs::create_dir_all(&sub).unwrap();

        let exe1 = base.join("foo.exe");
        let exe2 = sub.join("BAR.EXE");
        let not_exe = base.join("readme.txt");
        std::fs::write(&exe1, b"").unwrap();
        std::fs::write(&exe2, b"").unwrap();
        std::fs::write(&not_exe, b"").unwrap();

        let result = super::list_executables(base.to_str().unwrap());

        std::fs::remove_dir_all(&base).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p == exe1.to_str().unwrap()));
        assert!(result.iter().any(|p| p == exe2.to_str().unwrap()));
        assert!(!result.iter().any(|p| p.contains("readme")));
    }
}
