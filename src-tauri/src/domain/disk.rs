use serde::{Deserialize, Serialize};

use crate::ports::ProcessRunner;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsEditionInfo {
    pub current: String,
    pub targets: Vec<String>,
}

pub async fn convert_mbr_to_gpt(
    runner: &impl ProcessRunner,
    on_log: impl Fn(&str),
) -> Result<(), String> {
    on_log("Validando disco com mbr2gpt...");
    runner
        .run("mbr2gpt", &["/validate", "/allowFullOS"], None)
        .await?;
    on_log("Convertendo MBR para GPT...");
    runner
        .run("mbr2gpt", &["/convert", "/allowFullOS"], None)
        .await?;
    Ok(())
}

pub async fn get_windows_editions(
    runner: &impl ProcessRunner,
) -> Result<WindowsEditionInfo, String> {
    let current_output = runner
        .run(
            "dism",
            &["/online", "/english", "/Get-CurrentEdition"],
            None,
        )
        .await?;
    let targets_output = runner
        .run(
            "dism",
            &["/online", "/english", "/Get-TargetEditions"],
            None,
        )
        .await?;

    Ok(WindowsEditionInfo {
        current: parse_current_edition(&current_output)
            .unwrap_or_else(|| "Desconhecida".to_string()),
        targets: parse_target_editions(&targets_output),
    })
}

pub async fn change_windows_edition(
    runner: &impl ProcessRunner,
    target_edition: &str,
    product_key: &str,
    on_log: impl Fn(&str),
) -> Result<String, String> {
    validate_edition_name(target_edition)?;
    validate_product_key(product_key)?;
    on_log(&format!(
        "Alterando edição do Windows para {target_edition}..."
    ));
    let edition_arg = format!("/Set-Edition:{target_edition}");
    let key_arg = format!("/ProductKey:{product_key}");
    runner
        .run(
            "dism",
            &[
                "/online",
                edition_arg.as_str(),
                key_arg.as_str(),
                "/AcceptEula",
            ],
            None,
        )
        .await
}

pub fn parse_target_editions(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            line.split_once(':')
                .map(|(_, value)| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
        .collect()
}

fn parse_current_edition(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .map(|(_, value)| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
}

fn validate_edition_name(value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err("Edição do Windows inválida".to_string())
    }
}

fn validate_product_key(value: &str) -> Result<(), String> {
    let parts: Vec<&str> = value.split('-').collect();
    let valid = parts.len() == 5
        && parts
            .iter()
            .all(|part| part.len() == 5 && part.chars().all(|ch| ch.is_ascii_alphanumeric()));
    if valid {
        Ok(())
    } else {
        Err("Chave de produto deve usar o formato XXXXX-XXXXX-XXXXX-XXXXX-XXXXX".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::ProcessRunner;
    use std::sync::{Arc, Mutex};

    struct FakeRunner {
        calls: Arc<Mutex<Vec<String>>>,
        output: String,
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
            Ok(self.output.clone())
        }
    }

    #[tokio::test]
    async fn convert_mbr_to_gpt_validates_before_conversion() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
            output: String::new(),
        };

        convert_mbr_to_gpt(&runner, |_| {}).await.unwrap();

        assert_eq!(
            calls.lock().unwrap().clone(),
            vec![
                "mbr2gpt /validate /allowFullOS".to_string(),
                "mbr2gpt /convert /allowFullOS".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn parse_target_editions_reads_dism_output() {
        let output = "Target Edition : Professional\nTarget Edition : Enterprise";

        assert_eq!(
            parse_target_editions(output),
            vec!["Professional".to_string(), "Enterprise".to_string()]
        );
    }
}
