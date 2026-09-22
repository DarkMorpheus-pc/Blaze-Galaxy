use anyhow::{Context, Result};
use solar_common::{SolarEvent, SolarSystemState};
use solar_niri::{NiriClient, NiriEventSubscriber};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};
use zbus::Connection;

use crate::dbus_clients::network::{get_network_status, run_network_listener};
use crate::dbus_clients::upower::{get_battery_status, run_upower_listener};
use crate::ipc_server::SolarIpcServer;

pub struct SolarPlatformDaemon {
    state: Arc<RwLock<SolarSystemState>>,
    event_tx: broadcast::Sender<SolarEvent>,
}

impl SolarPlatformDaemon {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(128);
        Self {
            state: Arc::new(RwLock::new(SolarSystemState::default())),
            event_tx,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("Connecting to D-Bus system bus...");
        let system_conn = Connection::system()
            .await
            .context("Failed to connect to D-Bus system bus")?;

        // 1. Initial State Hydration
        let registry = Arc::new(RwLock::new(crate::registry::SolarWindowRegistry::new()));
        {
            let mut state = self.state.write().await;
            state.performance_profile = solar_common::HardwareTuningProfile::load();

            if let Ok(battery) = get_battery_status(&system_conn).await {
                info!(
                    "Initial battery state: {}% ({:?})",
                    battery.percentage, battery.state
                );
                state.battery = Some(battery);
            }
            if let Ok(network) = get_network_status(&system_conn).await {
                info!(
                    "Initial network state: connected={}, wifi={}",
                    network.is_connected, network.is_wifi
                );
                state.network = network;
            }
            if let Ok(niri) = NiriClient::new() {
                if let Ok(ws) = niri.get_workspaces().await {
                    info!("Initial Niri workspace: {}", ws.current_workspace);
                    state.workspaces = ws;
                }
                if let Ok(windows) = niri.get_windows().await {
                    info!("Initial Niri window count: {}", windows.len());
                    state.windows = windows.clone();
                    let reg_states = registry
                        .write()
                        .await
                        .update_from_niri(windows, state.workspaces.current_workspace);
                    state.window_registry = reg_states;
                }
            }
        }

        // Producers use a bounded lossless queue. Only committed state is broadcast
        // to clients, so a slow UI can resynchronize without corrupting core state.
        let (update_tx, update_rx) = mpsc::channel(128);
        tokio::spawn(sync_state(
            update_rx,
            self.state.clone(),
            registry,
            self.event_tx.clone(),
        ));

        // 3. Spawn UPower Listener
        let upower_conn = system_conn.clone();
        let upower_tx = update_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Err(err) = run_upower_listener(upower_conn.clone(), upower_tx.clone()).await
                {
                    warn!("UPower listener terminated: {err}; retrying");
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });

        // 4. Spawn NetworkManager Listener
        let nm_conn = system_conn.clone();
        let nm_tx = update_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Err(err) = run_network_listener(nm_conn.clone(), nm_tx.clone()).await {
                    warn!("NetworkManager listener terminated: {err}; retrying");
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });

        // 5. Spawn Niri Event Subscriber (if Niri is active)
        let niri_tx = update_tx.clone();
        tokio::spawn(async move {
            loop {
                match NiriEventSubscriber::new() {
                    Ok(subscriber) => {
                        if let Err(err) = subscriber.start_stream(niri_tx.clone()).await {
                            debug!("Niri stream disconnected: {}, retrying in 3s...", err);
                        }
                    }
                    Err(_) => {
                        // Niri might not be started yet (e.g. during bootup race)
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        // 6. Spawn IPC Server
        let ipc_server = SolarIpcServer::new(
            self.state.clone(),
            self.event_tx.clone(),
            system_conn.clone(),
        );

        tokio::select! {
            res = ipc_server.run() => {
                res.context("IPC server stopped")?;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("SolarCore received SIGINT, shutting down cleanly.");
            }
        }

        Ok(())
    }
}

async fn sync_state(
    mut updates: mpsc::Receiver<SolarEvent>,
    state: Arc<RwLock<SolarSystemState>>,
    registry: Arc<RwLock<crate::registry::SolarWindowRegistry>>,
    events: broadcast::Sender<SolarEvent>,
) {
    while let Some(event) = updates.recv().await {
        let mut s = state.write().await;
        let mut registry_changed = false;
        match &event {
            SolarEvent::Battery(b) => s.battery = Some(b.clone()),
            SolarEvent::Network(n) => s.network = n.clone(),
            SolarEvent::Audio(a) => s.audio = a.clone(),
            SolarEvent::Brightness(b) => s.brightness = Some(b.clone()),
            SolarEvent::Bluetooth(b) => s.bluetooth = b.clone(),
            SolarEvent::PerformanceProfileChanged(p) => s.performance_profile = p.clone(),
            SolarEvent::WindowList(w) => {
                s.windows = w.clone();
                registry_changed = true;
            }
            SolarEvent::Workspace(ws) => {
                s.workspaces = ws.clone();
                registry_changed = true;
            }
            _ => {}
        }
        if registry_changed {
            s.window_registry = registry
                .write()
                .await
                .update_from_niri(s.windows.clone(), s.workspaces.current_workspace);
        }
        // Broadcast while holding the state lock: snapshot + receiver reset is atomic
        // relative to all published events.
        let _ = events.send(event);
        if registry_changed {
            let _ = events.send(SolarEvent::WindowRegistrySync(s.window_registry.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn slow_subscriber_does_not_stop_state_updates() {
        let state = Arc::new(RwLock::new(SolarSystemState::default()));
        let (updates, rx) = mpsc::channel(2);
        let (events, mut slow) = broadcast::channel(2);
        let task = tokio::spawn(sync_state(
            rx,
            state.clone(),
            Arc::new(RwLock::new(crate::registry::SolarWindowRegistry::new())),
            events,
        ));
        for pct in 0..100 {
            updates
                .send(SolarEvent::Brightness(solar_common::BrightnessStatus {
                    percentage: pct,
                }))
                .await
                .unwrap();
        }
        drop(updates);
        task.await.unwrap();
        assert_eq!(
            state.read().await.brightness.as_ref().unwrap().percentage,
            99
        );
        assert!(matches!(
            slow.recv().await,
            Err(broadcast::error::RecvError::Lagged(_))
        ));
    }
}
