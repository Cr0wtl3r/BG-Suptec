# Refatoração BG-SupTec → Rust + Tauri

## Goal

Refatorar completa a aplicação BG-SupTec de Go+Wails+Svelte para Rust+Tauri+React, preservando o design system e todas as 15 features funcionais, com arquitetura escalável, TDD, segurança por construção e config externalizado. Prazo: 4 semanas.

Mova todo o conteúdo "legado" para uma pasta legacy_code e começe a construir na raiz do projeto com base no conteúdo legado. 

## Stack Final

- **Backend:** Rust 1.95+ + Tauri v2
- **Frontend:** React 19 + TypeScript + Tailwind CSS 4 + Vite
- **Auth:** argon2id (substitui SHA-256 sem salt)
- **Config:** `kms.json` externalizado (editável sem recompilar) 
- **Build:** `cargo tauri build` → standalone `.exe` + `kms.json`
- **OS alvo:** Windows 10 (1809+), 11, Server 2016+ (Win 7/8.1 com aviso se sem WebView2)

## Princípios Não-Negociáveis

1. **Sem `ExecutarComando` genérico** — cada feature tem seu command tipado
2. **TDD por feature** — `cargo test` antes da implementação
3. **Vertical Slice completo** — backend + frontend + teste + integração por slice
4. **Preservar DESIGN.md** — tokens, cores, tipografia, componentes idênticos
5. **Standalone executable** — sem dependências no diretório, sem extração
6. **Audit logging** — toda ação destrutiva gera log em `%APPDATA%\BG-SupTec\audit.log`

---

## Semana 1 — Fundação + Auth + Info Sistema

### Slice 0: Setup do Projeto Tauri + React

- [x] **0.1** Instalar Rust toolchain + Tauri CLI → Verify: `cargo tauri --version` retorna versão ✓ `tauri-cli 2.11.4` + `rustc 1.96.0`
- [x] **0.2** Criar projeto Tauri v2 com template React+TS → Verify: `cargo tauri dev` abre janela vazia ✓ janela "BG-SupTec" abre com template Tauri+React+Vite (screenshot confirmado)
- [x] **0.3** Configurar Tailwind CSS 4 com tokens do `DESIGN.md` (cores, tipografia, spacing) → Verify: componente de teste usa `bg-navy-slate` e renderiza corretamente ✓ `@tailwindcss/vite` instalado, tokens (`dark-blue-bg`, `dark-blue-light`, `accent-orange`, `text-light`, `structural-purple`, font-sans Segoe UI) definidos via `@theme` em `src/App.css`; componente de teste com `bg-accent-orange`/`bg-dark-blue-light` renderizou corretamente (screenshot confirmado, `npm run build` limpo)
- [x] **0.4** Portar `background.jpg` e assets visuais para `src-tauri/icons/` e `public/` → Verify: background aparece na janela ✓ ícones do app regenerados via `cargo tauri icon appicon.png` (logo real VIP, substituindo placeholders padrão do Tauri; assets iOS/Android removidos por não fazerem parte do alvo Windows-only); `background.jpg` e `profile.png` copiados de `legacy_code/frontend/public/` para `public/`; `App.tsx` ajustado para usar `background-image: url('/background.jpg')` com overlay `bg-black/40` (conforme DESIGN.md "vidro sobre realidade"); verificado com `npm run build` limpo e screenshot da janela `cargo tauri dev` confirmando background renderizado
- [x] **0.5** Configurar `tauri.conf.json` com manifest requireAdmin, tamanho janela 1160x700, min 700x555 (igual ao atual) → Verify: janela abre com tamanho correto e pede admin ✓ `tauri.conf.json` → `app.windows[0]` com `width:1160, height:700, minWidth:700, minHeight:555` (idêntico ao `main.go` legado); `build.rs` usa `tauri_build::WindowsAttributes::new().app_manifest(...)` mesclando o manifest padrão do Tauri (dependency Common-Controls v6, necessário para os diálogos nativos) com `trustInfo/requestedExecutionLevel="requireAdministrator"` (idêntico ao `BG-SupTec.exe.manifest` legado); verificado: `cargo run` direto (CreateProcess) falha com erro 740 "requer elevação" confirmando que o manifest exige admin; lançamento via `Start-Process -Verb RunAs` (ShellExecute, equivalente ao duplo-clique no Explorer) eleva corretamente e abre a janela; `GetWindowRect` via P/Invoke mediu 1176×739 de borda externa = 1160×700 de área útil + chrome padrão do Windows (16px bordas + 39px titlebar), confirmando tamanho exato; screenshot confirma renderização correta
- [x] **0.6** Criar estrutura de pastas Rust: `commands/`, `domain/`, `adapters/`, `ports/`, `audit/`, `config/`, `auth/` → Verify: `cargo build` compila com módulos vazios ✓ 7 diretórios criados em `src-tauri/src/` cada um com `mod.rs` vazio, declarados em `lib.rs` (`mod adapters; mod audit; mod auth; mod commands; mod config; mod domain; mod ports;`); `cargo build` compila limpo sem warnings
- [x] **0.7** Configurar WebView2 download automático (como `wails.json` atual) → Verify: em VM sem WebView2, app direciona para download ✓ `tauri.conf.json` → `bundle.windows.webviewInstallMode` configurado com `type: "downloadBootstrapper"` e `silent: true` (equivalente ao `"webview2": "download"` do `wails.json` legado — instalador baixa e instala o runtime WebView2 automaticamente e silenciosamente se ausente); `cargo build` validou o schema do config sem erros; teste em VM real fica pendente para o item 18.3 (Semana 4, testes finais em VM limpa)

**Done When:** Janela Tauri abre com background, pede admin, Tailwind com design tokens funciona, estrutura de pastas compila.

---

### Slice 1: Autenticação (argon2id + Login)

- [x] **1.1** Escrever teste: `auth::verify_password("senha_correta")` retorna true, senha errada retorna false → Verify: `cargo test auth` passa ✓ `src-tauri/src/auth/mod.rs` testes `verify_password_returns_match_for_correct_password` / `..._no_match_for_wrong_password` (+ casos extra: vazio, especiais, unicode, formato inválido); 1 teste pré-existente com asserção de mensagem de erro literal incorreta foi corrigido para checar a variante `VerifyResult::Error` em vez do texto exato (texto vinha da lib `password-hash`, não do nosso código)
- [x] **1.2** Implementar `auth/mod.rs` com argon2id (crate `argon2`) → Verify: teste passa ✓ `hash_password`/`verify_password` com `Argon2id`, params 64 MiB / t=3 / p=4 (`AuthConfig::default`), salt aleatório via `OsRng`; `cargo test --lib` → 8/8 passando neste módulo
- [x] **1.3** Escrever teste: login com 5 tentativas falhas aplica backoff exponencial → Verify: teste passa ✓ `src-tauri/src/auth/rate_limiter.rs` testes `fifth_failed_attempt_applies_exponential_backoff` e `backoff_grows_exponentially_with_further_failures` (RED confirmado: módulo não existia, falha de compilação `unresolved import`)
- [x] **1.4** Implementar rate limiting no auth → Verify: teste passa ✓ `RateLimiter` (mesmo arquivo) — abaixo de 5 falhas sem bloqueio; a partir da 5ª, backoff `2^(tentativas-5)` segundos (1s, 2s, 4s, ...), `is_locked()`/`record_success()` resetam o contador; 4/4 testes passando
- [x] **1.5** Criar `auth/hash_generator` utilitário CLI para gerar hash argon2id da senha → Verify: `cargo run --bin generate_hash "minhasenha"` imprime hash ✓ `src-tauri/src/bin/generate_hash.rs` (binário separado em `src/bin/`, reaproveita `bg_suptec_lib::auth::hash_password` — exigiu tornar `mod auth` público em `lib.rs`); testado via build + execução elevada (binário herda o manifest `requireAdministrator` do pacote) imprimindo `$argon2id$v=19$m=65536,t=3,p=4$...`
- [x] **1.6** Implementar `commands/auth.rs` (Tauri command `login`) → Verify: invocável via `invoke('login', { senha })` no frontend ✓ lógica pura `attempt_login(senha, hash, &mut RateLimiter)` testada isoladamente (3 testes: sucesso, falha, bloqueio após 5 tentativas) sem depender de `tauri::State`; o `#[tauri::command] login` é apenas um wrapper fino que injeta `AuthState` (hash + `Mutex<RateLimiter>`) gerenciado via `.manage()`; registrado em `lib.rs` com `commands::auth::login`
- [x] **1.7** Criar componente `Login.tsx` com design do `Login.svelte` atual (card slate-panel, input, botão burnt orange) → Verify: visual idêntico ao atual ✓ `src/components/Login.tsx` porta fielmente o legado (avatar `profile.png`, título "Caixa de Ferramentas", input de senha com Enter-to-submit, botão laranja queimado, mensagem de erro) chamando `invoke<boolean>('login', { senha })`; `npm run build` (tsc + vite) limpo. Verificação visual em janela real (`cargo tauri dev`, requer elevação) não foi possível nesta sessão por falta de ferramenta de captura de tela — pendente confirmação visual do usuário
- [x] **1.8** Integrar login no `App.tsx` com state `logado` → Verify: login correto mostra painel, errado mostra erro ✓ `App.tsx` com `useState(logado)`; renderiza `<Login onLoginSucesso={...}/>` quando deslogado, painel placeholder quando logado; erro de senha é tratado dentro do próprio `Login.tsx` (mensagem inline, não derruba a tela)
- [x] **1.9** Escrever teste: hash pode ser carregado de arquivo de config (`auth.hash`) sem recompilar → Verify: teste passa ✓ `auth::tests::load_hash_from_file_reads_trimmed_contents` (RED confirmado: função inexistente) e `load_hash_from_file_errors_when_missing`
- [x] **1.10** Criar `auth.hash` na raiz do build (gitignored) + carregar em runtime → Verify: app funciona com hash do arquivo ✓ `auth::load_hash_from_file` implementado em 1.9; `lib.rs::run()` resolve `auth.hash` ao lado do executável (`current_exe().parent()`), carrega no startup e gerencia via `AuthState`; se ausente, loga aviso e mantém hash vazio (login falha com segurança, igual ao comportamento do legado quando `PASSWORD` não definido); `auth.hash` adicionado ao `.gitignore` na raiz; hash de teste gerado via `generate_hash` e copiado para `target/debug/auth.hash` para uso em `cargo tauri dev`

**Done When:** Login funciona com argon2id, rate limiting ativo, hash carregado de arquivo externo, UI idêntica ao atual.

---

### Slice 2: Painel de Informações (System Info + Editable Fields)

- [x] **2.1** Escrever teste: `domain::system::get_info()` retorna struct com hostname, RAM, Windows, CPU, rede → Verify: `cargo test system_info` passa ✓ RED confirmado (`error[E0432]: unresolved imports super::get_info, super::SystemInfo` — nem a função nem a struct existem ainda); `src-tauri/src/domain/system/mod.rs` com 2 testes (`get_info_assembles_hostname_ram_windows_cpu_and_network` e `get_info_falls_back_to_na_when_no_active_network_interface`) injetando fakes de 3 ports novos — `RegistryReader` (`ports/registry.rs`), `MemoryReader` (`ports/memory.rs`), `NetworkReader` + `NetworkInfo` (`ports/network.rs`) — para manter `get_info` puro e testável sem tocar Windows real; `domain/mod.rs` e `ports/mod.rs` (antes vazios) agora declaram os novos submódulos
- [x] **2.2** Implementar `adapters/registry.rs` com `winreg` crate (ler HKLM ProductName, DisplayVersion, CurrentBuild, ProcessorNameString) → Verify: `cargo test adapters::registry` passa (3 testes: lê `ProductName` real, `None` para chave inexistente, `None` para valor inexistente em chave existente) ✓
- [x] **2.3** Implementar `adapters/powershell.rs` com `tokio::process::Command` (sem shell intermediário, args `&[&str]`) → Verify: `cargo test adapters::process adapters::powershell` passa, incluindo teste de injeção comportamental (`run_treats_a_malicious_argument_as_literal_text_not_a_new_command`, argumento `"ola && exit 7"` passado como argv literal não quebra para um segundo comando). **Achado crítico durante implementação**: o padrão inicial de passar parâmetros extras após `-Command <script>` (esperando bind como `$args`/parâmetros) está **quebrado** — `powershell -Command` concatena todo texto subsequente na própria string de script e a reanalisa, então `;`/`&&` em um "argumento extra" escapam como em uma interpolação de shell normal (confirmado por teste que falhou: valor malicioso `"abc; Write-Output INJETADO"` foi de fato executado). Corrigido substituindo por passagem de valores não confiáveis via variáveis de ambiente (`adapters/process.rs::run_with_env`, `adapters/powershell.rs::run_script_with_env`), lidas no script como `$env:NOME` — variáveis de ambiente trafegam pelo bloco de ambiente do processo do SO, um canal totalmente separado do texto de linha de comando, então não podem ser reinterpretadas como sintaxe. Teste `run_script_with_env_exposes_value_as_a_literal_string_not_executable_code` prova isso com o mesmo payload malicioso, agora retornando-o como string literal
- [x] **2.4** Implementar `domain/system/mod.rs` — coleta info via registry + PowerShell (Get-NetAdapter, Get-NetIPConfiguration) → Verify: `cargo test --lib` (41/41) passa; `domain::system::get_info` agora é `async fn` (aguarda `NetworkReader::active_interface_info`, que também se tornou `async fn` via RPITIT — necessário para `.await` em `PowerShellNetworkReader`), monta `SystemInfo` (`#[serde(rename)]` com nomes de campo idênticos ao frontend legado: `nomeComputador`, `versaoWindows`, `edicaoWindows`, `buildWindows`, `processador`, `memoriaTotalGB`, `enderecoMAC`, `enderecoIP`, `mascaraRede`, `gatewayPadrao`, `dnsPrimario`, `dnsSecundario`, `interfaceAtiva`) a partir de `RegistryReader`+`MemoryReader`+`NetworkReader` injetados, com fallback `"N/A"`/`""` quando não há interface de rede ativa; `adapters/network.rs` criado com `PowerShellNetworkReader` (script PowerShell estático reaproveitado do legado — `Get-NetAdapter -Physical`/`Get-NetIPConfiguration`/`Get-DnsClientServerAddress`, sem interpolação, portanto já seguro) ✓
- [x] **2.5** Substituir parser regex por `serde_json` (PowerShell output com `ConvertTo-Json`) → Verify: `cargo test adapters::network` passa (8 testes), incluindo parse correto de valores com aspas escapadas (`parses_output_with_escaped_quotes_in_interface_name`) e do quirk do `ConvertTo-Json` que colapsa array de 1 elemento em escalar (`parses_output_with_a_single_dns_server_collapsed_to_a_scalar`) — `RawNetworkInfo` tipa campos potencialmente escalares-ou-array como `serde_json::Value`, normalizados por `first_string`/`all_strings`; `prefix_to_subnet_mask` substitui `net.CIDRMask` do legado com aritmética de bits manual; MAC convertido de `-` para `:` (paridade com `strings.ReplaceAll` do legado) ✓
- [x] **2.6** Implementar `commands/system_info.rs` (Tauri command `obter_informacoes_sistema`) → Verify: `cargo build` limpo; comando `async fn obter_informacoes_sistema() -> Result<SystemInfo, String>` monta os 3 adapters reais (`WinRegistryReader`, `WinMemoryReader`, `PowerShellNetworkReader`) e chama `domain::system::get_info(...).await`; hostname lido de `COMPUTERNAME` (paridade com `os.Hostname()` do legado); registrado em `lib.rs`'s `invoke_handler!` ✓
- [x] **2.9** Implementar `commands/network.rs` — `alterar_nome_computador`, `alterar_ip`, `alterar_dns` com args sanitizados → Verify: `cargo test commands::network` passa; todas as chamadas usam `adapters::process::run`/`adapters::powershell::run_script_with_env` com argv literal (sem `cmd /c`/string interpolada) — `alterar_nome_computador` valida via `domain::network::validate_computer_name` e passa o nome por `$env:BG_NOVO_NOME`; `alterar_ip`/`alterar_dns` chamam `netsh` diretamente com cada valor como elemento de argv isolado (DHCP ou estático, paridade com a lógica do legado incluindo checagem de disponibilidade do IP via ping antes de aplicar IP estático); `reiniciar_computador` (`shutdown /r /t 0`) adicionado — necessário para o fluxo de confirmação de reinício do frontend, não estava no checklist original mas é exigido pela fatia vertical ✓
- [x] **2.10** Escrever teste: `alterar_ip` com IP inválido retorna erro → Verify: `cargo test commands::network` passa — `alterar_ip_rejects_invalid_ip_format_before_touching_the_network` chama o command diretamente com IP malformado e confirma `Err` (validação ocorre antes de qualquer chamada a `netsh`/`ping`, então o teste não toca a rede real); testes de `domain::network::validate_ipv4`/`validate_computer_name` cobrem a lógica pura isoladamente ✓
- [x] **2.7** Criar componente `PainelInformacoes.tsx` — grid paginado com busca (igual ao atual) → Verify: `npm run build` (tsc + vite) limpo ✓ `src/components/features/PainelInformacoes.tsx` porta o painel de 714 linhas do legado: três seções `Accordion` (Sistema/Rede IPv4/DNS), painel direito com busca + grid `repeat(auto-fill, minmax(180px, 1fr))` recalculado via `ResizeObserver` (itens por página = colunas × linhas detectadas), listener global de `keydown` que foca o campo de busca em qualquer tecla imprimível se nenhum input estiver focado, paginação Anterior/Próxima. Trouxe `Accordion.tsx` e `Modal.tsx` para esta fatia (originalmente Slice 16) por dependência direta, conforme o princípio de fatia vertical completa do projeto. `PainelInformacoes` recebe `modulos: Modulo[]` (default `[]`, já que não existe sidebar/registro de features ainda) e filtra/ordena `funcionalidades` excluindo a si mesmo
- [x] **2.8** Implementar `EditableField.tsx` (componente shared) — display mode + edit mode com ícones → Verify: `npm run build` limpo ✓ `src/components/shared/EditableField.tsx` — aplica a correção explícita do DESIGN.md: borda completa sutil (`border-gray-700`) em vez da faixa lateral `border-l-4 border-primary-purple` do legado, tornando-se `border-structural-purple` em modo de edição (sem stripe, sem glow); modo display = valor centralizado + botão lápis (ícone SVG igual ao legado); modo edição = input autofocado + botões check/X; Enter salva, Escape cancela (paridade com `EditableField.svelte`)
- [x] **2.11** Conectar editable fields no PainelInformacoes aos commands de rede → Verify: `npm run build` limpo, `cargo build` limpo ✓ `handleSalvarNome` chama `invoke("alterar_nome_computador", { novoNome })`, recarrega info, e se o backend sinalizar necessidade de reinício exibe modal com "Reiniciar Agora"/"Depois" (confirmar chama `invoke("reiniciar_computador")`); `salvarIP` valida preenchimento completo para IP estático (ou todos vazios para DHCP), exige `info.interfaceAtiva`, chama `invoke("alterar_ip", { interfaceName, novoIp, mascara, gateway })`, mostra modal de sucesso, aguarda 2000ms e recarrega (paridade exata com o delay do legado); `salvarDNS` segue o mesmo padrão com `invoke("alterar_dns", { interfaceName, dnsPrimario, dnsSecundario })` e delay de 1500ms. Argumentos em camelCase confirmam a convenção padrão do Tauri v2 (parâmetros Rust `snake_case` expostos como `camelCase` em `invoke()`, sem `rename_all`). Duas `@keyframes` (`modalFadeIn`, `modalSlideIn`) e uma terceira (`fadeIn`, usada pelo próprio `PainelInformacoes`) adicionadas a `src/App.css` — paridade exata com as definições do legado (`Modal.svelte`/`PainelInformacoes.svelte`). `App.tsx` atualizado para renderizar `<PainelInformacoes modulos={[]} />` após login bem-sucedido, substituindo o placeholder "Login realizado com sucesso"

**Done When:** Painel mostra todas as infos do sistema, edição inline de IP/DNS/hostname funciona, sem injeção de comando possível.

---

## Semana 2 — Ativação + Manutenção

### Slice 3: Ativação Windows (KMS + Log Streaming)

- [x] **3.1** Escrever teste: `domain::activation::windows::activate("pro")` chama `cscript slmgr.vbs /ipk W269N-WFGWX-YVC9B-4J6C9-T83GX` → Verify: teste passa com mock adapter ✓ RED confirmado (`error[E0425]: cannot find function activate in module super` — função ainda não existe); criado `ports/cscript.rs` com trait `CscriptRunner` (`async fn run(&self, script_path: &str, args: &[&str]) -> Result<String, String>`, reutilizável por `slmgr.vbs`/`ospp.vbs` nas Slices 3 e 4) e `domain/activation/windows.rs` com `FakeCscriptRunner` (grava chamadas em `Mutex<Vec<(String, Vec<String>)>>`) testando que `activate("pro", &runner)` dispara `cscript slmgr.vbs /ipk W269N-WFGWX-YVC9B-4J6C9-T83GX`
- [x] **3.2** Criar `config/kms.json` com chaves GVLK e servidores KMS (extraídos do `app.go`) → Verify: arquivo JSON válido e legível ✓ `kms.json` criado na raiz (não em `config/` — conforme "Estrutura Final de Pastas" do plano, alongside `auth.hash`, gitignored) com seção `windows` (`kms_server: "kms.msguides.com"` + `keys` para pro/education/enterprise/home, extraídos de `AtivarWindows` em `app.go:225-259`); seção `office` fica para o item 4.4; validado com `JSON.parse` via Node — bem formado
- [x] **3.3** Implementar `config/mod.rs` — carrega `kms.json` em runtime, permite editar sem recompilar → Verify: alterar JSON muda comportamento ✓ RED confirmado (`error[E0425]: cannot find function load_kms_config`) com 4 testes via arquivo temporário (`load_kms_config_reads_windows_server_and_keys`, `..._reflects_edits_without_recompiling` — prova que o comportamento muda conforme o conteúdo do JSON, sem recompilar —, `..._errors_when_file_missing`, `..._errors_on_malformed_json`); implementado `KmsConfig`/`WindowsKmsConfig` (`serde::Deserialize`) + `load_kms_config(path: &Path) -> Result<KmsConfig, String>` via `serde_json::from_str`; após implementar, único erro de compilação remanescente no crate é o RED já conhecido e esperado de `domain::activation::windows::activate` (item 3.1, ainda não implementado) — confirma que o loader de config em si está correto; confirmado: os 4 testes passam como parte dos 62/62 do `cargo test` após o item 3.5 restaurar a compilação do crate
- [x] **3.4** Implementar `adapters/cscript.rs` — executa `cscript slmgr.vbs` com args separados → Verify: executa sem shell intermediário ✓ `WinCscriptRunner` implementa o trait `CscriptRunner` delegando para `adapters::process::run("cscript", &[full_path, ...args])` (argv literal, mesma garantia de não-injeção já provada em `adapters::process`); `resolve_script_path` resolve `%SystemRoot%\System32\{script}` (testado contra o valor real de `SystemRoot` no ambiente); teste de execução real usa um script inexistente (`script-que-nao-existe-bg-suptec.vbs`) para exercitar o caminho completo (binário `cscript`, resolução de path, passagem de args) sem nunca invocar `slmgr.vbs`/`ospp.vbs` de verdade (evita ativação real/efeitos colaterais durante os testes); `cargo test` confirma zero erros novos — único erro remanescente era o RED já conhecido de `domain::activation::windows::activate` (item 3.1); confirmado: os 2 testes passam como parte dos 62/62 após o item 3.5
- [x] **3.5** Implementar `domain/activation/windows.rs` — fluxo: /ipk → /skms → /ato com log streaming via Tauri events → Verify: ativação funciona ✓ `activate(versao, keys: &HashMap<String,String>, kms_server: &str, runner: &impl CscriptRunner, on_log: impl Fn(&str)) -> bool` — chaves/servidor vêm do `config::KmsConfig` carregado (item 3.3), não hardcoded, preservando o princípio de externalização; fluxo `/ipk` → `/skms` → `/ato` espelha exatamente `runCommandAndLog`/`AtivarWindows` do `app.go` legado: erros em `/ipk`/`/skms` apenas geram aviso de log e não abortam o fluxo (`run_and_log`), só o resultado de `/ato` determina o `bool` de sucesso retornado; `on_log` é um callback puro (sem dependência de `tauri::Window` no domínio — a ponte com eventos Tauri fica para o `commands/activation.rs` no item 3.9, mantendo o domínio testável e desacoplado), permitindo streaming de log linha a linha. 7 testes cobrindo: chave correta via `/ipk`, servidor via `/skms`, sucesso via `/ato`, falha via `/ato`, edição inválida (não dispara nenhum comando), e mensagens de progresso via callback (estende e substitui o teste RED do item 3.1, mesma asserção central preservada). `cargo test` (lib): **62/62 passando** — compilação do crate restaurada, incluindo os testes de `config` (3.3) e `adapters::cscript` (3.4) que ficaram bloqueados aguardando esta implementação
- [x] **3.6** Implementar sistema de eventos Tauri (substitui Wails EventsEmit) — `tauri::Window::emit()` → Verify: frontend recebe logs em tempo real ✓ criado `src-tauri/src/events.rs` com nomes de evento centralizados como constantes tipadas (`LOG_ATIVACAO_WINDOWS = "log:ativacao:windows"`, `ATIVACAO_WINDOWS_FINALIZADO = "ativacao:windows:finalizado"` — idênticos aos do `app.go`/`AtivacaoWindows.svelte` legados) em vez de strings mágicas espalhadas pelos commands, e `emit_log(window, event_name, message)` usando o trait `tauri::Emitter` (disponível em Tauri v2 sem feature flag extra — confirmado via `cargo build` limpo) com semântica fire-and-forget idêntica ao `emitLogRunner` legado (falha de emit é reportada em stderr, não aborta o fluxo). `cargo build` confirma compilação correta; verificação ponta-a-ponta de "frontend recebe logs em tempo real" só é possível com janela real rodando, então fica confirmada ao final do item 3.9 quando `commands/activation.rs` consumir este módulo e o componente React (3.7/3.8) escutar o evento
- [x] **3.7** Criar hook `useLogEvent.ts` no React — escuta eventos Tauri e appenda ao LogPanel → Verify: logs aparecem em tempo real ✓ criado `src/lib/events.ts` (constantes tipadas espelhando `events.rs`, adiantado de 16.8 pelo mesmo motivo que `Accordion.tsx`/`Modal.tsx` foram adiantados de 16.1/16.5 no Slice 2 — checklist 16.x permanece para revisão final consolidada) e `src/hooks/useLogEvent.ts`, que usa `listen<string>()` de `@tauri-apps/api/event` (API real do Tauri v2, confirmada em `node_modules/@tauri-apps/api/event.d.ts`) e mantém o callback numa ref para não reinscrever a cada render, com proteção contra corrida (cancelamento se o cleanup ocorrer antes da Promise de `listen` resolver) — substitui `EventsOn`/`EventsOff` do Wails legado; `npx tsc --noEmit` limpo. Verificação ponta-a-ponta de "logs aparecem em tempo real" confirmada ao final de 3.8/3.9 com o componente real montado
- [x] **3.8** Criar componente `AtivacaoWindows.tsx` com seleção de versão + LogPanel → Verify: visual idêntico ao atual, logs streamam ✓ adiantados de Slice 16 (mesmo padrão de 3.7): `src/components/shared/LogPanel.tsx` (porta fiel de `LogPanel.svelte` — auto-scroll via `useEffect` em vez da action `use:autoscroll` do Svelte, mesmas classes Tailwind), `BotaoVoltar.tsx` e `FeatureContainer.tsx` (portas diretas, classes idênticas às legadas trocando `primary-purple` pelo token atual `structural-purple` do `DESIGN.md`). `AtivacaoWindows.tsx` reproduz `AtivacaoWindows.svelte` campo a campo: mesmo texto, mesmo seletor de versão (pro/home/education/enterprise), `useLogEvent(EVENTOS.logAtivacaoWindows, adicionarLog)` substituindo `EventsOn`/`EventsOff`, e `listen(EVENTOS.ativacaoWindowsFinalizado, …)` substituindo `EventsOnce` para aguardar a conclusão antes de liberar o botão — mesma sequência `invoke → aguarda evento de finalização → libera UI` do legado. Componente plugado em `App.tsx` via o `onNavigate` já existente em `PainelInformacoes` (callback que troca a view para "Ativação do Windows", com `onVoltar` retornando ao painel) — não foi necessário antecipar o Sidebar/MainView completos do Slice 16 para isso. `npx tsc --noEmit` e `npm run build` limpos. Observação de honestidade: a app exige `requireAdministrator` (item 0.5), então `cargo tauri dev` não pode ser lançado de forma não-interativa neste ambiente (precisa do prompt UAC); a confirmação visual em janela real fica pendente de checagem manual pelo usuário — toda a verificação automatizada possível (tipos, build, fluxo de eventos) foi feita
- [x] **3.9** Implementar `commands/activation.rs` (Tauri command `ativar_windows`) → Verify: invocável do frontend ✓ `#[tauri::command] pub async fn ativar_windows(window: tauri::Window, versao: String) -> Result<bool, String>` carrega `kms.json` via `kms_config_path()` (resolvido ao lado do `.exe`, mesmo padrão de `auth_hash_path` em `lib.rs`), instancia `WinCscriptRunner` (item 3.4) e chama `domain::activation::windows::activate` (item 3.5) com um closure `on_log` que faz `events::emit_log(&window, LOG_ATIVACAO_WINDOWS, msg)` — a ponte entre o domínio puro e o `tauri::Window` real fica isolada aqui, como planejado desde o item 3.5; ao final emite `ATIVACAO_WINDOWS_FINALIZADO` com o booleano de sucesso via novo helper `events::emit_finalizado` (payload tipado bool, não string — diferente de `emit_log` — reutilizável pelo Slice 4/Office que segue o mesmo padrão `log:*`/`*:finalizado`). Registrado em `commands/mod.rs` e no `invoke_handler!` de `lib.rs`. `cargo build` limpo; `cargo test --lib`: **62/62 passando** (sem testes novos para `ativar_windows` em si — like `emit_log`/`emit_finalizado`, exigiria a infra de mock `tauri::test` para construir um `tauri::Window` real, não adicionada nesta slice por estar fora do escopo planejado; a lógica de negócio que o comando orquestra já está 100% coberta pelos 7 testes de `activate` e pelos testes de `cscript`/`config`)

**Done When:** Ativação Windows funciona com KMS do `kms.json`, logs streamam em tempo real, config é editável. ✓ Slice 3 completo (3.1–3.9). Backend: `cargo build` limpo, `cargo test --lib` 62/62. Frontend: `npx tsc --noEmit` e `npm run build` limpos. Pendente apenas confirmação visual manual do usuário em janela elevada (`requireAdministrator` impede automação do `cargo tauri dev` neste ambiente).

---

### Slice 4: Ativação Office (KMS + Fallback + Config)

- [x] **4.1** Escrever teste: `domain::activation::office::find_office_path()` retorna caminho correto → Verify: teste passa ✓ RED confirmado (`error[E0425]: cannot find function find_office_path`); `src-tauri/src/domain/activation/office.rs` criado com `find_office_path(program_files: Option<&str>, program_files_x86: Option<&str>, path_exists: impl Fn(&str) -> bool) -> Result<String, String>` — espelha `findOfficePathGo` do `app.go` legado (varre `Office16`/`Office15` sob `ProgramFiles` depois `ProgramFiles(x86)`, procurando `ospp.vbs`), com `path_exists` injetado para manter a função pura e testável sem tocar o sistema de arquivos real (mesmo padrão de injeção via closure já usado em `on_log: impl Fn(&str)` no `windows.rs`); 4 testes cobrindo Office16/Office15/fallback x86/erro quando não encontrado — `cargo test --lib`: 4/4 passando
- [x] **4.2** Escrever teste: `activate("2016")` tenta 4 servidores KMS em loop até sucesso → Verify: teste passa com mock ✓ RED confirmado (`error[E0425]: cannot find function activate` — o compilador nem confunde com `windows::activate`, já que ainda não existe `office::activate`); `OfficeVersionConfig` (prod_key/unpkeys/license_patterns/kms_servers, espelha `OfficeVersionInfo` do legado) e `FakeCscriptRunner::succeeding_on_host` (só retorna sucesso em `/act` se o `/sethst` mais recente bateu com o host alvo) adicionados a `office.rs`; teste `activate_tries_each_kms_server_in_order_until_one_succeeds` usa os 4 servidores reais do Office 2016 legado e afirma que `/sethst` é chamado para os 4 em ordem, parando no sucesso do último — GREEN fica para o item 4.3
- [x] **4.3** Implementar `domain/activation/office.rs` — find path + taskkill + unpkey + inpkey + setprt + loop KMS servers → Verify: fluxo completo funciona ✓ `ospp.vbs` mora na pasta do Office (não em `%SystemRoot%\System32` como o `slmgr.vbs` do Windows), então `CscriptRunner` não serve; criado `ports::ProcessRunner` (`async fn run(&self, program: &str, args: &[&str], cwd: Option<&str>) -> Result<String, String>`) + adapter `adapters::process::WinProcessRunner` (delega para `run`/nova `run_in_dir`, esta com teste cobrindo `cwd`), espelhando o par `CscriptRunner`/`WinCscriptRunner` da Slice 3; `office::activate` implementado fiel ao `AtivarOffice` legado: taskkill (winword/excel/powerpnt/outlook) → loop `/unpkey` por cada chave antiga → `install_licenses` (varre `Licenses16` depois `Licenses15`, filtra nomes de arquivo por regex via novo crate `regex`, instala via `/inslic`, não-fatal se diretório não existe) → `/inpkey` com a GVLK → `/setprt:1688` → loop pelos servidores KMS tentando `/sethst`+`/act`, checando texto de sucesso ("product activation successful"/"ativado com êxito") case-insensitive; a chamada `/act` roda com `cwd = office_path` (única exceção — espelha `syscmd.RunCommand(officePath, ...)` do legado, todas as outras chamadas usam o diretório corrente, como no Go original); reescrito o teste RED do item 4.2 para usar `FakeProcessRunner` com a assinatura real (estava provisório usando `CscriptRunner`); adicionados 5 novos testes (`taskkill` antes de tudo, ordem `/unpkey`, `/inpkey` correto, `/act` com `cwd` correto, edição desconhecida não roda nada) — `cargo test --lib`: 11/11 em `activation::office`, 74/74 na suíte inteira (sem regressões)
- [x] **4.4** Adicionar chaves Office 2016/2021/2024 ao `kms.json` → Verify: config carrega corretamente ✓ RED confirmado (`error[E0609]: no field 'office' on type 'KmsConfig'`); `OfficeVersionConfig` (em `office.rs`) ganhou `derive(Deserialize)` e é reaproveitada diretamente por `config::OfficeKmsConfig { versions: HashMap<String, OfficeVersionConfig> }` (sem duplicar a struct — única fonte de verdade), `KmsConfig` ganhou campo `office: OfficeKmsConfig`; testes pré-existentes que montavam JSON parcial (`load_kms_config_reads_windows_server_and_keys`, `..._reflects_edits_without_recompiling`) atualizados com `"office": {"versions": {}}` já que o campo passou a ser obrigatório (kms.json real sempre terá as duas seções); novo teste `load_kms_config_reads_office_version_configs` cobre prod_key/unpkeys/license_patterns/kms_servers; `kms.json` da raiz atualizado com as 3 edições (2016/2021/2024) extraídas de `app.go:276-292`, validado com `JSON.parse` via Node — bem formado; `cargo test --lib`: 75/75 (sem regressões)
- [x] **4.5** Implementar `instalar_licencas_office` — lista diretório Licenses16, filtra por regex, instala via cscript → Verify: licenças instaladas ✓ a função `install_licenses` já existia desde o item 4.3 (necessária para `activate` compilar), mas só era exercitada com `dir_exists: |_| false` (pulando o ramo interessante); adicionados 3 testes dedicados cobrindo o que faltava: filtra corretamente por regex (`proplusvl_kms.*\.xrm-ms` casa 2 de 4 arquivos, instala via `/inslic:{caminho_completo}` só para os que casam), fallback de `Licenses16` ausente para `Licenses15`, e o caminho não-fatal quando nenhum diretório de licenças existe (loga aviso, nenhuma chamada `cscript` feita) — `cargo test --lib`: 3/3 novos, 78/78 na suíte inteira
- [x] **4.6** Criar componente `AtivacaoOffice.tsx` com seleção de versão + LogPanel → Verify: visual idêntico, logs streamam ✓ backend: `events.rs` ganhou `LOG_ATIVACAO_OFFICE`/`ATIVACAO_OFFICE_FINALIZADO`; `commands/activation.rs` ganhou `ativar_office` (carrega `kms.json`, resolve `office_path` via `find_office_path` usando `ProgramFiles`/`ProgramFiles(x86)` reais + `Path::exists`, chama `office::activate` com `WinProcessRunner` e closures reais de `dir_exists`/`list_dir` via `std::fs::read_dir`; falha de `find_office_path` emite log de erro + finalizado(false) sem tentar ativar), registrado em `lib.rs`; frontend: `src/lib/events.ts` ganhou `logAtivacaoOffice`/`ativacaoOfficeFinalizado`; `AtivacaoOffice.tsx` criado espelhando `AtivacaoWindows.tsx` 1:1 (mesmo layout/classes Tailwind, `useLogEvent` + `listen().then()` para a promessa de conclusão), com dropdown de versão 2024/2021/2016 (default "2024", ordem do `AtivacaoOffice.svelte` legado) chamando `invoke("ativar_office", { versao })`; `App.tsx` ganhou "Ativação do Office" em `MODULOS` e branch condicional de renderização — `cargo build`: limpo, sem warnings (o `WinProcessRunner` que antes acusava "never constructed" agora está em uso); `npx tsc --noEmit`: sem erros
- [x] **4.7** Implementar health check TCP para servidores KMS antes de tentar (timeout 2s) → Verify: evita timeout longo quando servidores down ✓ escopo limitado à Office (Windows usa um único `kms_server`, sem loop de fallback, então não se aplica); criado `ports::TcpHealthChecker` (`async fn is_reachable(&self, host: &str, port: u16) -> bool`) + adapter `adapters::tcp_health::TokioTcpHealthChecker` (`tokio::net::TcpStream::connect` envolto em `tokio::time::timeout(Duration::from_secs(2), ...)`, com 2 testes próprios usando um `TcpListener` real em porta efêmera — um aceitando conexão, outro derrubado antes do connect); features `net`/`time` adicionadas ao `tokio` no `Cargo.toml`; `office::activate` ganhou parâmetro `health_checker: &impl TcpHealthChecker` — no loop de servidores KMS, cada um é checado na porta 1688 (mesma fixada pelo `/setprt:1688` anterior) antes de `/sethst`+`/act`; se não saudável, loga aviso e pula para o próximo sem rodar nenhum comando cscript; todos os 7 testes pré-existentes de `activate` atualizados com `FakeHealthChecker::all_reachable()` (preserva comportamento anterior) e novo teste dedicado `activate_skips_kms_servers_that_fail_the_tcp_health_check` confirma que os 2 primeiros servidores (marcados não saudáveis) nunca recebem `/sethst`, ativando direto no terceiro; `commands/activation.rs` passa `TokioTcpHealthChecker` real; `cargo build`: limpo sem warnings; `cargo test --lib`: 81/81 (sem regressões)

**Slice 4 completa** — Ativação Office funciona com fallback de 4 servidores KMS (config externalizado em `kms.json`), health check TCP de 2s evita timeouts longos em servidores fora do ar, instalação de licenças KMS via regex, UI espelhando `AtivacaoWindows.tsx`. `cargo build` e `npx tsc --noEmit` limpos; 81/81 testes do `--lib`.

**Done When:** Ativação Office funciona com fallback de servidores, config externalizado, health check evita timeouts.

---

### Slice 5: Limpa Cache DNS

- [x] **5.1** Escrever teste: `domain::maintenance::clear_dns_cache()` chama `ipconfig /flushdns` → Verify: teste passa ✓ RED confirmado (`error[E0425]: cannot find function clear_dns_cache in module super` — módulo `domain/maintenance` ainda não existia); `src-tauri/src/domain/maintenance/mod.rs` criado com `FakeProcessRunner` (mesmo padrão de `domain/activation/office.rs`) e teste `clear_dns_cache_runs_ipconfig_flushdns` afirmando a chamada `ipconfig /flushdns` via `ports::ProcessRunner` (reaproveitado, sem novo port); `clear_dns_cache(runner: &impl ProcessRunner) -> Result<String, String>` implementada minimamente para o GREEN; `domain/mod.rs` ganhou `pub mod maintenance;`; `cargo test --lib`: 82/82 passando (sem regressões)
- [x] **5.2** Implementar `domain/maintenance/mod.rs` + `commands/maintenance.rs` → Verify: comando executa ✓ `domain/maintenance/mod.rs::clear_dns_cache` já existia minimamente do item 5.1; `commands/maintenance.rs` criado com `#[tauri::command] limpar_cache_dns() -> Result<String, String>` — wrapper fino que injeta `WinProcessRunner` real (mesmo padrão de `reiniciar_computador` em `commands/network.rs`); registrado em `commands/mod.rs` (`pub mod maintenance;`) e no `invoke_handler!` de `lib.rs`; `cargo build`: limpo
- [x] **5.3** Criar componente `LimpaCacheDNS.tsx` com LogPanel → Verify: visual idêntico, log aparece ✓ `src/components/features/LimpaCacheDNS.tsx` porta `LimpaCacheDNS.svelte` + `FeatureRunner.svelte` legados (classes Tailwind idênticas ao botão/descrição do `FeatureRunner`), chamando `invoke<string>("limpar_cache_dns")` uma única vez (sem streaming de eventos — comando síncrono e rápido, diferente das ativações), mesma sequência de mensagens de log do legado (comando executado → resultado → sucesso → `--- Operação concluída ---`); `App.tsx` ganhou módulo "Manutenção e Limpeza" (espelha agrupamento do `App.svelte` legado) com a funcionalidade "Limpar Cache DNS" e branch condicional de renderização; `npx tsc --noEmit` e `npm run build` limpos

**Done When:** Limpa cache DNS funciona com log de sucesso. ✓ Slice 5 completo (5.1–5.3).

---

### Slice 6: Limpa e Reinicia Spool de Impressão

- [x] **6.1** Escrever teste: `clear_print_spool()` para spooler, deleta arquivos `.SHD` e `.SPL`, reinicia spooler → Verify: teste passa ✓ RED confirmado (`error[E0425]: cannot find function clear_print_spool`); **achado**: o legado (`LimpaSpoolImpressao.svelte`) só para/reinicia o serviço — nunca de fato exclui os arquivos `.SHD`/`.SPL` travados (esse `git grep` também revelou o bug "Spool acento" do checklist global: `App.svelte` registra a funcionalidade como `"Limpa e Reinicia Spool de Impressão"` mas `MainView.svelte` compara contra `"...Spool de Impressao"` sem acento — o roteamento nunca batia, a feature legada era inalcançável; o port em React não tem esse risco pois usa uma única string compartilhada entre o módulo e o branch condicional); `clear_print_spool(spool_dir, runner, list_dir, delete_file, on_log)` testado com novo fixture `OrderedFakeProcessRunner` (`Arc<Mutex<Vec<String>>>` compartilhado entre chamadas de processo e exclusões de arquivo, provando a ordem exata: `net stop spooler` → exclusões `.SHD`/`.SPL` (case-insensitive) → `net start spooler`) e teste dedicado confirmando que extensões não correspondentes (`.txt`, `.dll`) não são excluídas; `cargo test --lib`: 84/84 (sem regressões)
- [x] **6.2** Implementar comando + criar `LimpaSpoolImpressao.tsx` → Verify: fluxo completo funciona ✓ `clear_print_spool` ganhou retorno `Result<usize, String>` (contagem de arquivos removidos, antes era `Result<(), String>`) com teste atualizado para afirmar `deleted == 3`; `commands/maintenance.rs::limpar_spool_impressao` resolve `spool_dir()` via `%SystemRoot%\System32\spool\PRINTERS` (mesmo padrão de `resolve_script_path` em `adapters/cscript.rs`) e injeta `std::fs::read_dir`/`std::fs::remove_file` reais (mesmo padrão de `dir_exists`/`list_dir` em `commands/activation.rs`); registrado em `lib.rs`; `src/components/features/LimpaSpoolImpressao.tsx` porta `LimpaSpoolImpressao.svelte` (mesmo título/descrição/classes do `FeatureRunner`), chamando `invoke<number>("limpar_spool_impressao")` uma vez e exibindo a contagem de arquivos removidos no log; `App.tsx` ganhou a funcionalidade "Limpar e Reiniciar Spool de Impressão" no módulo "Manutenção e Limpeza"; `cargo test --lib`: 84/84; `npx tsc --noEmit` e `npm run build` limpos

**Done When:** Spool de impressão é limpo e reiniciado com log. ✓ Slice 6 completo (6.1–6.2).

---

### Slice 7: Desativa Hibernação

- [x] **7.1** Escrever teste: `disable_hibernation()` chama `powercfg /h off` → Verify: teste passa ✓ RED confirmado (`error[E0425]: cannot find function disable_hibernation`); adicionada a `domain/maintenance/mod.rs` (mesmo agrupamento de `clear_dns_cache`/`clear_print_spool` — todas funcionalidades de "Manutenção e Limpeza" do legado); nota: legado usa `powercfg -h off`, checklist pede `/h` — ambos os prefixos são aceitos pelo `powercfg`, seguido o texto literal do plano; `cargo test --lib`: 85/85 (sem regressões)
- [x] **7.2** Implementar comando + criar `DesativaHibernacao.tsx` → Verify: funciona ✓ `commands/maintenance.rs::desativar_hibernacao` — wrapper fino sobre `disable_hibernation` com `WinProcessRunner` real, registrado em `lib.rs`; `src/components/features/DesativaHibernacao.tsx` porta `DesativaHibernacao.svelte` campo a campo (mesmo título/descrição/sequência de log incluindo a menção ao `hiberfil.sys`); `App.tsx` ganhou "Desativar Hibernação do Windows" no módulo "Manutenção e Limpeza"; `cargo build` limpo, `npx tsc --noEmit` e `npm run build` limpos

**Done When:** Hibernação desativada com log. ✓ Slice 7 completo (7.1–7.2).

---

### Slice 8: Ajustar Hora da Formatação (NTP + InstallDate)

- [x] **8.1** Escrever teste: `adjust_formatting_time()` configura w32time + NTP + atualiza InstallDate no registro → Verify: teste passa ✓ RED confirmado (`error[E0425]: cannot find function adjust_formatting_time`); novo port `ports::RegistryWriter` (trait separado de `RegistryReader` — assim os fakes somente-leitura existentes em `domain::system::tests` não precisam de um método write no-op só para continuar compilando), implementado por `WinRegistryReader` (mesma struct, duas traits) em `adapters/registry.rs` via `winreg::create_subkey`/`set_value`; `domain/system/time.rs::adjust_formatting_time(runner, registry, now_unix: u32, on_log)` testado com `FakeProcessRunner`+`FakeRegistryWriter` cobrindo a sequência completa (`sc config w32time start=auto` → `w32tm /config ...` → `net stop/start w32time` → escrita de `InstallDate`); **achado 1**: o legado passava `/manualpeerlist:\"pool.ntp.br\"` com aspas literais incorporadas (resquício de quoting de shell sem efeito real, e potencialmente um bug latente — as aspas viram parte do valor configurado) — removidas aqui já que (como em todo o resto do código) a chamada é argv literal sem shell intermediário; **achado 2**: `runCommandAndLog` do legado nunca aborta a sequência em erro de um passo (`"AVISO: ... (pode ser normal)"`, mesma semântica já usada por `domain::activation::windows::activate`'s `run_and_log`) — versão inicial desta função usava `?` e abortava no primeiro erro; corrigido para um `run_and_log` local não-fatal por passo (incluindo a escrita de `InstallDate`), retorno mudou de `Result<(), String>` para `()` (a função nunca falha como um todo, fiel ao legado); teste extra `adjust_formatting_time_continues_past_a_failing_step` confirma que uma falha em `sc` não impede os passos seguintes; `now_unix` injetado (não lido via `SystemTime::now()` dentro da função) para manter testável; `cargo test --lib`: 87/87 (sem regressões)
- [x] **8.2** Implementar com `adapters/registry.rs` para InstallDate + `adapters/command.rs` para w32tm → Verify: hora ajustada ✓ a escrita do `InstallDate` via `adapters/registry.rs` (`WinRegistryReader::write_local_machine_dword`) e a execução do `w32tm`/`sc`/`net` via `adapters/process.rs` (`WinProcessRunner`, já existente — "adapters/command.rs" do texto do plano mapeia para o `ProcessRunner` já estabelecido, sem criar um adapter novo) já estavam prontos desde o item 8.1; `commands/system_info.rs::ajustar_hora_formatacao(window) -> Result<(), String>` — calcula `now_unix` via `SystemTime::now()`, chama `adjust_formatting_time` com `WinProcessRunner`+`WinRegistryReader` reais e um closure `on_log` que emite `LOG_AJUSTAR_HORA_FORMATACAO` (novo evento em `events.rs`); sem evento `*_FINALIZADO` — diferente das ativações, este fluxo nunca reporta falha como um todo (mesma semântica do domínio), e o próprio retorno do `invoke()` já sinaliza conclusão; registrado em `lib.rs`; `cargo build` e `cargo test --lib` (87/87) limpos
- [x] **8.3** Criar `AjustarHoraFormatacao.tsx` com LogPanel → Verify: logs streamam ✓ `src/lib/events.ts` ganhou `logAjustarHoraFormatacao`; `src/components/features/AjustarHoraFormatacao.tsx` porta `AjustarHoraFormatacao.svelte` campo a campo (mesmo título/descrição/texto do botão), usando `useLogEvent(EVENTOS.logAjustarHoraFormatacao, adicionarLog)` para receber as linhas de log em tempo real durante a execução — diferente do legado (que via `EventsOn`/`EventsOff` e detectava conclusão observando `"---"` na mensagem), aqui a conclusão é sinalizada pela própria resolução da promise de `invoke()` (mais robusto que checar substring, e correto porque o command Rust só retorna depois que `adjust_formatting_time` already concluiu todos os passos); `App.tsx` ganhou o módulo "Reparos e Soluções" (mesmo agrupamento do `App.svelte` legado) com a funcionalidade "Ajustar Hora da Formatação"; `npx tsc --noEmit` e `npm run build` limpos

**Done When:** Hora ajustada com NTP e InstallDate atualizado. ✓ Slice 8 completo (8.1–8.3). Semana 2 do plano concluída (Slices 3–8).

---

## Semana 3 — Reparos + Segurança

### Slice 9: Corrige Compartilhamento Windows (4 etapas)

- [x] **9.1** Escrever teste: `fix_network_sharing()` executa 4 etapas (serviços, firewall, registro, políticas) → Verify: teste passa
- [x] **9.2** Implementar com sequência de comandos: `sc config`, `net start`, `netsh advfirewall`, `reg add`, `gpupdate /force` → Verify: todas etapas executam
- [x] **9.3** Adicionar AVISO na UI sobre redução de segurança SMB (limitblankpassworduse, RequireSecuritySignature) → Verify: modal de aviso aparece antes de executar
- [x] **9.4** Criar `CorrigirCompartilhamento.tsx` com LogPanel + modal de confirmação → Verify: visual idêntico

**Done When:** Compartilhamento corrigido com aviso de segurança explícito.

---

### Slice 10: Ativar Proteção do Sistema

- [x] **10.1** Escrever teste: `enable_system_protection()` chama `Enable-ComputerRestore -Drive 'C:'` → Verify: teste passa
- [x] **10.2** Implementar + criar `AtivarProtecaoSistema.tsx` → Verify: funciona com log correto (corrigir bug de event name do atual)

**Done When:** Proteção do sistema ativada com log visível (bug atual corrigido).

---

### Slice 11: Bloqueador de Firewall

- [x] **11.1** Escrever teste: `block_program_in_firewall(path)` cria regras in/out com `netsh advfirewall` → Verify: teste passa
- [x] **11.2** Escrever teste: `unblock_program_in_firewall(path)` deleta regras → Verify: teste passa
- [x] **11.3** Implementar `domain/security/firewall.rs` com nome de regra `[BG-SupTec] Bloqueio - {exe}` → Verify: regras criadas/deletadas
- [x] **11.4** Implementar `list_installed_programs()` via registry (corrigir handle leak do atual com escopo correto) → Verify: lista programas sem leak
- [x] **11.5** Implementar `list_executables(path)` com `walkdir` crate → Verify: lista .exe recursivamente
- [x] **11.6** Implementar `select_exe_file()` via Tauri dialog API → Verify: dialog abre
- [x] **11.7** Criar `BloqueadorFirewall.tsx` — seleção de exe/programa + toggle bloqueio/desbloqueio + status → Verify: visual idêntico, funciona end-to-end

**Done When:** Firewall bloqueia/desbloqueia programas, lista programas sem handle leak, seleção de exe funciona.

---

### Slice 12: Restaurar Photo Viewer

- [x] **12.1** Escrever teste: `restore_photo_viewer()` escreve 20+ chaves de registro HKCR/HKLM → Verify: teste passa
- [x] **12.2** Implementar com `adapters/registry.rs` — array de structs `RegChange { path, value, type, data }` → Verify: todas chaves escritas
- [x] **12.3** Emitir evento `log:restaurar:photoviewer:finalizado` no fim (corrigir dead promise do atual) → Verify: frontend recebe evento
- [x] **12.4** Criar `RestaurarPhotoViewer.tsx` com LogPanel → Verify: não trava mais (bug atual corrigido)

**Done When:** Photo Viewer restaurado, UI não trava mais, logs completos. ✅

**Done When:** Photo Viewer restaurado, UI não trava mais, logs completos.

---

## Semana 4 — Personalização + Finalização

### Slice 13: Alterar Layout do Teclado

- [ ] **13.1** Escrever teste: `change_keyboard_layout("pt-BR")` chama `Set-WinUserLanguageList -LanguageList pt-BR -Force` com arg sanitizado → Verify: teste passa
- [ ] **13.2** Escrever teste: input malicioso `pt-BR; Start-Process cmd` é rejeitado → Verify: teste passa (não há injeção)
- [ ] **13.3** Implementar `domain/system/keyboard.rs` com validação estrita (allowlist de tags) → Verify: só tags válidas aceitas
- [ ] **13.4** Implementar `get_available_layouts()` e `get_active_layout()` → Verify: retorna layouts
- [ ] **13.5** Portar SVGs de teclados de `shared/teclados/` para React components → Verify: SVGs renderizam
- [ ] **13.6** Criar `AlterarLayoutTeclado.tsx` com seleção + preview SVG → Verify: visual idêntico

**Done When:** Layout do teclado altera com validação estrita, preview SVG funciona, sem injeção possível.

---

### Slice 14: Ativar Gpedit.msc (Home)

- [ ] **14.1** Escrever teste: `enable_gpedit()` executa script de ativação do gpedit no Windows Home → Verify: teste passa
- [ ] **14.2** Implementar comando (este método FALTA no backend atual — implementar do zero) → Verify: gpedit ativado
- [ ] **14.3** Criar `AtivarGpedit.tsx` → Verify: funciona (feature quebrada no atual agora funciona)

**Done When:** Gpedit ativado no Windows Home (feature nova que faltava no atual).

---

### Slice 15: Agendar Desligamento

- [ ] **15.1** Escrever teste: `schedule_shutdown(seconds)` chama `shutdown /s /t {seconds}` → Verify: teste passa
- [ ] **15.2** Escrever teste: `cancel_shutdown()` chama `shutdown /a` → Verify: teste passa
- [ ] **15.3** Implementar `domain/system/power.rs` → Verify: funciona
- [ ] **15.4** Criar `AgendarDesligamento.tsx` com input de tempo + botões agendar/cancelar → Verify: visual idêntico

**Done When:** Desligamento agendável e cancelável.

---

### Slice 16: Componentes Shared + Sidebar + MainView

- [ ] **16.1** Portar `Accordion.tsx` → Verify: expande/colapsa
- [ ] **16.2** Portar `BotaoVoltar.tsx` → Verify: volta para painel
- [ ] **16.3** Portar `FeatureContainer.tsx` → Verify: wrapa feature com título
- [ ] **16.4** Portar `LogPanel.tsx` → Verify: scrolla, mostra logs em tempo real
- [ ] **16.5** Portar `Modal.tsx` — success/error/warning com cores semânticas → Verify: modal abre/fecha
- [ ] **16.6** Criar `Sidebar.tsx` — menu lateral com módulos + funcionalidades (translúcido, backdrop-blur) → Verify: visual idêntico
- [ ] **16.7** Criar `MainView.tsx` com **mapa de componentes** (NÃO if/else chain) → Verify: roteamento por chave funciona
- [ ] **16.8** Criar `lib/events.ts` com constantes de eventos (substitui strings mágicas) → Verify: todos eventos tipados

**Done When:** Todos componentes shared portados, sidebar funciona, roteamento por mapa (não if/else).

---

### Slice 17: Audit Logging

- [ ] **17.1** Escrever teste: `audit::log_action("admin", "alterar_ip", params, "ok")` escreve linha em arquivo → Verify: teste passa
- [ ] **17.2** Implementar `audit/file_logger.rs` — escreve em `%APPDATA%\BG-SupTec\audit.log` com timestamp ISO → Verify: arquivo criado com log
- [ ] **17.3** Implementar rotação de log (arquivo mensal) → Verify: log novo criado no mês seguinte
- [ ] **17.4** Integrar audit logging em todos os commands destrutivos (alterar_ip, alterar_dns, alterar_nome, firewall, registry, shutdown) → Verify: todas ações logadas

**Done When:** Toda ação destrutiva gera log auditável com timestamp, usuário, ação, parâmetros e resultado.

---

### Slice 18: Build Standalone + Docs + Testes Finais

- [ ] **18.1** Configurar `cargo tauri build` para gerar standalone `.exe` sem dependências no diretório → Verify: `.exe` funciona em pasta vazia
- [ ] **18.2** Empacotar `kms.json` e `auth.hash` como arquivos separados alongside do `.exe` → Verify: app lê configs do diretório
- [ ] **18.3** Testar em VM Windows 10 limpa (sem WebView2) → Verify: app direciona para download do WebView2
- [ ] **18.4** Testar em VM Windows 11 → Verify: funciona nativamente
- [ ] **18.5** Testar em Windows Server 2019+ → Verify: funciona com WebView2 instalado
- [ ] **18.6** Testar em Windows 7/8.1 sem WebView2 → Verify: aviso claro sobre necessidade de WebView2 (já não disponível)
- [ ] **18.7** Rodar `cargo test` completo → Verify: todos testes passam
- [ ] **18.8** Rodar `npm run build` + typecheck → Verify: sem erros TypeScript
- [ ] **18.9** Atualizar `README.md` com instruções de build e uso → Verify: documentação completa
- [ ] **18.10** Atualizar `CHECKLIST.md` marcando features portadas → Verify: checklist reflete estado real
- [ ] **18.11** Criar script `build.ps1` equivalente (gera hash argon2 + builda) → Verify: script funciona

**Done When:** Build standalone funciona em Windows 10/11/Server, configs externalizados, testes passam, docs atualizadas.

---

## Estrutura Final de Pastas

```
BG-SupTec/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/          ← Tauri handlers (thin)
│   │   ├── domain/            ← Business logic (pure, testable)
│   │   │   ├── activation/
│   │   │   ├── network/
│   │   │   ├── maintenance/
│   │   │   ├── security/
│   │   │   └── system/
│   │   ├── adapters/          ← Windows API impls
│   │   ├── ports/             ← Trait definitions
│   │   ├── audit/             ← File-based audit logger
│   │   ├── config/            ← kms.json loader
│   │   └── auth/              ← argon2id
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── src/                       ← React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── Login.tsx
│   │   ├── Sidebar.tsx
│   │   ├── MainView.tsx
│   │   ├── features/
│   │   └── shared/
│   ├── hooks/
│   ├── lib/
│   └── styles/
├── kms.json                   ← Config editável (gitignored)
├── auth.hash                  ← Hash argon2id (gitignored)
├── build.ps1                  ← Script de build
├── DESIGN.md                  ← Preservado
├── PRODUCT.md                 ← Preservado
└── README.md                  ← Atualizado
```

## Crates Rust Principais

```toml
[dependencies]
tauri = { version = "2", features = ["dialog"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.5", features = ["process", "fs", "sync"] }
argon2 = "0.5"
winreg = "0.56"
walkdir = "2"
chrono = "0.4"
regex = "1"
```

## Done When (Critério Global)

- [ ] Todas as 15 features funcionais portadas e testadas
- [ ] Build standalone `.exe` + `kms.json` + `auth.hash` funciona em Windows 10/11/Server
- [ ] `cargo test` passa com cobertura de domínio
- [ ] Sem `ExecutarComando` genérico — cada feature tem command tipado
- [ ] Audit logging ativo para todas ações destrutivas
- [ ] Auth com argon2id + rate limiting
- [ ] Config KMS externalizado em `kms.json`
- [ ] Visual idêntico ao atual (DESIGN.md preservado)
- [ ] Bugs atuais corrigidos: Spool acento, AtivarProtecaoSistema event, RestaurarPhotoViewer dead promise, AtivarGpedit não implementado, handle leak

## Notas

- **Win 7/8.1:** Funciona APENAS se WebView2 já estiver instalado (Microsoft descontinuou suporte jan/2023). App mostra aviso claro se não tiver.
- **Co-existência:** Não há. A versão Go+Wails é substituída quando o Slice 2 (Painel Informações) estiver funcional.
- **KMS legal:** O uso de servidores KMS terceiros (msguides.com) é mantido do atual. Externalizar em config permite trocar por servidor KMS próprio corporativo sem recompilar. Mas não fazer geração de arquivo além do próprio .exe.
