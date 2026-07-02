use crate::ports::ProcessRunner;

const BROWSERS: [&str; 6] = [
    "ccleaner64.exe",
    "ccleaner.exe",
    "msedge.exe",
    "firefox.exe",
    "vivaldi.exe",
    "brave.exe",
];

pub async fn clean_temp_files(
    runner: &impl ProcessRunner,
    user_profile: &str,
    windows_dir: &str,
    mut delete_target: impl FnMut(&str) -> Result<(), String>,
    on_log: impl Fn(&str),
) -> Result<usize, String> {
    for process in BROWSERS {
        run_best_effort(runner, &on_log, "taskkill", &["/F", "/IM", process]).await;
    }

    let targets = cleanup_targets(user_profile, windows_dir);
    let mut accepted = 0usize;
    for target in targets {
        validate_cleanup_target(user_profile, windows_dir, &target)?;
        on_log(&format!("Limpando {target}..."));
        delete_target(&target)?;
        accepted += 1;
    }
    Ok(accepted)
}

pub async fn clean_full_pc(
    runner: &impl ProcessRunner,
    user_profile: &str,
    windows_dir: &str,
    delete_shadow_copies: bool,
    delete_target: impl FnMut(&str) -> Result<(), String>,
    on_log: impl Fn(&str),
) -> Result<usize, String> {
    let cleaned =
        clean_temp_files(runner, user_profile, windows_dir, delete_target, &on_log).await?;
    run_best_effort(runner, &on_log, "cleanmgr", &["/sagerun:1"]).await;
    run_best_effort(runner, &on_log, "cleanmgr", &["/sagerun:65535"]).await;
    if delete_shadow_copies {
        run_best_effort(
            runner,
            &on_log,
            "vssadmin",
            &["delete", "shadows", "/all", "/quiet"],
        )
        .await;
    }
    Ok(cleaned)
}

pub fn cleanup_digisat_mongo_logs(
    log_dir: &str,
    files: &[String],
    today: &str,
    mut delete_file: impl FnMut(&str) -> Result<(), String>,
    on_log: impl Fn(&str),
) -> Result<usize, String> {
    let Some(latest) = latest_mongo_log(files) else {
        on_log("Nenhum log MongoDB DigiSat encontrado para limpeza.");
        return Ok(0);
    };

    let mut removed = 0usize;
    for file in files {
        let Some(date) = mongo_log_date(file) else {
            continue;
        };
        if file == latest || date == today {
            continue;
        }

        let full_path = format!(r"{log_dir}\{file}");
        on_log(&format!("Removendo log MongoDB antigo: {file}"));
        delete_file(&full_path)?;
        removed += 1;
    }

    Ok(removed)
}

pub fn validate_cleanup_target(
    user_profile: &str,
    windows_dir: &str,
    target: &str,
) -> Result<(), String> {
    let target = normalize(target);
    let user_profile = normalize(user_profile);
    let windows_dir = normalize(windows_dir);
    let allowed_prefixes = [
        format!(r"{user_profile}\appdata\local\temp"),
        format!(r"{user_profile}\appdata\local\microsoft"),
        format!(r"{user_profile}\appdata\local\mozilla"),
        format!(r"{user_profile}\appdata\local\vivaldi"),
        format!(r"{user_profile}\appdata\local\bravesoftware"),
        format!(r"{user_profile}\appdata\local\google"),
        format!(r"{windows_dir}\temp"),
        format!(r"{windows_dir}\logs"),
        format!(r"{windows_dir}\microsoft.net"),
        format!(r"{windows_dir}\softwaredistribution"),
        format!(r"{windows_dir}\panther"),
        format!(r"{windows_dir}\inf"),
    ];
    if allowed_prefixes
        .iter()
        .any(|prefix| target == *prefix || target.starts_with(&format!("{prefix}\\")))
    {
        Ok(())
    } else {
        Err(format!(
            "alvo de limpeza fora das raízes permitidas: {target}"
        ))
    }
}

fn cleanup_targets(user_profile: &str, windows_dir: &str) -> Vec<String> {
    let mut targets = vec![
        format!(r"{user_profile}\AppData\Local\Temp"),
        format!(r"{windows_dir}\Temp"),
        format!(r"{windows_dir}\Logs\CBS"),
        format!(r"{windows_dir}\Logs\MoSetup"),
        format!(r"{windows_dir}\Logs"),
        format!(r"{windows_dir}\Panther"),
        format!(r"{windows_dir}\inf"),
        format!(r"{windows_dir}\SoftwareDistribution\DataStore\Logs"),
        format!(r"{windows_dir}\Microsoft.NET"),
        format!(r"{user_profile}\AppData\Local\Microsoft\Windows\WebCache"),
        format!(r"{user_profile}\AppData\Local\Microsoft\Windows\SettingSync"),
        format!(r"{user_profile}\AppData\Local\Microsoft\Windows\Explorer\ThumbCacheToDelete"),
        format!(r"{user_profile}\AppData\Local\Microsoft\Terminal Server Client\Cache"),
        format!(r"{user_profile}\AppData\Local\Microsoft\Windows\INetCache"),
        format!(r"{user_profile}\AppData\Local\Mozilla\Firefox\Profiles"),
    ];

    for root in [
        format!(r"{user_profile}\AppData\Local\Microsoft\Edge\User Data"),
        format!(r"{user_profile}\AppData\Local\Vivaldi\User Data"),
        format!(r"{user_profile}\AppData\Local\BraveSoftware\Brave-Browser\User Data"),
        format!(r"{user_profile}\AppData\Local\Google\Chrome\User Data"),
    ] {
        targets.push(format!(r"{root}\GrShaderCache\GPUCache"));
        targets.push(format!(r"{root}\ShaderCache\GPUCache"));
        for profile in ["Default", "Profile 1", "Profile 2"] {
            targets.push(format!(r"{root}\{profile}\Cache"));
            targets.push(format!(r"{root}\{profile}\Service Worker\Database"));
            targets.push(format!(r"{root}\{profile}\Service Worker\CacheStorage"));
            targets.push(format!(r"{root}\{profile}\Service Worker\ScriptCache"));
            targets.push(format!(r"{root}\{profile}\GPUCache"));
            targets.push(format!(r"{root}\{profile}\Storage\ext"));
        }
    }

    targets
}

fn latest_mongo_log(files: &[String]) -> Option<&String> {
    files
        .iter()
        .filter(|file| mongo_log_date(file).is_some())
        .max_by_key(|file| mongo_log_date(file).unwrap_or_default())
}

fn mongo_log_date(file: &str) -> Option<&str> {
    file.strip_prefix("mongo.log.")
        .and_then(|rest| rest.get(0..10))
        .filter(|date| {
            date.len() == 10
                && date.chars().enumerate().all(|(idx, ch)| {
                    matches!(idx, 4 | 7) && ch == '-'
                        || !matches!(idx, 4 | 7) && ch.is_ascii_digit()
                })
        })
}

fn normalize(path: &str) -> String {
    path.trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase()
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
    async fn clean_temp_runs_browser_shutdowns_and_known_cleanup_targets() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };
        let mut targets = Vec::new();

        let result = clean_temp_files(
            &runner,
            r"C:\Users\Tecnico",
            r"C:\Windows",
            |target| {
                targets.push(target.to_string());
                Ok(())
            },
            |_| {},
        )
        .await;

        assert!(result.is_ok());
        assert!(calls
            .lock()
            .unwrap()
            .contains(&"taskkill /F /IM msedge.exe".to_string()));
        assert!(targets.contains(&r"C:\Users\Tecnico\AppData\Local\Temp".to_string()));
        assert!(targets.contains(&r"C:\Windows\Temp".to_string()));
        assert!(targets.contains(
            &r"C:\Users\Tecnico\AppData\Local\Google\Chrome\User Data\Default\Cache".to_string()
        ));
        assert!(!targets
            .contains(&r"C:\Users\Tecnico\AppData\Local\Google\Chrome\User Data".to_string()));
    }

    #[tokio::test]
    async fn cleanup_plan_rejects_paths_outside_allowed_roots() {
        let rejected = validate_cleanup_target(r"C:\Users\Tecnico", r"C:\Windows", r"C:\Dados");

        assert!(rejected.is_err());
    }

    #[test]
    fn digisat_mongo_cleanup_keeps_latest_and_today_logs() {
        let files = vec![
            "mongo.log.2026-06-29T00-00-00".to_string(),
            "mongo.log.2026-07-02T08-00-00".to_string(),
            "mongo.log.2026-07-01T08-00-00".to_string(),
            "outro.log".to_string(),
        ];
        let mut deleted = Vec::new();

        let removed = cleanup_digisat_mongo_logs(
            r"C:\DigiSat\SuiteG6\MongoDB\log",
            &files,
            "2026-07-01",
            |path| {
                deleted.push(path.to_string());
                Ok(())
            },
            |_| {},
        )
        .unwrap();

        assert_eq!(removed, 1);
        assert_eq!(
            deleted,
            vec![r"C:\DigiSat\SuiteG6\MongoDB\log\mongo.log.2026-06-29T00-00-00".to_string()]
        );
    }
}
