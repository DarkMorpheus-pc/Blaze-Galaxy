use anyhow::Result;
use tracing::{info, warn};

use crate::actions::ShellAction;
use crate::engine::{ShellEngine, SolarShellConfig, SurfaceProvider};
use crate::providers::{CaelestiaProvider, NoctaliaProvider, ShellProvider};

pub struct SolarSurfaceRouter {
    noctalia: NoctaliaProvider,
    caelestia: CaelestiaProvider,
}

impl Default for SolarSurfaceRouter {
    fn default() -> Self {
        Self {
            noctalia: NoctaliaProvider::new(),
            caelestia: CaelestiaProvider::new(),
        }
    }
}

impl SolarSurfaceRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Aksiyon için hangi Surface Provider'ın kullanılacağını belirler.
    pub fn provider_for(&self, action: ShellAction, shell_cfg: &SolarShellConfig) -> SurfaceProvider {
        match shell_cfg.engine {
            ShellEngine::Noctalia => SurfaceProvider::Noctalia,
            ShellEngine::Caelestia => SurfaceProvider::Caelestia,
            ShellEngine::Hybrid => match action {
                ShellAction::Launcher => shell_cfg.launcher_provider,
                ShellAction::Dashboard => shell_cfg.dashboard_provider,
                ShellAction::ControlCenter
                | ShellAction::Session
                | ShellAction::Settings
                | ShellAction::WindowSwitcher => shell_cfg.bar_provider,
                ShellAction::Overview => SurfaceProvider::SolarUI,
            },
        }
    }

    /// Aksiyonu konfigürasyona göre ilgili sağlayıcıya yönlendirir.
    pub async fn route(&self, action: ShellAction, shell_cfg: &SolarShellConfig) -> Result<()> {
        let provider = self.provider_for(action, shell_cfg);
        info!(
            "SolarSurfaceRouter: Aksiyon={:?}, Motor={:?}, Seçilen Sağlayıcı={:?}",
            action, shell_cfg.engine, provider
        );

        match provider {
            SurfaceProvider::Noctalia => {
                self.noctalia.dispatch(action).await
            }
            SurfaceProvider::Caelestia => {
                if !self.caelestia.health_check().await.unwrap_or(false) {
                    info!("Caelestia henüz çalışmıyor, talep üzerine (on-demand) başlatılıyor...");
                    let _ = self.caelestia.start().await;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                match self.caelestia.dispatch(action).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        warn!(
                            "Caelestia surface dispatch başarısız oldu ({}). Noctalia'ya güvenli geri dönüş (fallback) yapılıyor.",
                            e
                        );
                        self.noctalia.dispatch(action).await
                    }
                }
            }
            SurfaceProvider::SolarUI => {
                match action {
                    ShellAction::Overview => {
                        let _ = tokio::process::Command::new("niri")
                            .args(["msg", "action", "toggle-overview"])
                            .status()
                            .await;
                        Ok(())
                    }
                    ShellAction::Settings => {
                        tokio::process::Command::new("solar-shell")
                            .arg("settings")
                            .spawn()
                            .map(|_| ())
                            .map_err(|e| anyhow::anyhow!(e))
                    }
                    _ => {
                        // Diğer durumlar için Noctalia omurgasına yönlendir
                        self.noctalia.dispatch(action).await
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_noctalia_mode() {
        let router = SolarSurfaceRouter::new();
        let cfg = SolarShellConfig {
            engine: ShellEngine::Noctalia,
            ..Default::default()
        };

        assert_eq!(router.provider_for(ShellAction::Launcher, &cfg), SurfaceProvider::Noctalia);
        assert_eq!(router.provider_for(ShellAction::Dashboard, &cfg), SurfaceProvider::Noctalia);
        assert_eq!(router.provider_for(ShellAction::Session, &cfg), SurfaceProvider::Noctalia);
        assert_eq!(router.provider_for(ShellAction::Settings, &cfg), SurfaceProvider::Noctalia);
    }

    #[test]
    fn test_router_caelestia_mode() {
        let router = SolarSurfaceRouter::new();
        let cfg = SolarShellConfig {
            engine: ShellEngine::Caelestia,
            ..Default::default()
        };

        assert_eq!(router.provider_for(ShellAction::Launcher, &cfg), SurfaceProvider::Caelestia);
        assert_eq!(router.provider_for(ShellAction::Dashboard, &cfg), SurfaceProvider::Caelestia);
        assert_eq!(router.provider_for(ShellAction::Session, &cfg), SurfaceProvider::Caelestia);
    }

    #[test]
    fn test_router_hybrid_mode_custom_distribution() {
        let router = SolarSurfaceRouter::new();
        let cfg = SolarShellConfig {
            engine: ShellEngine::Hybrid,
            launcher_provider: SurfaceProvider::Caelestia,
            dashboard_provider: SurfaceProvider::Caelestia,
            bar_provider: SurfaceProvider::Noctalia,
            notifications_provider: SurfaceProvider::Noctalia,
            osd_provider: SurfaceProvider::Noctalia,
        };

        // Hybrid modunda Launcher ve Dashboard Caelestia'dan alınırken, Bar/Session Noctalia'dan alınır
        assert_eq!(router.provider_for(ShellAction::Launcher, &cfg), SurfaceProvider::Caelestia);
        assert_eq!(router.provider_for(ShellAction::Dashboard, &cfg), SurfaceProvider::Caelestia);
        assert_eq!(router.provider_for(ShellAction::Session, &cfg), SurfaceProvider::Noctalia);
        assert_eq!(router.provider_for(ShellAction::ControlCenter, &cfg), SurfaceProvider::Noctalia);
        assert_eq!(router.provider_for(ShellAction::Overview, &cfg), SurfaceProvider::SolarUI);
    }
}
