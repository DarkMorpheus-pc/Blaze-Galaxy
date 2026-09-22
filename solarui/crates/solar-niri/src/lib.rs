pub mod client;
pub mod minimized;

pub use client::*;
pub use niri_ipc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_window() {
        let json = r#"{
            "id": 42,
            "title": "Firefox Web Browser",
            "app_id": "firefox",
            "pid": 12345,
            "workspace_id": 1,
            "is_focused": true,
            "is_floating": false,
            "is_urgent": false,
            "layout": {
                "pos_in_scrolling_layout": [1, 1],
                "tile_size": [1920.0, 1080.0],
                "window_size": [1920, 1080],
                "tile_pos_in_workspace_view": [0.0, 0.0],
                "window_offset_in_tile": [0.0, 0.0]
            },
            "focus_timestamp": null
        }"#;

        let win: niri_ipc::Window =
            serde_json::from_str(json).expect("Failed to deserialize mock window");
        let mapped = map_window(win);
        assert_eq!(mapped.id, 42);
        assert_eq!(mapped.app_id.as_deref(), Some("firefox"));
        assert_eq!(mapped.title.as_deref(), Some("Firefox Web Browser"));
        assert!(mapped.is_focused);
        assert!(!mapped.is_floating);
        assert_eq!(mapped.workspace_id, Some(1));
    }
}
