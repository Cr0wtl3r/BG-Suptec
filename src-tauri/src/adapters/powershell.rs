use super::process;

const BASE_ARGS: [&str; 3] = ["-NoProfile", "-NonInteractive", "-Command"];

/// Runs a static PowerShell script that consumes untrusted values via
/// `$env:NAME`. Note: `powershell -Command <script> <more args>` does
/// *not* bind trailing arguments as `$args`/parameters — it concatenates
/// everything after `-Command` into one source string and re-parses it as
/// a script, so untrusted text placed there can break out with `;` or
/// `&&` just like the legacy shell-interpolation bug this rewrite exists
/// to fix. Environment variables don't have this problem: they travel
/// through the OS process environment block, a channel entirely separate
/// from command-line text, so a value can never be reinterpreted as
/// script syntax — the script only ever sees it as the string value of
/// `$env:NAME`.
pub async fn run_script_with_env(
    script: &str,
    env_vars: &[(&str, &str)],
) -> Result<String, String> {
    let mut args: Vec<&str> = BASE_ARGS.to_vec();
    args.push(script);
    process::run_with_env("powershell", &args, env_vars).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_script_with_env_returns_stdout_of_a_simple_script() {
        let result = run_script_with_env("Write-Output 'ola'", &[])
            .await
            .unwrap();
        assert_eq!(result, "ola");
    }

    #[tokio::test]
    async fn run_script_with_env_returns_err_on_powershell_error() {
        let result = run_script_with_env("throw 'falha proposital'", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_script_with_env_exposes_value_as_a_literal_string_not_executable_code() {
        let script = "Write-Output $env:BG_NOME";
        let malicious = "abc; Write-Output INJETADO";

        let result = run_script_with_env(script, &[("BG_NOME", malicious)])
            .await
            .unwrap();

        // The whole malicious string must come back as one literal value —
        // proof that `;` did not terminate the statement and start a new one.
        assert_eq!(result, malicious);
    }
}
