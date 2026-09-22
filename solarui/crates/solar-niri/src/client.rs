use anyhow::{bail, Context, Result};
use niri_ipc::{Action, Event, Reply, Request, Response, Window};
use solar_common::ipc::JsonLines;
use solar_common::{SolarEvent, SolarWindowInfo, SolarWorkspaceInfo};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::info;

pub struct NiriClient {
    socket_path: PathBuf,
}

impl NiriClient {
    pub fn new() -> Result<Self> {
        let socket_path = find_niri_socket()?;
        Ok(Self { socket_path })
    }

    pub fn with_socket_path(path: PathBuf) -> Self {
        Self { socket_path: path }
    }

    pub async fn send_request(&self, request: Request) -> Result<Response> {
        tokio::time::timeout(Duration::from_secs(3), self.request_inner(request))
            .await
            .context("Niri request timed out")?
    }

    async fn request_inner(&self, request: Request) -> Result<Response> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .with_context(|| {
                format!("Failed to connect to niri socket at {:?}", self.socket_path)
            })?;

        let (reader, mut writer) = stream.split();
        let mut reader = JsonLines::new(BufReader::new(reader), 8 * 1024 * 1024);

        let mut req_str = serde_json::to_string(&request)?;
        req_str.push('\n');
        writer.write_all(req_str.as_bytes()).await?;
        writer.flush().await?;

        let line = reader
            .next_frame()
            .await?
            .context("Niri closed before replying")?;
        let reply: Reply = serde_json::from_slice(&line).context("Invalid Niri reply")?;

        match reply {
            Ok(response) => Ok(response),
            Err(err) => bail!("Niri returned error: {err}"),
        }
    }

    pub async fn get_windows(&self) -> Result<Vec<SolarWindowInfo>> {
        let response = self.send_request(Request::Windows).await?;
        if let Response::Windows(windows) = response {
            Ok(windows.into_iter().map(map_window).collect())
        } else {
            bail!(
                "Unexpected response from niri for Windows request: {:?}",
                response
            )
        }
    }

    pub async fn get_workspaces(&self) -> Result<SolarWorkspaceInfo> {
        let response = self.send_request(Request::Workspaces).await?;
        if let Response::Workspaces(workspaces) = response {
            let mut active_id = 1;
            let mut ids = Vec::new();
            for ws in &workspaces {
                ids.push(ws.id);
                if ws.is_focused {
                    active_id = ws.id;
                }
            }
            Ok(SolarWorkspaceInfo {
                current_workspace: active_id,
                workspaces: ids,
                is_overview_open: false,
            })
        } else {
            bail!(
                "Unexpected response from niri for Workspaces request: {:?}",
                response
            )
        }
    }

    pub async fn focus_window(&self, id: u64) -> Result<()> {
        self.send_request(Request::Action(Action::FocusWindow { id }))
            .await?;
        Ok(())
    }

    pub async fn close_window(&self, id: u64) -> Result<()> {
        self.send_request(Request::Action(Action::CloseWindow { id: Some(id) }))
            .await?;
        Ok(())
    }

    pub async fn toggle_window_floating(&self, id: Option<u64>) -> Result<()> {
        self.send_request(Request::Action(Action::ToggleWindowFloating { id }))
            .await?;
        Ok(())
    }

    pub async fn maximize_column(&self) -> Result<()> {
        self.send_request(Request::Action(Action::MaximizeColumn {}))
            .await?;
        Ok(())
    }

    pub async fn toggle_overview(&self) -> Result<()> {
        self.send_request(Request::Action(Action::ToggleOverview {}))
            .await?;
        Ok(())
    }

    pub async fn switch_workspace(&self, id: u64) -> Result<()> {
        self.send_request(Request::Action(Action::FocusWorkspace {
            reference: niri_ipc::WorkspaceReferenceArg::Id(id),
        }))
        .await?;
        Ok(())
    }

    pub async fn move_window_to_workspace(
        &self,
        window_id: Option<u64>,
        reference: niri_ipc::WorkspaceReferenceArg,
        focus: bool,
    ) -> Result<()> {
        self.send_request(Request::Action(Action::MoveWindowToWorkspace {
            window_id,
            reference,
            focus,
        }))
        .await?;
        Ok(())
    }

    pub async fn get_focused_window(&self) -> Result<Option<SolarWindowInfo>> {
        let windows = self.get_windows().await?;
        Ok(windows.into_iter().find(|w| w.is_focused))
    }
}

pub struct NiriEventSubscriber {
    socket_path: PathBuf,
}

impl NiriEventSubscriber {
    pub fn new() -> Result<Self> {
        let socket_path = find_niri_socket()?;
        Ok(Self { socket_path })
    }

    pub async fn start_stream(self, event_tx: mpsc::Sender<SolarEvent>) -> Result<()> {
        info!("Connecting to Niri event stream at {:?}", self.socket_path);
        let mut stream = tokio::time::timeout(
            Duration::from_secs(3),
            UnixStream::connect(&self.socket_path),
        )
        .await??;
        let (reader, mut writer) = stream.split();
        let mut reader = JsonLines::new(BufReader::new(reader), 8 * 1024 * 1024);
        let mut request = serde_json::to_vec(&Request::EventStream)?;
        request.push(b'\n');
        tokio::time::timeout(Duration::from_secs(3), writer.write_all(&request)).await??;
        let reply = tokio::time::timeout(Duration::from_secs(3), reader.next_frame())
            .await??
            .context("Niri closed event stream before handshake")?;
        match serde_json::from_slice::<Reply>(&reply)? {
            Ok(Response::Handled) => {}
            other => bail!("Unexpected event stream reply: {other:?}"),
        }
        let mut cache = EventCache::default();
        while let Some(line) = reader.next_frame().await? {
            let event: Event = serde_json::from_slice(&line)?;
            if let Event::WindowClosed { id } = &event {
                let id = *id;
                // Wait off the stream reader if a minimize/restore transaction owns
                // the file lock. Closing a window must not leave a permanent record.
                tokio::spawn(async move {
                    for _ in 0..100 {
                        match solar_common::MinimizedStore::transaction() {
                            Ok(mut store) => {
                                if store.get(id).is_some() {
                                    if let Err(error) = store.remove(id) {
                                        tracing::warn!("Cannot clean closed window {id}: {error}");
                                    }
                                }
                                return;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                            Err(error) => {
                                tracing::warn!("Cannot clean closed window {id}: {error}");
                                return;
                            }
                        }
                    }
                    tracing::warn!("Timed out cleaning closed window {id}");
                });
            }
            for update in cache.apply(event) {
                event_tx
                    .send(update)
                    .await
                    .context("Core state receiver stopped")?;
            }
        }
        Ok(())
    }
}

/// Niri's stream starts with full workspace/window snapshots, followed by deltas.
/// Applying those deltas avoids opening a socket and querying every window on each focus change.
#[derive(Default)]
struct EventCache {
    windows: BTreeMap<u64, SolarWindowInfo>,
    workspaces: SolarWorkspaceInfo,
}

impl EventCache {
    fn apply(&mut self, event: Event) -> Vec<SolarEvent> {
        let mut window_changed = false;
        let mut workspace_changed = false;
        match event {
            Event::WindowsChanged { windows } => {
                self.windows = windows.into_iter().map(|w| (w.id, map_window(w))).collect();
                window_changed = true;
            }
            Event::WindowOpenedOrChanged { window } => {
                if window.is_focused {
                    for win in self.windows.values_mut() {
                        win.is_focused = false;
                    }
                }
                self.windows.insert(window.id, map_window(window));
                window_changed = true;
            }
            Event::WindowClosed { id } => {
                self.windows.remove(&id);
                window_changed = true;
            }
            Event::WindowFocusChanged { id } => {
                for win in self.windows.values_mut() {
                    win.is_focused = Some(win.id) == id;
                }
                window_changed = true;
            }
            Event::WorkspacesChanged { workspaces } => {
                self.workspaces.current_workspace =
                    workspaces.iter().find(|w| w.is_focused).map_or(0, |w| w.id);
                self.workspaces.workspaces = workspaces.into_iter().map(|w| w.id).collect();
                workspace_changed = true;
            }
            Event::WorkspaceActivated { id, focused: true } => {
                self.workspaces.current_workspace = id;
                workspace_changed = true;
            }
            Event::OverviewOpenedOrClosed { is_open } => {
                self.workspaces.is_overview_open = is_open;
                workspace_changed = true;
            }
            _ => {}
        }
        let mut updates = Vec::new();
        if workspace_changed {
            updates.push(SolarEvent::Workspace(self.workspaces.clone()));
        }
        if window_changed {
            updates.push(SolarEvent::WindowList(
                self.windows.values().cloned().collect(),
            ));
        }
        updates
    }
}

pub(crate) fn map_window(w: Window) -> SolarWindowInfo {
    SolarWindowInfo {
        id: w.id,
        title: w.title,
        app_id: w.app_id,
        is_focused: w.is_focused,
        is_floating: w.is_floating,
        workspace_id: w.workspace_id,
    }
}

pub fn find_niri_socket() -> Result<PathBuf> {
    if let Ok(sock) = std::env::var("NIRI_SOCKET") {
        let p = PathBuf::from(sock);
        if p.exists() {
            return Ok(p);
        }
    }

    // Try finding in XDG_RUNTIME_DIR
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = Path::new(&runtime_dir);
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("niri") && name_str.ends_with(".sock") {
                    return Ok(entry.path());
                }
            }
        }
    }

    bail!("Could not find running Niri socket. Is Niri running and $NIRI_SOCKET or $XDG_RUNTIME_DIR set?")
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    #[test]
    fn secondary_monitor_activation_does_not_steal_focused_workspace_or_reset_overview() {
        let mut cache = EventCache::default();
        cache.apply(Event::WorkspaceActivated {
            id: 900,
            focused: true,
        });
        cache.apply(Event::OverviewOpenedOrClosed { is_open: true });
        assert!(cache
            .apply(Event::WorkspaceActivated {
                id: 777,
                focused: false
            })
            .is_empty());
        assert_eq!(cache.workspaces.current_workspace, 900);
        assert!(cache.workspaces.is_overview_open);
    }
    #[test]
    fn focus_and_close_deltas_update_cached_windows_without_queries() {
        let mut cache = EventCache::default();
        cache.windows.insert(
            1,
            SolarWindowInfo {
                id: 1,
                is_focused: true,
                ..Default::default()
            },
        );
        cache.windows.insert(
            2,
            SolarWindowInfo {
                id: 2,
                ..Default::default()
            },
        );
        cache.apply(Event::WindowFocusChanged { id: Some(2) });
        assert!(!cache.windows[&1].is_focused);
        assert!(cache.windows[&2].is_focused);
        cache.apply(Event::WindowClosed { id: 2 });
        assert_eq!(cache.windows.len(), 1);
        cache.apply(Event::WindowFocusChanged { id: None });
        assert!(!cache.windows[&1].is_focused);
    }
    #[tokio::test]
    async fn request_to_unresponsive_compositor_times_out() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("niri.sock");
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            std::future::pending::<()>().await;
        });
        let error = NiriClient::with_socket_path(path)
            .send_request(Request::Windows)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("timed out"));
        server.abort();
    }
}
