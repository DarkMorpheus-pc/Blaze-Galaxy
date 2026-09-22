use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SolarCommand {
    // Audio commands
    SetVolume(u8),
    ToggleMute,

    // Brightness commands
    SetBrightness(u8),

    // Network commands
    ToggleWifi(bool),

    // Bluetooth commands
    ToggleBluetooth(bool),

    // Power commands
    Power(PowerAction),

    // Window Management actions (forwarded to Niri)
    FocusWindow(u64),
    CloseWindow(u64),
    ToggleWindowFloating(u64),
    MinimizeWindow(u64),
    RestoreWindow(u64),
    ToggleMinimizeWindow(u64),
    MaximizeColumn,
    ToggleOverview,
    SwitchWorkspace(u64),

    // App launch
    LaunchApp { exec: String },

    // State request
    RequestStateSync,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PowerAction {
    Lock,
    Suspend,
    Reboot,
    PowerOff,
}
