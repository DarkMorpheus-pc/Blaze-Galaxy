use anyhow::{Context, Result};
use solar_common::{get_solar_core_socket_path, SolarCommand, SolarEvent, SolarSystemState};
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

pub struct ShellIpcClient {
    _state: Arc<RwLock<SolarSystemState>>,
    cmd_tx: mpsc::Sender<SolarCommand>,
}

impl ShellIpcClient {
    pub fn new(state: Arc<RwLock<SolarSystemState>>) -> (Self, mpsc::Receiver<SolarCommand>) {
        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        (
            Self {
                _state: state,
                cmd_tx,
            },
            cmd_rx,
        )
    }

    pub async fn send_command(&self, cmd: SolarCommand) -> Result<()> {
        self.cmd_tx
            .send(cmd)
            .await
            .context("Failed to send command to IPC worker")
    }

    pub async fn run_loop(
        state: Arc<RwLock<SolarSystemState>>,
        mut cmd_rx: mpsc::Receiver<SolarCommand>,
        event_tx: Option<async_channel::Sender<SolarEvent>>,
    ) -> Result<()> {
        let socket_path = get_solar_core_socket_path();

        loop {
            info!("Connecting to SolarCore IPC at {:?}...", socket_path);
            match UnixStream::connect(&socket_path).await {
                Ok(stream) => {
                    info!("Connected to SolarCore IPC successfully.");
                    let (reader, mut writer) = stream.into_split();
                    let mut reader =
                        solar_common::ipc::JsonLines::new(BufReader::new(reader), 8 * 1024 * 1024);

                    loop {
                        tokio::select! {
                            // Read events from SolarCore
                            res = reader.next_frame() => {
                                match res {
                                    Ok(None) => {
                                        warn!("SolarCore disconnected.");
                                        break;
                                    }
                                    Ok(Some(line)) => {
                                        if let Ok(event) = serde_json::from_slice::<SolarEvent>(&line) {
                                            {
                                                let mut s = state.write().await;
                                                match &event {
                                                    SolarEvent::FullState(full) => {
                                                        *s = full.clone();
                                                    }
                                                    SolarEvent::Battery(b) => s.battery = Some(b.clone()),
                                                    SolarEvent::Network(n) => s.network = n.clone(),
                                                    SolarEvent::Audio(a) => s.audio = a.clone(),
                                                    SolarEvent::Brightness(br) => s.brightness = Some(br.clone()),
                                                    SolarEvent::Bluetooth(bt) => s.bluetooth = bt.clone(),
                                                    SolarEvent::WindowList(w) => s.windows = w.clone(),
                                                    SolarEvent::Workspace(ws) => s.workspaces = ws.clone(),
                                                    SolarEvent::WindowRegistrySync(w) => s.window_registry = w.clone(),
                                                    SolarEvent::PerformanceProfileChanged(p) => s.performance_profile = p.clone(),
                                                    SolarEvent::CommandFailed { message } => warn!("Command failed: {message}"),
                                                    _ => {}
                                                }
                                            }
                                            if let Some(ref tx) = event_tx {
                                                let _ = tx.send(event).await;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        error!("Error reading from SolarCore: {}", err);
                                        break;
                                    }
                                }
                            }

                            // Write outgoing commands to SolarCore
                            Some(cmd) = cmd_rx.recv() => {
                                if let Ok(mut json) = serde_json::to_string(&cmd) {
                                    json.push('\n');
                                    if let Err(err) = async {
                                        tokio::time::timeout(std::time::Duration::from_secs(3), writer.write_all(json.as_bytes())).await??;
                                        Ok::<_, anyhow::Error>(())
                                    }.await {
                                        error!("Failed to write command to SolarCore: {}", err);
                                        break;
                                    }
                                    let _ = writer.flush().await;
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // Core might still be starting
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}
