use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PerformanceTier {
    #[default]
    Balanced,
    Legacy,
    Ultra,
}

impl PerformanceTier {
    pub fn name_tr(&self) -> &'static str {
        match self {
            Self::Legacy => "Hafif / Eski Donanım (Legacy)",
            Self::Balanced => "Dengeli (Balanced)",
            Self::Ultra => "Yüksek Performans & Cam Blur (Ultra)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareInfo {
    pub gpu_vendor: String,
    pub gpu_devices: Vec<String>,
    pub total_ram_mb: u64,
    pub cpu_cores: usize,
    pub on_battery: bool,
    pub recommended_tier: PerformanceTier,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareTuningProfile {
    pub tier: PerformanceTier,
    pub blur_enabled: bool,
    pub shadows_enabled: bool,
    pub animation_duration_ms: u32,
    pub opacity: f64,
    #[serde(default = "default_true")]
    pub watchdog_enabled: bool,
    #[serde(default = "default_true")]
    pub idle_throttling_enabled: bool,
    #[serde(default = "default_true")]
    pub oom_shield_enabled: bool,
}

impl Default for HardwareTuningProfile {
    fn default() -> Self {
        Self::for_tier(PerformanceTier::Balanced)
    }
}

impl HardwareTuningProfile {
    pub fn for_tier(tier: PerformanceTier) -> Self {
        match tier {
            PerformanceTier::Legacy => Self {
                tier,
                blur_enabled: false,
                shadows_enabled: false,
                animation_duration_ms: 0,
                opacity: 0.96,
                watchdog_enabled: true,
                idle_throttling_enabled: true,
                oom_shield_enabled: true,
            },
            PerformanceTier::Balanced => Self {
                tier,
                blur_enabled: true,
                shadows_enabled: true,
                animation_duration_ms: 150,
                opacity: 0.82,
                watchdog_enabled: true,
                idle_throttling_enabled: true,
                oom_shield_enabled: true,
            },
            PerformanceTier::Ultra => Self {
                tier,
                blur_enabled: true,
                shadows_enabled: true,
                animation_duration_ms: 250,
                opacity: 0.72,
                watchdog_enabled: true,
                idle_throttling_enabled: true,
                oom_shield_enabled: true,
            },
        }
    }

    pub fn config_path() -> PathBuf {
        crate::paths::get_solar_config_dir().join("performance.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(p) = serde_json::from_str::<Self>(&content) {
                return p;
            }
        }
        let detected = HardwareDetector::detect();
        let default_profile = Self::for_tier(detected.recommended_tier);
        let _ = default_profile.save();
        default_profile
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, content)
    }
}

pub struct HardwareDetector;

impl HardwareDetector {
    pub fn detect() -> HardwareInfo {
        let (gpu_vendor, gpu_devices, has_discrete_gpu) = Self::detect_gpus();
        let total_ram_mb = Self::detect_ram_mb();
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let on_battery = Self::detect_on_battery();

        // Calculate recommended tier
        let recommended_tier = if on_battery {
            PerformanceTier::Balanced
        } else if (has_discrete_gpu
            || gpu_vendor.contains("NVIDIA")
            || gpu_vendor.contains("AMD"))
            && total_ram_mb >= 12000
        {
            PerformanceTier::Ultra
        } else if total_ram_mb <= 4096
            || gpu_vendor.contains("llvmpipe")
            || gpu_vendor.contains("Software")
        {
            PerformanceTier::Legacy
        } else {
            PerformanceTier::Balanced
        };

        HardwareInfo {
            gpu_vendor,
            gpu_devices,
            total_ram_mb,
            cpu_cores,
            on_battery,
            recommended_tier,
        }
    }

    fn detect_gpus() -> (String, Vec<String>, bool) {
        let mut devices = Vec::new();
        let mut vendors = Vec::new();
        let mut has_dgpu = false;

        let drm_dir = Path::new("/sys/class/drm");
        if let Ok(entries) = fs::read_dir(drm_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("card") && !name.contains('-') {
                    // Primary GPU card directory
                    let vendor_path = entry.path().join("device/vendor");
                    if let Ok(v_str) = fs::read_to_string(vendor_path) {
                        let v = v_str.trim();
                        let v_name = match v {
                            "0x8086" => "Intel",
                            "0x10de" => {
                                has_dgpu = true;
                                "NVIDIA"
                            }
                            "0x1002" => "AMD Radeon",
                            _ => "Generic GPU",
                        };
                        vendors.push(v_name.to_string());
                        devices.push(format!("{}: {}", name, v_name));
                    }
                }
            }
        }

        let vendor_str = if vendors.is_empty() {
            "Bilinmeyen / Tümleşik".to_string()
        } else {
            vendors.join(" + ")
        };

        (vendor_str, devices, has_dgpu)
    }

    fn detect_ram_mb() -> u64 {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return kb / 1024;
                        }
                    }
                }
            }
        }
        8192
    }

    fn detect_on_battery() -> bool {
        let ps_dir = Path::new("/sys/class/power_supply");
        if let Ok(entries) = fs::read_dir(ps_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("BAT") {
                    let status_path = entry.path().join("status");
                    if let Ok(status) = fs::read_to_string(status_path) {
                        let s = status.trim().to_lowercase();
                        if s == "discharging" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detector() {
        let hw = HardwareDetector::detect();
        assert!(hw.cpu_cores >= 1);
        assert!(hw.total_ram_mb > 0);
    }

    #[test]
    fn test_performance_profile() {
        let p_ultra = HardwareTuningProfile::for_tier(PerformanceTier::Ultra);
        assert!(p_ultra.blur_enabled);
        assert!(p_ultra.shadows_enabled);

        let p_legacy = HardwareTuningProfile::for_tier(PerformanceTier::Legacy);
        assert!(!p_legacy.blur_enabled);
        assert!(!p_legacy.shadows_enabled);
        assert_eq!(p_legacy.animation_duration_ms, 0);
    }
}

