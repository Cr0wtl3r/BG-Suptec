pub mod firewall;

use crate::ports::{ProcessRunner, RegistryWriter};

/// Network services restarted with auto-start, in order, as part of the
/// network sharing fix — mirrors legacy `CorrigirCompartilhamentoWindows`'s
/// `servicos` slice.
const SERVICOS: [&str; 6] = [
    "LanmanServer",
    "LanmanWorkstation",
    "FDResPub",
    "SSDPSRV",
    "IKEEXT",
    "PolicyAgent",
];

struct RegChange {
    path: &'static str,
    value: &'static str,
    data: u32,
    log_msg: &'static str,
}

/// Registry changes applied in Etapa 3/4 — mirrors legacy `changes` slice
/// exactly (path/value/type/data/log message). All five are `REG_DWORD`
/// writes under `HKEY_LOCAL_MACHINE`, applied via `RegistryWriter` (same
/// port `domain::system::time::adjust_formatting_time` uses for its
/// `InstallDate` write). `RequireSecuritySignature` and
/// `limitblankpassworduse` are the two changes the UI must warn about
/// before this function runs (SMB signing requirement removed, blank-
/// password guest logons allowed).
const REG_CHANGES: [RegChange; 5] = [
    RegChange {
        path: r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        value: "AllowInsecureGuestAuth",
        data: 1,
        log_msg: "Habilitando logons de convidado não seguros...",
    },
    RegChange {
        path: r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        value: "RequireSecuritySignature",
        data: 0,
        log_msg: "Ajustando política de assinatura digital (Require)...",
    },
    RegChange {
        path: r"SYSTEM\CurrentControlSet\Control\Print",
        value: "RpcAuthnLevelPrivacyEnabled",
        data: 0,
        log_msg: "Desativando privacidade RPC estrita para impressoras...",
    },
    RegChange {
        path: r"SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint",
        value: "RestrictDriverInstallationToAdministrators",
        data: 0,
        log_msg: "Permitindo instalação de drivers de impressão...",
    },
    RegChange {
        path: r"SYSTEM\CurrentControlSet\Control\Lsa",
        value: "limitblankpassworduse",
        data: 0,
        log_msg: "Habilitando acesso para usuários com senha em branco...",
    },
];

/// Applies a full set of fixes for common local-network folder/printer
/// sharing problems — mirrors legacy `CorrigirCompartilhamentoWindows`
/// (`app.go` lines 363-404) in four steps: (1) network services set to
/// auto-start and started, (2) firewall rule groups enabled, (3) registry
/// changes that relax SMB/print security to restore compatibility, (4)
/// Print Spooler restart + `gpupdate /force`. Every step is non-fatal —a
/// failure is logged as a warning but doesn't stop the rest of the
/// sequence, matching legacy `runCommandAndLog` semantics (same pattern as
/// `domain::system::time::adjust_formatting_time`). The registry changes
/// in step 3 reduce SMB security (disable signing requirement, allow
/// blank-password guest logons) — callers must warn the user before
/// invoking this function; the warning isn't enforced here since the
/// domain layer stays UI-agnostic.
pub async fn fix_network_sharing(
    runner: &impl ProcessRunner,
    registry: &impl RegistryWriter,
    on_log: impl Fn(&str),
) {
    on_log("INICIANDO CORREÇÃO DE COMPARTILHAMENTO DE REDE...");

    on_log("\n--> Etapa 1/4: Configurando Serviços de Rede...");
    for servico in SERVICOS {
        run_and_log(
            runner,
            &on_log,
            &format!("Configurando serviço: {servico}"),
            "sc",
            &["config", servico, "start=auto"],
        )
        .await;
        run_and_log(runner, &on_log, "", "net", &["start", servico]).await;
    }

    on_log("\n--> Etapa 2/4: Configurando Regras de Firewall do Windows...");
    run_and_log(
        runner,
        &on_log,
        "Habilitando grupo 'Compartilhamento de Arquivos e Impressoras'...",
        "netsh",
        &[
            "advfirewall",
            "firewall",
            "set",
            "rule",
            r#"group="File and Printer Sharing""#,
            "new",
            "enable=Yes",
        ],
    )
    .await;
    run_and_log(
        runner,
        &on_log,
        "Habilitando grupo 'Remote Service Management'...",
        "netsh",
        &[
            "advfirewall",
            "firewall",
            "set",
            "rule",
            r#"group="Remote Service Management""#,
            "new",
            "enable=yes",
        ],
    )
    .await;

    on_log("\n--> Etapa 3/4: Aplicando Configurações no Registro do Windows...");
    for change in REG_CHANGES {
        on_log(&format!("--> {}", change.log_msg));
        if let Err(e) = registry.write_local_machine_dword(change.path, change.value, change.data)
        {
            on_log(&format!(
                "AVISO: Comando encontrou um erro (pode ser normal): {e}"
            ));
        }
    }

    on_log("\n--> Etapa 4/4: Finalizando e Aplicando Políticas...");
    run_and_log(
        runner,
        &on_log,
        "Reiniciando Spooler de Impressão...",
        "net",
        &["stop", "spooler"],
    )
    .await;
    run_and_log(runner, &on_log, "", "net", &["start", "spooler"]).await;
    run_and_log(
        runner,
        &on_log,
        "Forçando atualização das políticas de grupo...",
        "gpupdate",
        &["/force"],
    )
    .await;

    on_log("\n--- OPERAÇÃO CONCLUÍDA ---");
    on_log("É altamente recomendável reiniciar o computador.");
}

/// Enables System Restore on drive `C:` and caps its shadow-storage usage
/// at 5% — this feature does not exist in the legacy Go app (the Svelte UI
/// called a Wails-bound `AtivarProtecaoSistema()` function that was never
/// implemented, and listened for a `"log:ativar:protecao"` event nothing
/// ever emitted); this is a first implementation based on the legacy
/// component's description text, not a port of working legacy logic.
/// `Enable-ComputerRestore` is a PowerShell cmdlet (no standalone .exe), so
/// it's invoked via `ProcessRunner` running `powershell` directly with
/// literal argv — same approach as `domain::system::time::adjust_formatting_time`
/// uses for `w32tm` — rather than `adapters::powershell::run_script`, which
/// hardcodes the real `WinProcessRunner` and isn't mockable. Restore must
/// be enabled before shadow storage can be sized, so step order matters.
/// Each step is non-fatal, matching every other slice's
/// `run_and_log` semantics.
pub async fn enable_system_protection(runner: &impl ProcessRunner, on_log: impl Fn(&str)) {
    on_log("Iniciando ativação da proteção do sistema...");

    run_and_log(
        runner,
        &on_log,
        "Habilitando Restauração do Sistema para a unidade C:...",
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Enable-ComputerRestore -Drive 'C:'",
        ],
    )
    .await;

    run_and_log(
        runner,
        &on_log,
        "Configurando uso de disco para 5%...",
        "vssadmin",
        &[
            "resize",
            "shadowstorage",
            "/for=C:",
            "/on=C:",
            "/maxsize=5%",
        ],
    )
    .await;

    on_log("--- PROTEÇÃO DO SISTEMA ATIVADA ---");
}

/// Runs one step, logging its outcome but never aborting the flow on
/// error — matches legacy `runCommandAndLog` semantics. Same shape as
/// `domain::system::time::run_and_log`.
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

    /// Records every call's program+args (joined) in a single shared,
    /// ordered log via `Arc<Mutex<..>>` so tests can assert exact
    /// sequencing across steps — same pattern as
    /// `domain::maintenance::tests::OrderedFakeProcessRunner`.
    struct OrderedFakeProcessRunner {
        ops: std::sync::Arc<Mutex<Vec<String>>>,
        fails_program: Option<&'static str>,
    }

    impl OrderedFakeProcessRunner {
        fn new() -> Self {
            Self {
                ops: std::sync::Arc::new(Mutex::new(Vec::new())),
                fails_program: None,
            }
        }

        fn failing_on(program: &'static str) -> Self {
            Self {
                ops: std::sync::Arc::new(Mutex::new(Vec::new())),
                fails_program: Some(program),
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
            self.ops
                .lock()
                .unwrap()
                .push(format!("{program} {}", args.join(" ")));
            if self.fails_program == Some(program) {
                return Err("comando falhou".to_string());
            }
            Ok(String::new())
        }
    }

    /// Records every `write_local_machine_dword` call (path, name, value)
    /// so tests can assert the 5 registry changes — same pattern as
    /// `domain::system::time::tests::FakeRegistryWriter`.
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
    async fn fix_network_sharing_issues_all_four_steps_in_order() {
        let runner = OrderedFakeProcessRunner::new();
        let ops = runner.ops.clone();
        let registry = FakeRegistryWriter::new();

        super::fix_network_sharing(&runner, &registry, |_| {}).await;

        let recorded = ops.lock().unwrap().clone();

        // Etapa 1/4: each of the 6 services gets `sc config ... start=auto`
        // then `net start <service>`, in order, before firewall rules.
        let expected_services: Vec<String> = [
            "LanmanServer",
            "LanmanWorkstation",
            "FDResPub",
            "SSDPSRV",
            "IKEEXT",
            "PolicyAgent",
        ]
        .iter()
        .flat_map(|s| {
            vec![
                format!("sc config {s} start=auto"),
                format!("net start {s}"),
            ]
        })
        .collect();
        assert_eq!(&recorded[0..12], expected_services.as_slice());

        // Etapa 2/4: firewall rule groups, exact group names/flags.
        assert_eq!(
            recorded[12],
            r#"netsh advfirewall firewall set rule group="File and Printer Sharing" new enable=Yes"#
        );
        assert_eq!(
            recorded[13],
            r#"netsh advfirewall firewall set rule group="Remote Service Management" new enable=yes"#
        );

        // Etapa 3/4: all 5 registry changes, written via RegistryWriter
        // (not ProcessRunner) — exact path/value/data, in order.
        assert_eq!(
            registry.writes.lock().unwrap().clone(),
            vec![
                (
                    r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters"
                        .to_string(),
                    "AllowInsecureGuestAuth".to_string(),
                    1,
                ),
                (
                    r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters"
                        .to_string(),
                    "RequireSecuritySignature".to_string(),
                    0,
                ),
                (
                    r"SYSTEM\CurrentControlSet\Control\Print".to_string(),
                    "RpcAuthnLevelPrivacyEnabled".to_string(),
                    0,
                ),
                (
                    r"SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint"
                        .to_string(),
                    "RestrictDriverInstallationToAdministrators".to_string(),
                    0,
                ),
                (
                    r"SYSTEM\CurrentControlSet\Control\Lsa".to_string(),
                    "limitblankpassworduse".to_string(),
                    0,
                ),
            ]
        );

        // Etapa 4/4: spooler restart then gpupdate /force, last.
        assert_eq!(recorded[14], "net stop spooler");
        assert_eq!(recorded[15], "net start spooler");
        assert_eq!(recorded[16], "gpupdate /force");

        assert_eq!(recorded.len(), 17);
    }

    #[tokio::test]
    async fn fix_network_sharing_continues_past_a_failing_step() {
        // "sc" fails for every service's config call; the rest of the
        // sequence (net start per service, firewall, registry, finalize)
        // must still run to completion — non-fatal-per-step, matching
        // legacy `runCommandAndLog`.
        let runner = OrderedFakeProcessRunner::failing_on("sc");
        let ops = runner.ops.clone();
        let registry = FakeRegistryWriter::new();

        super::fix_network_sharing(&runner, &registry, |_| {}).await;

        let recorded = ops.lock().unwrap().clone();
        assert_eq!(recorded.len(), 17);
        assert_eq!(recorded[16], "gpupdate /force");
        assert!(recorded.iter().any(|op| op == "net start spooler"));
        assert_eq!(registry.writes.lock().unwrap().len(), 5);
    }

    #[tokio::test]
    async fn enable_system_protection_calls_enable_computer_restore_then_vssadmin() {
        let runner = OrderedFakeProcessRunner::new();
        let ops = runner.ops.clone();

        super::enable_system_protection(&runner, |_| {}).await;

        let recorded = ops.lock().unwrap().clone();
        assert_eq!(
            recorded,
            vec![
                "powershell -NoProfile -NonInteractive -Command Enable-ComputerRestore -Drive 'C:'"
                    .to_string(),
                "vssadmin resize shadowstorage /for=C: /on=C: /maxsize=5%".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn enable_system_protection_continues_past_a_failing_restore_step() {
        // Even if `Enable-ComputerRestore` fails, the `vssadmin` step must
        // still run — non-fatal-per-step, matching every other slice.
        let runner = OrderedFakeProcessRunner::failing_on("powershell");
        let ops = runner.ops.clone();

        super::enable_system_protection(&runner, |_| {}).await;

        let recorded = ops.lock().unwrap().clone();
        assert_eq!(recorded.len(), 2);
        assert!(recorded[1].starts_with("vssadmin"));
    }
}
