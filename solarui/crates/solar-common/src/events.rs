use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SolarEvent {
    CommandFailed { message: String },
    Network(NetworkStatus),
    Battery(BatteryStatus),
    Audio(AudioStatus),
    Brightness(BrightnessStatus),
    Bluetooth(BluetoothStatus),
    WindowList(Vec<SolarWindowInfo>),
    WindowStateChanged(SolarWindowState),
    WindowRegistrySync(Vec<SolarWindowState>),
    Workspace(SolarWorkspaceInfo),
    Notification(SolarNotification),
    PerformanceProfileChanged(crate::hardware::HardwareTuningProfile),
    FullState(SolarSystemState),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SolarSystemState {
    pub network: NetworkStatus,
    pub battery: Option<BatteryStatus>,
    pub audio: AudioStatus,
    pub brightness: Option<BrightnessStatus>,
    pub bluetooth: BluetoothStatus,
    pub windows: Vec<SolarWindowInfo>,
    pub window_registry: Vec<SolarWindowState>,
    pub workspaces: SolarWorkspaceInfo,
    pub performance_profile: crate::hardware::HardwareTuningProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NetworkStatus {
    pub is_connected: bool,
    pub is_wifi: bool,
    pub ssid: Option<String>,
    pub signal_strength: u8, // 0 - 100
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum BatteryState {
    #[default]
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    PendingCharge,
    PendingDischarge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BatteryStatus {
    pub percentage: u8, // 0 - 100
    pub state: BatteryState,
    pub is_present: bool,
    pub time_to_empty_seconds: Option<i64>,
    pub time_to_full_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AudioStatus {
    pub volume: u8, // 0 - 100
    pub is_muted: bool,
    pub output_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BrightnessStatus {
    pub percentage: u8, // 0 - 100
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BluetoothStatus {
    pub is_powered: bool,
    pub connected_device_count: usize,
    pub connected_devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SolarWindowInfo {
    pub id: u64,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub is_focused: bool,
    pub is_floating: bool,
    pub workspace_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum WindowMode {
    #[default]
    Normal,
    Minimized {
        orig_workspace: u64,
    },
    Floating,
    Maximized,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SolarWindowState {
    pub id: u64,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub workspace_id: u64,
    pub output_id: Option<String>,
    pub mode: WindowMode,
    pub is_focused: bool,
    pub is_urgent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SolarWorkspaceInfo {
    pub current_workspace: u64,
    pub workspaces: Vec<u64>,
    pub is_overview_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolarNotification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: Option<String>,
    pub timeout_ms: i32,
}
