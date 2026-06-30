use tokio::process::Command;

async fn execute(mut command: Command) -> Result<String, String> {
    let program = format!("{:?}", command.as_std().get_program());

    let output = command
        .output()
        .await
        .map_err(|e| format!("falha ao executar {program}: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!(
            "{program} retornou código {}: {detail}",
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Runs an external program directly (no intermediate `cmd /c` or shell),
/// passing arguments as a literal argv array. Because each argument is a
/// separate OS-level string and nothing is concatenated into a command
/// line that gets re-parsed, no argument value can break out into a
/// second command — this is what makes the call injection-safe.
pub async fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args);
    execute(command).await
}

/// Like `run`, but also sets process environment variables. Untrusted
/// values that a script needs to consume (e.g. as `$env:NAME` in
/// PowerShell) should be passed this way rather than appended to the
/// command line — env vars travel through the OS process environment
/// block, a channel completely separate from command-line text, so they
/// can never be re-parsed as command/script syntax.
pub async fn run_with_env(
    program: &str,
    args: &[&str],
    env_vars: &[(&str, &str)],
) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args);
    for (key, value) in env_vars {
        command.env(key, value);
    }
    execute(command).await
}

/// Like `run`, but executes in `cwd` instead of the current process
/// directory. Some Office Software Protection Platform (`ospp.vbs`)
/// operations only resolve correctly when run from the Office install
/// directory — see `WinProcessRunner`.
pub async fn run_in_dir(program: &str, args: &[&str], cwd: &str) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args);
    command.current_dir(cwd);
    execute(command).await
}

/// Runs arbitrary external programs (no shell intermediary) for domain
/// orchestration code that depends on `ports::ProcessRunner` — e.g.
/// `taskkill` and `ospp.vbs` calls in Office activation.
pub struct WinProcessRunner;

impl crate::ports::ProcessRunner for WinProcessRunner {
    async fn run(&self, program: &str, args: &[&str], cwd: Option<&str>) -> Result<String, String> {
        match cwd {
            Some(dir) => run_in_dir(program, args, dir).await,
            None => run(program, args).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_captures_stdout_of_a_simple_command() {
        let result = run("cmd", &["/c", "echo", "ola"]).await.unwrap();
        assert_eq!(result, "ola");
    }

    #[tokio::test]
    async fn run_treats_a_malicious_argument_as_literal_text_not_a_new_command() {
        // If this argument were concatenated into a shell command string
        // (the legacy vulnerable pattern) instead of passed as an isolated
        // argv element, `&&` would chain a second statement (`exit 7`) and
        // the whole invocation would fail with that exit code. Properly
        // isolated, it's just literal text for `echo` and the call succeeds.
        let malicious = "ola && exit 7";
        let result = run("cmd", &["/c", "echo", malicious]).await;

        let output = result.expect(
            "comando não deveria falhar — argumento malicioso não deve ser interpretado como um segundo comando",
        );
        assert!(output.contains("exit 7"));
    }

    #[tokio::test]
    async fn run_returns_err_for_nonzero_exit_code() {
        let result = run("cmd", &["/c", "exit", "1"]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_returns_err_when_program_does_not_exist() {
        let result = run("programa-que-nao-existe-bg-suptec", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_with_env_exposes_the_variable_to_the_child_process() {
        let result = run_with_env("cmd", &["/c", "echo", "%BG_TEST_VAR%"], &[("BG_TEST_VAR", "ola")])
            .await
            .unwrap();
        assert_eq!(result, "ola");
    }

    #[tokio::test]
    async fn run_in_dir_executes_with_the_given_working_directory() {
        let cwd = std::env::temp_dir()
            .to_str()
            .unwrap()
            .trim_end_matches('\\')
            .to_string();

        let result = run_in_dir("cmd", &["/c", "cd"], &cwd).await.unwrap();

        assert_eq!(result.trim_end_matches('\\'), cwd);
    }
}
