# CHECKLIST — Features BG-SupTec (Go+Wails → Rust+Tauri)

Estado real das 15 features funcionais da migração descrita em [`refatoracao-rust-tauri.md`](refatoracao-rust-tauri.md). Todas portadas e cobertas por teste de domínio (`cargo test --lib`, 128/128) salvo nota em contrário.

| # | Feature | Slice | Backend | Frontend | Observação |
|---|---------|-------|:---:|:---:|---|
| 1 | Autenticação (argon2id + rate limiting) | 1 | ✅ | ✅ | Substitui SHA-256 sem salt do legado; backoff exponencial após 5 tentativas |
| 2 | Painel de Informações (sistema + edição inline) | 2 | ✅ | ✅ | IP/DNS/hostname editáveis sem injeção de comando |
| 3 | Ativação do Windows (KMS) | 3 | ✅ | ✅ | Chaves/servidor em `kms.json`, logs em tempo real via eventos Tauri |
| 4 | Ativação do Office (KMS) | 4 | ✅ | ✅ | Fallback de 4 servidores KMS + health check TCP de 2s |
| 5 | Limpar Cache DNS | 5 | ✅ | ✅ | |
| 6 | Limpar e Reiniciar Spool de Impressão | 6 | ✅ | ✅ | **Bug corrigido**: legado tinha mismatch de acentuação entre módulo e rota (`Impressão` vs `Impressao`) — feature era inalcançável |
| 7 | Desativar Hibernação do Windows | 7 | ✅ | ✅ | |
| 8 | Ajustar Hora da Formatação (NTP + InstallDate) | 8 | ✅ | ✅ | |
| 9 | Corrigir Compartilhamento de Rede | 9 | ✅ | ✅ | Aviso explícito de redução de segurança SMB antes de executar |
| 10 | Ativar Proteção do Sistema | 10 | ✅ | ✅ | **Bug corrigido**: legado chamava função Wails inexistente, nunca funcionou |
| 11 | Bloqueador de Programas no Firewall | 11 | ✅ | ✅ | **Bug corrigido**: handle leak na listagem de programas instalados |
| 12 | Restaurar Visualizador de Fotos (Photo Viewer) | 12 | ✅ | ✅ | **Bug corrigido**: promise nunca resolvia (evento de finalização nunca emitido), UI travava |
| 13 | Alterar Layout do Teclado | 13 | ✅ | ✅ | **Vulnerabilidade corrigida**: legado montava comando PowerShell via `fmt.Sprintf` sem sanitização |
| 14 | Ativar Gpedit.msc (Home) | 14 | ✅ | ✅ | **Feature nova**: legado chamava binding Wails nunca implementado, botão nunca funcionou |
| 15 | Agendar Desligamento | 15 | ✅ | ✅ | Preserva tratamento especial do exit code 1116 (`shutdown /a` sem agendamento ativo = sucesso amigável) |

## Infraestrutura transversal

| Item | Status | Observação |
|---|:---:|---|
| Componentes shared (Accordion, BotaoVoltar, FeatureContainer, LogPanel, Modal) | ✅ | Slice 16 |
| Sidebar (menu lateral translúcido) | ✅ | Slice 16 |
| MainView com roteamento por mapa de componentes (não if/else) | ✅ | Slice 16 — corrige o padrão if/else do `MainView.svelte` legado |
| `lib/events.ts` (constantes de evento tipadas) | ✅ | Slice 16 |
| Audit logging (`%APPDATA%\BG-SupTec\audit-YYYY-MM.log`) | ✅ | Slice 17 — cobre alterar IP/DNS/hostname, firewall, restaurar Photo Viewer, agendar/cancelar desligamento |
| Build standalone (`build.ps1` + `cargo tauri build`) | ✅ | Slice 18 — `.exe` + `kms.json` + `auth.hash` lado a lado, sem extração de recursos |
| Testes em VM (Windows 10/11/Server/7-8.1) | ⏳ | Não executável no ambiente de desenvolvimento sandboxed desta migração — pendente verificação manual |

## Critério Global (do plano)

- [x] Todas as 15 features funcionais portadas e testadas
- [x] Sem `ExecutarComando` genérico — cada feature tem command tipado
- [x] Audit logging ativo para todas as ações destrutivas listadas no plano
- [x] Auth com argon2id + rate limiting
- [x] Config KMS externalizado em `kms.json`
- [x] Visual idêntico ao legado (tokens de `DESIGN.md` preservados)
- [x] Bugs do legado corrigidos: Spool acentuação, AtivarProtecaoSistema, RestaurarPhotoViewer (dead promise), AtivarGpedit (não implementado), handle leak no firewall
- [ ] Build standalone validado em Windows 10/11/Server/7-8.1 reais — pendente verificação manual (ambiente sem acesso a VMs)
