use crate::adapters::{powershell, process};
use crate::audit;
use crate::domain::network::{validate_computer_name, validate_ipv4};

#[tauri::command]
pub async fn reiniciar_computador() -> Result<(), String> {
    let resultado = process::run("shutdown", &["/r", "/t", "0"]).await.map(|_| ());
    audit::record("reiniciar_computador", "", &audit::outcome(&resultado));
    resultado
}

#[tauri::command]
pub async fn alterar_nome_computador(novo_nome: String) -> Result<bool, String> {
    let resultado = alterar_nome_computador_inner(&novo_nome).await;
    audit::record(
        "alterar_nome_computador",
        &format!("novo_nome={novo_nome}"),
        &audit::outcome(&resultado),
    );
    resultado
}

async fn alterar_nome_computador_inner(novo_nome: &str) -> Result<bool, String> {
    let nome_sanitizado = validate_computer_name(novo_nome)?;

    powershell::run_script_with_env(
        "Rename-Computer -NewName $env:BG_NOVO_NOME -Force -PassThru",
        &[("BG_NOVO_NOME", nome_sanitizado.as_str())],
    )
    .await?;

    Ok(true)
}

/// Mirrors legacy semantics: "available" means the address did *not*
/// answer the ping (a non-zero exit / timeout), i.e. nothing currently
/// claims it on the network.
async fn ip_disponivel(ip: &str) -> bool {
    process::run("ping", &["-n", "1", "-w", "1000", ip])
        .await
        .is_err()
}

#[tauri::command]
pub async fn alterar_ip(
    interface_name: String,
    novo_ip: String,
    mascara: String,
    gateway: String,
) -> Result<(), String> {
    let resultado = alterar_ip_inner(&interface_name, &novo_ip, &mascara, &gateway).await;
    audit::record(
        "alterar_ip",
        &format!("interface={interface_name},ip={novo_ip},mascara={mascara},gateway={gateway}"),
        &audit::outcome(&resultado),
    );
    resultado
}

async fn alterar_ip_inner(
    interface_name: &str,
    novo_ip: &str,
    mascara: &str,
    gateway: &str,
) -> Result<(), String> {
    if interface_name.is_empty() {
        return Err("não foi possível identificar a interface de rede para alteração".to_string());
    }

    let name_arg = format!("name=\"{interface_name}\"");

    if novo_ip.is_empty() {
        process::run(
            "netsh",
            &[
                "interface",
                "ip",
                "set",
                "address",
                name_arg.as_str(),
                "source=dhcp",
            ],
        )
        .await?;
        return Ok(());
    }

    validate_ipv4(novo_ip)?;

    if !ip_disponivel(novo_ip).await {
        return Err("o IP informado já está em uso na rede".to_string());
    }

    process::run(
        "netsh",
        &[
            "interface",
            "ip",
            "set",
            "address",
            name_arg.as_str(),
            "static",
            novo_ip,
            mascara,
            gateway,
        ],
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn alterar_dns(
    interface_name: String,
    dns_primario: String,
    dns_secundario: String,
) -> Result<(), String> {
    let resultado = alterar_dns_inner(&interface_name, &dns_primario, &dns_secundario).await;
    audit::record(
        "alterar_dns",
        &format!(
            "interface={interface_name},dns_primario={dns_primario},dns_secundario={dns_secundario}"
        ),
        &audit::outcome(&resultado),
    );
    resultado
}

async fn alterar_dns_inner(
    interface_name: &str,
    dns_primario: &str,
    dns_secundario: &str,
) -> Result<(), String> {
    if interface_name.is_empty() {
        return Err("não foi possível identificar a interface de rede para alteração".to_string());
    }

    let name_arg = format!("name=\"{interface_name}\"");

    if dns_primario.is_empty() {
        process::run(
            "netsh",
            &["interface", "ip", "set", "dns", name_arg.as_str(), "source=dhcp"],
        )
        .await?;
        return Ok(());
    }

    validate_ipv4(dns_primario).map_err(|_| "DNS primário tem formato inválido".to_string())?;

    process::run(
        "netsh",
        &[
            "interface",
            "ip",
            "set",
            "dns",
            name_arg.as_str(),
            "static",
            dns_primario,
        ],
    )
    .await
    .map_err(|e| format!("erro ao configurar DNS primário: {e}"))?;

    if !dns_secundario.is_empty() {
        validate_ipv4(dns_secundario)
            .map_err(|_| "DNS secundário tem formato inválido".to_string())?;

        process::run(
            "netsh",
            &[
                "interface",
                "ip",
                "add",
                "dns",
                name_arg.as_str(),
                dns_secundario,
                "index=2",
            ],
        )
        .await
        .map_err(|e| format!("erro ao configurar DNS secundário: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn alterar_ip_rejects_invalid_ip_format_before_touching_the_network() {
        let result = alterar_ip(
            "Ethernet".to_string(),
            "not-an-ip".to_string(),
            "255.255.255.0".to_string(),
            "192.168.1.1".to_string(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn alterar_ip_rejects_empty_interface_name() {
        let result = alterar_ip(
            String::new(),
            "192.168.1.50".to_string(),
            "255.255.255.0".to_string(),
            "192.168.1.1".to_string(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn alterar_dns_rejects_invalid_primary_dns_format() {
        let result = alterar_dns(
            "Ethernet".to_string(),
            "not-an-ip".to_string(),
            String::new(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn alterar_nome_computador_rejects_name_longer_than_15_chars() {
        let result = alterar_nome_computador("NOME-MUITO-LONGO-DEMAIS".to_string()).await;
        assert!(result.is_err());
    }
}
