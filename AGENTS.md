# BG-SupTec

Windows support tool, migrated from Go+Wails+Svelte to **Rust+Tauri v2+React 19+TypeScript+Tailwind CSS 4**. The migration (`refatoracao-rust-tauri.md`, 18 slices across 4 weeks) is functionally complete — all 15 features ported, tested, and audited. Only real-VM testing (Windows 10/11/Server/7-8.1) remains, which can't be done from a dev sandbox; see `CHECKLIST.md` for per-feature status. Legacy code stays in `legacy_code/` as a permanent behavior/style reference — do not delete it, and do not treat it as dead weight to remove.

## Commands

```bash
# Frontend only (Vite dev server, port 1420)
npm run dev

# Tauri app (Rust+React, requires admin elevation on Windows — UAC prompt)
cargo tauri dev

# Frontend build check
npm run build          # runs tsc && vite build

# Rust checks (from src-tauri/)
cargo build
cargo test --lib       # 128/128 as of Slice 18 — use --lib, not plain `cargo test`
cargo clippy --lib

# Standalone release build (from repo root)
.\build.ps1 -Senha "suaSenha"    # generates auth.hash, runs cargo tauri build, packages dist-standalone/
.\build.ps1 -SkipBundle          # same, but skips NSIS/MSI installer generation (faster iteration)
```

`cargo test` (without `--lib`) also tries to launch the test harness for the `bg-suptec`/`generate_hash` **binaries**, which inherit the app's `requireAdministrator` manifest — this fails with error 740 in any non-elevated/non-interactive shell even though neither binary has any `#[test]` of its own. Not a real test failure; always use `cargo test --lib`.

No lint/formatter beyond `tsc` (inside `npm run build`) and `cargo clippy`. No CI workflows exist yet.

## Architecture

Two codebases in one repo:

- **`src/`** — React 19 + TypeScript + Tailwind CSS 4 frontend (Vite). Entry: `main.tsx` → `App.tsx`. Tailwind tokens via `@theme` in `App.css`. `App.tsx` renders `Sidebar` (hamburger-triggered overlay menu) and either `PainelInformacoes` (default view) or `MainView` (routes to whichever feature is active via a `Record<string, ComponentType>` map in `MainView.tsx` — never add an if/else chain here, extend the map).
- **`src-tauri/src/`** — Rust backend (Tauri v2), hexagonal (ports/adapters). Entry: `main.rs` → `lib.rs::run()`.
  - `commands/` — thin `#[tauri::command]` wrappers (one typed command per feature, no generic `ExecutarComando`). Registered in `lib.rs`'s `invoke_handler!`.
  - `domain/` — pure, testable business logic: `activation/` (Windows+Office KMS), `network/`, `maintenance/`, `security/` (firewall + network sharing), `personalization/`, `system/` (info, keyboard, gpedit, power, time). No `tauri::Window` dependency in domain code — progress callbacks are injected as `impl Fn(&str)` closures, wired to real Tauri events only at the `commands/` boundary.
  - `ports/` — trait definitions (`ProcessRunner`, `CscriptRunner`, `RegistryReader`/`RegistryWriter`, `NetworkReader`, `MemoryReader`, `TcpHealthChecker`, `AuditWriter`). `adapters/` implements them against the real Windows APIs; tests use in-memory fakes.
  - `audit/` — `log_action` + `FileAuditWriter`, writes `%APPDATA%\BG-SupTec\audit-YYYY-MM.log` (monthly rotation via filename). Called via `audit::record(action, params, outcome)` from destructive commands (network changes, firewall, registry, shutdown).
  - `auth/` — argon2id + exponential-backoff rate limiter. `config/` — loads `kms.json`.
  - `events.rs` — typed event-name constants (mirrored in `src/lib/events.ts`) instead of magic strings.

## Design System

All UI must conform to `DESIGN.md` and `.impeccable/design.json`. Key rules:

- **Single accent color:** `#ed5f0c` (burnt orange) — action buttons only, one per screen.
- **Glass surfaces:** `dark-blue-light` at 35–95% opacity + `backdrop-blur`, never opaque by default.
- **Fonts:** Segoe UI (body), Consolas (logs only). No second sans-serif family.
- **No decorative shadows:** `shadow-2xl` only on login card and modal.
- Tailwind theme tokens: `dark-blue-bg`, `dark-blue-light`, `accent-orange`, `text-light`, `structural-purple`, `state-success/error/warning`.

## Working in this codebase

The migration itself is done, but these principles still govern any future change:

1. **No generic `ExecutarComando`** — each feature gets its own typed Tauri command.
2. **TDD** — write a failing `cargo test` before implementing domain logic; ports+fakes over touching real Windows APIs in tests.
3. **Vertical slices** — backend + frontend + test + integration together, not layered across unrelated changes.
4. **DESIGN.md preserved** — tokens, colors, typography must stay identical.
5. **Standalone exe** — no runtime dependencies extracted into the directory; only `kms.json` and `auth.hash` live alongside the `.exe`, both externalized (gitignored, edited without recompiling).
6. **Audit logging** — every destructive command calls `audit::record(...)`; if you add a new destructive command, wire it in too.
7. **Never interpolate untrusted input into a shell command string.** Use argv-literal args (`process::run(prog, &[...])`) or, for PowerShell, environment variables (`powershell::run_script_with_env`) — `powershell -Command` concatenates trailing text back into the script and re-parses it, so "extra args" are not a safe boundary (see `adapters/powershell.rs` history for the concrete injection this caused).

## Gotchas

- Vite dev server must run on port 1420 (Tauri expects it; `strictPort: true`).
- `src-tauri/` and `legacy_code/` are excluded from Vite file watching.
- Rust crate name is `bg_suptec_lib` (underscore, not hyphen) due to Windows naming quirks.
- App (and `generate_hash`, which shares the same manifest) requires Windows admin elevation — `requireAdministrator` set via `build.rs`. This blocks `cargo run`, `cargo tauri dev` launch, and `cargo test` (non-`--lib`) in any non-interactive/non-elevated shell (sandboxed agents included) with error 740. GUI/elevation-dependent verification always needs manual confirmation from a human with an interactive elevated session.
- Target OS: Windows 10 (1809+), 11, Server 2016+; Windows 7/8.1 only if WebView2 is already installed (Microsoft stopped distributing it for those in Jan 2023). WebView2 auto-download bootstrapper is configured in `tauri.conf.json` but not verified on a clean VM.
- `tsconfig.json` enables `noUnusedLocals` and `noUnusedParameters` — remove dead code or compilation fails.
- `kms.json` and `auth.hash` are gitignored and required at runtime (resolved next to the running executable). Neither exists in a fresh checkout — generate `auth.hash` with `generate_hash` and copy a `kms.json` before running `cargo tauri dev`.
