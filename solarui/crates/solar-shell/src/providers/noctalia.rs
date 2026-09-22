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
            info!("Noctalia v5 zaten çalışıyor.");
            return Ok(());
        }

        if !self.get_pids().await.is_empty() {
            self.stop().await?;
        }

        info!(
            "Noctalia v5 native kabuğu başlatılıyor: {:?}",
            self.binary_path
        );

        let mut cmd = tokio::process::Command::new(&self.binary_path);
        cmd.arg("--daemon");
        cmd.env("NOCTALIA_ASSETS_DIR", &self.assets_dir);

        // Niri/Wayland ortam değişkenlerini aktar
        if let Ok(d) = std::env::var("WAYLAND_DISPLAY") {
            cmd.env("WAYLAND_DISPLAY", d);
        }
        if let Ok(s) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.env("XDG_RUNTIME_DIR", s);
        }

        let mut child = cmd.spawn()?;
        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        // Başlatılmasını doğrula (Reconciliation / Guard - VM veya llvmpipe altında 5-8 sn sürebilir)
        for i in 1..=80 {
            sleep(Duration::from_millis(100)).await;
            if self.health_check().await.unwrap_or(false) {
                info!("Noctalia v5 başarıyla başlatıldı ve doğrulandı ({}ms).", i * 100);
                return Ok(());
            }
        }

        bail!("Noctalia v5 başlatılamadı veya zaman aşımına uğradı!");
    }

    async fn stop(&self) -> Result<()> {
        let pids = self.get_pids().await;
        if pids.is_empty() {
            return Ok(());
        }

        info!("Noctalia v5 sonlandırılıyor (PID'ler: {:?})...", pids);

        // 1. Aşama: SIGTERM gönder
        let _ = tokio::process::Command::new("pkill")
            .args(["-u", &solar_common::current_uid().to_string()])
            .args(["-15", "-x", "noctalia"])
            .checked_status()
            .await;

        // 2. Aşama: Kapanmasını doğrula (en fazla 1.5 saniye)
        for _ in 0..15 {
            sleep(Duration::from_millis(100)).await;
            if self.get_pids().await.is_empty() {
                info!("Noctalia v5 başarıyla ve temiz bir şekilde kapandı.");
                return Ok(());
            }
        }

        // 3. Aşama: Hala kapanmadıysa SIGKILL ile zorla kapat
        warn!("Noctalia v5 SIGTERM'e yanıt vermedi, SIGKILL uygulanıyor...");
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
            bail!("Noctalia v5 süreçleri sonlandırılamadı!");
        }
    }

    async fn health_check(&self) -> Result<bool> {
        if self.get_pids().await.is_empty() {
            return Ok(false);
        }
        let output = tokio::process::Command::new(&self.binary_path)
            .args(["msg", "status"])
            .output_timeout()
            .await?;
        if !output.status.success() {
            return Ok(false);
        }
        let status: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        Ok(status
            .get("barVisible")
            .is_some_and(serde_json::Value::is_boolean))
    }

    async fn dispatch(&self, action: ShellAction) -> Result<()> {
        info!("Noctalia IPC yönlendiriliyor: {:?}", action);

        let res = match action {
            ShellAction::Launcher => tokio::process::Command::new(&self.binary_path)
                .args(["msg", "panel-toggle", "launcher"])
                .checked_status()
                .await
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!(e)),
            ShellAction::ControlCenter | ShellAction::Dashboard => {
                tokio::process::Command::new(&self.binary_path)
                    .args(["msg", "panel-toggle", "control-center"])
                    .checked_status()
                    .await
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!(e))
            }
            ShellAction::Session => tokio::process::Command::new(&self.binary_path)
                .args(["msg", "panel-toggle", "session"])
                .checked_status()
                .await
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!(e)),
            ShellAction::Settings => {
                let status = tokio::process::Command::new(&self.binary_path)
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
