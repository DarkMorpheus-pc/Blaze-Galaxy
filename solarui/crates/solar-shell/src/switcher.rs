use crate::engine::ShellEngine;
use crate::providers::{CaelestiaProvider, NoctaliaProvider, ShellProvider};
use crate::supervisor::{ActualEngine, SolarShellSupervisor};
use anyhow::{bail, Context, Result};
use std::time::Duration;
use tracing::{info, warn};

pub struct SolarShellSwitcher {
    supervisor: SolarShellSupervisor,
    noctalia: Box<dyn ShellProvider>,
    caelestia: Box<dyn ShellProvider>,
    full_caelestia: Box<dyn ShellProvider>,
}

impl Default for SolarShellSwitcher {
    fn default() -> Self {
        Self {
            supervisor: SolarShellSupervisor::new(),
            noctalia: Box::new(NoctaliaProvider::new()),
            caelestia: Box::new(CaelestiaProvider::for_hybrid()),
            full_caelestia: Box::new(CaelestiaProvider::for_full()),
        }
    }
}

impl SolarShellSwitcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn switch_engine(&self, target: ShellEngine) -> Result<()> {
        self.switch(Some(target)).await
    }

    pub async fn reconcile(&self) -> Result<()> {
        self.switch(None).await
    }

    async fn actual(&self) -> ActualEngine {
        let (n, c_hyb, c_full) = tokio::join!(
            self.noctalia.health_check(),
            self.caelestia.health_check(),
            self.full_caelestia.health_check()
        );
        let c_alive = c_hyb.unwrap_or(false) || c_full.unwrap_or(false);
        match (n.unwrap_or(false), c_alive) {
            (true, true) => ActualEngine::Hybrid,
            (true, false) => ActualEngine::Noctalia,
            (false, true) => ActualEngine::Caelestia,
            _ => ActualEngine::None,
        }
    }

    async fn switch(&self, requested: Option<ShellEngine>) -> Result<()> {
        let lock_path = solar_common::get_solar_runtime_dir().join("shell-transition.lock");
        let _lock = if requested.is_some() {
            let mut acquired = None;
            for _ in 0..50 {
                match solar_common::persistence::try_lock(&lock_path) {
                    Ok(l) => {
                        acquired = Some(l);
                        break;
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
            acquired.context("Another shell transition is in progress and did not finish in time")?
        } else {
            solar_common::persistence::try_lock(&lock_path)
                .context("Another shell transition is in progress")?
        };
        // Read desired state AFTER obtaining the cross-process lock. The watchdog
        // must never apply an old choice after a concurrent user switch completes.
        let mut desired = self.supervisor.load_desired_state()?;
        let target = requested.unwrap_or(desired.desired_engine);
        let previous = self.actual().await;
        if !SolarShellSupervisor::is_reconciled(target, &previous) {
            self.preflight_check(target).await?;
            self.transition_with_rollback(target, &previous).await?;
        }
        if desired.desired_engine != target {
            desired.desired_engine = target;
            if let Err(error) = self.supervisor.save_desired_state(&desired) {
                let rollback = self.rollback(&previous).await;
                bail!("Cannot persist shell choice: {error:#}; rollback: {rollback:?}");
            }
        }
        Ok(())
    }

    async fn preflight_check(&self, target: ShellEngine) -> Result<()> {
        let provider = match target {
            ShellEngine::Noctalia | ShellEngine::Hybrid => &self.noctalia,
            ShellEngine::Caelestia => &self.full_caelestia,
        };
        anyhow::ensure!(
            provider.is_installed().await,
            "Target shell is not installed: {}",
            provider.name()
        );
        Ok(())
    }

    async fn transition_with_rollback(
        &self,
        target: ShellEngine,
        previous: &ActualEngine,
    ) -> Result<()> {
        if let Err(error) = self.transition(target).await {
            match self.rollback(previous).await {
                Ok(()) => bail!("Shell transition failed: {error:#}; previous shell restored"),
                Err(rollback) => {
                    bail!("Shell transition failed: {error:#}; rollback also failed: {rollback:#}")
                }
            }
        }
        info!("Shell transition verified: {target:?}");
        Ok(())
    }

    async fn transition(&self, target: ShellEngine) -> Result<()> {
        match target {
            ShellEngine::Noctalia => {
                self.caelestia.stop().await?;
                self.noctalia.start().await?;
            }
            ShellEngine::Caelestia => {
                let auto_scale = crate::settings_gui::detect_screen_resolution_and_auto_scale();
                crate::settings_gui::apply_display_scale_dynamically(auto_scale);

                self.caelestia.stop().await?;
                self.noctalia.stop().await?;
                self.full_caelestia.start().await?;
            }
            ShellEngine::Hybrid => {
                let auto_scale = crate::settings_gui::detect_screen_resolution_and_auto_scale();
                crate::settings_gui::apply_display_scale_dynamically(auto_scale);

                self.caelestia.stop().await?;
                self.noctalia.start().await?;
                if let Err(error) = self.caelestia.start().await {
                    warn!("Hybrid surfaces unavailable; Noctalia remains active: {error:#}");
                }
            }
        }
        for _ in 0..100 {
            if SolarShellSupervisor::is_reconciled(target, &self.actual().await) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        bail!("Target shell did not become healthy: {target:?}")
    }

    async fn rollback(&self, previous: &ActualEngine) -> Result<()> {
        let target = match previous {
            ActualEngine::Noctalia => ShellEngine::Noctalia,
            ActualEngine::Caelestia => ShellEngine::Caelestia,
            ActualEngine::Hybrid => ShellEngine::Hybrid,
            _ => bail!("No previously healthy shell is available"),
        };
        self.transition(target).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    struct Fake {
        alive: Arc<AtomicBool>,
        fail_start: bool,
        fail_stop: bool,
        lie_about_start: bool,
    }
    #[async_trait]
    impl ShellProvider for Fake {
        fn name(&self) -> &'static str {
            "test shell"
        }
        async fn is_installed(&self) -> bool {
            true
        }
        async fn start(&self) -> Result<()> {
            anyhow::ensure!(!self.fail_start, "start failed");
            if !self.lie_about_start {
                self.alive.store(true, Ordering::SeqCst);
            }
            Ok(())
        }
        async fn stop(&self) -> Result<()> {
            anyhow::ensure!(!self.fail_stop, "stop failed");
            self.alive.store(false, Ordering::SeqCst);
            Ok(())
        }
        async fn health_check(&self) -> Result<bool> {
            Ok(self.alive.load(Ordering::SeqCst))
        }
        async fn dispatch(&self, _: crate::actions::ShellAction) -> Result<()> {
            Ok(())
        }
    }
    fn fixture(n: bool, c: bool, fail_n: bool, fail_c: bool) -> SolarShellSwitcher {
        let ns = Arc::new(AtomicBool::new(n));
        let cs = Arc::new(AtomicBool::new(c));
        SolarShellSwitcher {
            supervisor: SolarShellSupervisor::new(),
            noctalia: Box::new(Fake {
                alive: ns,
                fail_start: fail_n,
                fail_stop: false,
                lie_about_start: false,
            }),
            caelestia: Box::new(Fake {
                alive: cs.clone(),
                fail_start: fail_c,
                fail_stop: false,
                lie_about_start: false,
            }),
            full_caelestia: Box::new(Fake {
                alive: cs,
                fail_start: fail_c,
                fail_stop: false,
                lie_about_start: false,
            }),
        }
    }
    #[tokio::test]
    async fn successful_spawn_without_health_is_not_a_successful_switch() {
        let mut switcher = fixture(false, true, false, false);
        switcher.noctalia = Box::new(Fake {
            alive: Arc::new(AtomicBool::new(false)),
            fail_start: false,
            fail_stop: false,
            lie_about_start: true,
        });
        assert!(switcher
            .transition_with_rollback(ShellEngine::Noctalia, &ActualEngine::Caelestia)
            .await
            .is_err());
        assert_eq!(switcher.actual().await, ActualEngine::Caelestia);
    }

    #[tokio::test]
    async fn failed_noctalia_switch_restores_caelestia() {
        let switcher = fixture(false, true, true, false);
        assert!(switcher
            .transition_with_rollback(ShellEngine::Noctalia, &ActualEngine::Caelestia)
            .await
            .is_err());
        assert_eq!(switcher.actual().await, ActualEngine::Caelestia);
    }
    #[tokio::test]
    async fn failed_caelestia_switch_restores_noctalia() {
        let switcher = fixture(true, false, false, true);
        assert!(switcher
            .transition_with_rollback(ShellEngine::Caelestia, &ActualEngine::Noctalia)
            .await
            .is_err());
        assert_eq!(switcher.actual().await, ActualEngine::Noctalia);
    }
    #[tokio::test]
    async fn failed_rollback_is_reported() {
        let switcher = fixture(true, false, true, true);
        let err = switcher
            .transition_with_rollback(ShellEngine::Caelestia, &ActualEngine::Noctalia)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("rollback also failed"));
    }
}
