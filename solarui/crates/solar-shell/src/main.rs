pub mod actions;
pub mod apps;
pub mod bar;
pub mod engine;
pub mod gui;
pub mod ipc;
pub mod noctalia_config;
pub mod pixel_clock;
pub mod providers;
pub mod router;
pub mod settings_gui;
pub mod shelf;
pub mod snap_gui;
pub mod supervisor;
pub mod switcher;
pub mod welcome_gui;

use anyhow::Result;
use apps::scan_desktop_applications;
use gui::launch_gui_window;
use ipc::ShellIpcClient;
use shelf::ShelfViewModel;
use solar_common::{M3Theme, SolarConfig, SolarSystemState};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

async fn do_minimize(target_id: Option<u64>) -> Result<()> {
    {
        let client = solar_niri::NiriClient::new()?;
        let windows = client.get_windows().await?;
        let target = if let Some(id) = target_id {
            windows.into_iter().find(|w| w.id == id)
        } else {
            windows.into_iter().find(|w| {
                w.is_focused && {
                    let app = w.app_id.as_deref().unwrap_or("");
                    let title = w.title.as_deref().unwrap_or("");
                    let is_settings = title.contains("Ayarları") || app == "solar-settings";

                    if is_settings {
                        true
                    } else {
                        !app.contains("solar-shell")
                            && !app.contains("solar-snap")
                            && !app.contains("noctalia")
                    }
                }
            })
        };

        if let Some(win) = target {
            solar_niri::minimized::minimize(&client, win.id).await?;
        }
    }
    Ok(())
}

async fn do_restore(target_id: u64) -> Result<()> {
    solar_niri::minimized::restore(&solar_niri::NiriClient::new()?, target_id).await
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "solar_shell=info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let bin_name = args.get(0).map(|s| s.as_str()).unwrap_or("");
    let default_cmd = if bin_name.ends_with("solar-settings") {
        "settings"
    } else {
        "run"
    };
    let command = args.get(1).map(|s| s.trim()).unwrap_or(default_cmd);

    match command {
        "apps" => {
            println!("=== SolarUI Installed Applications ===");
            let apps = scan_desktop_applications();
            for app in apps {
                println!("{:<30} | {:<25} | {}", app.name, app.id, app.exec);
            }
            Ok(())
        }
        "status" => {
            let supervisor = supervisor::SolarShellSupervisor::new();
            let desired = supervisor.load_desired_state()?;
            let actual = supervisor.detect_actual_engine().await;

            let state = Arc::new(RwLock::new(SolarSystemState::default()));
            let (_client, rx) = ShellIpcClient::new(state.clone());
            let state_clone = state.clone();

            tokio::spawn(async move {
                let _ = ShellIpcClient::run_loop(state_clone, rx, None).await;
            });

            // Wait a moment for IPC connection
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

            let s = state.read().await;
            let vm = ShelfViewModel::from_state(&s);

            let is_in_sync = match (desired.desired_engine, &actual) {
                (engine::ShellEngine::Noctalia, supervisor::ActualEngine::Noctalia) => true,
                (engine::ShellEngine::Caelestia, supervisor::ActualEngine::Caelestia) => true,
                (
                    engine::ShellEngine::Hybrid,
                    supervisor::ActualEngine::Noctalia | supervisor::ActualEngine::Hybrid,
                ) => true,
                _ => false,
            };
            let sync_status = if is_in_sync {
                "Reconciled (OK)"
            } else {
                "Drifted (Repair Needed)"
            };

            println!("┌────────────────────────────────────────────────────────────────────────┐");
            println!("│                         SOLAR SHELL SUPERVISOR                         │");
            println!("├────────────────────────────────────────────────────────────────────────┤");
            println!(
                "│ Desired Shell Engine: {:<20} Profile: {:<17} │",
                desired.desired_engine.as_str(),
                desired.desired_profile
            );
            println!(
                "│ Actual Shell Engine:  {:<20} Status:  {:<17} │",
                actual.as_str(),
                sync_status
            );
            println!("├────────────────────────────────────────────────────────────────────────┤");
            println!(
                "│ Workspaces: {:<20} Time: {:<8} Date: {:<12}    │",
                vm.workspace_label, vm.clock_time, vm.clock_date
            );
            println!(
                "│ Network:    {:<20} Battery: {:<10}                     │",
                vm.network_label, vm.battery_label
            );
            println!("├────────────────────────────────────────────────────────────────────────┤");
            println!(
                "│ Active Running Windows ({}):                                            │",
                vm.open_windows.len()
            );
            for w in &vm.open_windows {
                let focused = if w.is_focused { "*" } else { " " };
                let floating = if w.is_floating { "(float)" } else { "(tile)" };
                println!(
                    "│  [{}{}] {:<35} {:<10} [id: {:<4}] │",
                    focused, w.id, w.name, floating, w.id
                );
            }
            println!("└────────────────────────────────────────────────────────────────────────┘");
            Ok(())
        }
        "bar" => {
            let state = Arc::new(RwLock::new(SolarSystemState::default()));
            let (_client, rx) = ShellIpcClient::new(state.clone());
            let state_clone = state.clone();

            tokio::spawn(async move {
                let _ = ShellIpcClient::run_loop(state_clone, rx, None).await;
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

            let s = state.read().await;
            let theme = M3Theme::default();
            let bar_model = bar::SolarBarModel::build(&s, &theme);
            println!("{}", bar_model.render_ascii_bar());
            Ok(())
        }
        "autostart" => {
            info!("SolarUI Autostart: Initializing portal environment & reconciling configured shell engine...");
            ensure_portal_and_graphical_session();
            let supervisor = Arc::new(supervisor::SolarShellSupervisor::new());
            if let Err(e) = supervisor.reconcile().await {
                tracing::error!("Failed to reconcile shell engine on autostart: {}", e);
            }

            // Start ChromeOS Ash-level Watchdog daemon (auto-recovers shell engines within 3 seconds if crashed)
            let _watchdog_handle = supervisor.start_watchdog_loop();

            let cfg = SolarConfig::load();
            let is_live = std::path::Path::new("/run/initramfs/live").exists()
                || std::path::Path::new("/dev/mapper/live-base").exists()
                || std::path::Path::new("/usr/bin/liveinst").exists();
            if cfg.shortcuts_hud.show_at_startup || is_live {
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_millis(2000));
                    let _ = std::process::Command::new("solar-shell")
                        .arg("welcome")
                        .spawn();
                });
            }

            if cfg.taskbar.enabled {
                info!("SolarUI Taskbar is enabled in configuration. Launching taskbar...");
                let state = Arc::new(RwLock::new(SolarSystemState::default()));
                let (client, rx) = ShellIpcClient::new(state.clone());
                launch_gui_window(state, client, rx);
            } else {
                info!("SolarUI Taskbar is disabled in configuration. Keeping Watchdog daemon active...");
                tokio::signal::ctrl_c().await?;
            }
            Ok(())
        }
        "settings" => {
            info!("Launching SolarUI Settings GUI...");
            tokio::task::block_in_place(|| {
                settings_gui::launch_settings_window();
            });
            Ok(())
        }
        "welcome" | "shortcuts" => {
            info!("Launching SolarUI Welcome & Shortcuts HUD...");
            tokio::task::block_in_place(|| {
                welcome_gui::launch_welcome_window();
            });
            Ok(())
        }
        "toggle-taskbar" => {
            let mut cfg = SolarConfig::load();
            cfg.taskbar.enabled = !cfg.taskbar.enabled;
            let _ = cfg.save();
            let _ = noctalia_config::sync_taskbar_state(cfg.taskbar.enabled);
            let _ = std::process::Command::new("pkill")
                .args(["-f", "solar-shell (run|gui|taskbar|window)"])
                .status();
            println!(
                "SolarUI Taskbar state toggled: enabled = {}",
                cfg.taskbar.enabled
            );
            Ok(())
        }
        "taskbar" => {
            let sub = args.get(2).map(|s| s.as_str()).unwrap_or("status");
            match sub {
                "get" | "status" => {
                    let cfg = SolarConfig::load();
                    println!("{}", cfg.taskbar.enabled);
                    Ok(())
                }
                "set" => {
                    if let Some(val_str) = args.get(3) {
                        let val = val_str.parse::<bool>().unwrap_or(false);
                        let mut cfg = SolarConfig::load();
                        cfg.taskbar.enabled = val;
                        let _ = cfg.save();
                        let _ = noctalia_config::sync_taskbar_state(val);
                        let _ = std::process::Command::new("pkill")
                            .args(["-f", "solar-shell (run|gui|taskbar|window)"])
                            .status();
                        println!("SolarUI Taskbar enabled = {}", val);
                    }
                    Ok(())
                }
                _ => {
                    let cfg = SolarConfig::load();
                    println!("{}", cfg.taskbar.enabled);
                    Ok(())
                }
            }
        }
        "run" | "window" | "gui" => {
            info!("Starting SolarUI KDE Taskbar (BlazeOS Desktop)...");
            let state = Arc::new(RwLock::new(SolarSystemState::default()));
            let (client, rx) = ShellIpcClient::new(state.clone());
            launch_gui_window(state, client, rx);
            Ok(())
        }
        "minimize" => {
            let target_id = args.get(2).and_then(|s| s.parse::<u64>().ok());
            do_minimize(target_id).await?;
            Ok(())
        }
        "restore" => {
            if let Some(target_id) = args.get(2).and_then(|s| s.parse::<u64>().ok()) {
                do_restore(target_id).await?;
            }
            Ok(())
        }
        "snap" | "snap-layout" => {
            info!("Launching SolarUI Snap Layouts HUD...");
            snap_gui::launch_snap_window();
            Ok(())
        }
        "clock" | "pixel-clock" => {
            info!("Launching SolarUI Google Pixel Clock Widget...");
            pixel_clock::launch_pixel_clock();
            Ok(())
        }
        "action" => {
            let action_type = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if action_type == "minimize" {
                let target_id = args.get(3).and_then(|s| s.parse::<u64>().ok());
                do_minimize(target_id).await?;
            } else if action_type == "restore" {
                if let Some(target_id) = args.get(3).and_then(|s| s.parse::<u64>().ok()) {
                    do_restore(target_id).await?;
                }
            } else if action_type == "snap" {
                snap_gui::launch_snap_window();
            }
            Ok(())
        }
        "route" => {
            let action_str = args.get(2).map(|s| s.as_str()).unwrap_or("");
            let action: actions::ShellAction =
                action_str.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let cfg = solar_common::SolarConfig::load();
            let router = router::SolarSurfaceRouter::new();
            router.route(action, &cfg.shell).await?;
            Ok(())
        }
        "switch" => {
            let engine_str = args.get(2).map(|s| s.as_str()).unwrap_or("");
            let engine: engine::ShellEngine =
                engine_str.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let switcher = switcher::SolarShellSwitcher::new();
            switcher.switch_engine(engine).await?;
            Ok(())
        }
        "engine" => {
            let sub = args.get(2).map(|s| s.as_str()).unwrap_or("get");
            if sub == "get" {
                let supervisor = supervisor::SolarShellSupervisor::new();
                let desired = supervisor.load_desired_state()?;
                println!("{}", desired.desired_engine.as_str());
            } else if sub == "set" {
                if let Some(eng_str) = args.get(3) {
                    let engine: engine::ShellEngine =
                        eng_str.parse().map_err(|e: String| anyhow::anyhow!(e))?;
                    let switcher = switcher::SolarShellSwitcher::new();
                    switcher.switch_engine(engine).await?;
                }
            }
            Ok(())
        }
        "noctalia-config" => {
            let action = args.get(2).map(|s| s.as_str()).unwrap_or("get");
            match action {
                "get" => {
                    let json = noctalia_config::get_config_json()?;
                    println!("{}", json);
                }
                "set" => {
                    if args.len() < 5 {
                        eprintln!("Usage: solar-shell noctalia-config set <key.path> <value>");
                    } else {
                        let key = &args[3];
                        let val = &args[4];
                        noctalia_config::set_config_value(key, val)?;
                        println!("OK");
                    }
                }
                "reload" => {
                    let _ = std::process::Command::new("noctalia")
                        .args(["msg", "reload"])
                        .status();
                    println!("RELOADED");
                }
                other => {
                    eprintln!(
                        "Unknown noctalia-config action: {}. Valid: get, set, reload",
                        other
                    );
                }
            }
            Ok(())
        }
        "supervisor" | "reconcile" => {
            let supervisor = supervisor::SolarShellSupervisor::new();
            supervisor.reconcile().await?;
            Ok(())
        }
        "sync-noctalia" => {
            sync_noctalia_settings().await?;
            Ok(())
        }
        other => {
            eprintln!("Unknown command: '{}'. Valid: route, switch, sync-noctalia, supervisor, run, autostart, settings, welcome, snap, minimize, restore, toggle-taskbar, apps, status, bar", other);
            Ok(())
        }
    }
}

async fn sync_noctalia_settings() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
    let state_file = std::path::PathBuf::from(home).join(".local/state/noctalia/settings.toml");

    if !state_file.exists() {
        println!("Noctalia settings file not found: {:?}", state_file);
        return Ok(());
    }

    let content = std::fs::read_to_string(&state_file)?;
    let parsed: toml::Value = toml::from_str(&content)?;

    let mut cfg = solar_common::SolarConfig::load();
    let mut changed = false;
    let mut engine_to_switch: Option<engine::ShellEngine> = None;

    if let Some(solarui) = parsed.get("solarui") {
        if let Some(engine_val) = solarui.get("shell_engine").and_then(|v| v.as_str()) {
            if let Ok(new_engine) = engine_val.parse::<engine::ShellEngine>() {
                if new_engine != cfg.shell.engine {
                    info!(
                        "Noctalia GUI'den kabuk motoru değişimi algılandı: {:?} -> {:?}",
                        cfg.shell.engine, new_engine
                    );
                    cfg.shell.engine = new_engine;
                    changed = true;
                    engine_to_switch = Some(new_engine);
                }
            }
        }

        if let Some(launcher_val) = solarui.get("launcher_provider").and_then(|v| v.as_str()) {
            if let Ok(new_launcher) = launcher_val.parse::<engine::SurfaceProvider>() {
                if new_launcher != cfg.shell.launcher_provider {
                    info!(
                        "Noctalia GUI'den başlatıcı sağlayıcı değişimi algılandı: {:?} -> {:?}",
                        cfg.shell.launcher_provider, new_launcher
                    );
                    cfg.shell.launcher_provider = new_launcher;
                    changed = true;
                }
            }
        }

        if let Some(dash_val) = solarui.get("dashboard_provider").and_then(|v| v.as_str()) {
            if let Ok(new_dash) = dash_val.parse::<engine::SurfaceProvider>() {
                if new_dash != cfg.shell.dashboard_provider {
                    info!(
                        "Noctalia GUI'den dashboard sağlayıcı değişimi algılandı: {:?} -> {:?}",
                        cfg.shell.dashboard_provider, new_dash
                    );
                    cfg.shell.dashboard_provider = new_dash;
                    changed = true;
                }
            }
        }

        if let Some(perf_val) = solarui.get("performance_profile").and_then(|v| v.as_str()) {
            if let Ok(new_perf) = perf_val.parse::<solar_common::PerformanceProfile>() {
                if new_perf != cfg.performance.profile {
                    info!(
                        "Noctalia GUI'den performans profili değişimi algılandı: {:?} -> {:?}",
                        cfg.performance.profile, new_perf
                    );
                    cfg.performance.profile = new_perf;
                    changed = true;
                }
            }
        }
    }

    if changed {
        let _ = cfg.save();
        info!("SolarConfig güncellendi ve kaydedildi.");
    }

    if let Some(target_engine) = engine_to_switch {
        let switcher = switcher::SolarShellSwitcher::new();
        let _ = switcher.switch_engine(target_engine).await;
    }

    let _ = std::process::Command::new("notify-send")
        .args([
            "-a",
            "SolarUI",
            "SolarUI Kabuk ve Yüzey Senkronizasyonu",
            &format!(
                "Motor: {:?} | Başlatıcı: {:?} | Dashboard: {:?}",
                cfg.shell.engine, cfg.shell.launcher_provider, cfg.shell.dashboard_provider
            ),
        ])
        .status();

    Ok(())
}

fn ensure_portal_and_graphical_session() {
    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::PathBuf::from(home);

        let portal_dir = home_path.join(".config/xdg-desktop-portal");
        let _ = std::fs::create_dir_all(&portal_dir);
        let portal_conf = portal_dir.join("solarui-portals.conf");
        if !portal_conf.exists() {
            let content = "[preferred]\ndefault=gnome;gtk;\norg.freedesktop.impl.portal.Access=gtk;\norg.freedesktop.impl.portal.Notification=gtk;\norg.freedesktop.impl.portal.Secret=gnome-keyring;\n";
            let _ = std::fs::write(&portal_conf, content);
        }

        let systemd_dir = home_path.join(".config/systemd/user");
        let _ = std::fs::create_dir_all(&systemd_dir);
        let service_file = systemd_dir.join("solarui-session.service");
        if !service_file.exists() {
            let content = "[Unit]\nDescription=SolarUI Session Manager\nBindsTo=graphical-session.target\nBefore=graphical-session.target\n\n[Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart=/usr/bin/true\nNice=-10\nOOMScoreAdjust=-500\n";
            let _ = std::fs::write(&service_file, content);
        }
    }

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let _ = std::process::Command::new("systemctl")
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
    let _ = std::process::Command::new("dbus-update-activation-environment")
        .args([
            "--systemd",
            "WAYLAND_DISPLAY",
            "XDG_CURRENT_DESKTOP",
            "XDG_SESSION_TYPE",
            "XDG_SESSION_DESKTOP",
            "DESKTOP_SESSION",
        ])
        .status();
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "start", "solarui-session.service"])
        .status();
}
