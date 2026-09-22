pub mod caelestia;
pub mod noctalia;

use crate::actions::ShellAction;
use anyhow::Result;
use async_trait::async_trait;

pub use caelestia::CaelestiaProvider;
pub use noctalia::NoctaliaProvider;

#[async_trait]
pub trait ProcessExt {
    async fn checked_status(&mut self) -> Result<()>;
    async fn output_timeout(&mut self) -> Result<std::process::Output>;
}

#[async_trait]
impl ProcessExt for tokio::process::Command {
    async fn checked_status(&mut self) -> Result<()> {
        let output = self.output_timeout().await?;
        anyhow::ensure!(
            output.status.success(),
            "Command failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        anyhow::ensure!(
            !output.stdout.starts_with(b"error:"),
            "IPC command failed: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        Ok(())
    }
    async fn output_timeout(&mut self) -> Result<std::process::Output> {
        use anyhow::Context;
        self.kill_on_drop(true);
        tokio::time::timeout(std::time::Duration::from_secs(2), self.output())
            .await
            .context("Shell IPC command timed out")?
            .context("Cannot run shell command")
    }
}

#[async_trait]
pub trait ShellProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn is_installed(&self) -> bool;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn health_check(&self) -> Result<bool>;
    async fn dispatch(&self, action: ShellAction) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn failed_commands_and_ipc_errors_are_not_success() {
        assert!(tokio::process::Command::new("false")
            .checked_status()
            .await
            .is_err());
        assert!(tokio::process::Command::new("printf")
            .arg("error: unavailable\n")
            .checked_status()
            .await
            .is_err());
        assert!(tokio::process::Command::new("true")
            .checked_status()
            .await
            .is_ok());
    }
}
