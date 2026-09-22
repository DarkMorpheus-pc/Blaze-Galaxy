use anyhow::{Context, Result};
use solar_common::ipc::JsonLines;
use solar_common::{
    get_solar_core_socket_path, get_solar_runtime_dir, PowerAction, SolarCommand, SolarEvent,
    SolarSystemState,
};
use solar_niri::NiriClient;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use zbus::Connection;

use crate::dbus_clients::logind::{
    execute_lock, execute_power_off, execute_reboot, execute_suspend,
};

pub struct SolarIpcServer {
    state: Arc<RwLock<SolarSystemState>>,
    event_tx: broadcast::Sender<SolarEvent>,
    system_conn: Connection,
}

impl SolarIpcServer {
    pub fn new(
        state: Arc<RwLock<SolarSystemState>>,
        event_tx: broadcast::Sender<SolarEvent>,
        system_conn: Connection,
    ) -> Self {
        Self {
            state,
            event_tx,
            system_conn,
        }
    }

    pub async fn run(self) -> Result<()> {
        let runtime_dir = get_solar_runtime_dir();
        std::fs::create_dir_all(&runtime_dir)
            .with_context(|| format!("Failed to create runtime dir {:?}", runtime_dir))?;

        let socket_path = get_solar_core_socket_path();
        let (_instance_lock, listener) = bind_exclusive(&socket_path).await?;
        info!("SolarCore IPC server listening on {:?}", socket_path);

        let server_state = self.state.clone();
        let event_tx = self.event_tx.clone();
        let system_conn = self.system_conn.clone();

        let clients = Arc::new(Semaphore::new(64));
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let Ok(permit) = clients.clone().try_acquire_owned() else {
                        continue;
                    };
                    let state = server_state.clone();
                    let mut rx = event_tx.subscribe();
                    let sys_conn = system_conn.clone();

                    tokio::spawn(async move {
                        let _permit = permit;
                        if let Err(err) = handle_client(stream, state, &mut rx, sys_conn).await {
                            debug!("Client disconnected: {}", err);
                        }
                    });
                }
                Err(err) => {
                    error!("Error accepting IPC client connection: {}", err);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

async fn handle_client(
    mut stream: UnixStream,
    state: Arc<RwLock<SolarSystemState>>,
    rx: &mut broadcast::Receiver<SolarEvent>,
    system_conn: Connection,
) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = JsonLines::new(BufReader::new(reader), 64 * 1024);
    write_event(&mut writer, &snapshot_and_reset(&state, rx).await).await?;
    loop {
        tokio::select! {
            result = rx.recv() => {
                let event = match result {
                    Ok(event) => event,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "IPC client lagged; sending a new snapshot");
                        snapshot_and_reset(&state, rx).await
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                write_event(&mut writer, &event).await?;
            }
            frame = reader.next_frame() => {
                let Some(frame) = frame? else { break; };
                match serde_json::from_slice::<SolarCommand>(&frame) {
                    Ok(SolarCommand::RequestStateSync) => {
                        write_event(&mut writer, &snapshot_and_reset(&state, rx).await).await?;
                    }
                    Ok(cmd) => {
                        if let Err(err) = dispatch_command(cmd, &system_conn).await {
                            warn!("SolarCore command failed: {err:#}");
                            write_event(&mut writer, &SolarEvent::CommandFailed { message: format!("{err:#}") }).await?;
                        }
                    }
                    Err(err) => {
                        warn!("Invalid command JSON: {err}");
                        write_event(&mut writer, &SolarEvent::CommandFailed { message: "Invalid command JSON".into() }).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn snapshot_and_reset(
    state: &Arc<RwLock<SolarSystemState>>,
    rx: &mut broadcast::Receiver<SolarEvent>,
) -> SolarEvent {
    let state = state.read().await;
    *rx = rx.resubscribe();
    SolarEvent::FullState(state.clone())
}

async fn write_event<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    event: &SolarEvent,
) -> Result<()> {
    let mut bytes = serde_json::to_vec(event)?;
    bytes.push(b'\n');
    tokio::time::timeout(Duration::from_secs(3), writer.write_all(&bytes))
        .await
        .context("IPC client stopped reading")??;
    Ok(())
}

async fn bind_exclusive(path: &Path) -> Result<(std::fs::File, UnixListener)> {
    use std::os::unix::fs::FileTypeExt;
    let lock = solar_common::persistence::try_lock(&path.with_extension("lock"))
        .context("Another SolarCore instance owns the socket")?;
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            anyhow::ensure!(
                meta.file_type().is_socket(),
                "Refusing to remove a non-socket IPC path"
            );
            // Also protect a live older daemon which does not yet use our lock.
            match tokio::time::timeout(Duration::from_millis(500), UnixStream::connect(path)).await
            {
                Ok(Err(err))
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
                    ) =>
                {
                    match std::fs::remove_file(path) {
                        Ok(()) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => return Err(e.into()),
                    }
                }
                _ => anyhow::bail!("IPC socket is live or its ownership cannot be determined"),
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let listener = UnixListener::bind(path)?;
    Ok((lock, listener))
}

async fn dispatch_command(cmd: SolarCommand, system_conn: &Connection) -> Result<()> {
    match cmd {
        SolarCommand::Power(action) => {
            let operation = async {
                match action {
                    PowerAction::Lock => execute_lock(system_conn).await,
                    PowerAction::Suspend => execute_suspend(system_conn).await,
                    PowerAction::Reboot => execute_reboot(system_conn).await,
                    PowerAction::PowerOff => execute_power_off(system_conn).await,
                }
            };
            tokio::time::timeout(Duration::from_secs(30), operation)
                .await
                .context("Power operation timed out")??;
        }
        SolarCommand::SetVolume(vol) => {
            run_command(
                "wpctl",
                &[
                    "set-volume",
                    "@DEFAULT_AUDIO_SINK@",
                    &format!("{}%", vol.min(100)),
                ],
            )
            .await?
        }
        SolarCommand::ToggleMute => {
            run_command("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).await?
        }
        SolarCommand::SetBrightness(pct) => {
            run_command("brightnessctl", &["set", &format!("{}%", pct.min(100))]).await?
        }
        SolarCommand::ToggleWifi(enable) => {
            run_command(
                "nmcli",
                &["radio", "wifi", if enable { "on" } else { "off" }],
            )
            .await?
        }
        SolarCommand::ToggleBluetooth(enable) => {
            run_command(
                "bluetoothctl",
                &["power", if enable { "on" } else { "off" }],
            )
            .await?
        }
        SolarCommand::LaunchApp { exec } => {
            let parts: Vec<&str> = exec.split_whitespace().collect();
            let (bin, args) = parts.split_first().context("Empty launch command")?;
            let mut child = tokio::process::Command::new(bin).args(args).spawn()?;
            tokio::spawn(async move {
                let _ = child.wait().await;
            });
        }
        SolarCommand::FocusWindow(id) => NiriClient::new()?.focus_window(id).await?,
        SolarCommand::CloseWindow(id) => NiriClient::new()?.close_window(id).await?,
        SolarCommand::ToggleWindowFloating(id) => {
            NiriClient::new()?.toggle_window_floating(Some(id)).await?
        }
        SolarCommand::MinimizeWindow(id) => {
            solar_niri::minimized::minimize(&NiriClient::new()?, id).await?
        }
        SolarCommand::RestoreWindow(id) => {
            solar_niri::minimized::restore(&NiriClient::new()?, id).await?
        }
        SolarCommand::ToggleMinimizeWindow(id) => {
            solar_niri::minimized::toggle(&NiriClient::new()?, id).await?
        }
        SolarCommand::MaximizeColumn => NiriClient::new()?.maximize_column().await?,
        SolarCommand::ToggleOverview => NiriClient::new()?.toggle_overview().await?,
        SolarCommand::SwitchWorkspace(id) => NiriClient::new()?.switch_workspace(id).await?,
        SolarCommand::RequestStateSync => {} // handled by the client connection
    }
    Ok(())
}

async fn run_command(bin: &str, args: &[&str]) -> Result<()> {
    let status = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new(bin)
            .args(args)
            .kill_on_drop(true)
            .status(),
    )
    .await
    .context("Command timed out")??;
    anyhow::ensure!(status.success(), "{bin} failed with {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn duplicate_core_does_not_replace_the_live_socket() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("core.sock");
        let (_lock, listener) = bind_exclusive(&path).await.unwrap();
        assert!(bind_exclusive(&path).await.is_err());
        let _client = UnixStream::connect(&path).await.unwrap();
        assert!(listener.accept().await.is_ok());
    }
    #[tokio::test]
    async fn protects_legacy_socket_and_recovers_stale_socket() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("core.sock");
        let legacy = UnixListener::bind(&path).unwrap();
        assert!(bind_exclusive(&path).await.is_err());
        drop(legacy);
        assert!(bind_exclusive(&path).await.is_ok());
    }
    #[tokio::test]
    async fn snapshot_discards_obsolete_queued_events() {
        let state = Arc::new(RwLock::new(SolarSystemState::default()));
        let (tx, mut rx) = broadcast::channel(2);
        for pct in 0..10 {
            tx.send(SolarEvent::Brightness(solar_common::BrightnessStatus {
                percentage: pct,
            }))
            .unwrap();
        }
        state.write().await.brightness = Some(solar_common::BrightnessStatus { percentage: 9 });
        let SolarEvent::FullState(full) = snapshot_and_reset(&state, &mut rx).await else {
            panic!()
        };
        assert_eq!(full.brightness.unwrap().percentage, 9);
        assert!(matches!(
            rx.try_recv(),
            Err(broadcast::error::TryRecvError::Empty)
        ));
        tx.send(SolarEvent::Brightness(solar_common::BrightnessStatus {
            percentage: 10,
        }))
        .unwrap();
        assert!(
            matches!(rx.recv().await.unwrap(), SolarEvent::Brightness(b) if b.percentage == 10)
        );
    }
}
