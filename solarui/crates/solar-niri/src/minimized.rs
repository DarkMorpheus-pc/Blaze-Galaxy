use crate::NiriClient;
use anyhow::{Context, Result};
use niri_ipc::{Action, Request, Response, WorkspaceReferenceArg};
use solar_common::{MinimizedStore, MinimizedTransaction, MinimizedWindowRecord};

pub async fn minimize(client: &NiriClient, id: u64) -> Result<()> {
    let mut store = MinimizedStore::transaction()?;
    minimize_with_store(client, id, &mut store).await
}

pub async fn restore(client: &NiriClient, id: u64) -> Result<()> {
    let mut store = MinimizedStore::transaction()?;
    restore_with_store(client, id, &mut store).await
}

pub async fn toggle(client: &NiriClient, id: u64) -> Result<()> {
    let mut store = MinimizedStore::transaction()?;
    if store.get(id).is_some() {
        restore_with_store(client, id, &mut store).await
    } else {
        minimize_with_store(client, id, &mut store).await
    }
}

async fn minimize_with_store(
    client: &NiriClient,
    id: u64,
    store: &mut MinimizedTransaction,
) -> Result<()> {
    if store.get(id).is_none() {
        let window = client
            .get_windows()
            .await?
            .into_iter()
            .find(|w| w.id == id)
            .context("Window no longer exists")?;
        let Response::Workspaces(workspaces) = client.send_request(Request::Workspaces).await?
        else {
            anyhow::bail!("Invalid workspace response");
        };
        let parked_workspace_id = workspaces
            .iter()
            .find(|ws| ws.name.as_deref() == Some("background"))
            .map(|ws| ws.id)
            .context("The background workspace is not configured")?;
        store.insert(
            id,
            MinimizedWindowRecord {
                parked_workspace_id: Some(parked_workspace_id),
                orig_workspace_id: window.workspace_id.context("Window has no workspace")?,
                was_floating: window.is_floating,
                translated: false,
            },
        )?;
    }
    // Preserve geometry and layout mode. A fixed +1000px translation is not
    // reversible when the compositor clamps positions or the display changes.
    // Persist first so a timeout/crash never loses the recovery destination.
    client
        .move_window_to_workspace(
            Some(id),
            WorkspaceReferenceArg::Name("background".into()),
            false,
        )
        .await
}

async fn restore_with_store(
    client: &NiriClient,
    id: u64,
    store: &mut MinimizedTransaction,
) -> Result<()> {
    let Some(record) = store.get(id) else {
        return Ok(());
    };
    let workspaces = client.get_workspaces().await?;
    let destination = if workspaces.workspaces.contains(&record.orig_workspace_id) {
        record.orig_workspace_id
    } else {
        // Empty original workspaces may have disappeared or their output was unplugged.
        anyhow::ensure!(
            workspaces
                .workspaces
                .contains(&workspaces.current_workspace),
            "No focused workspace available for restore"
        );
        workspaces.current_workspace
    };
    client
        .move_window_to_workspace(Some(id), WorkspaceReferenceArg::Id(destination), true)
        .await?;
    if record.translated {
        if record.was_floating {
            client
                .send_request(Request::Action(Action::MoveFloatingWindow {
                    id: Some(id),
                    x: niri_ipc::PositionChange::AdjustFixed(0.0),
                    y: niri_ipc::PositionChange::AdjustFixed(-1000.0),
                }))
                .await?;
        } else {
            client
                .send_request(Request::Action(Action::MoveWindowToTiling { id: Some(id) }))
                .await?;
        }
        // Do not apply a legacy relative translation twice if focusing fails.
        store.insert(
            id,
            MinimizedWindowRecord {
                translated: false,
                ..record
            },
        )?;
    }
    client.focus_window(id).await?;
    store.remove(id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use niri_ipc::{Reply, Response, Workspace};
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::UnixListener,
    };

    fn workspaces(id: u64) -> Reply {
        Ok(Response::Workspaces(vec![Workspace {
            id,
            idx: 1,
            name: None,
            output: Some("test-output".into()),
            is_urgent: false,
            is_active: true,
            is_focused: true,
            active_window_id: Some(42),
        }]))
    }

    async fn mock(
        dir: &std::path::Path,
        replies: Vec<Reply>,
    ) -> (NiriClient, tokio::task::JoinHandle<Vec<Request>>) {
        let path = dir.join("niri.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let task = tokio::spawn(async move {
            let mut requests = Vec::new();
            for reply in replies {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut line = String::new();
                BufReader::new(&mut stream)
                    .read_line(&mut line)
                    .await
                    .unwrap();
                requests.push(serde_json::from_str(&line).unwrap());
                let mut bytes = serde_json::to_vec(&reply).unwrap();
                bytes.push(b'\n');
                stream.write_all(&bytes).await.unwrap();
            }
            requests
        });
        (NiriClient::with_socket_path(path), task)
    }

    #[tokio::test]
    async fn restore_uses_stable_id_larger_than_u8() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = MinimizedStore::transaction_at(&dir.path().join("state.json")).unwrap();
        store
            .insert(
                42,
                MinimizedWindowRecord {
                    orig_workspace_id: 900,
                    was_floating: true,
                    parked_workspace_id: Some(901),
                    translated: false,
                },
            )
            .unwrap();
        let (client, server) = mock(
            dir.path(),
            vec![
                workspaces(900),
                Ok(Response::Handled),
                Ok(Response::Handled),
            ],
        )
        .await;
        restore_with_store(&client, 42, &mut store).await.unwrap();
        let requests = server.await.unwrap();
        assert!(matches!(
            requests[1],
            Request::Action(Action::MoveWindowToWorkspace {
                reference: WorkspaceReferenceArg::Id(900),
                ..
            })
        ));
        assert!(store.get(42).is_none());
    }

    #[tokio::test]
    async fn failed_restore_preserves_recovery_record() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = MinimizedStore::transaction_at(&dir.path().join("state.json")).unwrap();
        store
            .insert(
                42,
                MinimizedWindowRecord {
                    orig_workspace_id: 900,
                    was_floating: false,
                    parked_workspace_id: Some(901),
                    translated: false,
                },
            )
            .unwrap();
        let (client, server) = mock(
            dir.path(),
            vec![workspaces(900), Err("compositor rejected move".into())],
        )
        .await;
        assert!(restore_with_store(&client, 42, &mut store).await.is_err());
        server.await.unwrap();
        assert_eq!(store.get(42).unwrap().orig_workspace_id, 900);
    }

    #[tokio::test]
    async fn missing_original_workspace_uses_focused_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = MinimizedStore::transaction_at(&dir.path().join("state.json")).unwrap();
        store
            .insert(
                42,
                MinimizedWindowRecord {
                    orig_workspace_id: 900,
                    was_floating: false,
                    parked_workspace_id: Some(901),
                    translated: false,
                },
            )
            .unwrap();
        let (client, server) = mock(
            dir.path(),
            vec![
                workspaces(777),
                Ok(Response::Handled),
                Ok(Response::Handled),
            ],
        )
        .await;
        restore_with_store(&client, 42, &mut store).await.unwrap();
        assert!(matches!(
            server.await.unwrap()[1],
            Request::Action(Action::MoveWindowToWorkspace {
                reference: WorkspaceReferenceArg::Id(777),
                ..
            })
        ));
    }
}
