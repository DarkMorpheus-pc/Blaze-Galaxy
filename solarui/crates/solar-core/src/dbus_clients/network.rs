use anyhow::Result;
use futures_util::StreamExt;
use solar_common::{NetworkStatus, SolarEvent};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use zbus::Connection;

#[zbus::proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
pub trait NetworkManager {
    #[zbus(property)]
    fn primary_connection_type(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;
}

pub async fn get_network_status(conn: &Connection) -> Result<NetworkStatus> {
    let proxy = NetworkManagerProxy::new(conn).await?;
    let conn_type = proxy.primary_connection_type().await.unwrap_or_default();
    let connectivity = proxy.connectivity().await.unwrap_or(0);
    let is_connected = connectivity >= 3; // 3 = LIMITED, 4 = FULL

    let is_wifi = conn_type == "802-11-wireless";

    let mut ssid = None;
    let mut signal_strength = 0;

    if is_wifi {
        // Try getting SSID from iwgetid or nmcli quick status
        if let Ok(Ok(out)) = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::process::Command::new("iwgetid")
                .arg("-r")
                .kill_on_drop(true)
                .output(),
        )
        .await
        {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                ssid = Some(s);
            }
        }
        signal_strength = 80; // default active wifi representation
    }

    Ok(NetworkStatus {
        is_connected,
        is_wifi,
        ssid,
        signal_strength,
        ip_address: None,
    })
}

pub async fn run_network_listener(
    conn: Connection,
    event_tx: mpsc::Sender<SolarEvent>,
) -> Result<()> {
    info!("Starting NetworkManager event-driven listener...");
    let proxy = match NetworkManagerProxy::new(&conn).await {
        Ok(p) => p,
        Err(err) => {
            error!("NetworkManager proxy could not be created: {}", err);
            return Err(err.into());
        }
    };

    let mut conn_stream = proxy.receive_connectivity_changed().await;
    let mut primary_stream = proxy.receive_primary_connection_type_changed().await;

    if let Ok(status) = get_network_status(&conn).await {
        event_tx.send(SolarEvent::Network(status)).await?;
    }

    loop {
        tokio::select! {
            Some(_change) = conn_stream.next() => {
                debug!("Network connectivity changed");
                if let Ok(status) = get_network_status(&conn).await {
                    event_tx.send(SolarEvent::Network(status)).await?;
                }
            }
            Some(_change) = primary_stream.next() => {
                debug!("Primary connection type changed");
                if let Ok(status) = get_network_status(&conn).await {
                    event_tx.send(SolarEvent::Network(status)).await?;
                }
            }
            else => break,
        }
    }

    Ok(())
}
