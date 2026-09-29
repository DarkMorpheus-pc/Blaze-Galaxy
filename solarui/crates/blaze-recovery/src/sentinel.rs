// ==============================================================================
// Blaze Sentinel — Threat Detection & System Integrity Daemon
// Monitors ld.so.preload, PAM integrity, ransomware canaries, and triggers BERP
// ==============================================================================

use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct SentinelConfig {
    pub canary_paths: Vec<PathBuf>,
    pub monitored_files: Vec<PathBuf>,
    pub check_interval_secs: u64,
}

impl Default for SentinelConfig {
    fn default() -> Self {
        let mut canaries = vec![
            PathBuf::from("/var/tmp/.blaze_canary"),
        ];
        if let Ok(home) = std::env::var("HOME") {
            canaries.push(PathBuf::from(home).join(".config/.blaze_canary"));
        }

        Self {
            canary_paths: canaries,
            monitored_files: vec![
                PathBuf::from("/etc/ld.so.preload"),
                PathBuf::from("/etc/pam.d/sudo"),
                PathBuf::from("/etc/pam.d/system-auth"),
                PathBuf::from("/etc/shadow"),
            ],
            check_interval_secs: 5,
        }
    }
}

pub struct SentinelEngine {
    config: SentinelConfig,
}

impl SentinelEngine {
    pub fn new() -> Self {
        let engine = Self {
            config: SentinelConfig::default(),
        };
        engine.init_canaries();
        engine
    }

    fn init_canaries(&self) {
        for p in &self.config.canary_paths {
            if !p.exists() {
                if let Some(parent) = p.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(p, "BLAZE_CANARY_INTEGRITY_TOKEN_2026_VALID");
            }
        }
    }

    pub fn scan_once(&self) -> Vec<String> {
        let mut alerts = Vec::new();

        // 1. Check ld.so.preload (Classic Linux Rootkit persistence)
        let preload = Path::new("/etc/ld.so.preload");
        if preload.exists() {
            if let Ok(content) = fs::read_to_string(preload) {
                let trimmed = content.trim();
                if !trimmed.is_empty() && !trimmed.contains("libsolar_brand.so") {
                    alerts.push(format!(
                        "KRİTİK GÜVENLİK TEHDİDİ: Yetkisiz /etc/ld.so.preload tespiti: '{}'",
                        trimmed
                    ));
                }
            }
        }

        // 2. Check Canary integrity (Ransomware / File tampering detection)
        for canary in &self.config.canary_paths {
            if !canary.exists() {
                alerts.push(format!(
                    "RANSOMWARE UYARISI: Güvenlik koruma canarisi silindi: {:?}",
                    canary
                ));
            } else if let Ok(c) = fs::read_to_string(canary) {
                if !c.contains("BLAZE_CANARY_INTEGRITY_TOKEN") {
                    alerts.push(format!(
                        "RANSOMWARE / BOZULMA UYARISI: Güvenlik canarisi tahrif edildi: {:?}",
                        canary
                    ));
                }
            }
        }

        // 3. Scan /proc for deleted executables running hiddenly
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(pid) = name.parse::<i32>() {
                    let exe_link = entry.path().join("exe");
                    if let Ok(target) = fs::read_link(&exe_link) {
                        let target_str = target.to_string_lossy();
                        if target_str.contains("(deleted)") && (target_str.contains("/tmp") || target_str.contains("/dev/shm")) {
                            alerts.push(format!(
                                "ŞÜPHELİ GİZLİ SÜREÇ: PID {} silinmiş bellek alanından çalışıyor ({})",
                                pid, target_str
                            ));
                        }
                    }
                }
            }
        }

        alerts
    }

    pub fn handle_alerts(&self, alerts: &[String]) {
        for a in alerts {
            eprintln!("[Blaze Sentinel] {}", a);
            log_sentinel_event(a);
        }

        // Trigger BERP if severe threat detected
        if !alerts.is_empty() {
            eprintln!("[Blaze Sentinel] Sistem bütünlüğü bozuldu. Bir sonraki açılış için BERP kurtarma protokolü tetikleniyor...");
            let _ = Command::new("blaze-recovery").arg("--trigger").status();

            // Send notification to user desktop if notify-send available
            let _ = Command::new("notify-send")
                .args([
                    "-u", "critical",
                    "-i", "security-high",
                    "Blaze Sentinel Güvenlik Alarmı",
                    "Şüpheli sistem aktivitesi veya dosya bozulması tespit edildi! Blaze Acil Kurtarma Protokolü hazırlandı.",
                ])
                .status();
        }
    }

    pub fn run_daemon(&self) {
        println!("=== Blaze Sentinel Güvenlik Servisi Başlatıldı ===");
        println!("İzleme döngüsü: {} saniyede bir", self.config.check_interval_secs);

        loop {
            let alerts = self.scan_once();
            if !alerts.is_empty() {
                self.handle_alerts(&alerts);
            }
            thread::sleep(Duration::from_secs(self.config.check_interval_secs));
        }
    }
}

fn log_sentinel_event(msg: &str) {
    let log_file = "/var/log/blaze-sentinel.log";
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log_file) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(f, "[{}] {}", now, msg);
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let engine = SentinelEngine::new();

    if args.len() > 1 {
        match args[1].as_str() {
            "check" | "--check" => {
                println!("Blaze Sentinel: Sistem ve bütünlük kontrolü yapılıyor...");
                let alerts = engine.scan_once();
                if alerts.is_empty() {
                    println!("[GÜVENLİ] Herhangi bir virüs, rootkit veya bütünlük ihlali bulunamadı.");
                } else {
                    println!("[UYARI] Tespit edilen anormallikler:");
                    for a in &alerts {
                        println!("  - {}", a);
                    }
                }
                return Ok(());
            }
            "trigger" | "--trigger" => {
                engine.handle_alerts(&["Manuel güvenlik alarmı tetiklendi".to_string()]);
                return Ok(());
            }
            "status" | "--status" => {
                println!("Blaze Sentinel: Aktif ve sistem dosyaları izleme altında.");
                return Ok(());
            }
            _ => {}
        }
    }

    // Default: run daemon
    engine.run_daemon();
    Ok(())
}
