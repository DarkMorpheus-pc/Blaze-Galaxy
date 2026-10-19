use super::{ProcessExt, ShellProvider};
use crate::actions::ShellAction;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Supervised end4-pC Quickshell provider.
///
/// The provider deliberately selects the config by an absolute path. It does
/// not copy executable QML into the user's home and never uses `killall qs`,
/// because other SolarUI engines may also use Quickshell.
pub struct End4PcProvider {
    qs_bin: PathBuf,
    config_dir: PathBuf,
}

impl Default for End4PcProvider {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/liveuser".to_string());
        let checkout = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../vendor/end4-pC");
        let candidates = [
            std::env::var_os("SOLARUI_END4_PC_DIR").map(PathBuf::from),
            Some(PathBuf::from("/usr/share/quickshell/end4-pC")),
            Some(PathBuf::from("/etc/xdg/quickshell/end4-pC")),
            Some(PathBuf::from(&home).join(".config/quickshell/end4-pC")),
            Some(checkout),
        ];
        let config_dir = candidates
            .into_iter()
            .flatten()
            .find(|path| path.join("shell.qml").is_file())
            .unwrap_or_else(|| PathBuf::from("/usr/share/quickshell/end4-pC"));

        let qs_bin = [
            // end4-pC imports Qt5Compat.GraphicalEffects. The Fedora build is
            // kept beside BlazeOS' bundled Quickshell so its private Qt ABI
            // always matches Fedora's Qt QML modules.
            PathBuf::from("/usr/bin/quickshell-fedora"),
            PathBuf::from("/usr/bin/qs"),
            PathBuf::from("/usr/bin/quickshell"),
        ]
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("qs"));

        Self { qs_bin, config_dir }
    }
}

impl End4PcProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn cmd(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(&self.qs_bin);
        cmd.args(["--path"])
            .arg(&self.config_dir)
            .env("XDG_CURRENT_DESKTOP", "SolarUI:Niri")
            .env("XDG_SESSION_DESKTOP", "SolarUI:Niri")
            .env("QT_QPA_PLATFORM", "wayland");
        if let Ok(value) = std::env::var("WAYLAND_DISPLAY") {
            cmd.env("WAYLAND_DISPLAY", value);
        }
        if let Ok(value) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.env("XDG_RUNTIME_DIR", value);
        }
        cmd
    }

    async fn get_pids(&self) -> Vec<i32> {
        let uid = solar_common::current_uid();
        let needle = self.config_dir.to_string_lossy();
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return Vec::new();
        };

        entries
            .flatten()
            .filter_map(|entry| {
                let pid = entry.file_name().to_string_lossy().parse::<i32>().ok()?;
                let metadata = std::fs::metadata(entry.path()).ok()?;
                use std::os::unix::fs::MetadataExt;
                if metadata.uid() != uid {
                    return None;
                }
                let cmdline = std::fs::read(entry.path().join("cmdline")).ok()?;
                let text = String::from_utf8_lossy(&cmdline).replace('\0', " ");
                (text.contains("qs") || text.contains("quickshell"))
                    .then_some(())
                    .filter(|_| text.contains(needle.as_ref()))
                    .map(|_| pid)
            })
            .collect()
    }

    async fn ipc(&self, target: &str, function: &str) -> Result<std::process::Output> {
        self.cmd()
            .args(["ipc", "call", target, function])
            .output_timeout()
            .await
    }

    async fn ipc_checked(&self, target: &str, function: &str) -> Result<()> {
        let output = self.ipc(target, function).await?;
        anyhow::ensure!(
            output.status.success(),
            "end4-pC IPC failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    fn ipc_target(action: ShellAction) -> (&'static str, &'static str) {
        match action {
            ShellAction::Launcher | ShellAction::Overview | ShellAction::WindowSwitcher => {
                ("search", "toggle")
            }
            ShellAction::ControlCenter | ShellAction::Dashboard => ("sidebarRight", "toggle"),
            ShellAction::Session => ("session", "toggle"),
            ShellAction::Settings => ("settings", "toggle"),
        }
    }
}

#[async_trait]
impl ShellProvider for End4PcProvider {
    fn name(&self) -> &'static str {
        "end4-pC"
    }

    async fn is_installed(&self) -> bool {
        self.config_dir.join("shell.qml").is_file()
            && (self.qs_bin.is_file() || self.qs_bin == Path::new("qs"))
    }

    async fn start(&self) -> Result<()> {
        if self.health_check().await.unwrap_or(false) {
            return Ok(());
        }
        if !self.get_pids().await.is_empty() {
            self.stop().await?;
        }
        anyhow::ensure!(
            self.is_installed().await,
            "end4-pC or Quickshell is not installed"
        );

        info!("Starting end4-pC from {:?}", self.config_dir);
        self.cmd()
            .arg("--daemonize")
            .spawn()
            .context("Cannot start end4-pC")?;

        // First launch creates configuration and shader caches. Software
        // rendering and slower disks can need well over five seconds.
        for _ in 0..300 {
            sleep(Duration::from_millis(100)).await;
            if self.health_check().await.unwrap_or(false) {
                return Ok(());
            }
        }
        let diagnostics = self
            .cmd()
            .arg("log")
            .output_timeout()
            .await
            .map(|out| {
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                )
            })
            .unwrap_or_else(|error| format!("log unavailable: {error:#}"));
        let _ = self.stop().await;
        let tail = diagnostics
            .lines()
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        bail!("end4-pC did not become IPC-ready within 30 seconds\n{tail}")
    }

    async fn stop(&self) -> Result<()> {
        let pids = self.get_pids().await;
        for pid in &pids {
            let _ = tokio::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status()
                .await;
        }
        for _ in 0..20 {
            if self.get_pids().await.is_empty() {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }
        warn!("end4-pC ignored SIGTERM; terminating owned PIDs");
        for pid in self.get_pids().await {
            let _ = tokio::process::Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status()
                .await;
        }
        anyhow::ensure!(
            self.get_pids().await.is_empty(),
            "end4-pC processes could not be stopped"
        );
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        if self.get_pids().await.is_empty() {
            return Ok(false);
        }
        let output = match self.ipc("solarui", "ping").await {
            Ok(output) => output,
            Err(_) => return Ok(false),
        };
        Ok(output.status.success()
            && String::from_utf8_lossy(&output.stdout)
                .to_lowercase()
                .contains("ready"))
    }

    async fn dispatch(&self, action: ShellAction) -> Result<()> {
        let (target, function) = Self::ipc_target(action);
        self.ipc_checked(target, function).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solar_actions_have_stable_end4_pc_ipc_targets() {
        assert_eq!(
            End4PcProvider::ipc_target(ShellAction::Launcher),
            ("search", "toggle")
        );
        assert_eq!(
            End4PcProvider::ipc_target(ShellAction::ControlCenter),
            ("sidebarRight", "toggle")
        );
        assert_eq!(
            End4PcProvider::ipc_target(ShellAction::Session),
            ("session", "toggle")
        );
        assert_eq!(
            End4PcProvider::ipc_target(ShellAction::Settings),
            ("settings", "toggle")
        );
    }
}
