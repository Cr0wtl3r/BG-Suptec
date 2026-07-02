use crate::ports::ProcessRunner;

pub async fn reset_anydesk(
    runner: &impl ProcessRunner,
    all_users_profile: &str,
    roaming_app_data: &str,
    mut delete_path: impl FnMut(&str) -> Result<(), String>,
    anydesk_id_ready: impl Fn() -> bool,
    on_log: impl Fn(&str),
) -> Result<(), String> {
    on_log("Parando AnyDesk...");
    run_best_effort(runner, &on_log, "sc", &["stop", "AnyDesk"]).await;
    run_best_effort(runner, &on_log, "taskkill", &["/f", "/im", "AnyDesk.exe"]).await;

    let system_conf = format!(r"{all_users_profile}\AnyDesk\service.conf");
    let user_service_conf = format!(r"{roaming_app_data}\AnyDesk\service.conf");
    delete_path(&system_conf)?;
    delete_path(&user_service_conf)?;

    on_log("Iniciando AnyDesk...");
    run_best_effort(runner, &on_log, "sc", &["start", "AnyDesk"]).await;
    if !anydesk_id_ready() {
        on_log("AVISO: ID do AnyDesk ainda não foi detectado após reinício do serviço.");
    }

    Ok(())
}

async fn run_best_effort(
    runner: &impl ProcessRunner,
    on_log: &impl Fn(&str),
    program: &str,
    args: &[&str],
) {
    match runner.run(program, args, None).await {
        Ok(output) => {
            let output = output.trim();
            if !output.is_empty() {
                on_log(output);
            }
        }
        Err(e) => on_log(&format!("AVISO: {program} falhou: {e}")),
    }
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
    async fn reset_anydesk_stops_deletes_and_restarts_service() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };
        let mut deleted = Vec::new();

        reset_anydesk(
            &runner,
            r"C:\ProgramData",
            r"C:\Users\Tecnico\AppData\Roaming",
            |path| {
                deleted.push(path.to_string());
                Ok(())
            },
            || true,
            |_| {},
        )
        .await
        .unwrap();

        assert!(calls
            .lock()
            .unwrap()
            .contains(&"sc stop AnyDesk".to_string()));
        assert!(deleted.contains(&r"C:\ProgramData\AnyDesk\service.conf".to_string()));
        assert!(calls
            .lock()
            .unwrap()
            .contains(&"sc start AnyDesk".to_string()));
    }
}
