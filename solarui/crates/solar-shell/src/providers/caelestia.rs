use super::{ProcessExt, ShellProvider};
use crate::actions::ShellAction;
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct CaelestiaProvider {
    caelestia_bin: PathBuf,
    is_full_mode: bool,
}

impl Default for CaelestiaProvider {
    fn default() -> Self {
        Self::for_hybrid()
    }
}

impl CaelestiaProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_hybrid() -> Self {
        Self {
            caelestia_bin: Self::discover_bin(),
            is_full_mode: false,
        }
    }

    pub fn for_full() -> Self {
        Self {
            caelestia_bin: Self::discover_bin(),
            is_full_mode: true,
        }
    }

    fn discover_bin() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
        let candidates = [
            PathBuf::from("/usr/bin/caelestia"),
            PathBuf::from("/usr/local/bin/caelestia"),
            PathBuf::from(&home).join(".local/bin/caelestia"),
        ];

        for c in candidates {
            if c.exists() {
                return c;
            }
        }

        PathBuf::from("/usr/bin/caelestia")
    }

    fn cmd(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(&self.caelestia_bin);
        cmd.env("QT_QPA_PLATFORMTHEME", "");
        cmd.env("QT_QPA_PLATFORM", "wayland");
        cmd.env("QT_PLUGIN_PATH", "/usr/lib64/quickshell/plugins:/usr/lib64/qt6/plugins");
        cmd.env("QT_QPA_PLATFORM_PLUGIN_PATH", "/usr/lib64/quickshell/plugins/platforms");
        cmd.env("QML2_IMPORT_PATH", "/usr/lib64/quickshell/qml:/usr/lib64/qt6/qml");
        cmd.env("QML_IMPORT_PATH", "/usr/lib64/quickshell/qml:/usr/lib64/qt6/qml");

        let current_ld = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let ld_path = if current_ld.is_empty() {
            "/usr/lib64/quickshell:/usr/lib64/quickshell/qml/Caelestia/lib:/usr/lib64".to_string()
        } else {
            format!("/usr/lib64/quickshell:/usr/lib64/quickshell/qml/Caelestia/lib:{}", current_ld)
        };
        cmd.env("LD_LIBRARY_PATH", ld_path);

        if let Ok(d) = std::env::var("WAYLAND_DISPLAY") {
            cmd.env("WAYLAND_DISPLAY", d);
        } else if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let s = name.to_string_lossy();
                    if s.starts_with("wayland-") && !s.ends_with(".lock") {
                        cmd.env("WAYLAND_DISPLAY", s.to_string());
                        break;
                    }
                }
            }
        }

        if let Ok(s) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.env("XDG_RUNTIME_DIR", s);
        }

        cmd
    }

    pub async fn is_bar_open(&self) -> Result<bool> {
        let output = self
            .cmd()
            .args(["shell", "drawers", "isOpen", "bar"])
            .output_timeout()
            .await?;
        anyhow::ensure!(output.status.success(), "Caelestia bar query failed");
        match String::from_utf8_lossy(&output.stdout).trim() {
            "1" => Ok(true),
            "0" => Ok(false),
            _ => bail!("Invalid Caelestia bar response"),
        }
    }

    pub async fn ensure_bar_state(&self, should_be_open: bool) -> Result<()> {
        let mut last_err = None;
        for _ in 0..15 {
            match self.is_bar_open().await {
                Ok(current) => {
                    if current != should_be_open {
                        info!(
                            "Caelestia bar durumu ayarlanıyor: mevcut={}, hedeflenen={}",
                            current, should_be_open
                        );
                        let _ = self
                            .cmd()
                            .args(["shell", "drawers", "toggle", "bar"])
                            .checked_status()
                            .await;
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_err = Some(e);
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
        if let Some(e) = last_err {
            warn!("Caelestia bar durumu sorgulanamadı (devam ediliyor): {}", e);
        }
        Ok(())
    }

    async fn get_pids(&self) -> Vec<i32> {
        let out = tokio::process::Command::new("pgrep")
            .args(["-u", &solar_common::current_uid().to_string()])
            .args(["-f", "(qs|quickshell).*caelestia"])
            .output_timeout()
            .await;

        if let Ok(output) = out {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let pids: Vec<i32> = stdout
                    .lines()
                    .filter_map(|l| l.trim().parse::<i32>().ok())
                    .collect();
                if !pids.is_empty() {
                    return pids;
                }
            }
        }

        // Fallback without UID constraint if running in certain session container contexts
        let fallback_out = tokio::process::Command::new("pgrep")
            .args(["-f", "quickshell.*caelestia"])
            .output_timeout()
            .await;

        if let Ok(output) = fallback_out {
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
impl ShellProvider for CaelestiaProvider {
    fn name(&self) -> &'static str {
        if self.is_full_mode {
            "Caelestia Shell (Official)"
        } else {
            "Caelestia Hybrid Surfaces (Official)"
        }
    }

    async fn is_installed(&self) -> bool {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/liveuser".to_string());
        let qml = PathBuf::from(&home).join(".config/quickshell/caelestia/shell.qml");
        let xdg_qml = PathBuf::from("/etc/xdg/quickshell/caelestia/shell.qml");
        let skel_qml = PathBuf::from("/etc/skel/.config/quickshell/caelestia/shell.qml");

        self.caelestia_bin.exists() && (qml.exists() || xdg_qml.exists() || skel_qml.exists())
    }

    async fn start(&self) -> Result<()> {
        if self.health_check().await.unwrap_or(false) {
            info!("Caelestia shell zaten çalışıyor.");
            self.ensure_bar_state(self.is_full_mode).await?;
            return Ok(());
        }

        if !self.get_pids().await.is_empty() {
            self.stop().await?;
        }

        if !self.is_installed().await {
            bail!(
                "Caelestia kurulu değil veya shell.qml bulunamadı: {:?}",
                self.caelestia_bin
            );
        }

        info!(
            "Resmi Caelestia Shell başlatılıyor: {:?} (FullMode: {})",
            self.caelestia_bin, self.is_full_mode
        );

        let mut cmd = self.cmd();
        cmd.args(["shell", "-d"]);

        let mut child = cmd.spawn()?;
        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        // Başlatılmasını doğrula (Guard döngüsü - llvmpipe veya düşük kaynaklı VM'lerde 6 sn)
        for _ in 0..60 {
            sleep(Duration::from_millis(100)).await;
            if self.health_check().await.unwrap_or(false) {
                info!("Resmi Caelestia Shell başarıyla aktifleşti.");
                self.ensure_bar_state(self.is_full_mode).await?;
                return Ok(());
            }
        }

        bail!("Caelestia Shell başlatılamadı veya zaman aşımına uğradı!");
    }

    async fn stop(&self) -> Result<()> {
        let pids = self.get_pids().await;
        if pids.is_empty() {
            return Ok(());
        }

        info!("Caelestia Shell sonlandırılıyor (PID'ler: {:?})...", pids);

        // 1. Resmi Caelestia kill komutu (-k)
        let _ = self.cmd().args(["shell", "-k"]).checked_status().await;

        // 2. Kapanmasını doğrula
        for _ in 0..15 {
            sleep(Duration::from_millis(100)).await;
            if self.get_pids().await.is_empty() {
                info!("Caelestia Shell başarıyla sonlandırıldı.");
                return Ok(());
            }
        }

        // 3. SIGTERM gönder
        let _ = tokio::process::Command::new("pkill")
            .args(["-u", &solar_common::current_uid().to_string()])
            .args(["-15", "-f", "(qs|quickshell).*caelestia"])
            .checked_status()
            .await;

        for _ in 0..10 {
            sleep(Duration::from_millis(100)).await;
            if self.get_pids().await.is_empty() {
                info!("Caelestia Shell SIGTERM ile temizlendi.");
                return Ok(());
            }
        }

        // 4. Zorla kapat
        warn!("Caelestia Shell SIGTERM'e yanıt vermedi, SIGKILL uygulanıyor...");
        let _ = tokio::process::Command::new("pkill")
            .args(["-u", &solar_common::current_uid().to_string()])
            .args(["-9", "-f", "(qs|quickshell).*caelestia"])
            .checked_status()
            .await;

        sleep(Duration::from_millis(150)).await;
        if self.get_pids().await.is_empty() {
            info!("Caelestia Shell zorlanarak temizlendi.");
            Ok(())
        } else {
            bail!("Caelestia Shell süreçleri kapatılamadı!");
        }
    }

    async fn health_check(&self) -> Result<bool> {
        let pids = self.get_pids().await;
        if pids.is_empty() {
            return Ok(false);
        }
        let output = self
            .cmd()
            .args(["shell", "drawers", "isOpen", "bar"])
            .output_timeout()
            .await;
        if let Ok(out) = output {
            if out.status.success() && matches!(String::from_utf8_lossy(&out.stdout).trim(), "0" | "1") {
                return Ok(true);
            }
        }
        // If drawer query hasn't responded yet or returned non-zero, but process is alive:
        Ok(true)
    }

    async fn dispatch(&self, action: ShellAction) -> Result<()> {
        info!("Resmi Caelestia IPC yönlendiriliyor: {:?}", action);

        match action {
            ShellAction::Launcher => {
                self.cmd()
                    .args(["shell", "drawers", "toggle", "launcher"])
                    .checked_status()
                    .await?;
                Ok(())
            }
            ShellAction::Dashboard => {
                self.cmd()
                    .args(["shell", "drawers", "toggle", "dashboard"])
                    .checked_status()
                    .await?;
                Ok(())
            }
            ShellAction::ControlCenter => {
                self.cmd()
                    .args(["shell", "drawers", "toggle", "sidebar"])
                    .checked_status()
                    .await?;
                Ok(())
            }
            ShellAction::Session => {
                self.cmd()
                    .args(["shell", "drawers", "toggle", "session"])
                    .checked_status()
                    .await?;
                Ok(())
            }
            ShellAction::Settings => {
                // SolarUI Settings GUI
                tokio::process::Command::new("solar-shell")
                    .arg("settings")
                    .spawn()
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!(e))
            }
            ShellAction::Overview | ShellAction::WindowSwitcher => {
                // Overview
                let _ = tokio::process::Command::new("niri")
                    .args(["msg", "action", "toggle-overview"])
                    .checked_status()
                    .await?;
                Ok(())
            }
        }
    }
}
