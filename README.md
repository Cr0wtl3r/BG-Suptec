# BG-SupTec

Ferramenta interna de suporte técnico para Windows. Centraliza e padroniza procedimentos que hoje seriam executados manualmente via terminal/PowerShell/registro — cada funcionalidade é uma ação tipada e auditável, não um terminal genérico. Roda localmente, protegida por senha, com privilégios de administrador.

Reescrita de Go+Wails+Svelte para **Rust + Tauri v2 + React 19 + TypeScript + Tailwind CSS 4**. O código legado (referência de comportamento e estilo) está preservado em [`legacy_code/`](legacy_code/). O plano de execução completo da migração está em [`refatoracao-rust-tauri.md`](refatoracao-rust-tauri.md); o estado de cada feature em [`CHECKLIST.md`](CHECKLIST.md).

## Stack

- **Backend:** Rust + Tauri v2
- **Frontend:** React 19 + TypeScript + Tailwind CSS 4 + Vite
- **Auth:** argon2id com rate limiting (backoff exponencial após 5 tentativas)
- **Config:** `kms.json` e `auth.hash` externalizados — editáveis sem recompilar
- **Audit:** toda ação destrutiva grava uma linha em `%APPDATA%\BG-SupTec\audit-YYYY-MM.log`
- **OS alvo:** Windows 10 (1809+), 11, Server 2016+ (Windows 7/8.1 funcionam apenas se o WebView2 Runtime já estiver instalado — a Microsoft descontinuou a distribuição dele para essas versões em jan/2023)

## Pré-requisitos

- [Rust](https://rustup.rs/) (toolchain MSVC no Windows) + `cargo tauri` CLI (`cargo install tauri-cli --version "^2"`)
- [Node.js](https://nodejs.org/) 18+ e `npm`
- Windows com [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (já vem pré-instalado no Windows 11 e na maioria das instalações atualizadas do Windows 10)

## Desenvolvimento

```sh
npm install
npm run tauri dev
```

A janela abre pedindo elevação (UAC) — o app exige administrador (`requireAdministrator`) para poder executar as ações de sistema. Em desenvolvimento, `auth.hash` precisa existir ao lado do binário (`src-tauri/target/debug/auth.hash`); gere um com:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --bin generate_hash -- "suaSenha"
```

## Configuração externa

Dois arquivos ficam **ao lado do `.exe`** (não embutidos no binário) e são lidos em runtime — editáveis sem recompilar:

- **`kms.json`** — chaves GVLK e servidores KMS para ativação de Windows/Office. Veja a seção `windows`/`office` do arquivo para o formato; nunca é commitado (gitignored).
- **`auth.hash`** — hash argon2id da senha de acesso ao app, gerado pelo binário `generate_hash` (veja acima). Também gitignored.

Nenhum dos dois é extraído de dentro do `.exe` em runtime — diferente do app legado Wails, o Tauri não empacota `resources` que precisem ser descompactados, então o `.exe` funciona em uma pasta vazia desde que esses dois arquivos estejam ao lado dele.

## Build standalone

```powershell
.\build.ps1 -Senha "suaSenhaForte"
```

Isso gera (ou reaproveita, se já existir) o `auth.hash`, roda `cargo tauri build` e monta um pacote pronto para distribuir em `dist-standalone\` com `bg-suptec.exe` + `kms.json` + `auth.hash` lado a lado. Parâmetros:

- `-Senha "..."` — gera um novo `auth.hash` a partir desta senha (omitir reaproveita um `auth.hash` já existente na raiz do projeto)
- `-SkipBundle` — pula a geração de instaladores NSIS/MSI, produzindo apenas o `.exe` (mais rápido para iteração local)

Para buildar manualmente sem o script:

```sh
cargo tauri build
```

O executável final fica em `src-tauri/target/release/bg-suptec.exe`; copie-o junto com `kms.json` e `auth.hash` para distribuir.

## Audit log

Toda ação destrutiva (alterar IP/DNS/hostname, bloquear/desbloquear programa no firewall, restaurar Photo Viewer, agendar/cancelar desligamento) grava uma linha em `%APPDATA%\BG-SupTec\audit-YYYY-MM.log` com timestamp ISO 8601, usuário, ação, parâmetros e resultado (`ok` ou `erro: ...`). O arquivo roda mensalmente pelo próprio nome (um arquivo novo por mês, sem necessidade de limpeza manual).

## Features

As 15 features funcionais (autenticação, painel de informações + 13 ações técnicas) estão listadas com status de port em [`CHECKLIST.md`](CHECKLIST.md).

## Testes

```sh
cd src-tauri && cargo test --lib   # backend
npm run build                       # frontend (tsc + vite)
```
