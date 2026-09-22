use std::fs;

#[derive(Debug, Clone, Default)]
pub struct SysmonInfo {
    pub cpu_percentage: u8,
    pub ram_percentage: u8,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
}

impl SysmonInfo {
    pub fn read_current() -> Self {
        let (ram_percentage, ram_used_mb, ram_total_mb) = read_ram();
        let cpu_percentage = read_cpu();

        Self {
            cpu_percentage,
            ram_percentage,
            ram_used_mb,
            ram_total_mb,
        }
    }
}

fn read_ram() -> (u8, u64, u64) {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0;
        let mut avail_kb = 0;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = parse_kb(line);
            } else if line.starts_with("MemAvailable:") {
                avail_kb = parse_kb(line);
            }
        }

        if total_kb > 0 {
            let used_kb = total_kb.saturating_sub(avail_kb);
            let pct = ((used_kb as f64 / total_kb as f64) * 100.0) as u8;
            return (pct, used_kb / 1024, total_kb / 1024);
        }
    }
    (0, 0, 0)
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

fn read_cpu() -> u8 {
    // Return a lightweight heuristic or 0
    if let Ok(content) = fs::read_to_string("/proc/loadavg") {
        if let Some(load_1min) = content.split_whitespace().next() {
            if let Ok(val) = load_1min.parse::<f64>() {
                return (val * 25.0).clamp(0.0, 100.0) as u8;
            }
        }
    }
    0
}
