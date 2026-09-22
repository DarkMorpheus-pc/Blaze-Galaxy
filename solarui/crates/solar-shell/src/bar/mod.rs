pub mod widgets;

use chrono::Local;
use solar_common::{BatteryState, M3Theme, SolarSystemState};
use widgets::{MediaInfo, PrivacyInfo, SysmonInfo};

#[derive(Debug, Clone)]
pub struct SolarBarModel {
    pub theme: M3Theme,
    // Start Section
    pub launcher_label: String,
    pub workspaces: Vec<WorkspacePill>,
    pub active_window_title: Option<String>,
    pub open_windows: Vec<TaskPill>,

    // Center Section
    pub clock_time: String,
    pub clock_date: String,
    pub media: Option<MediaInfo>,

    // End Section
    pub privacy: PrivacyInfo,
    pub sysmon: SysmonInfo,
    pub network_pill: String,
    pub bluetooth_pill: Option<String>,
    pub audio_pill: String,
    pub brightness_pill: Option<String>,
    pub battery_pill: String,
    pub keyboard_layout: String,
}

#[derive(Debug, Clone)]
pub struct WorkspacePill {
    pub id: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct TaskPill {
    pub id: u64,
    pub name: String,
    pub is_focused: bool,
    pub is_floating: bool,
}

impl SolarBarModel {
    pub fn build(state: &SolarSystemState, theme: &M3Theme) -> Self {
        let now = Local::now();
        let clock_time = now.format("%H:%M").to_string();
        let clock_date = now.format("%a, %d %b").to_string();

        // 1. Workspaces
        let active_ws = state.workspaces.current_workspace;
        let mut sorted_ws = state.workspaces.workspaces.clone();
        sorted_ws.sort();
        if sorted_ws.is_empty() {
            sorted_ws.push(1);
        }

        let workspaces = sorted_ws
            .into_iter()
            .map(|id| WorkspacePill {
                id,
                is_active: id == active_ws,
            })
            .collect();

        // 2. Tasks / Windows
        let mut active_window_title = None;
        let open_windows: Vec<TaskPill> = state
            .windows
            .iter()
            .map(|w| {
                let name = w
                    .title
                    .as_deref()
                    .or(w.app_id.as_deref())
                    .unwrap_or("App")
                    .to_string();

                if w.is_focused {
                    active_window_title = Some(name.clone());
                }

                let short = if name.chars().count() > 18 {
                    format!("{}…", name.chars().take(16).collect::<String>())
                } else {
                    name
                };

                TaskPill {
                    id: w.id,
                    name: short,
                    is_focused: w.is_focused,
                    is_floating: w.is_floating,
                }
            })
            .collect();

        // 3. Status Pills
        let network_pill = if state.network.is_connected {
            if let Some(ref ssid) = state.network.ssid {
                format!("📶 {}", ssid)
            } else if state.network.is_wifi {
                "📶 Wi-Fi".to_string()
            } else {
                "🌐 Ethernet".to_string()
            }
        } else {
            "⚠ Offline".to_string()
        };

        let bluetooth_pill = if state.bluetooth.is_powered {
            Some(format!("󰂯 {}", state.bluetooth.connected_device_count))
        } else {
            None
        };

        let audio_pill = if state.audio.is_muted {
            "󰝟 Muted".to_string()
        } else {
            format!("󰕾 {}%", state.audio.volume)
        };

        let brightness_pill = state
            .brightness
            .as_ref()
            .map(|b| format!("󰃠 {}%", b.percentage));

        let battery_pill = if let Some(ref b) = state.battery {
            let symbol = match b.state {
                BatteryState::Charging | BatteryState::PendingCharge => "⚡",
                BatteryState::FullyCharged => "󰁹",
                _ => "󰁿",
            };
            format!("{} {}%", symbol, b.percentage)
        } else {
            "󰚥 AC".to_string()
        };

        let media = MediaInfo::detect_current();
        let privacy = PrivacyInfo::detect();
        let sysmon = SysmonInfo::read_current();

        Self {
            theme: theme.clone(),
            launcher_label: "◉ Apps".to_string(),
            workspaces,
            active_window_title,
            open_windows,
            clock_time,
            clock_date,
            media,
            privacy,
            sysmon,
            network_pill,
            bluetooth_pill,
            audio_pill,
            brightness_pill,
            battery_pill,
            keyboard_layout: "TR".to_string(),
        }
    }

    pub fn render_ascii_bar(&self) -> String {
        let mut ws_str = String::new();
        for ws in &self.workspaces {
            if ws.is_active {
                ws_str.push_str(&format!(" [ {} ]", ws.id));
            } else {
                ws_str.push_str(&format!("  {}  ", ws.id));
            }
        }

        let media_str = if let Some(ref m) = self.media {
            let icon = if m.is_playing { "▶" } else { "⏸" };
            format!(" 🎵 {} {} ", icon, m.title)
        } else {
            "".to_string()
        };

        let active_win_str = self
            .active_window_title
            .as_ref()
            .map(|t| format!(" | {}", t))
            .unwrap_or_default();

        let sysmon_str = format!("CPU: {}% RAM: {}%", self.sysmon.cpu_percentage, self.sysmon.ram_percentage);

        format!(
            "┌──────────────────────────────────────────────────────────────────────────────────────────────────┐\n\
             │  {} {} {:<22} │  {}  {}  │  {} {}  {}  {}  [⚙ Quick]  │\n\
             └──────────────────────────────────────────────────────────────────────────────────────────────────┘",
            self.launcher_label,
            ws_str.trim(),
            active_win_str,
            self.clock_time,
            self.clock_date,
            media_str,
            sysmon_str,
            self.network_pill,
            self.battery_pill
        )
    }
}
