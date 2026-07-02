use serde::{Deserialize, Serialize};

use crate::ports::ProcessRunner;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeC2rInstall {
    pub client_exe: String,
    pub click_to_run_exe: String,
    pub install_root: String,
    pub platform: String,
    pub culture: String,
    pub version: String,
    pub audience_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficeUpdateChannel {
    pub id: &'static str,
    pub name: &'static str,
    pub ffn: &'static str,
}

pub fn office_update_channels() -> Vec<OfficeUpdateChannel> {
    vec![
        OfficeUpdateChannel {
            id: "CC",
            name: "Monthly Current",
            ffn: "492350F6-3A01-4F97-B9C0-C7C6DDF67D60",
        },
        OfficeUpdateChannel {
            id: "MEC",
            name: "Monthly Enterprise",
            ffn: "55336B82-A18D-4DD6-B5F6-9E5095C314A6",
        },
        OfficeUpdateChannel {
            id: "DC",
            name: "Semi Annual",
            ffn: "7FFBC6BF-BC32-4F92-8982-F9DD17FD3114",
        },
        OfficeUpdateChannel {
            id: "LTSC2021",
            name: "Perpetual 2021 VL",
            ffn: "5030841D-C919-4594-8D2D-84AE4F96E58E",
        },
        OfficeUpdateChannel {
            id: "LTSC2024",
            name: "Perpetual 2024 VL",
            ffn: "7983BAC0-E531-40CF-BE00-FD24FE66619C",
        },
    ]
}

pub async fn change_office_channel(
    runner: &impl ProcessRunner,
    install: &OfficeC2rInstall,
    channel_id: &str,
    on_log: impl Fn(&str),
) -> Result<String, String> {
    let channel = office_update_channels()
        .into_iter()
        .find(|channel| channel.id.eq_ignore_ascii_case(channel_id))
        .ok_or_else(|| "Canal Office não suportado".to_string())?;
    let cdn_arg = format!(
        "updatebaseurl=http://officecdn.microsoft.com/pr/{}",
        channel.ffn
    );
    let channel_arg = format!(
        "cdnbaseurl=http://officecdn.microsoft.com/pr/{}",
        channel.ffn
    );
    on_log(&format!(
        "Alterando canal do Office para {}...",
        channel.name
    ));
    runner
        .run(
            &install.client_exe,
            &[
                "/update",
                "user",
                "displaylevel=false",
                "forceappshutdown=true",
                "updatepromptuser=false",
                cdn_arg.as_str(),
                channel_arg.as_str(),
            ],
            Some(&install.install_root),
        )
        .await
}

pub async fn add_office_product(
    runner: &impl ProcessRunner,
    install: &OfficeC2rInstall,
    product_id: &str,
    excluded_apps: &[String],
    on_log: impl Fn(&str),
) -> Result<String, String> {
    validate_product_id(product_id)?;
    for app in excluded_apps {
        validate_product_id(app)?;
    }
    let product_arg = format!("productstoadd={}.16_{}_x-none", product_id, install.culture);
    let excluded = if excluded_apps.is_empty() {
        "flt.useteamsaddon=disabled".to_string()
    } else {
        format!(
            "{}.excludedapps.16=groove,{}",
            product_id,
            excluded_apps.join(",")
        )
    };
    on_log(&format!("Adicionando produto Office {product_id}..."));
    runner
        .run(
            &install.click_to_run_exe,
            &[
                install.platform.as_str(),
                install.culture.as_str(),
                product_arg.as_str(),
                excluded.as_str(),
            ],
            Some(&install.install_root),
        )
        .await
}

pub async fn remove_office_product(
    runner: &impl ProcessRunner,
    install: &OfficeC2rInstall,
    product_id: &str,
    on_log: impl Fn(&str),
) -> Result<String, String> {
    validate_product_id(product_id)?;
    let product_arg = format!("productstoremove={product_id}.16");
    on_log(&format!("Removendo produto Office {product_id}..."));
    runner
        .run(
            &install.click_to_run_exe,
            &[install.platform.as_str(), product_arg.as_str()],
            Some(&install.install_root),
        )
        .await
}

fn validate_product_id(value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|ch| ch.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err("Identificador de produto Office inválido".to_string())
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
            cwd: Option<&str>,
        ) -> Result<String, String> {
            self.calls.lock().unwrap().push(format!(
                "{} {} @{}",
                program,
                args.join(" "),
                cwd.unwrap_or("")
            ));
            Ok(String::new())
        }
    }

    #[test]
    fn office_channel_catalog_contains_supported_channels_only() {
        let channels = office_update_channels();

        assert!(channels.iter().any(|c| c.id == "CC" && c.ffn.contains('-')));
        assert!(!channels
            .iter()
            .any(|c| c.name.to_ascii_lowercase().contains("ohook")));
    }

    #[tokio::test]
    async fn change_office_channel_calls_click_to_run_with_update_prompt() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };
        let install = OfficeC2rInstall {
            client_exe: r"C:\Office\OfficeC2RClient.exe".to_string(),
            click_to_run_exe: r"C:\Office\OfficeClickToRun.exe".to_string(),
            install_root: r"C:\Office".to_string(),
            platform: "x64".to_string(),
            culture: "pt-br".to_string(),
            version: "16.0.1".to_string(),
            audience_id: "old".to_string(),
        };

        change_office_channel(&runner, &install, "CC", |_| {})
            .await
            .unwrap();

        assert!(calls.lock().unwrap()[0].contains("/update user"));
    }
}
