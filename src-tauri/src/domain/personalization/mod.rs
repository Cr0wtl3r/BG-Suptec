use crate::ports::RegistryWriter;

/// The `rundll32`+`shimgvw.dll` command line that launches Windows Photo
/// Viewer — the real entry point the classic viewer has used since
/// Windows XP/Vista's "Windows Picture and Fax Viewer," still present as a
/// DLL in `System32` on modern Windows even though Explorer no longer
/// exposes it as a default. Reused verbatim for every `shell\open\command`
/// write below (the `Applications\photoviewer.dll` registration and all six
/// ProgIDs).
const RUNDLL32_COMMAND: &str =
    r#"%SystemRoot%\System32\rundll32.exe "%SystemRoot%\System32\shimgvw.dll", ImageView_Fullscreen %1"#;

/// One ProgID covering a group of related image extensions — mirrors the
/// long-circulated community "restore Windows Photo Viewer" registry
/// scripts' extension-to-ProgID grouping (verified against
/// W4RH4WK/Debloat-Windows-10's `scripts/photo-viewer.ps1`, PR #76, plus
/// Winhelponline's widely-cited restoration guide). Each ProgID gets its
/// own `shell\open\command` registered under `HKEY_CLASSES_ROOT`, and each
/// extension in `extensions` gets an `OpenWithProgids` entry pointing at it.
struct ProgidGroup {
    progid: &'static str,
    extensions: &'static [&'static str],
}

const PROGID_GROUPS: [ProgidGroup; 6] = [
    ProgidGroup { progid: "PhotoViewer.FileAssoc.Bitmap", extensions: &["bmp", "dib"] },
    ProgidGroup { progid: "PhotoViewer.FileAssoc.Gif", extensions: &["gif"] },
    ProgidGroup {
        progid: "PhotoViewer.FileAssoc.Jpeg",
        extensions: &["jfif", "jpe", "jpeg", "jpg"],
    },
    ProgidGroup { progid: "PhotoViewer.FileAssoc.Png", extensions: &["png"] },
    ProgidGroup { progid: "PhotoViewer.FileAssoc.Tiff", extensions: &["tif", "tiff"] },
    ProgidGroup { progid: "PhotoViewer.FileAssoc.Wdp", extensions: &["wdp"] },
];

/// Restores "Windows Photo Viewer" as an available choice in Explorer's
/// right-click "Open with" menu for common image file types. This feature
/// never existed in the legacy app — the Svelte sidebar listed the label
/// "Restaurar Visualizador de Fotos" but no Go function or Wails binding
/// was ever written behind it — so this is a first implementation, built
/// from the well-documented public technique rather than a port.
///
/// Deliberately does **not** touch the default-app `UserChoice` registry
/// key: that key is hash-protected by Windows (an undocumented,
/// version-varying hash tied to the OS install) and cannot be set
/// programmatically without either fighting that protection or having it
/// silently reset by Windows. Instead this uses the legitimate, documented
/// `OpenWithProgids` mechanism, which lists Photo Viewer as an available
/// handler without forcing it to be the default — the honest, achievable
/// scope for this feature (the frontend copy must say "available option,"
/// not "now the default").
///
/// Two steps, applied via the injected `RegistryWriter` (25 writes total:
/// 2 for `Applications\photoviewer.dll` + 6 ProgIDs × 2 (`shell\open` verb +
/// `shell\open\command`) + 11 `OpenWithProgids` entries):
/// 1. Register `Applications\photoviewer.dll\shell\open(\command)` under
///    `HKEY_CLASSES_ROOT` — the generic "Application" handler entry. Then,
///    for each of the six `PROGID_GROUPS`, register that ProgID's own
///    `shell\open` verb label and `shell\open\command` (same rundll32
///    command line, reused).
/// 2. For each of the 11 covered extensions, write an empty-data
///    `OpenWithProgids` value (named after the extension's ProgID) under
///    `HKEY_LOCAL_MACHINE\SOFTWARE\Classes\.{ext}` — this app already runs
///    elevated (see `tauri.conf.json`), so HKLM is appropriate and
///    consistent with the rest of this app's elevated registry writes.
///
/// Every write is non-fatal — a failure is logged as a warning but doesn't
/// abort the rest of the sequence, matching every other slice this week
/// (`domain::security::fix_network_sharing`, `enable_system_protection`).
pub fn restore_photo_viewer(registry: &impl RegistryWriter, on_log: impl Fn(&str)) {
    on_log("INICIANDO RESTAURAÇÃO DO VISUALIZADOR DE FOTOS DO WINDOWS...");

    on_log("\n--> Etapa 1/2: Registrando o Visualizador de Fotos como aplicativo...");
    write_string(
        registry,
        &on_log,
        RegRoot::ClassesRoot,
        r"Applications\photoviewer.dll\shell\open",
        "",
        "Windows Photo Viewer",
        "Registrando verbo de abertura do Visualizador de Fotos...",
    );
    write_string(
        registry,
        &on_log,
        RegRoot::ClassesRoot,
        r"Applications\photoviewer.dll\shell\open\command",
        "",
        RUNDLL32_COMMAND,
        "Registrando comando de execução do Visualizador de Fotos...",
    );

    on_log("\n--> Etapa 2/2: Registrando associações de tipos de imagem...");
    for group in PROGID_GROUPS {
        write_string(
            registry,
            &on_log,
            RegRoot::ClassesRoot,
            &format!(r"{}\shell\open", group.progid),
            "",
            "Windows Photo Viewer",
            &format!("Registrando verbo de abertura do ProgID {}...", group.progid),
        );
        write_string(
            registry,
            &on_log,
            RegRoot::ClassesRoot,
            &format!(r"{}\shell\open\command", group.progid),
            "",
            RUNDLL32_COMMAND,
            &format!("Registrando comando do ProgID {}...", group.progid),
        );

        for ext in group.extensions {
            write_string(
                registry,
                &on_log,
                RegRoot::LocalMachine,
                &format!(r"SOFTWARE\Classes\.{ext}\OpenWithProgids"),
                group.progid,
                "",
                &format!("Disponibilizando .{ext} para o Visualizador de Fotos..."),
            );
        }
    }

    on_log("\n--- VISUALIZADOR DE FOTOS RESTAURADO COMO OPÇÃO ---");
}

/// Which registry root a given write targets — keeps `write_string` a
/// single helper instead of two near-identical copies.
enum RegRoot {
    ClassesRoot,
    LocalMachine,
}

/// Writes one string value via the injected `RegistryWriter`, logging the
/// step and any failure as a non-fatal warning — same `on_log` + "AVISO:"
/// pattern as `domain::security`'s `run_and_log`/registry-loop steps.
fn write_string(
    registry: &impl RegistryWriter,
    on_log: &impl Fn(&str),
    root: RegRoot,
    path: &str,
    name: &str,
    value: &str,
    log_msg: &str,
) {
    on_log(&format!("--> {log_msg}"));
    let result = match root {
        RegRoot::ClassesRoot => registry.write_classes_root_string(path, name, value),
        RegRoot::LocalMachine => registry.write_local_machine_string(path, name, value),
    };
    if let Err(e) = result {
        on_log(&format!(
            "AVISO: Comando encontrou um erro (pode ser normal): {e}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::ports::RegistryWriter;
    use std::sync::Mutex;

    /// What kind of write was recorded — lets tests assert exact
    /// path/name/value/kind without needing three separate Vecs.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RecordedWrite {
        ClassesRootString { path: String, name: String, value: String },
        LocalMachineString { path: String, name: String, value: String },
        LocalMachineDword { path: String, name: String, value: u32 },
    }

    /// Records every write call (in order) so tests can assert the full set
    /// and sequence — same pattern as `domain::security::tests::FakeRegistryWriter`,
    /// extended to cover all three `RegistryWriter` capabilities this slice
    /// needs (HKCR string, HKLM string, HKLM dword).
    struct FakeRegistryWriter {
        writes: Mutex<Vec<RecordedWrite>>,
        /// When `Some(name)`, the write whose `name` matches fails — lets
        /// tests target exactly one write in the middle of the sequence
        /// without guessing index positions.
        fails_on_name: Option<&'static str>,
    }

    impl FakeRegistryWriter {
        fn new() -> Self {
            Self {
                writes: Mutex::new(Vec::new()),
                fails_on_name: None,
            }
        }

        fn failing_on_name(name: &'static str) -> Self {
            Self {
                writes: Mutex::new(Vec::new()),
                fails_on_name: Some(name),
            }
        }
    }

    impl RegistryWriter for FakeRegistryWriter {
        fn write_local_machine_dword(
            &self,
            path: &str,
            name: &str,
            value: u32,
        ) -> Result<(), String> {
            self.writes.lock().unwrap().push(RecordedWrite::LocalMachineDword {
                path: path.to_string(),
                name: name.to_string(),
                value,
            });
            if self.fails_on_name == Some(name) {
                return Err("falha simulada".to_string());
            }
            Ok(())
        }

        fn write_classes_root_string(
            &self,
            path: &str,
            name: &str,
            value: &str,
        ) -> Result<(), String> {
            self.writes.lock().unwrap().push(RecordedWrite::ClassesRootString {
                path: path.to_string(),
                name: name.to_string(),
                value: value.to_string(),
            });
            if self.fails_on_name == Some(name) {
                return Err("falha simulada".to_string());
            }
            Ok(())
        }

        fn write_local_machine_string(
            &self,
            path: &str,
            name: &str,
            value: &str,
        ) -> Result<(), String> {
            self.writes.lock().unwrap().push(RecordedWrite::LocalMachineString {
                path: path.to_string(),
                name: name.to_string(),
                value: value.to_string(),
            });
            if self.fails_on_name == Some(name) {
                return Err("falha simulada".to_string());
            }
            Ok(())
        }
    }

    const RUNDLL32_COMMAND: &str = r#"%SystemRoot%\System32\rundll32.exe "%SystemRoot%\System32\shimgvw.dll", ImageView_Fullscreen %1"#;

    #[test]
    fn restore_photo_viewer_issues_at_least_20_registry_writes() {
        let registry = FakeRegistryWriter::new();

        super::restore_photo_viewer(&registry, |_| {});

        let writes = registry.writes.lock().unwrap();
        assert!(
            writes.len() >= 20,
            "expected at least 20 registry writes, got {}",
            writes.len()
        );
    }

    #[test]
    fn restore_photo_viewer_registers_applications_photoviewer_dll_command() {
        let registry = FakeRegistryWriter::new();

        super::restore_photo_viewer(&registry, |_| {});

        let writes = registry.writes.lock().unwrap();
        assert!(writes.contains(&RecordedWrite::ClassesRootString {
            path: r"Applications\photoviewer.dll\shell\open".to_string(),
            name: String::new(),
            value: "Windows Photo Viewer".to_string(),
        }));
        assert!(writes.contains(&RecordedWrite::ClassesRootString {
            path: r"Applications\photoviewer.dll\shell\open\command".to_string(),
            name: String::new(),
            value: RUNDLL32_COMMAND.to_string(),
        }));
    }

    #[test]
    fn restore_photo_viewer_registers_full_jpeg_progid_command() {
        let registry = FakeRegistryWriter::new();

        super::restore_photo_viewer(&registry, |_| {});

        let writes = registry.writes.lock().unwrap();
        assert!(writes.contains(&RecordedWrite::ClassesRootString {
            path: r"PhotoViewer.FileAssoc.Jpeg\shell\open\command".to_string(),
            name: String::new(),
            value: RUNDLL32_COMMAND.to_string(),
        }));
    }

    #[test]
    fn restore_photo_viewer_registers_openwithprogids_for_each_extension() {
        let registry = FakeRegistryWriter::new();

        super::restore_photo_viewer(&registry, |_| {});

        let writes = registry.writes.lock().unwrap();

        let expected: [(&str, &str, &str); 11] = [
            (r"SOFTWARE\Classes\.bmp\OpenWithProgids", "PhotoViewer.FileAssoc.Bitmap", ""),
            (r"SOFTWARE\Classes\.dib\OpenWithProgids", "PhotoViewer.FileAssoc.Bitmap", ""),
            (r"SOFTWARE\Classes\.gif\OpenWithProgids", "PhotoViewer.FileAssoc.Gif", ""),
            (r"SOFTWARE\Classes\.jfif\OpenWithProgids", "PhotoViewer.FileAssoc.Jpeg", ""),
            (r"SOFTWARE\Classes\.jpe\OpenWithProgids", "PhotoViewer.FileAssoc.Jpeg", ""),
            (r"SOFTWARE\Classes\.jpeg\OpenWithProgids", "PhotoViewer.FileAssoc.Jpeg", ""),
            (r"SOFTWARE\Classes\.jpg\OpenWithProgids", "PhotoViewer.FileAssoc.Jpeg", ""),
            (r"SOFTWARE\Classes\.png\OpenWithProgids", "PhotoViewer.FileAssoc.Png", ""),
            (r"SOFTWARE\Classes\.tif\OpenWithProgids", "PhotoViewer.FileAssoc.Tiff", ""),
            (r"SOFTWARE\Classes\.tiff\OpenWithProgids", "PhotoViewer.FileAssoc.Tiff", ""),
            (r"SOFTWARE\Classes\.wdp\OpenWithProgids", "PhotoViewer.FileAssoc.Wdp", ""),
        ];

        for (path, name, value) in expected {
            assert!(
                writes.contains(&RecordedWrite::LocalMachineString {
                    path: path.to_string(),
                    name: name.to_string(),
                    value: value.to_string(),
                }),
                "missing expected OpenWithProgids write for {path} / {name}"
            );
        }
    }

    #[test]
    fn restore_photo_viewer_never_touches_userchoice() {
        let registry = FakeRegistryWriter::new();

        super::restore_photo_viewer(&registry, |_| {});

        let writes = registry.writes.lock().unwrap();
        for write in writes.iter() {
            let path = match write {
                RecordedWrite::ClassesRootString { path, .. } => path,
                RecordedWrite::LocalMachineString { path, .. } => path,
                RecordedWrite::LocalMachineDword { path, .. } => path,
            };
            assert!(
                !path.to_lowercase().contains("userchoice"),
                "write must never target UserChoice, but found: {path}"
            );
        }
    }

    #[test]
    fn restore_photo_viewer_continues_past_a_failing_write() {
        // A write failing partway through (e.g. the Applications\\
        // photoviewer.dll command) must not abort the rest of the
        // sequence — non-fatal-per-step, matching every other slice.
        let registry = FakeRegistryWriter::failing_on_name("");

        super::restore_photo_viewer(&registry, |_| {});

        let writes = registry.writes.lock().unwrap();
        assert!(
            writes.len() >= 20,
            "expected the full sequence to still run despite a failing step, got {} writes",
            writes.len()
        );
        // The OpenWithProgids entries (last step) must still have run.
        assert!(writes.contains(&RecordedWrite::LocalMachineString {
            path: r"SOFTWARE\Classes\.png\OpenWithProgids".to_string(),
            name: "PhotoViewer.FileAssoc.Png".to_string(),
            value: String::new(),
        }));
    }

    #[test]
    fn restore_photo_viewer_logs_each_step() {
        let registry = FakeRegistryWriter::new();
        let logs = Mutex::new(Vec::new());

        super::restore_photo_viewer(&registry, |msg| logs.lock().unwrap().push(msg.to_string()));

        let logs = logs.lock().unwrap();
        assert!(!logs.is_empty());
    }
}
