use serde::Serialize;

use crate::ports::ProcessRunner;

/// One selectable keyboard layout — mirrors legacy `app.go`'s
/// `TecladoInfo`/`tecladosDisponiveis`. Multiple entries can share the same
/// `tag_idioma` (e.g. the two Brazilian ABNT/ABNT2 variants both apply via
/// `pt-BR`): `id` identifies the exact Windows input method, `tag_idioma` is
/// the BCP-47 language tag handed to `Set-WinUserLanguageList`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KeyboardLayout {
    pub id: &'static str,
    pub nome: &'static str,
    #[serde(rename = "tagIdioma")]
    pub tag_idioma: &'static str,
}

const KEYBOARD_LAYOUTS: [KeyboardLayout; 7] = [
    KeyboardLayout {
        id: "0416:00000416",
        nome: "Português (Brasil ABNT)",
        tag_idioma: "pt-BR",
    },
    KeyboardLayout {
        id: "0416:00010416",
        nome: "Português (Brasil ABNT2)",
        tag_idioma: "pt-BR",
    },
    KeyboardLayout {
        id: "0816:00000816",
        nome: "Português (Portugal)",
        tag_idioma: "pt-PT",
    },
    KeyboardLayout {
        id: "0409:00000409",
        nome: "Inglês (Estados Unidos)",
        tag_idioma: "en-US",
    },
    KeyboardLayout {
        id: "0409:00020409",
        nome: "Inglês (Estados Unidos-Internacional)",
        tag_idioma: "en-US",
    },
    KeyboardLayout {
        id: "0c0a:0000040a",
        nome: "Espanhol (Espanha - Internacional)",
        tag_idioma: "es-ES",
    },
    KeyboardLayout {
        id: "080a:0000080a",
        nome: "Espanhol (México/América Latina)",
        tag_idioma: "es-419",
    },
];

/// Returns every selectable layout, sorted by display name — mirrors legacy
/// `ObterLayoutsDisponiveis`'s `sort.Slice` by `Nome`.
pub fn get_available_layouts() -> Vec<KeyboardLayout> {
    let mut layouts = KEYBOARD_LAYOUTS.to_vec();
    layouts.sort_by(|a, b| a.nome.cmp(b.nome));
    layouts
}

/// Validates a language tag against the strict allowlist of tags actually
/// used by `KEYBOARD_LAYOUTS` — anything else (including shell-metacharacter
/// payloads like `"pt-BR; Start-Process cmd"`) is rejected before it can
/// ever reach a command line. Returns the matching `'static` allowlisted
/// string (never the caller's own buffer), so a caller that only ever
/// builds a command from this return value can't end up interpolating
/// unvalidated text.
pub fn validate_language_tag(tag: &str) -> Result<&'static str, String> {
    KEYBOARD_LAYOUTS
        .iter()
        .map(|l| l.tag_idioma)
        .find(|&allowed| allowed == tag)
        .ok_or_else(|| format!("tag de idioma não suportada: {tag}"))
}

/// Applies a keyboard layout system-wide via
/// `Set-WinUserLanguageList -LanguageList <tag> -Force` — mirrors legacy
/// `AlterarLayoutDeTeclado`. Unlike the legacy Go version (which built the
/// PowerShell command with unsanitized `fmt.Sprintf` interpolation), `tag`
/// is validated against the allowlist first; only a known-safe constant
/// ever reaches the command string, so injection is impossible regardless
/// of what the caller passes in.
pub async fn change_keyboard_layout(
    tag: &str,
    runner: &impl ProcessRunner,
) -> Result<String, String> {
    let validated = validate_language_tag(tag)?;
    let script = format!("Set-WinUserLanguageList -LanguageList {validated} -Force");
    runner
        .run("powershell", &["-NoProfile", "-NonInteractive", "-Command", &script], None)
        .await
}

/// Reads the currently active input method's ID — mirrors legacy
/// `ObterLayoutAtivo`.
pub async fn get_active_layout(runner: &impl ProcessRunner) -> Result<String, String> {
    let output = runner
        .run(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-WinUserLanguageList)[0].InputMethodTips[0]",
            ],
            None,
        )
        .await?;
    Ok(output.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeProcessRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        output: Result<String, String>,
    }

    impl FakeProcessRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                output: Ok(String::new()),
            }
        }

        fn returning(output: Result<String, String>) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                output,
            }
        }
    }

    impl ProcessRunner for FakeProcessRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> Result<String, String> {
            self.calls.lock().unwrap().push((
                program.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
            ));
            self.output.clone()
        }
    }

    #[test]
    fn get_available_layouts_returns_all_seven_sorted_by_name() {
        let layouts = get_available_layouts();
        assert_eq!(layouts.len(), 7);
        let names: Vec<&str> = layouts.iter().map(|l| l.nome).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn validate_language_tag_accepts_allowlisted_tag() {
        assert_eq!(validate_language_tag("pt-BR"), Ok("pt-BR"));
    }

    #[test]
    fn validate_language_tag_rejects_tag_not_in_the_allowlist() {
        assert!(validate_language_tag("fr-FR").is_err());
    }

    #[test]
    fn validate_language_tag_rejects_injection_payload() {
        assert!(validate_language_tag("pt-BR; Start-Process cmd").is_err());
    }

    #[tokio::test]
    async fn change_keyboard_layout_runs_set_winuserlanguagelist_with_sanitized_arg() {
        let runner = FakeProcessRunner::new();

        change_keyboard_layout("pt-BR", &runner).await.unwrap();

        let calls = runner.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        let (program, args) = &calls[0];
        assert_eq!(program, "powershell");
        assert!(args
            .iter()
            .any(|a| a == "Set-WinUserLanguageList -LanguageList pt-BR -Force"));
    }

    #[tokio::test]
    async fn change_keyboard_layout_rejects_injection_payload_without_running_anything() {
        let runner = FakeProcessRunner::new();

        let result = change_keyboard_layout("pt-BR; Start-Process cmd", &runner).await;

        assert!(result.is_err());
        assert!(runner.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_active_layout_returns_trimmed_output() {
        let runner = FakeProcessRunner::returning(Ok("  0409:00000409\n".to_string()));

        let result = get_active_layout(&runner).await.unwrap();

        assert_eq!(result, "0409:00000409");
    }
}
