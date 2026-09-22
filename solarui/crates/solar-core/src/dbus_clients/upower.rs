use anyhow::Result;
use futures_util::StreamExt;
use solar_common::{BatteryState, BatteryStatus, SolarEvent};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use zbus::Connection;

#[zbus::proxy(
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower/devices/DisplayDevice",
    interface = "org.freedesktop.UPower.Device"
)]
pub trait UPowerDevice {
    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn is_present(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn time_to_empty(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn time_to_full(&self) -> zbus::Result<i64>;
}

pub async fn get_battery_status(conn: &Connection) -> Result<BatteryStatus> {
    let proxy = UPowerDeviceProxy::new(conn).await?;
    let is_present = proxy.is_present().await.unwrap_or(false);
    let percentage = proxy.percentage().await.unwrap_or(0.0) as u8;
    let state_raw = proxy.state().await.unwrap_or(0);
    let time_to_empty = proxy.time_to_empty().await.ok();
    let time_to_full = proxy.time_to_full().await.ok();

    let state = match state_raw {
        1 => BatteryState::Charging,
        2 => BatteryState::Discharging,
        3 => BatteryState::Empty,
        4 => BatteryState::FullyCharged,
        5 => BatteryState::PendingCharge,
        6 => BatteryState::PendingDischarge,
        _ => BatteryState::Unknown,
    };

    Ok(BatteryStatus {
        percentage,
        state,
        is_present,
        time_to_empty_seconds: time_to_empty,
        time_to_full_seconds: time_to_full,
    })
}

pub async fn run_upower_listener(
    conn: Connection,
    event_tx: mpsc::Sender<SolarEvent>,
) -> Result<()> {
    info!("Starting UPower event-driven listener (zero polling)...");
    let proxy = match UPowerDeviceProxy::new(&conn).await {
        Ok(p) => p,
        Err(err) => {
            error!("UPower device proxy could not be created: {}", err);
            return Err(err.into());
        }
    };

    let mut percentage_stream = proxy.receive_percentage_changed().await;
    let mut state_stream = proxy.receive_state_changed().await;

    if let Ok(status) = get_battery_status(&conn).await {
        event_tx.send(SolarEvent::Battery(status)).await?;
    }

    loop {
        tokio::select! {
            Some(change) = percentage_stream.next() => {
                if let Ok(pct) = change.get().await {
                    debug!("UPower battery percentage updated: {}%", pct);
                    if let Ok(status) = get_battery_status(&conn).await {
                        event_tx.send(SolarEvent::Battery(status)).await?;
                    }
                }
            }
            Some(change) = state_stream.next() => {
                if let Ok(state) = change.get().await {
                    debug!("UPower battery state updated: {}", state);
                    if let Ok(status) = get_battery_status(&conn).await {
                        event_tx.send(SolarEvent::Battery(status)).await?;
                    }
                }
            }
            else => break,
        }
    }

    Ok(())
}
