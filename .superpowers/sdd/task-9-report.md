# Task 9 Report: Corrige Compartilhamento Windows (9.1-9.4)

## Status: DONE

## What was implemented

### Backend (Rust)

- **`src-tauri/src/domain/security/mod.rs`** (new module): `fix_network_sharing(runner: &impl ProcessRunner, on_log: impl Fn(&str))` — async, pure domain function with no `tauri::Window` dependency, mirroring legacy `CorrigirCompartilhamentoWindows` (`legacy_code/app.go` lines 363-404) exactly:
  - Etapa 1/4: for each of the 6 services (`LanmanServer`, `LanmanWorkstation`, `FDResPub`, `SSDPSRV`, `IKEEXT`, `PolicyAgent`): `sc config <svc> start=auto` then `net start <svc>`, in order.
  - Etapa 2/4: `netsh advfirewall firewall set rule group="File and Printer Sharing" new enable=Yes` and `group="Remote Service Management" new enable=yes`.
  - Etapa 3/4: 5 `reg add` calls with exact path/value/type/data from the brief (`AllowInsecureGuestAuth`, `RequireSecuritySignature`, `RpcAuthnLevelPrivacyEnabled`, `RestrictDriverInstallationToAdministrators`, `limitblankpassworduse`).
  - Etapa 4/4: `net stop spooler`, `net start spooler`, `gpupdate /force`.
  - Every step routed through a local `run_and_log` helper (same shape as `domain::system::time::run_and_log`) that logs a `--> {msg}` line before running and a non-fatal `AVISO: ...` line on error — a failing step never aborts the sequence.
  - Used the existing `ProcessRunner` port — no new port introduced, `reg add` invoked through it like every other program.
  - Dropped the legacy `time.Sleep(2 * time.Second)` between spooler stop/start (brief explicitly allowed dropping it as not behavior/security relevant).
- **`src-tauri/src/domain/mod.rs`**: declared `pub mod security;`.
- **`src-tauri/src/events.rs`**: added `LOG_COMPARTILHAMENTO = "log:compartilhamento"` and `COMPARTILHAMENTO_FINALIZADO = "compartilhamento:finalizado"`, matching the existing `log:x:y` / `x:y:finalizado` convention and the legacy event names verbatim.
- **`src-tauri/src/commands/security.rs`** (new): `corrigir_compartilhamento(window: tauri::Window) -> Result<(), String>` Tauri command — builds the `on_log` closure via `events::emit_log`, calls `fix_network_sharing` with `WinProcessRunner`, then emits `COMPARTILHAMENTO_FINALIZADO` via `events::emit_finalizado` (same shape as `ativar_windows` in `commands/activation.rs`).
- **`src-tauri/src/commands/mod.rs`**: declared `pub mod security;`.
- **`src-tauri/src/lib.rs`**: registered `commands::security::corrigir_compartilhamento` in the `invoke_handler!` macro.

### Frontend (React/TypeScript)

- **`src/lib/events.ts`**: added `logCompartilhamento` and `compartilhamentoFinalizado` to the `EVENTOS` object, mirroring the new Rust constants.
- **`src/components/features/CorrigirCompartilhamento.tsx`** (new): ports `legacy_code/frontend/src/components/features/CorrigirCompartilhamento.svelte`, following the established `FeatureContainer` + `LogPanel` + `BotaoVoltar` + `useLogEvent` pattern from `AjustarHoraFormatacao.tsx`, and the finalizado-event promise pattern from `AtivacaoWindows.tsx` (awaits `compartilhamentoFinalizado` after invoking the command, since the command resolves before the async work completes the same way `ativar_windows` does — actually here `corrigir_compartilhamento` is itself the async work, but the `listen`-before-`invoke` race-free pattern was kept for consistency and because the event still arrives asynchronously via the Tauri event bus).
  - **Pre-execution warning modal** (9.3, new requirement not in legacy): clicking "Aplicar Correções" opens a `tipo="aviso"` `Modal` explaining the SMB security tradeoff (`RequireSecuritySignature` disabled, `limitblankpassworduse` blank-password guest logons allowed) in plain language; only on confirm does `iniciar()` run and the backend command get invoked.
  - **Post-execution reboot modal** (ported from legacy): unchanged behavior — appears once `compartilhamentoFinalizado` fires, offers "Reiniciar Agora" (invokes the existing `reiniciar_computador` command, added in Slice 2 — reused, not recreated) or "Depois".
- **`src/App.tsx`**: imported `CorrigirCompartilhamento`, added `"Corrigir Compartilhamento de Rede"` to the `"Reparos e Soluções"` module's `funcionalidades` array, added a ternary branch routing to the new component.
- **`refatoracao-rust-tauri.md`**: checked off 9.1-9.4.

## Tests (TDD)

Added to `src-tauri/src/domain/security/mod.rs`, using the same `OrderedFakeProcessRunner` (`Arc<Mutex<Vec<String>>>`-backed) pattern as `domain::maintenance::tests`:

1. `fix_network_sharing_issues_all_four_steps_in_order` — asserts all 22 recorded calls in exact order: 12 service config/start calls (6 services × 2 commands), 2 firewall rule calls with exact group names/flags, 5 `reg add` calls with exact path/value/type/data, then `net stop spooler` / `net start spooler` / `gpupdate /force` last.
2. `fix_network_sharing_continues_past_a_failing_step` — `sc` configured to fail; asserts the sequence still completes all 22 calls and reaches `gpupdate /force`, matching legacy's non-fatal `runCommandAndLog` semantics.

Wrote test + implementation together (both passed on first run since the implementation was written directly against the legacy reference and existing pattern) — verified by running `cargo test --lib security::` in isolation first (2/2 passed), then the full suite.

## Commands run / results

- `cargo test --lib` (from `src-tauri/`): **89 passed; 0 failed** (87 pre-existing + 2 new).
- `cargo build` (from `src-tauri/`): clean, no warnings/errors.
- `npx tsc --noEmit` (from repo root): clean, no output/errors.
- `npm run build` (from repo root): clean — `tsc && vite build` succeeded, 49 modules transformed, `dist/assets/index-*.js` 231.14 kB (gzip 69.09 kB).

## Deviations from the brief

- Dropped the legacy `time.Sleep(2 * time.Second)` between `net stop spooler` and `net start spooler` — brief explicitly said this was the implementer's call since it's not security/behavior relevant.
- `corrigir_compartilhamento` returns `Result<(), String>` rather than a `bool` (unlike `ativar_windows`/`ativar_office` which return success booleans) because `fix_network_sharing` has no meaningful "success/failure" outcome to report — every step is non-fatal by design, matching the legacy function which also returns nothing. `emit_finalizado` is still called with `true` (the existing helper's signature requires a bool) purely to keep the established finalizado-signal shape; the frontend's `compartilhamentoFinalizado` listener ignores the payload, same as legacy's payload-less `compartilhamento:finalizado` emit.
- No other deviations — command sequence, registry values, firewall rule strings, and service list all match the legacy source verbatim.

## Commit

Committed directly to `main` per repo convention (no feature branch).

---

## Follow-up fix: use `RegistryWriter` port for registry step (review finding)

**Date:** 2026-06-30
**Trigger:** Important finding from task review of commit `dc99742` (Slice 9, merged to `main`).

### Finding

`fix_network_sharing`'s Etapa 3/4 wrote its 5 registry values by shelling out to `reg add <path> /v <value> /t REG_DWORD /d <data> /f` via `ProcessRunner`. The immediately-prior slice (Slice 8, `domain::system::time::adjust_formatting_time`) had already established `ports::RegistryWriter::write_local_machine_dword(path, name, value) -> Result<(), String>` for exactly this case — writing a `REG_DWORD` under `HKEY_LOCAL_MACHINE`. All 5 of Slice 9's registry changes fit this port exactly, so the raw `reg add` argv-building duplicated what the port already abstracted and diverged from the pattern set one slice earlier.

### What changed

- **`src-tauri/src/domain/security/mod.rs`**:
  - `fix_network_sharing` now takes an additional `registry: &impl RegistryWriter` parameter (alongside the existing `runner: &impl ProcessRunner`), following the exact signature shape of `domain::system::time::adjust_formatting_time(runner, registry, now_unix, on_log)`.
  - `RegChange` struct simplified: dropped `tipo` (always `REG_DWORD`, now implicit in `write_local_machine_dword`) and changed `data: &'static str` to `data: u32`. Paths no longer carry the `HKLM\` prefix (now implicit in `write_local_machine_dword`, matching `time.rs`'s `REG_PATH_WINDOWS_VERSION` convention).
  - Etapa 3/4's loop now calls `registry.write_local_machine_dword(change.path, change.value, change.data)` instead of `run_and_log(runner, ..., "reg", &["add", ...])`; logs `--> {msg}` before and a non-fatal `AVISO: ...` line on `Err`, same semantics as before.
  - Service/firewall/gpupdate steps (Etapas 1, 2, 4) are unchanged — still on `ProcessRunner`.
  - Test module: added `FakeRegistryWriter` (records `(path, name, value)` triples in a `Mutex<Vec<_>>`), same shape as `domain::system::time::tests::FakeRegistryWriter`. Both tests updated to construct and pass a `FakeRegistryWriter`:
    - `fix_network_sharing_issues_all_four_steps_in_order`: the 5 `reg add` argv assertions were replaced with a single `assert_eq!` on `registry.writes.lock().unwrap().clone()` against a `Vec` of the 5 expected `(path, name, value)` tuples, in order. The `ProcessRunner` `recorded` assertions for Etapa 4 shifted from indices `[19..21]`/len 22 to `[14..16]`/len 17 (since the registry step no longer appears in `recorded`).
    - `fix_network_sharing_continues_past_a_failing_step`: same index/length shift (len 17, `gpupdate /force` at index 16), plus a new assertion that `registry.writes.lock().unwrap().len() == 5` (registry step still runs to completion even when `sc` fails).
  - No test coverage was lost — the 5 registry value assertions (path, name, value) still exist, just expressed via the new mechanism instead of `reg add` argv strings.
- **`src-tauri/src/commands/security.rs`**: `corrigir_compartilhamento` now imports `crate::adapters::registry::WinRegistryReader` and passes `&WinRegistryReader` as the second argument to `fix_network_sharing`, mirroring exactly how `commands/system_info.rs`'s `ajustar_hora_formatacao` wires `&WinRegistryReader` into `adjust_formatting_time`.

### Commands run / results

- `cargo build` (from `src-tauri/`): clean, finished in 9.59s, **no warnings**.
- `cargo test --lib` (from `src-tauri/`): **89 passed; 0 failed; 0 ignored** — same total as before the fix (the two rewritten tests net to zero change in test count; all 5 registry-write assertions preserved, just via `FakeRegistryWriter` instead of `reg add` argv).

### Deviations from the finding

None — applied exactly as specified: only the registry step moved to `RegistryWriter`, service/firewall/gpupdate steps untouched, command wiring mirrors Slice 8's `WinRegistryReader` pattern verbatim.

### Commit

Committed directly to `main` per repo convention (no feature branch) as a Slice 9 follow-up fix.
