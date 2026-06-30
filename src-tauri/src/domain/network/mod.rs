use std::net::Ipv4Addr;

const NOME_COMPUTADOR_MAX_LEN: usize = 15;

/// Validates a proposed computer name (length 1-15, matching legacy's
/// constraint) and returns it with spaces stripped, ready to hand to
/// `Rename-Computer`.
pub fn validate_computer_name(name: &str) -> Result<String, String> {
    if name.is_empty() || name.chars().count() > NOME_COMPUTADOR_MAX_LEN {
        return Err("nome do computador deve ter entre 1 e 15 caracteres".to_string());
    }
    Ok(name.replace(' ', ""))
}

/// Validates that a string is a well-formed IPv4 address.
pub fn validate_ipv4(ip: &str) -> Result<(), String> {
    ip.parse::<Ipv4Addr>()
        .map(|_| ())
        .map_err(|_| "formato de IP inválido".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_computer_name_accepts_name_within_length_and_strips_spaces() {
        assert_eq!(
            validate_computer_name("PC TESTE"),
            Ok("PCTESTE".to_string())
        );
    }

    #[test]
    fn validate_computer_name_rejects_empty_name() {
        assert!(validate_computer_name("").is_err());
    }

    #[test]
    fn validate_computer_name_rejects_name_longer_than_15_chars() {
        assert!(validate_computer_name("NOME-MUITO-LONGO-DEMAIS").is_err());
    }

    #[test]
    fn validate_ipv4_accepts_valid_address() {
        assert!(validate_ipv4("192.168.1.50").is_ok());
    }

    #[test]
    fn validate_ipv4_rejects_invalid_format() {
        assert!(validate_ipv4("not-an-ip").is_err());
    }
}
