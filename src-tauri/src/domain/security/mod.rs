use crate::ports::ProcessRunner;

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
    tipo: &'static str,
    data: &'static str,
    log_msg: &'static str,
}

/// Registry changes applied in Etapa 3/4 — mirrors legacy `changes` slice
/// exactly (path/value/type/data/log message). `RequireSecuritySignature`
/// and `limitblankpassworduse` are the two changes the UI must warn about
/// before this function runs (SMB signing requirement removed, blank-
/// password guest logons allowed).
const REG_CHANGES: [RegChange; 5] = [
    RegChange {
        path: r"HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        value: "AllowInsecureGuestAuth",
        tipo: "REG_DWORD",
        data: "1",
        log_msg: "Habilitando logons de convidado não seguros...",
    },
    RegChange {
        path: r"HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        value: "RequireSecuritySignature",
        tipo: "REG_DWORD",
        data: "0",
        log_msg: "Ajustando política de assinatura digital (Require)...",
    },
    RegChange {
        path: r"HKLM\SYSTEM\CurrentControlSet\Control\Print",
        value: "RpcAuthnLevelPrivacyEnabled",
        tipo: "REG_DWORD",
        data: "0",
        log_msg: "Desativando privacidade RPC estrita para impressoras...",
    },
    RegChange {
        path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint",
        value: "RestrictDriverInstallationToAdministrators",
        tipo: "REG_DWORD",
        data: "0",
        log_msg: "Permitindo instalação de drivers de impressão...",
    },
    RegChange {
        path: r"HKLM\SYSTEM\CurrentControlSet\Control\Lsa",
        value: "limitblankpassworduse",
        tipo: "REG_DWORD",
        data: "0",
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
pub async fn fix_network_sharing(runner: &impl ProcessRunner, on_log: impl Fn(&str)) {
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
        run_and_log(
            runner,
            &on_log,
            change.log_msg,
            "reg",
            &[
                "add",
                change.path,
                "/v",
                change.value,
                "/t",
                change.tipo,
                "/d",
                change.data,
                "/f",
            ],
        )
        .await;
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
    use crate::ports::ProcessRunner;
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

    #[tokio::test]
    async fn fix_network_sharing_issues_all_four_steps_in_order() {
        let runner = OrderedFakeProcessRunner::new();
        let ops = runner.ops.clone();

        super::fix_network_sharing(&runner, |_| {}).await;

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

        // Etapa 3/4: all 5 registry changes, exact path/value/type/data.
        assert_eq!(
            recorded[14],
            r"reg add HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters /v AllowInsecureGuestAuth /t REG_DWORD /d 1 /f"
        );
        assert_eq!(
            recorded[15],
            r"reg add HKLM\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters /v RequireSecuritySignature /t REG_DWORD /d 0 /f"
        );
        assert_eq!(
            recorded[16],
            r"reg add HKLM\SYSTEM\CurrentControlSet\Control\Print /v RpcAuthnLevelPrivacyEnabled /t REG_DWORD /d 0 /f"
        );
        assert_eq!(
            recorded[17],
            r"reg add HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Printers\PointAndPrint /v RestrictDriverInstallationToAdministrators /t REG_DWORD /d 0 /f"
        );
        assert_eq!(
            recorded[18],
            r"reg add HKLM\SYSTEM\CurrentControlSet\Control\Lsa /v limitblankpassworduse /t REG_DWORD /d 0 /f"
        );

        // Etapa 4/4: spooler restart then gpupdate /force, last.
        assert_eq!(recorded[19], "net stop spooler");
        assert_eq!(recorded[20], "net start spooler");
        assert_eq!(recorded[21], "gpupdate /force");

        assert_eq!(recorded.len(), 22);
    }

    #[tokio::test]
    async fn fix_network_sharing_continues_past_a_failing_step() {
        // "sc" fails for every service's config call; the rest of the
        // sequence (net start per service, firewall, registry, finalize)
        // must still run to completion — non-fatal-per-step, matching
        // legacy `runCommandAndLog`.
        let runner = OrderedFakeProcessRunner::failing_on("sc");
        let ops = runner.ops.clone();

        super::fix_network_sharing(&runner, |_| {}).await;

        let recorded = ops.lock().unwrap().clone();
        assert_eq!(recorded.len(), 22);
        assert_eq!(recorded[21], "gpupdate /force");
        assert!(recorded.iter().any(|op| op == "net start spooler"));
    }
}
