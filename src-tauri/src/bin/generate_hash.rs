use bg_suptec_lib::auth;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let password = match args.get(1) {
        Some(p) if !p.is_empty() => p,
        _ => {
            eprintln!("Uso: generate_hash <senha>");
            std::process::exit(1);
        }
    };

    match auth::hash_password(password) {
        Ok(hash) => println!("{}", hash),
        Err(e) => {
            eprintln!("Erro ao gerar hash: {}", e);
            std::process::exit(1);
        }
    }
}
