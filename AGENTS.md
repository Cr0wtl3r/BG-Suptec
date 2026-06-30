# BG-SupTec

Windows support tool migrating from Go+Wails+Svelte → **Rust+Tauri v2+React 19+TypeScript+Tailwind CSS 4**. All legacy code lives in `legacy_code/` for reference. The full incremental migration plan is in `refatoracao-rust-tauri.md`.

## Commands

```bash
# Frontend only (Vite dev server, port 1420)
npm run dev

# Tauri app (Rust+React, requires admin elevation on Windows)
cargo tauri dev

# Frontend build check
npm run build          # runs tsc && vite build

# Rust checks (from src-tauri/)
cargo build
cargo test
cargo clippy
```

No lint, formatter, or typecheck scripts are configured beyond `tsc` inside `npm run build`. No CI workflows exist yet.

## Architecture

Two codebases in one repo:

- **`src/`** — React 19 + TypeScript + Tailwind CSS 4 frontend (Vite). Entry: `main.tsx` → `App.tsx`. Tailwind tokens defined via `@theme` in `App.css`.
- **`src-tauri/`** — Rust backend (Tauri v2). Entry: `main.rs` → `lib.rs`. Modules declared but mostly empty: `commands/`, `domain/`, `adapters/`, `ports/`, `audit/`, `config/`, `auth/`.

The Rust lib uses hexagonal architecture (ports/adapters pattern). Currently only a placeholder `greet` command is wired. Follow the numbered slices in `refatoracao-rust-tauri.md` for what to build next.

## Design System

All UI must conform to `DESIGN.md` and `.impeccable/design.json`. Key rules:

- **Single accent color:** `#ed5f0c` (burnt orange) — action buttons only, one per screen.
- **Glass surfaces:** `dark-blue-light` at 35–95% opacity + `backdrop-blur`, never opaque by default.
- **Fonts:** Segoe UI (body), Consolas (logs only). No second sans-serif family.
- **No decorative shadows:** `shadow-2xl` only on login card and modal.
- Tailwind theme tokens: `dark-blue-bg`, `dark-blue-light`, `accent-orange`, `text-light`, `structural-purple`, `state-success/error/warning`.

## Migration Principles

1. **No generic `ExecutarComando`** — each feature gets its own typed Tauri command.
2. **TDD per feature** — write `cargo test` before implementing.
3. **Vertical slices** — backend + frontend + test + integration, one slice at a time.
4. **DESIGN.md preserved** — tokens, colors, typography identical to legacy.
5. **Standalone exe** — no runtime dependencies in directory; `kms.json` and `auth.hash` externalized.
6. **Audit logging** — every destructive action logs to `%APPDATA%\BG-SupTec\audit.log`.

## Gotchas

- Vite dev server must run on port 1420 (Tauri expects it; `strictPort: true`).
- `src-tauri/` and `legacy_code/` are excluded from Vite file watching.
- Rust crate name is `bg_suptec_lib` (underscore, not hyphen) due to Windows naming quirks.
- App requires Windows admin elevation (manifest in `build.rs` sets `requireAdministrator`).
- Target OS: Windows 10 (1809+), 11, Server 2016+. WebView2 auto-download is configured but untested.
- `tsconfig.json` enables `noUnusedLocals` and `noUnusedParameters` — remove dead code or compilation fails.
