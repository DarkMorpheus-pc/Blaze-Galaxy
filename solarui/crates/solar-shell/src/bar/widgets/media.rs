use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub is_playing: bool,
    pub title: String,
    pub artist: String,
}

impl MediaInfo {
    pub fn detect_current() -> Option<Self> {
        // Quick playerctl check if available
        let status_out = Command::new("playerctl").arg("status").output().ok()?;
        if !status_out.status.success() {
            return None;
        }

        let status_str = String::from_utf8_lossy(&status_out.stdout).trim().to_string();
        let is_playing = status_str == "Playing";

        let title_out = Command::new("playerctl")
            .args(["metadata", "title"])
            .output()
            .ok()?;
        let title = String::from_utf8_lossy(&title_out.stdout).trim().to_string();

        let artist_out = Command::new("playerctl")
            .args(["metadata", "artist"])
            .output()
            .ok()?;
        let artist = String::from_utf8_lossy(&artist_out.stdout).trim().to_string();

        if title.is_empty() {
            return None;
        }

        Some(Self {
            is_playing,
            title,
            artist,
        })
    }
}
