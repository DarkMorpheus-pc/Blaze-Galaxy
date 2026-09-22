use anyhow::Result;
use tracing::info;
use zbus::Connection;

#[zbus::proxy(
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1",
    interface = "org.freedesktop.login1.Manager"
)]
pub trait LogindManager {
    fn power_off(&self, interactive: bool) -> zbus::Result<()>;
    fn reboot(&self, interactive: bool) -> zbus::Result<()>;
    fn suspend(&self, interactive: bool) -> zbus::Result<()>;
    fn lock_session(&self, session_id: &str) -> zbus::Result<()>;
}

pub async fn execute_power_off(conn: &Connection) -> Result<()> {
    info!("Requesting PowerOff via logind");
    let proxy = LogindManagerProxy::new(conn).await?;
    proxy.power_off(true).await?;
    Ok(())
}

pub async fn execute_reboot(conn: &Connection) -> Result<()> {
    info!("Requesting Reboot via logind");
    let proxy = LogindManagerProxy::new(conn).await?;
    proxy.reboot(true).await?;
    Ok(())
}

pub async fn execute_suspend(conn: &Connection) -> Result<()> {
    info!("Requesting Suspend via logind");
    let proxy = LogindManagerProxy::new(conn).await?;
    proxy.suspend(true).await?;
    Ok(())
}

pub async fn execute_lock(conn: &Connection) -> Result<()> {
    info!("Requesting Session Lock via logind");
    let proxy = LogindManagerProxy::new(conn).await?;
    proxy.lock_session("").await?;
    Ok(())
}
