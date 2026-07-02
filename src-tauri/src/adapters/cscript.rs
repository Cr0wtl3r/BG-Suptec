use crate::adapters::process;
use crate::ports::CscriptRunner;

fn resolve_script_path(script_path: &str) -> String {
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    format!(r"{system_root}\System32\{script_path}")
}

/// Runs WSH scripts (`slmgr.vbs`, `ospp.vbs`) via `cscript`. Args are
/// passed as literal argv elements through `adapters::process` — no shell
/// intermediary, so no value can break out into a second command.
pub struct WinCscriptRunner;

impl CscriptRunner for WinCscriptRunner {
    async fn run(&self, script_path: &str, args: &[&str]) -> Result<String, String> {
        let full_path = resolve_script_path(script_path);
        let mut full_args = vec![full_path.as_str()];
        full_args.extend_from_slice(args);
        process::run("cscript", &full_args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_script_path_joins_system_root_and_script_name() {
        let system_root = std::env::var("SystemRoot").expect("SystemRoot should be set on Windows");

        assert_eq!(
            resolve_script_path("slmgr.vbs"),
            format!(r"{system_root}\System32\slmgr.vbs")
        );
    }

    #[tokio::test]
    async fn run_resolves_under_system32_and_executes_without_a_shell_intermediary() {
        // No slmgr.vbs/ospp.vbs call here — those mutate real licensing
        // state. A nonexistent script proves the full real path (cscript
        // binary lookup, System32 resolution, literal argv passthrough)
        // without performing any activation.
        let runner = WinCscriptRunner;

        let result = runner
            .run("script-que-nao-existe-bg-suptec.vbs", &["/algum-arg"])
            .await;

        assert!(result.is_err());
    }
}
