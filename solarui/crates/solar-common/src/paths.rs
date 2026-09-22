use std::path::PathBuf;

extern "C" {
    fn getuid() -> u32;
}

pub fn current_uid() -> u32 {
    unsafe { getuid() }
}

pub fn get_solar_runtime_dir() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("solar")
    } else {
        let uid = unsafe { getuid() };
        PathBuf::from(format!("/run/user/{}/solar", uid))
    }
}

pub fn get_solar_config_dir() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("solarui")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config/solarui")
    } else {
        PathBuf::from("/tmp/solarui")
    }
}

pub fn get_solar_core_socket_path() -> PathBuf {
    get_solar_runtime_dir().join("core.sock")
}
