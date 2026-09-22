use gdk_pixbuf::PixbufLoader;
use gtk4::gdk;
use gtk4::glib;
use gtk4::pango;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, CssProvider, Image, Label, Orientation, Separator, Window};
use solar_common::{M3Theme, SolarConfig, SolarSystemState};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ipc::ShellIpcClient;

// Real Google Material 3 and vector SVG assets - Zero Emojis!
const ICON_KICKOFF: &str = include_str!("../../../data/icons/kickoff.svg");
const ICON_TERMINAL: &str = include_str!("../../../data/icons/terminal.svg");
const ICON_FOLDER: &str = include_str!("../../../data/icons/folder.svg");
const ICON_BROWSER: &str = include_str!("../../../data/icons/browser.svg");
const ICON_DESKTOP: &str = include_str!("../../../data/icons/desktop.svg");
const ICON_WIFI: &str = include_str!("../../../data/icons/wifi.svg");
const ICON_VOLUME: &str = include_str!("../../../data/icons/volume_up.svg");
const ICON_BATTERY: &str = include_str!("../../../data/icons/battery_charging.svg");
const ICON_POWER: &str = include_str!("../../../data/icons/power_settings_new.svg");
const ICON_MINIMIZE: &str = include_str!("../../../data/icons/minimize.svg");
const ICON_CLOSE: &str = include_str!("../../../data/icons/close.svg");
const ICON_SNAP: &str = include_str!("../../../data/icons/snap.svg");

// Cleaned transparent SolarUI Bird Logo
const LOGO_PNG: &[u8] = include_bytes!("../../../data/icons/solar-logo-48.png");

#[derive(Clone, Debug)]
pub struct ShellDesktopSnapshot {
    pub windows: Vec<solar_common::SolarWindowInfo>,
    pub active_workspace_id: u64,
}

fn create_svg_image(svg_data: &str, size: i32) -> Image {
    let loader = match PixbufLoader::with_type("svg") {
        Ok(l) => l,
        Err(_) => return Image::new(),
    };
    loader.set_size(size, size);
    if loader.write(svg_data.as_bytes()).is_ok() && loader.close().is_ok() {
        if let Some(pixbuf) = loader.pixbuf() {
            let texture = gdk4::Texture::for_pixbuf(&pixbuf);
            let img = Image::from_paintable(Some(&texture));
            img.set_pixel_size(size);
            return img;
        }
    }
    Image::new()
}

fn create_png_image(png_data: &[u8], size: i32) -> Image {
    let loader = match PixbufLoader::with_type("png") {
        Ok(l) => l,
        Err(_) => return Image::new(),
    };
    loader.set_size(size, size);
    if loader.write(png_data).is_ok() && loader.close().is_ok() {
        if let Some(pixbuf) = loader.pixbuf() {
            let texture = gdk4::Texture::for_pixbuf(&pixbuf);
            let img = Image::from_paintable(Some(&texture));
            img.set_pixel_size(size);
            return img;
        }
    }
    Image::new()
}

pub fn launch_gui_window(
    state: Arc<RwLock<SolarSystemState>>,
    client: ShellIpcClient,
    cmd_rx: tokio::sync::mpsc::Receiver<solar_common::SolarCommand>,
) {
    glib::set_prgname(Some("solar-shell"));
    glib::set_application_name("solar-shell");

    if let Err(err) = gtk4::init() {
        eprintln!("Failed to initialize GTK4: {}", err);
        return;
    }

    let main_loop = glib::MainLoop::new(None, false);
    build_ui(state, Arc::new(client), main_loop.clone(), cmd_rx);
    main_loop.run();
}

fn generate_taskbar_css(
    cfg: &SolarConfig,
    profile: &solar_common::HardwareTuningProfile,
) -> String {
    let theme = M3Theme::default();
    let is_light = cfg.taskbar.theme_mode == "light";
    let bg_color = if is_light {
        format!("alpha(#f5f7fa, {:.2})", cfg.taskbar.opacity)
    } else {
        format!("alpha(#16191c, {:.2})", cfg.taskbar.opacity)
    };
    let border_color = if is_light {
        "alpha(#b0bac2, 0.65)"
    } else {
        "alpha(#525c62, 0.45)"
    };
    let task_bg = if is_light {
        "alpha(#e2e6eb, 0.75)"
    } else {
        "alpha(#23272a, 0.65)"
    };
    let task_color = if is_light { "#1e2429" } else { "#e1e3e5" };
    let task_hover = if is_light {
        "alpha(#d2d9df, 0.95)"
    } else {
        "alpha(#2f3539, 0.9)"
    };
    let active_bg = if is_light {
        "alpha(#c3e7ff, 0.85)"
    } else {
        "alpha(#004d67, 0.55)"
    };
    let active_color = if is_light { "#00364c" } else { "#ffffff" };
    let active_bottom = if is_light { "#0095c7" } else { "#38c8ff" };
    let clock_bg = if is_light {
        "alpha(#e2e6eb, 0.85)"
    } else {
        "alpha(#23272a, 0.7)"
    };
    let clock_color = if is_light { "#192734" } else { "#c4e7ff" };
    let tray_color = if is_light { "#3b454d" } else { "#b0b8be" };
    let pinned_color = if is_light { "#1a2a36" } else { "#c4e7ff" };
    let pinned_hover = if is_light {
        "alpha(#000000, 0.08)"
    } else {
        "alpha(#ffffff, 0.12)"
    };

    let box_shadow = if profile.shadows_enabled {
        "box-shadow: 0 4px 20px rgba(0, 0, 0, 0.55);"
    } else {
        "box-shadow: none;"
    };

    let anim_duration = profile.animation_duration_ms;
    let transition_prop = if anim_duration > 0 {
        format!("transition: all {}ms ease;", anim_duration)
    } else {
        "transition: none;".to_string()
    };

    format!(
        r#"
        {}

        window {{
            background-color: transparent;
        }}

        .kde-taskbar-container {{
            background-color: {};
            border: 1px solid {};
            border-radius: 14px;
            padding: 3px 8px;
            margin: 2px 14px 4px 14px;
            {}
        }}

        .kde-kickoff-btn {{
            background: linear-gradient(135deg, #00688b, #0095c7);
            color: #ffffff;
            border-radius: 6px;
            padding: 4px 8px;
            margin-right: 6px;
            border: none;
            {}
        }}

        .kde-kickoff-btn:hover {{
            background: linear-gradient(135deg, #007ba4, #14a8dc);
        }}

        .kde-pinned-btn {{
            background-color: transparent;
            border-radius: 6px;
            padding: 6px 8px;
            margin: 0 2px;
            border: none;
            color: {};
            {}
        }}

        .kde-pinned-btn:hover {{
            background-color: {};
        }}

        .kde-task-btn {{
            background-color: {};
            color: {};
            border-radius: 6px;
            border: 1px solid transparent;
            border-bottom: 2px solid transparent;
            padding: 4px 12px;
            margin: 0 3px;
            font-size: 13px;
            font-weight: 500;
            {}
        }}

        .kde-task-btn:hover {{
            background-color: {};
            border-color: alpha(#616c72, 0.4);
        }}

        .kde-task-active {{
            background-color: {};
            border: 1px solid alpha(#0095c7, 0.45);
            border-bottom: 2px solid {};
            color: {};
            font-weight: 600;
        }}

        .kde-task-minimized {{
            opacity: 0.65;
            font-style: italic;
            border-bottom: 2px dashed #0095c7;
        }}

        .kde-task-action-btn {{
            background-color: transparent;
            border: none;
            border-radius: 4px;
            padding: 2px 4px;
            margin-left: 2px;
            color: alpha(#ffffff, 0.7);
            {}
        }}

        .kde-task-action-btn:hover {{
            background-color: alpha(#ffffff, 0.2);
            color: #ffffff;
        }}

        .kde-task-close-btn {{
            background-color: transparent;
            border: none;
            border-radius: 4px;
            padding: 2px 4px;
            margin-left: 2px;
            color: alpha(#ffffff, 0.7);
            {}
        }}

        .kde-task-close-btn:hover {{
            background-color: rgba(220, 53, 69, 0.85);
            color: #ffffff;
        }}

        .kde-clock-pill {{
            background-color: {};
            color: {};
            border-radius: 6px;
            padding: 4px 12px;
            font-weight: 600;
            font-size: 13px;
            margin: 0 4px;
        }}

        .kde-tray-item {{
            background-color: transparent;
            border: none;
            border-radius: 6px;
            padding: 4px 6px;
            margin: 0 2px;
            color: {};
        }}

        .kde-tray-item:hover {{
            background-color: alpha(#ffffff, 0.1);
        }}

        .kde-show-desktop {{
            background-color: transparent;
            border: none;
            border-left: 1px solid alpha(#525c62, 0.4);
            border-radius: 0;
            padding: 4px 8px;
            margin-left: 6px;
            color: #8a9297;
        }}

        .kde-show-desktop:hover {{
            background-color: alpha(#38c8ff, 0.3);
            color: #ffffff;
        }}

        .kde-separator {{
            margin: 4px 6px;
            background-color: alpha(#525c62, 0.4);
        }}
        "#,
        theme.to_gtk_css(),
        bg_color,
        border_color,
        box_shadow,
        transition_prop,
        pinned_color,
        transition_prop,
        pinned_hover,
        task_bg,
        task_color,
        transition_prop,
        task_hover,
        active_bg,
        active_bottom,
        active_color,
        transition_prop,
        transition_prop,
        clock_bg,
        clock_color,
        tray_color
    )
}

fn build_ui(
    state: Arc<RwLock<SolarSystemState>>,
    _client: Arc<ShellIpcClient>,
    main_loop: glib::MainLoop,
    cmd_rx: tokio::sync::mpsc::Receiver<solar_common::SolarCommand>,
) {
    let cfg = SolarConfig::load();
    let perf_profile = solar_common::HardwareTuningProfile::load();

    // 1. Load KDE Plasma & CachyOS Glassmorphism CSS styling with PerformanceProfile
    let provider = CssProvider::new();
    let css = generate_taskbar_css(&cfg, &perf_profile);
    provider.load_from_string(&css);

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // 2. Create Taskbar Window
    let window = Window::builder()
        .title("SolarUI Taskbar")
        .default_width(1920)
        .default_height(cfg.taskbar.height)
        .resizable(false)
        .decorated(false)
        .build();

    let loop_quit = main_loop.clone();
    window.connect_close_request(move |_| {
        loop_quit.quit();
        glib::Propagation::Proceed
    });

    let root_box = GtkBox::new(Orientation::Horizontal, 0);
    root_box.add_css_class("kde-taskbar-container");

    // ── 1. KICKOFF START MENU BUTTON (Left) ────────────────────────────────────
    let start_btn = Button::new();
    if cfg.taskbar.start_menu_icon == "kickoff" {
        start_btn.set_child(Some(&create_svg_image(ICON_KICKOFF, cfg.taskbar.icon_size)));
    } else {
        start_btn.set_child(Some(&create_png_image(LOGO_PNG, cfg.taskbar.icon_size + 4)));
    }
    start_btn.add_css_class("kde-kickoff-btn");
    start_btn.set_tooltip_text(Some("SolarUI Uygulama Menüsü"));
    start_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("noctalia")
                .args(["msg", "panel-toggle", "launcher"])
                .status();
        });
    });
    root_box.append(&start_btn);

    // ── 2. PINNED QUICK LAUNCHERS ──────────────────────────────────────────────
    let pinned_box = GtkBox::new(Orientation::Horizontal, 2);

    // Terminal
    let term_btn = Button::new();
    term_btn.set_child(Some(&create_svg_image(ICON_TERMINAL, 18)));
    term_btn.add_css_class("kde-pinned-btn");
    term_btn.set_tooltip_text(Some("Uçbirim (Terminal)"));
    term_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("alacritty").spawn();
        });
    });
    pinned_box.append(&term_btn);

    // File Manager
    let files_btn = Button::new();
    files_btn.set_child(Some(&create_svg_image(ICON_FOLDER, 18)));
    files_btn.add_css_class("kde-pinned-btn");
    files_btn.set_tooltip_text(Some("Dosyalar (Nautilus)"));
    files_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("nautilus").spawn();
        });
    });
    pinned_box.append(&files_btn);

    // Browser
    let web_btn = Button::new();
    web_btn.set_child(Some(&create_svg_image(ICON_BROWSER, 18)));
    web_btn.add_css_class("kde-pinned-btn");
    web_btn.set_tooltip_text(Some("Web Tarayıcısı"));
    web_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("google-chrome-stable")
                .spawn()
                .or_else(|_| std::process::Command::new("google-chrome").spawn())
                .or_else(|_| std::process::Command::new("firefox").spawn());
        });
    });
    pinned_box.append(&web_btn);

    if cfg.taskbar.show_pinned {
        root_box.append(&pinned_box);
        let sep = Separator::new(Orientation::Vertical);
        sep.add_css_class("kde-separator");
        root_box.append(&sep);
    }

    // ── 3. DYNAMIC WINDOW TASK MANAGER (Center/Left) ───────────────────────────
    let tasks_box = GtkBox::new(Orientation::Horizontal, 4);
    tasks_box.set_hexpand(true);
    tasks_box.set_halign(gtk4::Align::Start);
    if cfg.taskbar.show_tasks {
        root_box.append(&tasks_box);
    }

    // ── 4. TRAY & STATUS AREA (Right) ──────────────────────────────────────────
    let right_box = GtkBox::new(Orientation::Horizontal, 2);
    right_box.set_halign(gtk4::Align::End);

    let (maybe_net_btn, maybe_vol_btn, maybe_batt_btn) = if cfg.taskbar.show_tray {
        // Wifi Button (Opens Noctalia Network panel)
        let net_btn = Button::new();
        net_btn.set_child(Some(&create_svg_image(ICON_WIFI, 16)));
        net_btn.add_css_class("kde-tray-item");
        net_btn.set_tooltip_text(Some("Ağ ve Bağlantılar"));
        net_btn.connect_clicked(|_| {
            std::thread::spawn(|| {
                let _ = std::process::Command::new("noctalia")
                    .args(["msg", "panel-toggle", "control-center", "network"])
                    .status();
            });
        });
        right_box.append(&net_btn);

        // Volume Button (Opens Noctalia Audio mixer & per-app sliders)
        let vol_btn = Button::new();
        vol_btn.set_child(Some(&create_svg_image(ICON_VOLUME, 16)));
        vol_btn.add_css_class("kde-tray-item");
        vol_btn.set_tooltip_text(Some("Ses Ayarları ve Mikser"));
        vol_btn.connect_clicked(|_| {
            std::thread::spawn(|| {
                let _ = std::process::Command::new("noctalia")
                    .args(["msg", "panel-toggle", "control-center", "audio"])
                    .status();
            });
        });
        right_box.append(&vol_btn);

        // Battery Button (Opens Noctalia Power profile panel)
        let batt_btn = Button::new();
        batt_btn.set_child(Some(&create_svg_image(ICON_BATTERY, 16)));
        batt_btn.add_css_class("kde-tray-item");
        batt_btn.set_tooltip_text(Some("Pil ve Performans"));
        batt_btn.connect_clicked(|_| {
            std::thread::spawn(|| {
                let _ = std::process::Command::new("noctalia")
                    .args(["msg", "panel-toggle", "control-center"])
                    .status();
            });
        });
        right_box.append(&batt_btn);

        (Some(net_btn), Some(vol_btn), Some(batt_btn))
    } else {
        (None, None, None)
    };

    let maybe_clock_label = if cfg.taskbar.show_clock {
        // Digital Clock Button (Opens Noctalia Calendar)
        let clock_btn = Button::new();
        let clock_label = Label::new(Some("14:00"));
        clock_btn.set_child(Some(&clock_label));
        clock_btn.add_css_class("kde-clock-pill");
        clock_btn.set_tooltip_text(Some("Takvim ve Bildirimler"));
        clock_btn.connect_clicked(|_| {
            std::thread::spawn(|| {
                let _ = std::process::Command::new("noctalia")
                    .args(["msg", "panel-toggle", "control-center", "calendar"])
                    .status();
            });
        });
        right_box.append(&clock_btn);
        Some(clock_label)
    } else {
        None
    };

    // Power / Session Button (Opens Noctalia Logout / Power menu)
    let power_btn = Button::new();
    power_btn.set_child(Some(&create_svg_image(ICON_POWER, 16)));
    power_btn.add_css_class("kde-tray-item");
    power_btn.set_tooltip_text(Some("Oturumu Kapat / Bilgisayarı Kapat"));
    power_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("noctalia")
                .args(["msg", "panel-toggle", "session"])
                .status();
        });
    });
    // Windows 11-Style Snap Layouts HUD Button (Mod+Z)
    let snap_btn = Button::new();
    snap_btn.set_child(Some(&create_svg_image(ICON_SNAP, 16)));
    snap_btn.add_css_class("kde-tray-item");
    snap_btn.set_tooltip_text(Some("Pencere Yerleşim Düzenleri (Snap Layouts - Mod+Z)"));
    snap_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("solar-shell")
                .arg("snap")
                .spawn();
        });
    });
    right_box.append(&snap_btn);

    // Show Desktop Button (Classic KDE / Windows)
    let show_desktop_btn = Button::new();
    show_desktop_btn.set_child(Some(&create_svg_image(ICON_DESKTOP, 16)));
    show_desktop_btn.add_css_class("kde-show-desktop");
    show_desktop_btn.set_tooltip_text(Some("Masaüstünü Göster"));
    show_desktop_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = std::process::Command::new("niri")
                .args(["msg", "action", "toggle-overview"])
                .status();
        });
    });
    right_box.append(&show_desktop_btn);

    root_box.append(&right_box);
    window.set_child(Some(&root_box));

    // ── 5. ZERO-POLLING EVENT-DRIVEN TASKBAR & TRAY ───────────────────────────
    struct TaskbarLiveState {
        windows: Vec<solar_common::SolarWindowState>,
        active_workspace_id: u64,
    }

    let live_state = std::rc::Rc::new(std::cell::RefCell::new(TaskbarLiveState {
        windows: Vec::new(),
        active_workspace_id: 1,
    }));

    // Reactive listener for SolarEventBus using async_channel and spawn_local (Zero Polling)
    let (event_tx, event_rx) = async_channel::bounded::<solar_common::SolarEvent>(128);

    // SolarCore's FullState snapshot owns initial hydration and reconnection.
    // A parallel Niri query could arrive late and overwrite a newer core state.

    let container = tasks_box.clone();
    let state_ref = live_state.clone();
    let net_ref = maybe_net_btn;
    let vol_ref = maybe_vol_btn;
    let batt_ref = maybe_batt_btn;
    let provider_ref = provider.clone();
    let cfg_ref = cfg.clone();

    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                solar_common::SolarEvent::WindowRegistrySync(windows) => {
                    if state_ref.borrow().windows == windows {
                        continue;
                    }
                    let active_ws = state_ref.borrow().active_workspace_id;
                    state_ref.borrow_mut().windows = windows.clone();
                    update_tasks_ui(&container, &windows, active_ws);
                }
                solar_common::SolarEvent::WindowStateChanged(win_state) => {
                    let active_ws = state_ref.borrow().active_workspace_id;
                    let mut cur = state_ref.borrow().windows.clone();
                    if let Some(idx) = cur.iter().position(|w| w.id == win_state.id) {
                        cur[idx] = win_state;
                    } else {
                        cur.push(win_state);
                    }
                    state_ref.borrow_mut().windows = cur.clone();
                    update_tasks_ui(&container, &cur, active_ws);
                }
                solar_common::SolarEvent::Workspace(ws_info) => {
                    let active_ws = ws_info.current_workspace;
                    state_ref.borrow_mut().active_workspace_id = active_ws;
                    let windows = state_ref.borrow().windows.clone();
                    update_tasks_ui(&container, &windows, active_ws);
                }
                solar_common::SolarEvent::FullState(full_state) => {
                    let active_ws = full_state.workspaces.current_workspace;
                    state_ref.borrow_mut().active_workspace_id = active_ws;
                    state_ref.borrow_mut().windows = full_state.window_registry.clone();
                    update_tasks_ui(&container, &full_state.window_registry, active_ws);

                    if let Some(ref btn) = batt_ref {
                        if let Some(batt) = full_state.battery {
                            btn.set_tooltip_text(Some(&format!(
                                "Pil: %{} ({:?})",
                                batt.percentage, batt.state
                            )));
                        }
                    }
                    if let Some(ref btn) = net_ref {
                        btn.set_tooltip_text(Some(&format!(
                            "Ağ: {}",
                            if full_state.network.is_connected {
                                "Bağlı"
                            } else {
                                "Bağlantı Yok"
                            }
                        )));
                    }
                    if let Some(ref btn) = vol_ref {
                        btn.set_tooltip_text(Some(&format!(
                            "Ses: %{} {}",
                            full_state.audio.volume,
                            if full_state.audio.is_muted {
                                "(Sessiz)"
                            } else {
                                ""
                            }
                        )));
                    }
                }
                solar_common::SolarEvent::Battery(b) => {
                    if let Some(ref btn) = batt_ref {
                        btn.set_tooltip_text(Some(&format!(
                            "Pil: %{} ({:?})",
                            b.percentage, b.state
                        )));
                    }
                }
                solar_common::SolarEvent::Network(n) => {
                    if let Some(ref btn) = net_ref {
                        btn.set_tooltip_text(Some(&format!(
                            "Ağ: {}",
                            if n.is_connected {
                                "Bağlı"
                            } else {
                                "Bağlantı Yok"
                            }
                        )));
                    }
                }
                solar_common::SolarEvent::Audio(a) => {
                    if let Some(ref btn) = vol_ref {
                        btn.set_tooltip_text(Some(&format!(
                            "Ses: %{} {}",
                            a.volume,
                            if a.is_muted { "(Sessiz)" } else { "" }
                        )));
                    }
                }
                solar_common::SolarEvent::PerformanceProfileChanged(p) => {
                    let new_css = generate_taskbar_css(&cfg_ref, &p);
                    provider_ref.load_from_string(&new_css);
                }
                _ => {}
            }
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        let _ = ShellIpcClient::run_loop(state_clone, cmd_rx, Some(event_tx)).await;
    });

    // Clock updater (every 1 second, near-zero timer wakeups)
    if let Some(clock_ref) = maybe_clock_label {
        let now = chrono::Local::now();
        clock_ref.set_text(&now.format("%H:%M  |  %d %b").to_string());
        glib::timeout_add_seconds_local(1, move || {
            let now = chrono::Local::now();
            clock_ref.set_text(&now.format("%H:%M  |  %d %b").to_string());
            glib::ControlFlow::Continue
        });
    }

    window.present();
}

fn update_tasks_ui(
    tasks_container: &GtkBox,
    windows: &[solar_common::SolarWindowState],
    active_ws: u64,
) {
    let mut visible_windows: Vec<solar_common::SolarWindowState> = windows.to_vec();

    // Filter out self (solar-shell) and desktop panels
    visible_windows.retain(|win| {
        if let Some(ref app_id) = win.app_id {
            let id_lower = app_id.to_lowercase();
            if id_lower.contains("solar-shell")
                || id_lower.contains("solarui")
                || id_lower.contains("noctalia")
            {
                return false;
            }
        }
        if let Some(ref title) = win.title {
            let title_lower = title.to_lowercase();
            if title_lower.contains("solarui taskbar") || title_lower.contains("solarshell") {
                return false;
            }
        }

        // Workspace filtering:
        // If minimized, check if its original workspace matches active_ws
        match win.mode {
            solar_common::WindowMode::Minimized { orig_workspace } => orig_workspace == active_ws,
            _ => win.workspace_id == active_ws,
        }
    });

    // Clear old buttons
    while let Some(child) = tasks_container.first_child() {
        tasks_container.remove(&child);
    }

    // Add task buttons for windows on current workspace
    for win in visible_windows {
        let (is_minimized, _orig_ws) = match win.mode {
            solar_common::WindowMode::Minimized { orig_workspace } => (true, orig_workspace),
            _ => (false, win.workspace_id),
        };

        let task_btn = Button::new();
        task_btn.add_css_class("kde-task-btn");

        if win.is_focused && !is_minimized {
            task_btn.add_css_class("kde-task-active");
        }
        if is_minimized {
            task_btn.add_css_class("kde-task-minimized");
        }

        let btn_content = GtkBox::new(Orientation::Horizontal, 6);

        // Real application icon or fallback folder/window icon
        let icon_img = if let Some(app_id) = &win.app_id {
            let img = Image::from_icon_name(app_id);
            img.set_pixel_size(16);
            img
        } else {
            create_svg_image(ICON_FOLDER, 16)
        };
        btn_content.append(&icon_img);

        // Window title with truncation
        let title_text = win
            .title
            .clone()
            .unwrap_or_else(|| win.app_id.clone().unwrap_or_else(|| "Window".into()));
        let title_label = Label::new(Some(&title_text));
        title_label.set_ellipsize(pango::EllipsizeMode::End);
        title_label.set_max_width_chars(20);
        btn_content.append(&title_label);

        // Explicit Arka Plana At (-) button
        let min_btn = Button::new();
        min_btn.set_child(Some(&create_svg_image(ICON_MINIMIZE, 12)));
        min_btn.add_css_class("kde-task-action-btn");
        min_btn.set_tooltip_text(Some(if is_minimized {
            "Ön Plana Getir"
        } else {
            "Arka Plana At (Simge Durumuna Küçült)"
        }));
        let min_win_id = win.id;
        let min_is_min = is_minimized;
        min_btn.connect_clicked(move |_| {
            let id = min_win_id;
            let currently_min = min_is_min;
            std::thread::spawn(move || {
                if currently_min {
                    let _ = std::process::Command::new("solar-shell")
                        .args(["restore", &id.to_string()])
                        .status();
                } else {
                    let _ = std::process::Command::new("solar-shell")
                        .args(["minimize", &id.to_string()])
                        .status();
                }
            });
        });
        btn_content.append(&min_btn);

        // Explicit Kapat (✕) button
        let close_btn = Button::new();
        close_btn.set_child(Some(&create_svg_image(ICON_CLOSE, 12)));
        close_btn.add_css_class("kde-task-close-btn");
        close_btn.set_tooltip_text(Some("Pencereyi Kapat"));
        let close_win_id = win.id;
        close_btn.connect_clicked(move |_| {
            let id = close_win_id;
            std::thread::spawn(move || {
                let _ = std::process::Command::new("niri")
                    .args(["msg", "action", "close-window", "--id", &id.to_string()])
                    .status();
            });
        });
        btn_content.append(&close_btn);

        task_btn.set_child(Some(&btn_content));
        task_btn.set_tooltip_text(Some(&title_text));

        // Clicking the main task pill body:
        let click_win_id = win.id;
        let click_is_focused = win.is_focused;
        let click_is_min = is_minimized;
        task_btn.connect_clicked(move |_| {
            let id = click_win_id;
            let focused = click_is_focused;
            let min = click_is_min;
            std::thread::spawn(move || {
                if min {
                    let _ = std::process::Command::new("solar-shell")
                        .args(["restore", &id.to_string()])
                        .status();
                } else if focused {
                    let _ = std::process::Command::new("solar-shell")
                        .args(["minimize", &id.to_string()])
                        .status();
                } else {
                    let _ = std::process::Command::new("niri")
                        .args(["msg", "action", "focus-window", "--id", &id.to_string()])
                        .status();
                }
            });
        });

        tasks_container.append(&task_btn);
    }
}
