pub mod commands;
pub mod config;
pub mod events;
pub mod hardware;
pub mod ipc;
pub mod minimized;
pub mod paths;
pub mod persistence;
pub mod theme;

pub use commands::*;
pub use config::*;
pub use events::*;
pub use hardware::*;
pub use minimized::*;
pub use paths::*;
pub use theme::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let event = SolarEvent::Network(NetworkStatus {
            is_connected: true,
            is_wifi: true,
            ssid: Some("BlazeNet_5G".to_string()),
            signal_strength: 85,
            ip_address: Some("192.168.1.50".to_string()),
        });

        let json = serde_json::to_string(&event).expect("Serialization failed");
        let deserialized: SolarEvent = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(event, deserialized);
    }
}
