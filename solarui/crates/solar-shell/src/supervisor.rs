use anyhow::Result;
use serde::{Deserialize, Serialize};
use solar_common::SolarConfig;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::engine::ShellEngine;
use crate::providers::{CaelestiaProvider, NoctaliaProvider, ShellProvider};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DesiredState {
    #[serde(default)]
    pub desired_engine: ShellEngine,
    #[serde(default = "default_profile")]
    pub desired_profile: String,
}

fn default_profile() -> String {
    "balanced".to_string()
}

impl Default for DesiredState {
    fn default() -> Self {
        Self {
            desired_engine: ShellEngine::Noctalia,
            desired_profile: default_profile(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualEngine {
    Noctalia,
    Caelestia,
    Hybrid,
    None,
    Failed(String),
}

impl ActualEngine {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActualEngine::Noctalia => "noctalia",
            ActualEngine::Caelestia => "caelestia",
            ActualEngine::Hybrid => "hybrid",
            ActualEngine::None => "none",
            ActualEngine::Failed(_) => "failed",
        }
    }
}

pub struct SolarShellSupervisor {
    state_file: PathBuf,
    noctalia: NoctaliaProvider,
    caelestia: CaelestiaProvider,
}

impl Default for SolarShellSupervisor {
    fn default() -> Self {
        let state_file = solar_common::get_solar_config_dir().join("shell_state.json");
        Self {
            state_file,
            noctalia: NoctaliaProvider::new(),
            caelestia: CaelestiaProvider::new(),
        }
    }
}

impl SolarShellSupervisor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Single Source-of-Truth: SolarConfig üzerinden istenen durumu okur.
    pub fn load_desired_state(&self) -> Result<DesiredState> {
        let cfg = SolarConfig::try_load()?;
        Ok(DesiredState {
            desired_engine: cfg.shell.engine,
            desired_profile: cfg.performance.profile.as_str().to_string(),
        })
    }

    pub fn save_desired_state(&self, state: &DesiredState) -> Result<()> {
        let profile = state
            .desired_profile
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;
        SolarConfig::update(|cfg| {
            cfg.shell.engine = state.desired_engine;
            cfg.performance.profile = profile;
        })?;
        // Config is authoritative; a diagnostic mirror failure must not undo a
        // successful transition or make the watchdog rewrite the configuration.
        if let Err(e) = solar_common::persistence::atomic_write(
            &self.state_file,
            &serde_json::to_vec_pretty(state)?,
        ) {
            warn!("Could not update diagnostic shell state: {e}");
        }
        Ok(())
    }

    pub fn is_reconciled(desired: ShellEngine, actual: &ActualEngine) -> bool {
        match (desired, actual) {
            (ShellEngine::Noctalia, ActualEngine::Noctalia) => true,
            (ShellEngine::Caelestia, ActualEngine::Caelestia) => true,
            (ShellEngine::Hybrid, ActualEngine::Noctalia | ActualEngine::Hybrid) => true,
            _ => false,
        }
    }

    pub async fn detect_actual_engine(&self) -> ActualEngine {
        let n_alive = self.noctalia.health_check().await.unwrap_or(false);
        let c_alive = self.caelestia.health_check().await.unwrap_or(false);

        match (n_alive, c_alive) {
            (true, true) => ActualEngine::Hybrid,
            (true, false) => ActualEngine::Noctalia,
            (false, true) => ActualEngine::Caelestia,
            (false, false) => ActualEngine::None,
        }
    }

    pub async fn reconcile(&self) -> Result<()> {
        let desired = self.load_desired_state()?;
        let actual = self.detect_actual_engine().await;
        if Self::is_reconciled(desired.desired_engine, &actual) {
            return Ok(());
        }
        crate::switcher::SolarShellSwitcher::new().reconcile().await
    }

    pub fn start_watchdog_loop(self: std::sync::Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("SolarShell watchdog started");
            let mut delay = 3;
            loop {
                // Delay after completion: do not replay missed ticks in a restart storm.
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                match self.reconcile().await {
                    Ok(()) => delay = 3,
                    Err(e) => {
                        tracing::error!("Watchdog reconciliation failed: {e:#}");
                        delay = (delay * 2).min(60);
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_reconciled_matching() {
        assert!(SolarShellSupervisor::is_reconciled(
            ShellEngine::Noctalia,
            &ActualEngine::Noctalia
        ));
        assert!(SolarShellSupervisor::is_reconciled(
            ShellEngine::Caelestia,
            &ActualEngine::Caelestia
        ));
        // Hybrid modunda Noctalia omurgası yeterlidir
        assert!(SolarShellSupervisor::is_reconciled(
            ShellEngine::Hybrid,
            &ActualEngine::Noctalia
        ));
        assert!(SolarShellSupervisor::is_reconciled(
            ShellEngine::Hybrid,
            &ActualEngine::Hybrid
        ));

        // Uyuşmazlık durumları
        assert!(!SolarShellSupervisor::is_reconciled(
            ShellEngine::Noctalia,
            &ActualEngine::Caelestia
        ));
        assert!(!SolarShellSupervisor::is_reconciled(
            ShellEngine::Caelestia,
            &ActualEngine::Noctalia
        ));
        assert!(!SolarShellSupervisor::is_reconciled(
            ShellEngine::Noctalia,
            &ActualEngine::None
        ));
    }
}
