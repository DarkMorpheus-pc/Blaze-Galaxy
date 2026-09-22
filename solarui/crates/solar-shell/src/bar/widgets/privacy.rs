#[derive(Debug, Clone, Default)]
pub struct PrivacyInfo {
    pub is_mic_active: bool,
    pub is_camera_active: bool,
}

impl PrivacyInfo {
    pub fn detect() -> Self {
        // In Linux/PipeWire, active recording streams can be checked via pw-dump or /dev/video
        let is_camera_active = std::path::Path::new("/dev/video0").exists()
            && std::fs::read_link("/proc/self/fd/0").is_err(); // Placeholder indicator
        
        Self {
            is_mic_active: false,
            is_camera_active,
        }
    }
}
