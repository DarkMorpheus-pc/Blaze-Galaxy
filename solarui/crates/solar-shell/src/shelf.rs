use chrono::Local;
use solar_common::{BatteryState, SolarSystemState};

pub struct ShelfViewModel {
    pub clock_time: String,
    pub clock_date: String,
    pub battery_label: String,
    pub network_label: String,
    pub workspace_label: String,
    pub open_windows: Vec<WindowPill>,
}

pub struct WindowPill {
    pub id: u64,
    pub name: String,
    pub is_focused: bool,
    pub is_floating: bool,
}

impl ShelfViewModel {
    pub fn from_state(state: &SolarSystemState) -> Self {
        let now = Local::now();
        let clock_time = now.format("%H:%M").to_string();
        let clock_date = now.format("%d %b, %a").to_string();

        let battery_label = if let Some(ref b) = state.battery {
            let icon = match b.state {
                BatteryState::Charging | BatteryState::PendingCharge => "⚡",
                BatteryState::FullyCharged => "✓",
                _ => "",
            };
            format!("{}% {}", b.percentage, icon).trim().to_string()
        } else {
            "AC".to_string()
        };

        let network_label = if state.network.is_connected {
            if let Some(ref ssid) = state.network.ssid {
                format!("📶 {}", ssid)
            } else if state.network.is_wifi {
                "📶 Wi-Fi".to_string()
            } else {
                "🌐 Wired".to_string()
            }
        } else {
            "⚠ Offline".to_string()
        };

        let active_ws = state.workspaces.current_workspace;
        let mut ws_parts = Vec::new();
        let mut sorted_ws = state.workspaces.workspaces.clone();
        sorted_ws.sort();
        if sorted_ws.is_empty() {
            sorted_ws.push(1);
        }

        for id in sorted_ws {
            if id == active_ws {
                ws_parts.push(format!("[{}]", id));
            } else {
                ws_parts.push(format!(" {} ", id));
            }
        }
        let workspace_label = ws_parts.join("");

        let open_windows = state
            .windows
            .iter()
            .map(|w| {
                let name = w
                    .title
                    .as_deref()
                    .or(w.app_id.as_deref())
                    .unwrap_or("App")
                    .to_string();
                let short_name = if name.chars().count() > 22 {
                    format!("{}…", name.chars().take(20).collect::<String>())
                } else {
                    name
                };

                WindowPill {
                    id: w.id,
                    name: short_name,
                    is_focused: w.is_focused,
                    is_floating: w.is_floating,
                }
            })
            .collect();

        Self {
            clock_time,
            clock_date,
            battery_label,
            network_label,
            workspace_label,
            open_windows,
        }
    }
}
