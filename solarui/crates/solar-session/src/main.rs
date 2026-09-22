use anyhow::{Context, Result};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "solar_session=info".into()),
        )
        .init();

    info!("=========================================");
    info!("   Starting BlazeOS SolarUI Session      ");
    info!("=========================================");

    // 1. Setup Wayland & Desktop environment variables
    std::env::set_var("XDG_CURRENT_DESKTOP", "SolarUI:gnome:niri");
    std::env::set_var("XDG_SESSION_TYPE", "wayland");
    std::env::set_var("XDG_SESSION_DESKTOP", "SolarUI");
    std::env::set_var("DESKTOP_SESSION", "SolarUI");
    std::env::set_var("MOZ_ENABLE_WAYLAND", "1");
    std::env::set_var("QT_QPA_PLATFORM", "wayland;xcb");
    std::env::set_var("GDK_BACKEND", "wayland,x11");
    std::env::set_var("SDL_VIDEODRIVER", "wayland");
    std::env::set_var("CLUTTER_BACKEND", "wayland");

    // ChromeOS Ash Performance & Low-Power Idle Optimizations
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "opengl");
    std::env::set_var("QT_WAYLAND_DISABLE_WINDOWDECORATION", "1");

    // Initialize portal configs & systemd graphical session target for PipeWire / OBS screencast
    setup_desktop_portals();

    // Branding belongs in shell assets; do not inject GTK/Pango overrides into
    // every application launched by this session.

    // Configure Noctalia to use SolarUI branded assets (Signature SolarUI translations & icons)
    if let Some(assets_dir) = find_noctalia_assets_dir() {
        info!("Using SolarUI Noctalia assets directory: {:?}", assets_dir);
        std::env::set_var("NOCTALIA_ASSETS_DIR", &assets_dir);
    }

    // 2. Prepare Solar runtime directories
    let runtime_dir = solar_common::get_solar_runtime_dir();
    let _ = std::fs::create_dir_all(&runtime_dir);

    // 3. Locate Solar Niri config
    let config_path = find_solar_niri_config();
    info!("Using Niri configuration: {:?}", config_path);

    // 4. Start SolarCore in background if not already running via systemd
    let solar_core_bin =
        find_binary("solar-core").unwrap_or_else(|| PathBuf::from("/usr/bin/solar-core"));
    if solar_core_bin.exists() {
        info!(
            "Spawning SolarCore platform daemon from {:?}",
            solar_core_bin
        );
        let _ = Command::new(&solar_core_bin).spawn();
    }

    // 4.5. Synchronize display scale if configured
    let solar_cfg = solar_common::SolarConfig::load();
    if (solar_cfg.display.scale - 1.0).abs() > 0.001 {
        let scale = solar_cfg.display.scale;
        info!("Applying persistent display scale: {}", scale);
        std::thread::spawn(move || {
            for _ in 0..40 {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let out = Command::new("niri")
                    .args(["msg", "--json", "outputs"])
                    .output();
                if let Ok(output) = out {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout);
                        let scale_str = format!("{:.2}", scale);
                        for token in text.split('"') {
                            if token == "Virtual-1"
                                || token.starts_with("eDP")
                                || token.starts_with("HDMI")
                                || token.starts_with("DP-")
                            {
                                let _ = Command::new("niri")
                                    .args(["msg", "output", token, "scale", &scale_str])
                                    .status();
                            }
                        }
                        let _ = Command::new("gsettings")
                            .args([
                                "set",
                                "org.gnome.desktop.interface",
                                "text-scaling-factor",
                                &scale_str,
                            ])
                            .status();
                        break;
                    }
                }
            }
        });
    }

    // 5. Exec into Niri compositor
    let niri_bin = find_binary("niri").unwrap_or_else(|| PathBuf::from("/usr/bin/niri"));
    if !niri_bin.exists() {
        error!("Fatal: niri compositor binary not found at {:?}", niri_bin);
        anyhow::bail!("Niri compositor is required to run SolarUI. Please install niri.");
    }

    info!("Executing Niri Wayland session: {:?}", niri_bin);

    let mut cmd = Command::new(&niri_bin);
    cmd.arg("--session");

    if let Some(cfg) = config_path {
        cmd.arg("-c").arg(cfg);
    }

    // exec replaces current process with Niri so signals and exit codes are cleanly passed
    let err = cmd.exec();
    error!("Failed to exec into Niri: {}", err);
    Err(err).context("Failed to exec niri")
}

fn find_binary(name: &str) -> Option<PathBuf> {
    // 1. Check same directory as current exe (for dev/testing)
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let candidate = parent.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. Check PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 3. Common system locations
    for loc in &["/usr/local/bin", "/usr/bin"] {
        let candidate = Path::new(loc).join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn find_solar_niri_config() -> Option<PathBuf> {
    // 1. User config: ~/.config/solarui/niri.kdl
    if let Ok(home) = std::env::var("HOME") {
        let user_cfg = PathBuf::from(home).join(".config/solarui/niri.kdl");
        if user_cfg.exists() {
            return Some(user_cfg);
        }
    }

    // 2. System config: /etc/solarui/niri.kdl or /usr/share/solarui/niri.kdl
    let sys_cfg = PathBuf::from("/etc/solarui/niri.kdl");
    if sys_cfg.exists() {
        return Some(sys_cfg);
    }
    let share_cfg = PathBuf::from("/usr/share/solarui/niri.kdl");
    if share_cfg.exists() {
        return Some(share_cfg);
    }

    // 3. Workspace config in project
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let workspace_cfg = parent.join("../../data/niri.kdl");
            if workspace_cfg.exists() {
                return Some(workspace_cfg);
            }
        }
    }

    None
}

fn find_noctalia_assets_dir() -> Option<PathBuf> {
    // 1. User local directory: ~/.local/share/solarui/noctalia-assets
    if let Ok(home) = std::env::var("HOME") {
        let user_assets = PathBuf::from(home).join(".local/share/solarui/noctalia-assets");
        if user_assets.exists() && user_assets.join("translations/en.json").exists() {
            return Some(user_assets);
        }
    }

    // 2. System directory: /usr/share/solarui/noctalia-assets
    let sys_assets = PathBuf::from("/usr/share/solarui/noctalia-assets");
    if sys_assets.exists() && sys_assets.join("translations/en.json").exists() {
        return Some(sys_assets);
    }

    // 3. Workspace directory: <root>/data/noctalia-assets
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let ws_assets = parent.join("../../data/noctalia-assets");
            if ws_assets.exists() && ws_assets.join("translations/en.json").exists() {
                return Some(ws_assets);
            }
        }
    }

    None
}

fn setup_desktop_portals() {
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);

        // 1. Ensure ~/.config/xdg-desktop-portal/solarui-portals.conf exists
        let portal_dir = home_path.join(".config/xdg-desktop-portal");
        let _ = std::fs::create_dir_all(&portal_dir);
        let portal_conf = portal_dir.join("solarui-portals.conf");
        if !portal_conf.exists() {
            let content = "[preferred]\ndefault=gnome;gtk;\norg.freedesktop.impl.portal.Access=gtk;\norg.freedesktop.impl.portal.Notification=gtk;\norg.freedesktop.impl.portal.Secret=gnome-keyring;\n";
            let _ = std::fs::write(&portal_conf, content);
        }

        // 2. Ensure ~/.config/systemd/user/solarui-session.service exists
        let systemd_dir = home_path.join(".config/systemd/user");
        let _ = std::fs::create_dir_all(&systemd_dir);
        let service_file = systemd_dir.join("solarui-session.service");
        if !service_file.exists() {
            let content = "[Unit]\nDescription=SolarUI Session Manager\nBindsTo=graphical-session.target\nBefore=graphical-session.target\n\n[Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart=/usr/bin/true\nNice=-10\nOOMScoreAdjust=-500\n";
            let _ = std::fs::write(&service_file, content);
        }
    }

    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let _ = Command::new("systemctl")
        .args([
            "--user",
            "import-environment",
            "WAYLAND_DISPLAY",
            "XDG_CURRENT_DESKTOP",
            "XDG_SESSION_TYPE",
            "XDG_SESSION_DESKTOP",
            "DESKTOP_SESSION",
        ])
        .status();
    let _ = Command::new("dbus-update-activation-environment")
        .args([
            "--systemd",
            "WAYLAND_DISPLAY",
            "XDG_CURRENT_DESKTOP",
            "XDG_SESSION_TYPE",
            "XDG_SESSION_DESKTOP",
            "DESKTOP_SESSION",
        ])
        .status();
    let _ = Command::new("systemctl")
        .args(["--user", "start", "solarui-session.service"])
        .status();
    let _ = Command::new("systemctl")
        .args(["--user", "restart", "xdg-desktop-portal"])
        .status();
}
