use crate::ports::{ProcessRunner, RegistryWriter};

pub async fn fix_start_menu_search(
    runner: &impl ProcessRunner,
    registry: &impl RegistryWriter,
    on_log: impl Fn(&str),
) {
    on_log("Corrigindo indexação/busca do menu iniciar...");
    if let Err(e) = registry.write_current_user_dword(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
        "EnableXamlStartMenu",
        0,
    ) {
        on_log(&format!("AVISO: {e}"));
    }
    run_best_effort(runner, &on_log, "taskkill", &["/f", "/im", "explorer.exe"]).await;
    run_best_effort(runner, &on_log, "explorer.exe", &[]).await;
}

pub async fn run_dism_restore_health(
    runner: &impl ProcessRunner,
    on_log: impl Fn(&str),
) -> Result<String, String> {
    on_log("Executando DISM RestoreHealth...");
    runner
        .run(
            "dism",
            &["/English", "/Online", "/Cleanup-Image", "/RestoreHealth"],
            None,
        )
        .await
}

pub async fn run_sfc_scannow(
    runner: &impl ProcessRunner,
    on_log: impl Fn(&str),
) -> Result<String, String> {
    on_log("Executando SFC /scannow...");
    runner.run("sfc", &["/scannow"], None).await
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
    use crate::ports::{ProcessRunner, RegistryWriter};
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

    #[derive(Default)]
    struct FakeRegistry {
        dwords: Mutex<Vec<(String, String, u32)>>,
    }

    impl RegistryWriter for FakeRegistry {
        fn write_local_machine_dword(
            &self,
            path: &str,
            name: &str,
            value: u32,
        ) -> Result<(), String> {
            self.dwords
                .lock()
                .unwrap()
                .push((path.to_string(), name.to_string(), value));
            Ok(())
        }

        fn write_classes_root_string(
            &self,
            _path: &str,
            _name: &str,
            _value: &str,
        ) -> Result<(), String> {
            Ok(())
        }

        fn write_local_machine_string(
            &self,
            _path: &str,
            _name: &str,
            _value: &str,
        ) -> Result<(), String> {
            Ok(())
        }

        fn write_current_user_dword(
            &self,
            path: &str,
            name: &str,
            value: u32,
        ) -> Result<(), String> {
            self.dwords
                .lock()
                .unwrap()
                .push((path.to_string(), name.to_string(), value));
            Ok(())
        }
    }

    #[tokio::test]
    async fn fix_start_menu_search_sets_registry_and_restarts_explorer() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };
        let registry = FakeRegistry::default();

        fix_start_menu_search(&runner, &registry, |_| {}).await;

        assert!(registry.dwords.lock().unwrap().contains(&(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced".to_string(),
            "EnableXamlStartMenu".to_string(),
            0
        )));
        assert_eq!(
            calls.lock().unwrap().clone(),
            vec![
                "taskkill /f /im explorer.exe".to_string(),
                "explorer.exe ".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn dism_restore_health_uses_official_dism_command() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };

        run_dism_restore_health(&runner, |_| {}).await.unwrap();

        assert_eq!(
            calls.lock().unwrap().clone(),
            vec!["dism /English /Online /Cleanup-Image /RestoreHealth".to_string()]
        );
    }
}
