use serde::{Deserialize, Serialize};

use crate::ports::{ProcessRunner, RegistryWriter};

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryHive {
    ClassesRoot,
    CurrentUser,
    LocalMachine,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowsUpdateMode {
    Disable,
    NotificationsOnly,
    Enable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DefenderPolicy {
    Enable,
    Disable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PowerProfile {
    Balanced,
    HighPerformance,
}

const WINDOWS_UPDATE: &str = r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate";
const WINDOWS_UPDATE_AU: &str = r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU";
const WINDOWS_SELF_HOST: &str = r"SOFTWARE\Microsoft\WindowsSelfHost";
const WINDOWS_SELF_HOST_AUTO_UPDATE: &str = r"SOFTWARE\Microsoft\WindowsSelfHost\AutoUpdate";

pub async fn configure_windows_update(
    runner: &impl ProcessRunner,
    registry: &impl RegistryWriter,
    mode: WindowsUpdateMode,
    on_log: impl Fn(&str),
) {
    match mode {
        WindowsUpdateMode::Disable => {
            on_log("Desativando Windows Update e políticas relacionadas...");
            write_hklm_dword(registry, WINDOWS_SELF_HOST, "NoAutoUpdate", 1, &on_log);
            write_hklm_dword(registry, WINDOWS_SELF_HOST, "EnableAutoUpdate", 0, &on_log);
            write_hklm_dword(registry, WINDOWS_UPDATE_AU, "AUOptions", 1, &on_log);
            for name in [
                "NoAutoUpdate",
                "DisableOSUpgrade",
                "SetAutoRestartDeadline",
                "SetActiveHours",
                "SetDisableUXWUAccess",
                "DoNotConnectToWindowsUpdateInternetLocations",
                "ExcludeWUDriversInQualityUpdate",
                "ElevateNonAdmins",
            ] {
                write_hklm_dword(registry, WINDOWS_UPDATE, name, 1, &on_log);
            }
            for name in [
                "NoAutoUpdate",
                "AlwaysAutoRebootAtScheduledTime",
                "AutoInstallMinorUpdates",
                "IncludeRecommendedUpdates",
                "RebootRelaunchTimeoutEnabled",
                "DetectionFrequencyEnabled",
            ] {
                write_hklm_dword(registry, WINDOWS_UPDATE_AU, name, 1, &on_log);
            }
            run_and_log(runner, &on_log, "net", &["stop", "wuauserv"]).await;
            run_and_log(runner, &on_log, "sc", &["stop", "wuauserv"]).await;
            run_and_log(
                runner,
                &on_log,
                "sc",
                &["config", "wuauserv", "start=", "disabled"],
            )
            .await;
        }
        WindowsUpdateMode::NotificationsOnly => {
            on_log("Configurando Windows Update para notificar antes de baixar/instalar...");
            write_hklm_dword(
                registry,
                WINDOWS_SELF_HOST_AUTO_UPDATE,
                "AUOptions",
                2,
                &on_log,
            );
            delete_hklm_value(registry, WINDOWS_UPDATE_AU, "NoAutoUpdate", &on_log);
            write_hklm_dword(registry, WINDOWS_UPDATE_AU, "NoAutoUpdate", 1, &on_log);
            write_hkcu_dword(
                registry,
                r"Software\Microsoft\Windows\CurrentVersion\EOSNotify",
                "DiscontinueEOS",
                1,
                &on_log,
            );
            write_hklm_dword(registry, WINDOWS_UPDATE, "DisableGwx", 1, &on_log);
        }
        WindowsUpdateMode::Enable => {
            on_log("Reativando Windows Update e limpando políticas antigas...");
            for name in ["NoAutoUpdate", "EnableAutoUpdate"] {
                delete_hklm_value(registry, WINDOWS_SELF_HOST, name, &on_log);
            }
            delete_hkcu_value(
                registry,
                r"Software\Microsoft\Windows\CurrentVersion\EOSNotify",
                "DiscontinueEOS",
                &on_log,
            );
            for name in [
                "NoAutoUpdate",
                "SetAutoRestartDeadline",
                "SetActiveHours",
                "SetDisableUXWUAccess",
                "DoNotConnectToWindowsUpdateInternetLocations",
                "ExcludeWUDriversInQualityUpdate",
                "ElevateNonAdmins",
            ] {
                delete_hklm_value(registry, WINDOWS_UPDATE, name, &on_log);
            }
            for name in [
                "NoAutoUpdate",
                "AlwaysAutoRebootAtScheduledTime",
                "AutoInstallMinorUpdates",
                "IncludeRecommendedUpdates",
                "RebootRelaunchTimeoutEnabled",
                "DetectionFrequencyEnabled",
            ] {
                delete_hklm_value(registry, WINDOWS_UPDATE_AU, name, &on_log);
            }
            write_hklm_dword(registry, WINDOWS_UPDATE, "SetDisableUXWUAccess", 0, &on_log);
            write_hklm_dword(registry, WINDOWS_UPDATE_AU, "AUOptions", 3, &on_log);
            run_and_log(runner, &on_log, "gpupdate", &["/force"]).await;
            run_and_log(runner, &on_log, "net", &["start", "wuauserv"]).await;
            run_and_log(runner, &on_log, "sc", &["start", "wuauserv"]).await;
            run_and_log(
                runner,
                &on_log,
                "sc",
                &["config", "wuauserv", "start=", "auto"],
            )
            .await;
        }
    }
}

pub async fn configure_defender_policy(
    runner: &impl ProcessRunner,
    registry: &impl RegistryWriter,
    policy: DefenderPolicy,
    on_log: impl Fn(&str),
) {
    let disable_value = match policy {
        DefenderPolicy::Enable => 0,
        DefenderPolicy::Disable => 1,
    };

    if matches!(policy, DefenderPolicy::Enable) {
        delete_hklm_tree(
            registry,
            r"SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection",
            &on_log,
        );
        run_and_log(
            runner,
            &on_log,
            "setx",
            &["/M", "MP_FORCE_USE_SANDBOX", "1"],
        )
        .await;
        write_hklm_dword(
            registry,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management",
            "MoveImages",
            2,
            &on_log,
        );
    }

    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection",
        "DisableBehaviorMonitoring",
        1,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SOFTWARE\Microsoft\Windows Defender",
        "DisableDeleteAlerts",
        disable_value,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows Defender",
        "DisableDeleteScan",
        disable_value,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows Defender",
        "DisableAntiSpyware",
        disable_value,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection",
        "DisableAntiSpywareNetworkProtection",
        1,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection",
        "DisableOnAccessProtection",
        1,
        &on_log,
    );
    run_and_log(runner, &on_log, "net", &["stop", "MpCmdRun"]).await;
    run_and_log(runner, &on_log, "net", &["start", "MpCmdRun"]).await;
}

pub fn disable_smartscreen(registry: &impl RegistryWriter, on_log: impl Fn(&str)) {
    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows\System",
        "EnableSmartScreen",
        0,
        &on_log,
    );
    write_hklm_string(
        registry,
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer",
        "SmartScreenEnabled",
        "Off",
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SOFTWARE\Microsoft\Internet Explorer\PhishingFilter",
        "EnabledV9",
        0,
        &on_log,
    );
}

pub fn configure_onedrive_integration(
    registry: &impl RegistryWriter,
    enabled: bool,
    on_log: impl Fn(&str),
) {
    write_hklm_dword(
        registry,
        r"SOFTWARE\Policies\Microsoft\Windows\OneDrive",
        "DisableFileSyncNGSC",
        if enabled { 0 } else { 1 },
        &on_log,
    );
}

pub fn apply_windows11_smb_policies(registry: &impl RegistryWriter, on_log: impl Fn(&str)) {
    write_hklm_dword(
        registry,
        r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        "AllowInsecureGuestAuth",
        1,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        "RequireSecuritySignature",
        0,
        &on_log,
    );
    write_hklm_dword(
        registry,
        r"SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters",
        "EnableSecuritySignature",
        0,
        &on_log,
    );
}

pub fn restore_classic_context_menu(registry: &impl RegistryWriter, on_log: impl Fn(&str)) {
    on_log("Restaurando menu de contexto clássico do Windows 11...");
    if let Err(e) = registry.write_current_user_string(
        r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
        "",
        "",
    ) {
        on_log(&format!("AVISO: {e}"));
    }
}

pub async fn set_power_profile(
    runner: &impl ProcessRunner,
    profile: PowerProfile,
    on_log: impl Fn(&str),
) -> Result<String, String> {
    let guid = match profile {
        PowerProfile::Balanced => "381b4222-f694-41f0-9685-ff5bb260df2e",
        PowerProfile::HighPerformance => "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c",
    };
    on_log(&format!("Aplicando perfil de energia {guid}..."));
    runner.run("powercfg", &["/s", guid], None).await
}

async fn run_and_log(
    runner: &impl ProcessRunner,
    on_log: &impl Fn(&str),
    program: &str,
    args: &[&str],
) {
    match runner.run(program, args, None).await {
        Ok(output) => {
            let output = output.trim();
            if !output.is_empty() {
                on_log(output);
            }
        }
        Err(e) => on_log(&format!("AVISO: {program} falhou: {e}")),
    }
}

fn write_hklm_dword(
    registry: &impl RegistryWriter,
    path: &str,
    name: &str,
    value: u32,
    on_log: &impl Fn(&str),
) {
    if let Err(e) = registry.write_local_machine_dword(path, name, value) {
        on_log(&format!("AVISO: {e}"));
    }
}

fn write_hkcu_dword(
    registry: &impl RegistryWriter,
    path: &str,
    name: &str,
    value: u32,
    on_log: &impl Fn(&str),
) {
    if let Err(e) = registry.write_current_user_dword(path, name, value) {
        on_log(&format!("AVISO: {e}"));
    }
}

fn write_hklm_string(
    registry: &impl RegistryWriter,
    path: &str,
    name: &str,
    value: &str,
    on_log: &impl Fn(&str),
) {
    if let Err(e) = registry.write_local_machine_string(path, name, value) {
        on_log(&format!("AVISO: {e}"));
    }
}

fn delete_hklm_value(
    registry: &impl RegistryWriter,
    path: &str,
    name: &str,
    on_log: &impl Fn(&str),
) {
    if let Err(e) = registry.delete_local_machine_value(path, name) {
        on_log(&format!("AVISO: {e}"));
    }
}

fn delete_hkcu_value(
    registry: &impl RegistryWriter,
    path: &str,
    name: &str,
    on_log: &impl Fn(&str),
) {
    if let Err(e) = registry.delete_current_user_value(path, name) {
        on_log(&format!("AVISO: {e}"));
    }
}

fn delete_hklm_tree(registry: &impl RegistryWriter, path: &str, on_log: &impl Fn(&str)) {
    if let Err(e) = registry.delete_local_machine_tree(path) {
        on_log(&format!("AVISO: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{ProcessRunner, RegistryWriter};
    use std::sync::{Arc, Mutex};

    struct FakeRunner {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl ProcessRunner for FakeRunner {
        async fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> Result<String, String> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{program} {}", args.join(" ")));
            Ok(String::new())
        }
    }

    #[derive(Default)]
    struct FakeRegistry {
        dwords: Mutex<Vec<(RegistryHive, String, String, u32)>>,
        strings: Mutex<Vec<(RegistryHive, String, String, String)>>,
        deletes: Mutex<Vec<(RegistryHive, String, Option<String>)>>,
    }

    impl RegistryWriter for FakeRegistry {
        fn write_local_machine_dword(
            &self,
            path: &str,
            name: &str,
            value: u32,
        ) -> Result<(), String> {
            self.dwords.lock().unwrap().push((
                RegistryHive::LocalMachine,
                path.to_string(),
                name.to_string(),
                value,
            ));
            Ok(())
        }

        fn write_classes_root_string(
            &self,
            path: &str,
            name: &str,
            value: &str,
        ) -> Result<(), String> {
            self.strings.lock().unwrap().push((
                RegistryHive::ClassesRoot,
                path.to_string(),
                name.to_string(),
                value.to_string(),
            ));
            Ok(())
        }

        fn write_local_machine_string(
            &self,
            path: &str,
            name: &str,
            value: &str,
        ) -> Result<(), String> {
            self.strings.lock().unwrap().push((
                RegistryHive::LocalMachine,
                path.to_string(),
                name.to_string(),
                value.to_string(),
            ));
            Ok(())
        }

        fn write_current_user_dword(
            &self,
            path: &str,
            name: &str,
            value: u32,
        ) -> Result<(), String> {
            self.dwords.lock().unwrap().push((
                RegistryHive::CurrentUser,
                path.to_string(),
                name.to_string(),
                value,
            ));
            Ok(())
        }

        fn write_current_user_string(
            &self,
            path: &str,
            name: &str,
            value: &str,
        ) -> Result<(), String> {
            self.strings.lock().unwrap().push((
                RegistryHive::CurrentUser,
                path.to_string(),
                name.to_string(),
                value.to_string(),
            ));
            Ok(())
        }

        fn delete_local_machine_value(&self, path: &str, name: &str) -> Result<(), String> {
            self.deletes.lock().unwrap().push((
                RegistryHive::LocalMachine,
                path.to_string(),
                Some(name.to_string()),
            ));
            Ok(())
        }

        fn delete_current_user_value(&self, path: &str, name: &str) -> Result<(), String> {
            self.deletes.lock().unwrap().push((
                RegistryHive::CurrentUser,
                path.to_string(),
                Some(name.to_string()),
            ));
            Ok(())
        }

        fn delete_local_machine_tree(&self, path: &str) -> Result<(), String> {
            self.deletes
                .lock()
                .unwrap()
                .push((RegistryHive::LocalMachine, path.to_string(), None));
            Ok(())
        }
    }

    #[tokio::test]
    async fn disable_windows_update_sets_policies_and_disables_service() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = FakeRunner {
            calls: calls.clone(),
        };
        let registry = FakeRegistry::default();

        configure_windows_update(&runner, &registry, WindowsUpdateMode::Disable, |_| {}).await;

        assert!(registry
            .dwords
            .lock()
            .unwrap()
            .iter()
            .any(|(_, path, name, value)| {
                path == r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate"
                    && name == "NoAutoUpdate"
                    && *value == 1
            }));
        assert!(calls
            .lock()
            .unwrap()
            .contains(&"sc config wuauserv start= disabled".to_string()));
    }

    #[tokio::test]
    async fn classic_context_menu_writes_hkcu_default_value() {
        let registry = FakeRegistry::default();

        restore_classic_context_menu(&registry, |_| {});

        assert!(registry.strings.lock().unwrap().iter().any(|(hive, path, name, value)| {
            *hive == RegistryHive::CurrentUser
                && path == r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32"
                && name.is_empty()
                && value.is_empty()
        }));
    }
}
