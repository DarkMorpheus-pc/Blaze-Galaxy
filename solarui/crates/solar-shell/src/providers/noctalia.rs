use super::{ProcessExt, ShellProvider};
use crate::actions::ShellAction;
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct NoctaliaProvider {
    binary_path: PathBuf,
    assets_dir: PathBuf,
}

impl Default for NoctaliaProvider {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
        let usr_bin = PathBuf::from("/usr/bin/noctalia");
        let local_bin = PathBuf::from(&home).join(".local/bin/noctalia");
        let bin = if usr_bin.exists() {
            usr_bin
        } else if local_bin.exists() {
            local_bin
        } else {
            PathBuf::from("/usr/bin/noctalia")
        };

        // Prefer our product assets, including when running directly from a checkout.
        // The session's branding must not be overwritten with upstream translations.
        let mut candidates = vec![
            PathBuf::from(&home).join(".local/share/solarui/noctalia-assets"),
            PathBuf::from("/usr/share/solarui/noctalia-assets"),
        ];
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("../../data/noctalia-assets"));
            }
        }
        if let Some(assets) = std::env::var_os("NOCTALIA_ASSETS_DIR") {
            candidates.push(PathBuf::from(assets));
        }
        candidates.push(PathBuf::from(&home).join(".local/share/noctalia/assets"));
        let assets = candidates.into_iter()
            .find(|path| path.join("translations/en.json").is_file())
            .unwrap_or_else(|| PathBuf::from("/usr/share/noctalia/assets"));

        Self {
            binary_path: bin,
            assets_dir: assets,
        }
    }
}

impl NoctaliaProvider {
    pub fn new() -> Self {
        Self::default()
    }

    async fn get_pids(&self) -> Vec<i32> {
        let out = tokio::process::Command::new("pgrep")
            .args(["-u", &solar_common::current_uid().to_string()])
            .arg("-x")
            .arg("noctalia")
            .output_timeout()
            .await;

        if let Ok(output) = out {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout
                    .lines()
                    .filter_map(|l| l.trim().parse::<i32>().ok())
                    .collect();
            }
        }
        Vec::new()
    }

    fn cmd(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(&self.binary_path);
        cmd.env("NOCTALIA_ASSETS_DIR", &self.assets_dir);

        let wayland_disp = if let Ok(d) = std::env::var("WAYLAND_DISPLAY") {
            Some(d)
        } else if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let mut found = None;
            if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
                let mut sockets: Vec<String> = entries
                    .flatten()
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with("wayland-") && !name.ends_with(".lock") {
                            Some(name)
                        } else {
                            None
                        }
                    })
                    .collect();
                sockets.sort();
                if let Some(last) = sockets.last() {
                    found = Some(last.clone());
                }
            }
            found
        } else {
            None
        };

        if let Some(d) = wayland_disp {
            cmd.env("WAYLAND_DISPLAY", d);
        }
        if let Ok(s) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.env("XDG_RUNTIME_DIR", s);
        }
        cmd.env("XDG_CURRENT_DESKTOP", "SolarUI");
        cmd.env("XDG_SESSION_DESKTOP", "SolarUI");
        cmd
    }
}

#[async_trait]
impl ShellProvider for NoctaliaProvider {
    fn name(&self) -> &'static str {
        "Noctalia v5"
    }

    async fn is_installed(&self) -> bool {
        self.binary_path.exists() || Path::new("/usr/bin/noctalia").exists()
    }

    async fn start(&self) -> Result<()> {
        if self.health_check().await.unwrap_or(false) {
            info!("Noctalia v5 zaten calisiyor.");
            return Ok(());
        }

        if !self.get_pids().await.is_empty() {
            self.stop().await?;
        }

        info!(
            "Noctalia v5 native kabugu baslatiliyor: {:?}",
            self.binary_path
        );

        let mut cmd = self.cmd();
        cmd.arg("--daemon");

        let mut child = cmd.spawn()?;
        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        // Baslatilmasini dogrula (Reconciliation / Guard - aninda tepki icin 20ms yoklama)
        for i in 1..=500 {
            sleep(Duration::from_millis(20)).await;
            if self.health_check().await.unwrap_or(false) {
                info!("Noctalia v5 basariyla baslatildi ve dogrulandi ({}ms).", i * 20);
                return Ok(());
            }
        }

        bail!("Noctalia v5 baslatilamadi veya zaman asimina ugradi!");
    }

    async fn stop(&self) -> Result<()> {
        let pids = self.get_pids().await;
        if pids.is_empty() {
            return Ok(());
        }

        info!("Noctalia v5 sonlandiriliyor (PID'ler: {:?})...", pids);

        // 1. Asama: SIGTERM gonder
        let _ = tokio::process::Command::new("pkill")
            .args(["-u", &solar_common::current_uid().to_string()])
            .args(["-15", "-x", "noctalia"])
            .checked_status()
            .await;

        // 2. Asama: Kapanmasini dogrula (en fazla 1.5 saniye)
        for _ in 0..15 {
            sleep(Duration::from_millis(100)).await;
            if self.get_pids().await.is_empty() {
                info!("Noctalia v5 basariyla ve temiz bir sekilde kapandi.");
                return Ok(());
            }
        }

        // 3. Asama: Hala kapanmadiysa SIGKILL ile zorla kapat
        warn!("Noctalia v5 SIGTERM'e yanit vermedi, SIGKILL uygulaniyor...");
        let _ = tokio::process::Command::new("pkill")
            .args(["-u", &solar_common::current_uid().to_string()])
            .args(["-9", "-x", "noctalia"])
            .checked_status()
            .await;

        sleep(Duration::from_millis(150)).await;
        if self.get_pids().await.is_empty() {
            info!("Noctalia v5 zorlanarak temizlendi.");
            Ok(())
        } else {
            bail!("Noctalia v5 surecleri sonlandirilamadi!");
        }
    }

    async fn health_check(&self) -> Result<bool> {
        if self.get_pids().await.is_empty() {
            return Ok(false);
        }
        let mut cmd = self.cmd();
        cmd.args(["msg", "status"]);
        let output = match cmd.output_timeout().await {
            Ok(o) => o,
            Err(_) => return Ok(false),
        };
        if !output.status.success() {
            return Ok(false);
        }
        let status: serde_json::Value = match serde_json::from_slice(&output.stdout) {
            Ok(v) => v,
            Err(_) => return Ok(false),
        };
        Ok(status.is_object())
    }

    async fn dispatch(&self, action: ShellAction) -> Result<()> {
        info!("Noctalia IPC yonlendiriliyor: {:?}", action);

        let res = match action {
            ShellAction::Launcher => self.cmd()
                .args(["msg", "panel-toggle", "launcher"])
                .checked_status()
                .await
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!(e)),
            ShellAction::ControlCenter | ShellAction::Dashboard => {
                self.cmd()
                    .args(["msg", "panel-toggle", "control-center"])
                    .checked_status()
                    .await
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!(e))
            }
            ShellAction::Session => self.cmd()
                .args(["msg", "panel-toggle", "session"])
                .checked_status()
                .await
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!(e)),
            ShellAction::Settings => {
                let status = self.cmd()
                    .args(["msg", "settings-toggle"])
                    .checked_status()
                    .await;
                if status.is_ok() {
                    Ok(())
                } else {
                    // Fallback to internal solar-shell settings window
                    tokio::process::Command::new("solar-shell")
                        .arg("settings")
                        .spawn()
                        .map(|_| ())
                        .map_err(|e| anyhow::anyhow!(e))
                }
            }
            ShellAction::Overview | ShellAction::WindowSwitcher => {
                tokio::process::Command::new(&self.binary_path)
                    .args(["msg", "window-switcher"])
                    .checked_status()
                    .await
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!(e))
            }
        };

        res
    }
}
