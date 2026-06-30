use std::collections::HashMap;

use regex::Regex;
use serde::Deserialize;

use crate::ports::{ProcessRunner, TcpHealthChecker};

/// Standard KMS port — fixed by `/setprt:1688` earlier in the flow, so the
/// health check before each server attempt targets the same port.
const KMS_PORT: u16 = 1688;

/// GVLK key, existing-license unpkeys, KMS license file name patterns and
/// KMS server fallback list for one Office edition (`2016`/`2021`/`2024`),
/// sourced from the externalized `kms.json` — mirrors legacy
/// `OfficeVersionInfo`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OfficeVersionConfig {
    pub prod_key: String,
    pub unpkeys: Vec<String>,
    pub license_patterns: Vec<String>,
    pub kms_servers: Vec<String>,
}

/// Locates the Office installation folder (the one containing `ospp.vbs`)
/// by checking `%ProgramFiles%\Microsoft Office\{Office16,Office15}` and
/// the same under `%ProgramFiles(x86)%`, in that order — mirrors legacy
/// `findOfficePathGo`. `path_exists` is injected so this stays unit
/// testable without touching the real filesystem.
pub fn find_office_path(
    program_files: Option<&str>,
    program_files_x86: Option<&str>,
    path_exists: impl Fn(&str) -> bool,
) -> Result<String, String> {
    let base_paths = [program_files, program_files_x86]
        .into_iter()
        .flatten()
        .map(|base| format!(r"{base}\Microsoft Office"));
    let version_folders = ["Office16", "Office15"];

    for base_path in base_paths {
        for version_folder in version_folders {
            let full_path = format!(r"{base_path}\{version_folder}");
            if path_exists(&format!(r"{full_path}\ospp.vbs")) {
                return Ok(full_path);
            }
        }
    }

    Err("a pasta de instalação do Office (com ospp.vbs) não foi encontrada".to_string())
}

/// Activates Office via `ospp.vbs`: closes running Office apps, removes any
/// existing product keys, installs the matching KMS license files, installs
/// the GVLK key, then tries each KMS server in `config.kms_servers` in turn
/// until one reports a successful activation. `office_path` is the folder
/// found by `find_office_path` (where `ospp.vbs` lives — not under
/// `%SystemRoot%\System32`, so this uses the generic `ProcessRunner` rather
/// than `CscriptRunner`). Mirrors legacy `AtivarOffice`.
pub async fn activate(
    versao: &str,
    office_path: &str,
    versions: &HashMap<String, OfficeVersionConfig>,
    runner: &impl ProcessRunner,
    health_checker: &impl TcpHealthChecker,
    dir_exists: impl Fn(&str) -> bool,
    list_dir: impl Fn(&str) -> Vec<String>,
    on_log: impl Fn(&str),
) -> bool {
    let Some(config) = versions.get(versao) else {
        on_log("ERRO: Versão do Office inválida.");
        on_log("--- FALHA NA ATIVAÇÃO ---");
        return false;
    };

    let ospp_path = format!(r"{office_path}\ospp.vbs");

    run_and_log(
        runner,
        &on_log,
        "Fechando processos do Office...",
        "taskkill",
        &[
            "/f", "/im", "winword.exe", "/im", "excel.exe", "/im", "powerpnt.exe", "/im",
            "outlook.exe",
        ],
        None,
    )
    .await;

    for unpkey in &config.unpkeys {
        run_and_log(
            runner,
            &on_log,
            &format!("Desinstalando chave do produto existente ({unpkey})..."),
            "cscript",
            &[ospp_path.as_str(), &format!("/unpkey:{unpkey}")],
            None,
        )
        .await;
    }

    if !config.license_patterns.is_empty() {
        install_licenses(
            &ospp_path,
            office_path,
            &config.license_patterns,
            runner,
            &dir_exists,
            &list_dir,
            &on_log,
        )
        .await;
    }

    run_and_log(
        runner,
        &on_log,
        &format!("Instalando chave do produto GVLK ({})...", config.prod_key),
        "cscript",
        &[ospp_path.as_str(), &format!("/inpkey:{}", config.prod_key)],
        None,
    )
    .await;

    run_and_log(
        runner,
        &on_log,
        "Definindo porta KMS: 1688 (padrão)...",
        "cscript",
        &[ospp_path.as_str(), "/setprt:1688"],
        None,
    )
    .await;

    for server in &config.kms_servers {
        if !health_checker.is_reachable(server, KMS_PORT).await {
            on_log(&format!(
                "--- Servidor KMS {server} não respondeu ao health check (porta {KMS_PORT}). Pulando... ---"
            ));
            continue;
        }

        run_and_log(
            runner,
            &on_log,
            &format!("Definindo servidor KMS: {server}..."),
            "cscript",
            &[ospp_path.as_str(), &format!("/sethst:{server}")],
            None,
        )
        .await;

        on_log(&format!("--> Tentando ativar com {server}..."));
        // Mirrors legacy: only the `/act` call runs with cwd = office_path.
        let result = runner
            .run("cscript", &[ospp_path.as_str(), "/act"], Some(office_path))
            .await;
        let success = matches!(&result, Ok(output) if activation_succeeded(output));
        log_output(&on_log, &result);

        if success {
            on_log("--- ATIVAÇÃO CONCLUÍDA COM SUCESSO ---");
            return true;
        }
        on_log(&format!(
            "--- Falha na ativação com {server}. Tentando próximo... ---"
        ));
    }

    on_log("--- FALHA NA ATIVAÇÃO: NENHUM SERVIDOR KMS FUNCIONOU. ---");
    false
}

fn activation_succeeded(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.contains("product activation successful") || lower.contains("ativado com êxito")
}

/// Installs the KMS license `.xrm-ms` files matching `patterns` from
/// whichever of `Licenses16`/`Licenses15` (siblings of `office_path`'s
/// parent `root` folder) exists, via `ospp.vbs /inslic:`. Mirrors legacy
/// `instalarLicencasOffice`; logs and returns without error if no licenses
/// directory is found (non-fatal, matches legacy "Pulando esta etapa").
async fn install_licenses(
    ospp_path: &str,
    office_path: &str,
    patterns: &[String],
    runner: &impl ProcessRunner,
    dir_exists: &impl Fn(&str) -> bool,
    list_dir: &impl Fn(&str) -> Vec<String>,
    on_log: &impl Fn(&str),
) {
    let candidates = [
        format!(r"{office_path}\..\root\Licenses16"),
        format!(r"{office_path}\..\root\Licenses15"),
    ];

    let Some(licenses_dir) = candidates.into_iter().find(|dir| dir_exists(dir)) else {
        on_log("Aviso: Diretório de licenças KMS não encontrado. Pulando esta etapa.");
        return;
    };

    let files = list_dir(&licenses_dir);
    for pattern in patterns {
        let Ok(re) = Regex::new(pattern) else {
            on_log(&format!("Aviso: padrão de licença inválido: {pattern}"));
            continue;
        };
        for file in files.iter().filter(|f| re.is_match(f)) {
            let license_path = format!(r"{licenses_dir}\{file}");
            run_and_log(
                runner,
                on_log,
                &format!("Instalando licença KMS: {file}"),
                "cscript",
                &[ospp_path, &format!("/inslic:{license_path}")],
                None,
            )
            .await;
        }
    }
}

/// Runs one `ospp.vbs`/`taskkill` step, logging its outcome but never
/// aborting the flow on error — matches legacy `runCommandAndLog`
/// semantics where a non-fatal step error ("pode ser normal") is just
/// surfaced as a warning.
async fn run_and_log(
    runner: &impl ProcessRunner,
    on_log: &impl Fn(&str),
    log_msg: &str,
    program: &str,
    args: &[&str],
    cwd: Option<&str>,
) {
    on_log(&format!("--> {log_msg}"));
    let result = runner.run(program, args, cwd).await;
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

    struct FakeProcessRunner {
        calls: Mutex<Vec<(String, Vec<String>, Option<String>)>>,
        succeeds_on_host: Option<String>,
    }

    impl FakeProcessRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                succeeds_on_host: None,
            }
        }

        fn succeeding_on_host(host: &str) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                succeeds_on_host: Some(host.to_string()),
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
            let mut calls = self.calls.lock().unwrap();
            calls.push((
                program.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
                cwd.map(|c| c.to_string()),
            ));

            if args.last() == Some(&"/act") {
                let last_sethst = calls.iter().rev().find_map(|(_, a, _)| {
                    a.iter()
                        .find(|arg| arg.starts_with("/sethst:"))
                        .cloned()
                });
                let target = self
                    .succeeds_on_host
                    .as_ref()
                    .map(|h| format!("/sethst:{h}"));
                return if last_sethst == target {
                    Ok("Product activation successful.".to_string())
                } else {
                    Err("0x8007232B Falha ao ativar.".to_string())
                };
            }
            Ok(String::new())
        }
    }

    struct FakeHealthChecker {
        unreachable_hosts: Vec<String>,
    }

    impl FakeHealthChecker {
        fn all_reachable() -> Self {
            Self {
                unreachable_hosts: Vec::new(),
            }
        }

        fn unreachable(hosts: &[&str]) -> Self {
            Self {
                unreachable_hosts: hosts.iter().map(|h| h.to_string()).collect(),
            }
        }
    }

    impl TcpHealthChecker for FakeHealthChecker {
        async fn is_reachable(&self, host: &str, _port: u16) -> bool {
            !self.unreachable_hosts.iter().any(|h| h == host)
        }
    }

    fn office_2016_config() -> OfficeVersionConfig {
        OfficeVersionConfig {
            prod_key: "XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99".to_string(),
            unpkeys: vec!["BTDRB".to_string(), "KHGM9".to_string(), "CPQVG".to_string()],
            license_patterns: vec![r"proplusvl_kms.*\.xrm-ms".to_string()],
            kms_servers: vec![
                "23.226.136.46".to_string(),
                "107.173.230.24".to_string(),
                "kms8.msguides.com".to_string(),
                "kms9.msguides.com".to_string(),
            ],
        }
    }

    const OFFICE_PATH: &str = r"C:\Program Files\Microsoft Office\Office16";

    #[tokio::test]
    async fn activate_tries_each_kms_server_in_order_until_one_succeeds() {
        let runner = FakeProcessRunner::succeeding_on_host("kms9.msguides.com");
        let health_checker = FakeHealthChecker::all_reachable();
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        let success = activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        assert!(success, "deveria ativar com sucesso no último servidor KMS");

        let sethst_calls: Vec<String> = runner
            .calls_for("cscript")
            .into_iter()
            .filter_map(|args| {
                args.into_iter()
                    .find(|a| a.starts_with("/sethst:"))
            })
            .collect();

        assert_eq!(
            sethst_calls,
            vec![
                "/sethst:23.226.136.46",
                "/sethst:107.173.230.24",
                "/sethst:kms8.msguides.com",
                "/sethst:kms9.msguides.com",
            ],
            "deveria tentar os 4 servidores KMS em ordem até um funcionar"
        );
    }

    #[tokio::test]
    async fn activate_returns_false_when_no_kms_server_succeeds() {
        let runner = FakeProcessRunner::new();
        let health_checker = FakeHealthChecker::all_reachable();
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        let success = activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        assert!(!success);
    }

    #[tokio::test]
    async fn activate_skips_kms_servers_that_fail_the_tcp_health_check() {
        let runner = FakeProcessRunner::succeeding_on_host("kms8.msguides.com");
        let health_checker =
            FakeHealthChecker::unreachable(&["23.226.136.46", "107.173.230.24"]);
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        let success = activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        assert!(
            success,
            "deveria pular os 2 servidores não saudáveis e ativar no terceiro"
        );

        let sethst_calls: Vec<String> = runner
            .calls_for("cscript")
            .into_iter()
            .filter_map(|args| args.into_iter().find(|a| a.starts_with("/sethst:")))
            .collect();

        assert_eq!(
            sethst_calls,
            vec!["/sethst:kms8.msguides.com"],
            "não deveria tentar /sethst para servidores que falharam o health check"
        );
    }

    #[tokio::test]
    async fn activate_closes_office_processes_via_taskkill_before_anything_else() {
        let runner = FakeProcessRunner::succeeding_on_host("23.226.136.46");
        let health_checker = FakeHealthChecker::all_reachable();
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        let taskkill_calls = runner.calls_for("taskkill");
        assert_eq!(
            taskkill_calls,
            vec![vec![
                "/f".to_string(),
                "/im".to_string(),
                "winword.exe".to_string(),
                "/im".to_string(),
                "excel.exe".to_string(),
                "/im".to_string(),
                "powerpnt.exe".to_string(),
                "/im".to_string(),
                "outlook.exe".to_string(),
            ]]
        );
    }

    #[tokio::test]
    async fn activate_removes_existing_product_keys_via_unpkey_before_installing_the_gvlk() {
        let runner = FakeProcessRunner::succeeding_on_host("23.226.136.46");
        let health_checker = FakeHealthChecker::all_reachable();
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        let unpkey_calls: Vec<String> = runner
            .calls_for("cscript")
            .into_iter()
            .filter_map(|args| args.into_iter().find(|a| a.starts_with("/unpkey:")))
            .collect();

        assert_eq!(
            unpkey_calls,
            vec!["/unpkey:BTDRB", "/unpkey:KHGM9", "/unpkey:CPQVG"]
        );
    }

    #[tokio::test]
    async fn activate_installs_the_gvlk_key_via_inpkey() {
        let runner = FakeProcessRunner::succeeding_on_host("23.226.136.46");
        let health_checker = FakeHealthChecker::all_reachable();
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        let inpkey_calls: Vec<String> = runner
            .calls_for("cscript")
            .into_iter()
            .filter_map(|args| args.into_iter().find(|a| a.starts_with("/inpkey:")))
            .collect();

        assert_eq!(inpkey_calls, vec!["/inpkey:XQNVK-8JYDB-WJ9W3-YJ8YR-WFG99"]);
    }

    #[tokio::test]
    async fn activate_runs_the_act_attempt_with_office_path_as_the_working_directory() {
        let runner = FakeProcessRunner::succeeding_on_host("23.226.136.46");
        let health_checker = FakeHealthChecker::all_reachable();
        let mut versions = HashMap::new();
        versions.insert("2016".to_string(), office_2016_config());

        activate(
            "2016",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        let calls = runner.calls.lock().unwrap();
        let act_call = calls
            .iter()
            .find(|(program, args, _)| program == "cscript" && args.last() == Some(&"/act".to_string()))
            .expect("deveria ter tentado /act");

        assert_eq!(act_call.2.as_deref(), Some(OFFICE_PATH));
    }

    #[tokio::test]
    async fn activate_returns_false_for_unknown_office_edition_without_running_any_command() {
        let runner = FakeProcessRunner::new();
        let health_checker = FakeHealthChecker::all_reachable();
        let versions = HashMap::new();

        let success = activate(
            "ultimate",
            OFFICE_PATH,
            &versions,
            &runner,
            &health_checker,
            |_| false,
            |_| Vec::new(),
            |_| {},
        )
        .await;

        assert!(!success);
        assert!(runner.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn find_office_path_returns_the_first_base_path_containing_ospp_vbs() {
        let existing = format!(
            r"{}\Microsoft Office\Office16\ospp.vbs",
            r"C:\Program Files"
        );

        let path = find_office_path(Some(r"C:\Program Files"), Some(r"C:\Program Files (x86)"), |p| {
            p == existing
        })
        .expect("deveria encontrar a pasta do Office");

        assert_eq!(path, r"C:\Program Files\Microsoft Office\Office16");
    }

    #[test]
    fn find_office_path_falls_back_to_office15_when_office16_is_absent() {
        let existing = format!(
            r"{}\Microsoft Office\Office15\ospp.vbs",
            r"C:\Program Files"
        );

        let path = find_office_path(Some(r"C:\Program Files"), None, |p| p == existing)
            .expect("deveria encontrar a pasta do Office15");

        assert_eq!(path, r"C:\Program Files\Microsoft Office\Office15");
    }

    #[test]
    fn find_office_path_falls_back_to_program_files_x86() {
        let existing = format!(
            r"{}\Microsoft Office\Office16\ospp.vbs",
            r"C:\Program Files (x86)"
        );

        let path = find_office_path(Some(r"C:\Program Files"), Some(r"C:\Program Files (x86)"), |p| {
            p == existing
        })
        .expect("deveria encontrar a pasta do Office em Program Files (x86)");

        assert_eq!(path, r"C:\Program Files (x86)\Microsoft Office\Office16");
    }

    #[test]
    fn find_office_path_errors_when_ospp_vbs_is_nowhere_to_be_found() {
        let result = find_office_path(Some(r"C:\Program Files"), Some(r"C:\Program Files (x86)"), |_| {
            false
        });

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn install_licenses_installs_only_files_matching_the_pattern_via_inslic() {
        let runner = FakeProcessRunner::new();
        let licenses_dir = format!(r"{OFFICE_PATH}\..\root\Licenses16");
        let files = vec![
            "proplusvl_kms_2016.xrm-ms".to_string(),
            "proplusvl_kms_2016-ppd.xrm-ms".to_string(),
            "visiovl_kms_2016.xrm-ms".to_string(),
            "readme.txt".to_string(),
        ];

        install_licenses(
            "ospp.vbs",
            OFFICE_PATH,
            &[r"proplusvl_kms.*\.xrm-ms".to_string()],
            &runner,
            &|dir: &str| dir == licenses_dir,
            &|dir: &str| if dir == licenses_dir { files.clone() } else { Vec::new() },
            &|_| {},
        )
        .await;

        let inslic_calls: Vec<String> = runner
            .calls_for("cscript")
            .into_iter()
            .filter_map(|args| args.into_iter().find(|a| a.starts_with("/inslic:")))
            .collect();

        assert_eq!(
            inslic_calls,
            vec![
                format!("/inslic:{licenses_dir}\\proplusvl_kms_2016.xrm-ms"),
                format!("/inslic:{licenses_dir}\\proplusvl_kms_2016-ppd.xrm-ms"),
            ]
        );
    }

    #[tokio::test]
    async fn install_licenses_falls_back_to_licenses15_when_licenses16_is_absent() {
        let runner = FakeProcessRunner::new();
        let licenses15_dir = format!(r"{OFFICE_PATH}\..\root\Licenses15");
        let files = vec!["proplusvl_kms_2013.xrm-ms".to_string()];

        install_licenses(
            "ospp.vbs",
            OFFICE_PATH,
            &[r"proplusvl_kms.*\.xrm-ms".to_string()],
            &runner,
            &|dir: &str| dir == licenses15_dir,
            &|dir: &str| if dir == licenses15_dir { files.clone() } else { Vec::new() },
            &|_| {},
        )
        .await;

        let inslic_calls: Vec<String> = runner
            .calls_for("cscript")
            .into_iter()
            .filter_map(|args| args.into_iter().find(|a| a.starts_with("/inslic:")))
            .collect();

        assert_eq!(
            inslic_calls,
            vec![format!("/inslic:{licenses15_dir}\\proplusvl_kms_2013.xrm-ms")]
        );
    }

    #[tokio::test]
    async fn install_licenses_skips_silently_when_no_licenses_directory_exists() {
        let runner = FakeProcessRunner::new();
        let logs = Mutex::new(Vec::new());

        install_licenses(
            "ospp.vbs",
            OFFICE_PATH,
            &[r"proplusvl_kms.*\.xrm-ms".to_string()],
            &runner,
            &|_| false,
            &|_| Vec::new(),
            &|msg: &str| logs.lock().unwrap().push(msg.to_string()),
        )
        .await;

        assert!(runner.calls.lock().unwrap().is_empty());
        assert!(logs
            .lock()
            .unwrap()
            .iter()
            .any(|m| m.contains("Diretório de licenças KMS não encontrado")));
    }
}
