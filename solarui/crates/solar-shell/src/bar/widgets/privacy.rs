#[derive(Debug, Clone, Default)]
pub struct PrivacyInfo {
    pub is_mic_active: bool,
    pub is_camera_active: bool,
}

impl PrivacyInfo {
    pub fn detect() -> Self {
        // 1. Microphone capture detection via ALSA PCM status in /proc/asound
        let is_mic_active = std::fs::read_dir("/proc/asound")
            .map(|entries| {
                entries.flatten().any(|e| {
                    let name = e.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("card") {
                        std::fs::read_dir(e.path())
                            .map(|pcm_entries| {
                                pcm_entries.flatten().any(|pe| {
                                    let pname = pe.file_name();
                                    let pname_str = pname.to_string_lossy();
                                    if pname_str.starts_with("pcm") && pname_str.ends_with('c') {
                                        let status_path = pe.path().join("sub0/status");
                                        std::fs::read_to_string(status_path)
                                            .map(|content| content.contains("RUNNING"))
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    }
                                })
                            })
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        // 2. Camera detection via active file descriptors pointing to /dev/video*
        let is_camera_active = std::fs::read_dir("/proc")
            .map(|proc_entries| {
                proc_entries.flatten().any(|pe| {
                    let pname = pe.file_name();
                    let pname_str = pname.to_string_lossy();
                    if pname_str.chars().all(|c| c.is_ascii_digit()) {
                        let fd_dir = pe.path().join("fd");
                        std::fs::read_dir(fd_dir)
                            .map(|fd_entries| {
                                fd_entries.flatten().any(|fe| {
                                    std::fs::read_link(fe.path())
                                        .map(|link| {
                                            let link_str = link.to_string_lossy();
                                            link_str.starts_with("/dev/video")
                                        })
                                        .unwrap_or(false)
                                })
                            })
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        Self {
            is_mic_active,
            is_camera_active,
        }
    }
}
