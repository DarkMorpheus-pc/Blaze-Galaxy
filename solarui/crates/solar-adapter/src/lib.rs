use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistroType {
    Debian,
    Fedora,
    ArchOrCachy,
    Unknown,
}

impl DistroType {
    pub fn name(&self) -> &'static str {
        match self {
            DistroType::Debian => "Debian (apt)",
            DistroType::Fedora => "Fedora (dnf)",
            DistroType::ArchOrCachy => "Arch/CachyOS (pacman)",
            DistroType::Unknown => "Unknown Linux",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistroInfo {
    pub distro_type: DistroType,
    pub pretty_name: String,
    pub id: String,
    pub version_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageUpdate {
    pub name: String,
    pub current_version: String,
    pub new_version: String,
}

#[async_trait]
pub trait DistroAdapter: Send + Sync {
    fn distro_type(&self) -> DistroType;
    async fn check_updates(&self) -> Result<Vec<PackageUpdate>>;
    async fn search_packages(&self, query: &str) -> Result<Vec<String>>;
}

pub fn detect_host_distro() -> DistroInfo {
    let mut distro_type = DistroType::Unknown;
    let mut pretty_name = "Linux System".to_string();
    let mut id = "linux".to_string();
    let mut version_id = "".to_string();

    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                id = val.trim_matches('"').to_lowercase();
            } else if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                pretty_name = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
                version_id = val.trim_matches('"').to_string();
            }
        }

        distro_type = match id.as_str() {
            "debian" | "ubuntu" | "linuxmint" | "pop" => DistroType::Debian,
            "fedora" | "rhel" | "centos" | "almalinux" | "rocky" => DistroType::Fedora,
            "arch" | "cachyos" | "manjaro" | "endeavouros" => DistroType::ArchOrCachy,
            _ => DistroType::Unknown,
        };
    }

    DistroInfo {
        distro_type,
        pretty_name,
        id,
        version_id,
    }
}

pub fn create_adapter() -> Box<dyn DistroAdapter> {
    let info = detect_host_distro();
    info!("Detected host distro: {} ({})", info.pretty_name, info.distro_type.name());
    match info.distro_type {
        DistroType::Debian => Box::new(DebianAdapter),
        DistroType::Fedora => Box::new(FedoraAdapter),
        DistroType::ArchOrCachy => Box::new(ArchAdapter),
        DistroType::Unknown => Box::new(GenericAdapter),
    }
}

pub struct DebianAdapter;

#[async_trait]
impl DistroAdapter for DebianAdapter {
    fn distro_type(&self) -> DistroType {
        DistroType::Debian
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        debug!("Checking Debian package updates via apt-get -s upgrade");
        let output = tokio::task::spawn_blocking(|| {
            Command::new("apt-get")
                .args(["-s", "upgrade"])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut updates = Vec::new();
        // Inst libssl3 [3.0.11-1] (3.0.13-1~deb12u1 Debian:12.5/stable [amd64])
        for line in stdout.lines() {
            if line.starts_with("Inst ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    updates.push(PackageUpdate {
                        name: parts[1].to_string(),
                        current_version: parts.get(2).unwrap_or(&"").trim_matches(&['[', ']'][..]).to_string(),
                        new_version: parts.get(3).unwrap_or(&"").trim_matches(&['(', ')'][..]).to_string(),
                    });
                }
            }
        }
        Ok(updates)
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<String>> {
        let q = query.to_string();
        let output = tokio::task::spawn_blocking(move || {
            Command::new("apt-cache")
                .args(["search", "--names-only", &q])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = stdout
            .lines()
            .take(20)
            .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
            .collect();
        Ok(results)
    }
}

pub struct FedoraAdapter;

#[async_trait]
impl DistroAdapter for FedoraAdapter {
    fn distro_type(&self) -> DistroType {
        DistroType::Fedora
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        debug!("Checking Fedora package updates via dnf check-update");
        let output = tokio::task::spawn_blocking(|| {
            Command::new("dnf")
                .args(["check-update", "-q"])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut updates = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                updates.push(PackageUpdate {
                    name: parts[0].to_string(),
                    current_version: "installed".to_string(),
                    new_version: parts[1].to_string(),
                });
            }
        }
        Ok(updates)
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<String>> {
        let q = query.to_string();
        let output = tokio::task::spawn_blocking(move || {
            Command::new("dnf")
                .args(["repoquery", "-q", &format!("*{}*", q)])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = stdout.lines().take(20).map(|s| s.to_string()).collect();
        Ok(results)
    }
}

pub struct ArchAdapter;

#[async_trait]
impl DistroAdapter for ArchAdapter {
    fn distro_type(&self) -> DistroType {
        DistroType::ArchOrCachy
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        debug!("Checking Arch/CachyOS updates via checkupdates");
        let output = tokio::task::spawn_blocking(|| {
            Command::new("checkupdates").output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut updates = Vec::new();
        // pkgname 1.0.0-1 -> 1.0.1-1
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[2] == "->" {
                updates.push(PackageUpdate {
                    name: parts[0].to_string(),
                    current_version: parts[1].to_string(),
                    new_version: parts[3].to_string(),
                });
            }
        }
        Ok(updates)
    }

    async fn search_packages(&self, query: &str) -> Result<Vec<String>> {
        let q = query.to_string();
        let output = tokio::task::spawn_blocking(move || {
            Command::new("pacman")
                .args(["-Ssq", &q])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = stdout.lines().take(20).map(|s| s.to_string()).collect();
        Ok(results)
    }
}

pub struct GenericAdapter;

#[async_trait]
impl DistroAdapter for GenericAdapter {
    fn distro_type(&self) -> DistroType {
        DistroType::Unknown
    }

    async fn check_updates(&self) -> Result<Vec<PackageUpdate>> {
        bail!("No package manager adapter configured for unknown Linux distribution")
    }

    async fn search_packages(&self, _query: &str) -> Result<Vec<String>> {
        bail!("No package manager adapter configured for unknown Linux distribution")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_distro() {
        let info = detect_host_distro();
        assert_ne!(info.distro_type, DistroType::Unknown);
        println!("Detected distro: {:?}", info);
    }
}
