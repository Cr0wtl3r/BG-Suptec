# BG-SupTec

Ferramenta de suporte técnico Windows, refatorada de Go+Wails+Svelte para **Rust+Tauri v2+React 19+TypeScript+Tailwind CSS 4**. A refatoração (`refatoracao-rust-tauri.md`, 18 slices em 4 semanas) está funcionalmente completa — as 15 features portadas, testadas e auditadas. Só falta testar em VMs Windows reais (10/11/Server/7-8.1), que não é executável a partir de um ambiente sandboxed; ver `CHECKLIST.md` para o status por feature.

O código legado está em `legacy_code/` como referência permanente de comportamento e estilo — **não é enviado ao GitHub** (`legacy_code/` está no `.gitignore`), mas continua no disco local e não deve ser apagado nem tratado como lixo a remover.

## Comandos

```bash
# Frontend (Vite dev server, porta 1420)
npm run dev

# App Tauri completo (exige elevação de admin no Windows — prompt UAC)
cargo tauri dev

# Build/typecheck do frontend
npm run build          # roda tsc && vite build

# Backend Rust (a partir de src-tauri/)
cargo build
cargo test --lib       # 128/128 — usar --lib, não `cargo test` puro
cargo clippy --lib

# Build standalone (a partir da raiz)
.\build.ps1 -Senha "suaSenha"    # gera auth.hash, roda cargo tauri build, empacota dist-standalone/
.\build.ps1 -SkipBundle          # idem, pulando instaladores NSIS/MSI (mais rápido)
```

`cargo test` sem `--lib` tenta lançar o harness de teste dos binários `bg-suptec`/`generate_hash`, que herdam o manifest `requireAdministrator` do app — falha com erro 740 em qualquer shell não-elevado/não-interativo, mesmo sem nenhum `#[test]` próprio nesses binários. Não é falha de teste real; sempre usar `cargo test --lib`.

## Arquitetura

- **`src/`** — React + TypeScript + Tailwind (Vite). `App.tsx` renderiza `Sidebar` (menu overlay via hambúrguer) e ou `PainelInformacoes` (view padrão) ou `MainView` (roteia para a feature ativa via `Record<string, ComponentType>` em `MainView.tsx` — nunca adicionar cadeia if/else aqui, estender o mapa).
- **`src-tauri/src/`** — Rust (Tauri v2), arquitetura hexagonal (ports/adapters). `commands/` = wrappers finos `#[tauri::command]` (um comando tipado por feature, nunca um `ExecutarComando` genérico), registrados em `lib.rs`. `domain/` = lógica de negócio pura e testável (`activation/`, `network/`, `maintenance/`, `security/`, `personalization/`, `system/`), sem dependência de `tauri::Window` — callbacks de progresso são closures `impl Fn(&str)` injetadas, só viram eventos Tauri reais na borda de `commands/`. `ports/` = traits (`ProcessRunner`, `RegistryReader`/`Writer`, `AuditWriter` etc.), `adapters/` = implementações reais contra o Windows; testes usam fakes em memória. `audit/` = grava `%APPDATA%\BG-SupTec\audit-YYYY-MM.log` (rotação mensal pelo nome do arquivo), chamado via `audit::record(...)` nos commands destrutivos.

## Design Context

- `PRODUCT.md` — registro, usuários, propósito e princípios de design do produto.
- `DESIGN.md` — sistema de tokens (cores, tipografia, componentes) extraído do legado, mais sidecar `.impeccable/design.json`.
- North Star: "O Painel de Controle (HUD do Técnico)" — superfícies de vidro translúcido sobre o `background.jpg` da estação real; um único acento (#ed5f0c laranja queimado); sem sombras decorativas.

## Princípios (ainda valem para qualquer mudança futura)

1. **Sem `ExecutarComando` genérico** — cada feature tem seu command tipado.
2. **TDD** — teste `cargo test` falhando antes de implementar lógica de domínio; usar ports+fakes em vez de tocar APIs reais do Windows em teste.
3. **Vertical slice** — backend + frontend + teste + integração juntos, não em camadas espalhadas por mudanças não relacionadas.
4. **Nunca interpolar input não confiável em string de comando de shell.** Usar args argv-literal (`process::run(prog, &[...])`) ou, para PowerShell, variáveis de ambiente (`powershell::run_script_with_env`) — `powershell -Command` concatena texto restante de volta na string do script e reanalisa, então "argumentos extras" não são um limite seguro (ver histórico de `adapters/powershell.rs` para a injeção real que isso causou).
5. **Audit logging** — todo command destrutivo chama `audit::record(...)`; se adicionar um novo, integrar aqui também.

## Gotchas

- App (e `generate_hash`, que compartilha o manifest) exige elevação de admin no Windows (`requireAdministrator`, definido em `build.rs`). Isso bloqueia `cargo run`, o lançamento de `cargo tauri dev` e `cargo test` (sem `--lib`) em qualquer shell não-interativo/não-elevado (agentes sandboxed inclusive) com erro 740. Verificação que depende de GUI/elevação sempre precisa de confirmação manual de um humano em sessão interativa elevada.
- `kms.json` e `auth.hash` são gitignored e exigidos em runtime (resolvidos ao lado do executável). Nenhum dos dois existe em um checkout limpo — gerar `auth.hash` com `generate_hash` e copiar um `kms.json` antes de rodar `cargo tauri dev`.
- OS alvo: Windows 10 (1809+), 11, Server 2016+; Windows 7/8.1 só se o WebView2 já estiver instalado (Microsoft parou de distribuí-lo para essas versões em jan/2023).
- `tsconfig.json` tem `noUnusedLocals`/`noUnusedParameters` ativos — código morto quebra a build.
