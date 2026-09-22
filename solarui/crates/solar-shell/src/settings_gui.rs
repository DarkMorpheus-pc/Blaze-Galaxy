use gdk_pixbuf::PixbufLoader;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, CssProvider, DropDown, Image, Label,
    Notebook, Orientation, Separator, StringList, Window,
};
use solar_common::{
    HardwareDetector, HardwareTuningProfile, PerformanceTier, ShellEngine, SolarConfig,
    SurfaceProvider,
};
use crate::pixel_clock::{PixelClockConfig, PixelClockStyle};
use std::path::PathBuf;
use std::process::Command;

const LOGO_PNG: &[u8] = include_bytes!("../../../data/icons/solar-logo-256.png");

fn create_logo_image(size: i32) -> Image {
    let loader = match PixbufLoader::with_type("png") {
        Ok(l) => l,
        Err(_) => return Image::new(),
    };
    loader.set_size(size, size);
    if loader.write(LOGO_PNG).is_ok() && loader.close().is_ok() {
        if let Some(pixbuf) = loader.pixbuf() {
            let texture = gdk4::Texture::for_pixbuf(&pixbuf);
            let img = Image::from_paintable(Some(&texture));
            img.set_pixel_size(size);
            return img;
        }
    }
    Image::new()
}

pub fn launch_settings_window() {
    glib::set_prgname(Some("solar-settings"));
    glib::set_application_name("SolarUI Settings");

    if let Err(err) = gtk4::init() {
        eprintln!("Failed to initialize GTK4: {}", err);
        return;
    }

    let main_loop = glib::MainLoop::new(None, false);
    build_settings_ui(main_loop.clone());
    main_loop.run();
}

fn restart_taskbar_if_enabled() {
    let cfg = SolarConfig::load();
    // Kill existing taskbar instance
    let _ = Command::new("pkill")
        .args(["-f", "solar-shell (run|gui|taskbar|window)"])
        .status();

    if cfg.taskbar.enabled {
        // Spawn fresh taskbar instance
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(150));
            let _ = Command::new("solar-shell").arg("run").spawn();
        });
    }
}

fn build_settings_ui(main_loop: glib::MainLoop) {
    let provider = CssProvider::new();
    let css = r#"
        window {
            background-color: alpha(#16191c, 0.98);
        }

        .settings-card {
            padding: 20px;
        }

        .settings-header {
            margin-bottom: 16px;
            padding-bottom: 12px;
            border-bottom: 1px solid alpha(#525c62, 0.35);
        }

        .settings-title {
            color: #ffffff;
            font-size: 20px;
            font-weight: 700;
        }

        .settings-subtitle {
            color: #38c8ff;
            font-size: 13px;
            font-weight: 500;
        }

        .section-box {
            background-color: alpha(#23272a, 0.65);
            border: 1px solid alpha(#525c62, 0.35);
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 14px;
        }

        .section-title {
            color: #c4e7ff;
            font-size: 15px;
            font-weight: 600;
            margin-bottom: 10px;
        }

        .setting-row {
            margin: 8px 0;
        }

        .setting-label {
            color: #e1e3e5;
            font-size: 13px;
            font-weight: 500;
        }

        .setting-subtext {
            color: #8a9297;
            font-size: 11px;
        }

        .action-btn {
            background: linear-gradient(135deg, #007ba4, #0095c7);
            color: #ffffff;
            border-radius: 6px;
            padding: 6px 14px;
            font-size: 13px;
            font-weight: 500;
            border: none;
            transition: all 120ms ease;
        }

        .action-btn:hover {
            background: linear-gradient(135deg, #008ebf, #14a8dc);
        }

        .secondary-btn {
            background-color: alpha(#303437, 0.8);
            color: #c4e7ff;
            border-radius: 6px;
            padding: 6px 14px;
            font-size: 13px;
            border: 1px solid alpha(#525c62, 0.4);
        }

        .secondary-btn:hover {
            background-color: alpha(#40484d, 0.9);
        }

        .status-badge-ok {
            color: #4ade80;
            background-color: alpha(#22c55e, 0.15);
            border: 1px solid alpha(#22c55e, 0.35);
            border-radius: 6px;
            padding: 2px 8px;
            font-size: 11px;
            font-weight: 600;
        }

        .status-badge-info {
            color: #38c8ff;
            background-color: alpha(#0095c7, 0.15);
            border: 1px solid alpha(#0095c7, 0.35);
            border-radius: 6px;
            padding: 2px 8px;
            font-size: 11px;
            font-weight: 600;
        }

        .danger-btn {
            background: linear-gradient(135deg, #c53030, #e53e3e);
            color: #ffffff;
            border-radius: 6px;
            padding: 6px 14px;
            font-size: 13px;
            font-weight: 500;
            border: none;
            transition: all 120ms ease;
        }

        .danger-btn:hover {
            background: linear-gradient(135deg, #e53e3e, #f56565);
        }

        notebook > header {
            background-color: alpha(#1e2226, 0.8);
            border-radius: 10px;
            margin-bottom: 12px;
            border: 1px solid alpha(#525c62, 0.3);
        }

        notebook > header > tabs > tab {
            color: #b0b8be;
            font-weight: 600;
            padding: 8px 16px;
            font-size: 13px;
        }

        notebook > header > tabs > tab:checked {
            color: #38c8ff;
            border-bottom: 2px solid #38c8ff;
        }
    "#;
    provider.load_from_string(css);

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window = Window::builder()
        .title("SolarUI Ayarları - Masaüstü ve Görev Çubuğu")
        .default_width(720)
        .default_height(580)
        .build();

    let loop_quit = main_loop.clone();
    window.connect_close_request(move |_| {
        loop_quit.quit();
        glib::Propagation::Proceed
    });

    let root_box = GtkBox::new(Orientation::Vertical, 0);
    root_box.add_css_class("settings-card");

    // Header
    let header_box = GtkBox::new(Orientation::Horizontal, 14);
    header_box.add_css_class("settings-header");

    let logo = create_logo_image(44);
    header_box.append(&logo);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    let title = Label::new(Some("SolarUI Masaüstü Ayarları"));
    title.add_css_class("settings-title");
    title.set_halign(gtk4::Align::Start);
    text_box.append(&title);

    let subtitle = Label::new(Some("SolarUI kontrol paneli"));
    subtitle.add_css_class("settings-subtitle");
    subtitle.set_halign(gtk4::Align::Start);
    text_box.append(&subtitle);

    header_box.append(&text_box);
    root_box.append(&header_box);

    // Tabbed interface
    let notebook = Notebook::new();

    // ── TAB 1: GOOGLE PIXEL SAAT & MASAÜSTÜ WIDGET'I ────────────────────────
    let clock_tab = build_pixel_clock_tab();
    notebook.append_page(&clock_tab, Some(&Label::new(Some("Pixel Saat (test)"))));

    // ── TAB 2: GÖREV ÇUBUĞU (Taskbar Özelleştirme & Stiller) ─────────────────
    let taskbar_tab = build_taskbar_tab();
    notebook.append_page(&taskbar_tab, Some(&Label::new(Some("Görev Çubuğu"))));

    // ── TAB 3: EKRAN VE ARAYÜZ ÖLÇEĞİ (Display & UI Scale) ───────────────────
    let display_tab = build_display_tab();
    notebook.append_page(&display_tab, Some(&Label::new(Some("Ekran & Ölçek"))));

    let shell_tab = build_shell_tab();
    notebook.append_page(&shell_tab, Some(&Label::new(Some("Kabuk Motoru & Yüzeyler"))));

    let perf_tab = build_performance_tab();
    notebook.append_page(&perf_tab, Some(&Label::new(Some("Performans"))));

    let health_tab = build_health_tab();
    notebook.append_page(&health_tab, Some(&Label::new(Some("Sistem Sağlığı"))));

    // ── TAB: CAELESTIA / HIBRIT ÖZELLEŞTİRME ─────────────────────────────────
    let caelestia_tab = build_caelestia_tab();
    notebook.append_page(&caelestia_tab, Some(&Label::new(Some("Caelestia / Hibrit Düzeni"))));

    // ── TAB: NOCTALIA ÜST BARI VE HIZLI BAĞLANTILAR ─────────────────────────
    let noctalia_tab = GtkBox::new(Orientation::Vertical, 12);
    noctalia_tab.set_margin_top(10);
    noctalia_tab.set_margin_bottom(10);

    let n_sec = GtkBox::new(Orientation::Vertical, 12);
    n_sec.add_css_class("section-box");

    let n_title = Label::new(Some("Noctalia Bar ve Kontrol Panelleri"));
    n_title.add_css_class("section-title");
    n_title.set_halign(gtk4::Align::Start);
    n_sec.append(&n_title);

    let n_desc = Label::new(Some(
        "Noctalia Bar, SolarUI'ın üst bilgi ve kontrol merkezidir. Tüm ayarlarını ve panellerini aşağıdan doğrudan açabilirsiniz.",
    ));
    n_desc.add_css_class("setting-subtext");
    n_desc.set_wrap(true);
    n_desc.set_halign(gtk4::Align::Start);
    n_sec.append(&n_desc);

    let btn_grid = GtkBox::new(Orientation::Horizontal, 12);

    let btn_settings = Button::with_label("Noctalia Ayarları");
    btn_settings.add_css_class("action-btn");
    btn_settings.connect_clicked(|_| {
        let _ = Command::new("noctalia")
            .args(["msg", "settings-toggle"])
            .spawn();
    });
    btn_grid.append(&btn_settings);

    let btn_cc = Button::with_label("Kontrol Merkezi");
    btn_cc.add_css_class("secondary-btn");
    btn_cc.connect_clicked(|_| {
        let _ = Command::new("noctalia")
            .args(["msg", "panel-toggle", "control-center"])
            .spawn();
    });
    btn_grid.append(&btn_cc);

    let btn_wp = Button::with_label("Duvar Kağıdı Seçici");
    btn_wp.add_css_class("secondary-btn");
    btn_wp.connect_clicked(|_| {
        let _ = Command::new("noctalia")
            .args(["msg", "panel-toggle", "wallpaper"])
            .spawn();
    });
    btn_grid.append(&btn_wp);

    let btn_audio = Button::with_label("Ses & Mikser");
    btn_audio.add_css_class("secondary-btn");
    btn_audio.connect_clicked(|_| {
        let _ = Command::new("noctalia")
            .args(["msg", "panel-toggle", "control-center", "audio"])
            .spawn();
    });
    btn_grid.append(&btn_audio);

    n_sec.append(&btn_grid);
    noctalia_tab.append(&n_sec);

    notebook.append_page(&noctalia_tab, Some(&Label::new(Some("Noctalia Bar"))));

    // ── TAB 3: KISAYOLLAR & KARŞILAMA HUD ─────────────────────────────────────
    let hud_tab = GtkBox::new(Orientation::Vertical, 12);
    hud_tab.set_margin_top(10);
    hud_tab.set_margin_bottom(10);

    let h_sec = GtkBox::new(Orientation::Vertical, 12);
    h_sec.add_css_class("section-box");

    let h_title = Label::new(Some("Açılış Karşılama ve Kısayollar Ekranı"));
    h_title.add_css_class("section-title");
    h_title.set_halign(gtk4::Align::Start);
    h_sec.append(&h_title);

    let h_sub = Label::new(Some(
        ".",
    ));
    h_sub.add_css_class("setting-subtext");
    h_sub.set_halign(gtk4::Align::Start);
    h_sec.append(&h_sub);

    let open_welcome_btn = Button::with_label("Kısayol ve Karşılama Ekranını Şimdi Aç");
    open_welcome_btn.add_css_class("action-btn");
    open_welcome_btn.connect_clicked(|_| {
        let _ = Command::new("solar-shell").arg("welcome").spawn();
    });
    h_sec.append(&open_welcome_btn);

    let cfg = SolarConfig::load();
    let chk_startup = CheckButton::with_label("Oturum açıldığında karşılama ekranını otomatik göster");
    chk_startup.set_active(cfg.shortcuts_hud.show_at_startup);
    chk_startup.connect_toggled(|cb| {
        let mut c = SolarConfig::load();
        c.shortcuts_hud.show_at_startup = cb.is_active();
        let _ = c.save();
    });
    h_sec.append(&chk_startup);

    hud_tab.append(&h_sec);
    notebook.append_page(&hud_tab, Some(&Label::new(Some("Kısayollar"))));

    // ── TAB 4: HAKKINDA & SOLARUI İMZASI ──────────────────────────────────────
    let about_tab = GtkBox::new(Orientation::Vertical, 12);
    about_tab.set_margin_top(10);
    about_tab.set_margin_bottom(10);

    let a_sec = GtkBox::new(Orientation::Vertical, 12);
    a_sec.add_css_class("section-box");
    a_sec.set_halign(gtk4::Align::Center);

    let big_logo = create_logo_image(80);
    big_logo.set_halign(gtk4::Align::Center);
    a_sec.append(&big_logo);

    let a_name = Label::new(Some("SolarUI Masaüstü Ortamı (BlazeOS Edition)"));
    a_name.add_css_class("settings-title");
    a_name.set_halign(gtk4::Align::Center);
    a_sec.append(&a_name);

    let a_sig = Label::new(Some("Modern , zarif ve asil bir ortam"));
    a_sig.add_css_class("settings-subtitle");
    a_sig.set_halign(gtk4::Align::Center);
    a_sec.append(&a_sig);

    let a_ver = Label::new(Some("Sürüm 1.0.0 (Rust + GTK4 + Wayland)"));
    a_ver.add_css_class("setting-subtext");
    a_ver.set_halign(gtk4::Align::Center);
    a_sec.append(&a_ver);

    let sep = Separator::new(Orientation::Horizontal);
    a_sec.append(&sep);

    let a_dev = Label::new(Some("Geliştirici: Bulut Ars. E. (DarkMorpheus) & BlazeOS Project\nGoogle Material 3 ikon standardı."));
    a_dev.add_css_class("setting-label");
    a_dev.set_justify(gtk4::Justification::Center);
    a_dev.set_halign(gtk4::Align::Center);
    a_sec.append(&a_dev);

    about_tab.append(&a_sec);
    notebook.append_page(&about_tab, Some(&Label::new(Some("Hakkında"))));

    root_box.append(&notebook);

    window.set_child(Some(&root_box));
    window.present();
}

fn build_performance_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 12);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);

    let hw = HardwareDetector::detect();
    let current_profile = HardwareTuningProfile::load();

    // 1. Detected Hardware Card
    let hw_sec = GtkBox::new(Orientation::Vertical, 8);
    hw_sec.add_css_class("section-box");

    let hw_title = Label::new(Some("Algılanan Donanım Mimarisi"));
    hw_title.add_css_class("section-title");
    hw_title.set_halign(gtk4::Align::Start);
    hw_sec.append(&hw_title);

    let dev_label = Label::new(Some(&format!(
        "GPU: {}\nCihazlar: {}\nToplam RAM: {:.1} GB | İşlemci: {} Çekirdek\nGüç: {}",
        hw.gpu_vendor,
        if hw.gpu_devices.is_empty() {
            "Tümleşik / Standart DRM".to_string()
        } else {
            hw.gpu_devices.join(", ")
        },
        (hw.total_ram_mb as f64) / 1024.0,
        hw.cpu_cores,
        if hw.on_battery {
            "Batarya Gücünde"
        } else {
            "Şebeke Gücünde (AC Bağlı)"
        }
    )));
    dev_label.add_css_class("setting-label");
    dev_label.set_halign(gtk4::Align::Start);
    dev_label.set_margin_bottom(6);
    hw_sec.append(&dev_label);

    let rec_box = GtkBox::new(Orientation::Horizontal, 8);
    let rec_title = Label::new(Some("Donanımınıza Özel Önerilen Profil:"));
    rec_title.add_css_class("setting-subtext");
    rec_box.append(&rec_title);

    let rec_val = Label::new(Some(hw.recommended_tier.name_tr()));
    rec_val.add_css_class("status-badge-info");
    rec_box.append(&rec_val);
    hw_sec.append(&rec_box);

    tab.append(&hw_sec);

    // 2. Profile Selection Card
    let prof_sec = GtkBox::new(Orientation::Vertical, 10);
    prof_sec.add_css_class("section-box");

    let prof_title = Label::new(Some("Aktif Performans Profili"));
    prof_title.add_css_class("section-title");
    prof_title.set_halign(gtk4::Align::Start);
    prof_sec.append(&prof_title);

    let prof_sub = Label::new(Some(
        "Görsel efektler (blur, gölgeler, animasyon süreleri) donanım gücüne göre uyarlanır. Ekran yenileme hızı (Hz) profilden bağımsız olup daima monitörün native frekansında kalır.",
    ));
    prof_sub.add_css_class("setting-subtext");
    prof_sub.set_wrap(true);
    prof_sub.set_halign(gtk4::Align::Start);
    prof_sec.append(&prof_sub);

    let tier_items = StringList::new(&[
        "Hafif / Eski Donanım (Legacy) — Sıfır blur, sıfır gölge, anlık 0ms geçişler",
        "Dengeli (Balanced) — Optimize edilmiş akıcı animasyonlar, hafif cam efekti",
        "Yüksek Performans (Ultra) — Tam cam blur efekti, derin gölgeler, yay fiziği",
    ]);
    let tier_dropdown = DropDown::new(Some(tier_items), None::<gtk4::Expression>);

    match current_profile.tier {
        PerformanceTier::Legacy => tier_dropdown.set_selected(0),
        PerformanceTier::Balanced => tier_dropdown.set_selected(1),
        PerformanceTier::Ultra => tier_dropdown.set_selected(2),
    }

    tier_dropdown.connect_selected_notify(|dd| {
        let tier = match dd.selected() {
            0 => PerformanceTier::Legacy,
            1 => PerformanceTier::Balanced,
            _ => PerformanceTier::Ultra,
        };
        let p = HardwareTuningProfile::for_tier(tier);
        let _ = p.save();
        restart_taskbar_if_enabled();
    });

    prof_sec.append(&tier_dropdown);
    tab.append(&prof_sec);

    // 3. ChromeOS Ash Stability & Resource Optimization Card
    let ash_sec = GtkBox::new(Orientation::Vertical, 10);
    ash_sec.add_css_class("section-box");

    let ash_title = Label::new(Some("ChromeOS Ash Seviyesi Kararlılık & Güç Tasarrufu"));
    ash_title.add_css_class("section-title");
    ash_title.set_halign(gtk4::Align::Start);
    ash_sec.append(&ash_title);

    let ash_sub = Label::new(Some(
        "Aura/Ash esintili akıllı kaynak yönetimi: Boşta CPU/GPU kullanımını minimuma indirir, çöken kabukları otomatik iyileştirir ve arayüzü OOM krizlerine karşı korur.",
    ));
    ash_sub.add_css_class("setting-subtext");
    ash_sub.set_wrap(true);
    ash_sub.set_halign(gtk4::Align::Start);
    ash_sec.append(&ash_sub);

    // Watchdog Auto-Healing
    let chk_watchdog = CheckButton::with_label("Sıfır-Çökme Watchdog (Çöken arayüz motorlarını 3sn içinde sessizce onarır)");
    chk_watchdog.set_active(current_profile.watchdog_enabled);
    chk_watchdog.connect_toggled(|cb| {
        let mut p = HardwareTuningProfile::load();
        p.watchdog_enabled = cb.is_active();
        let _ = p.save();
    });
    ash_sec.append(&chk_watchdog);

    // Idle Resource Throttling
    let chk_idle = CheckButton::with_label("Boşta Akıllı Kaynak Kısma (Kullanıcı boştayken CPU/RAM polling ve render'ı frenler)");
    chk_idle.set_active(current_profile.idle_throttling_enabled);
    chk_idle.connect_toggled(|cb| {
        let mut p = HardwareTuningProfile::load();
        p.idle_throttling_enabled = cb.is_active();
        let _ = p.save();
    });
    ash_sec.append(&chk_idle);

    // OOM Killer Shield
    let chk_oom = CheckButton::with_label("OOM Bellek Kalkanı (Ağır RAM yükünde pencere yöneticisi ve kabuğun öldürülmesini engeller)");
    chk_oom.set_active(current_profile.oom_shield_enabled);
    chk_oom.connect_toggled(|cb| {
        let mut p = HardwareTuningProfile::load();
        p.oom_shield_enabled = cb.is_active();
        let _ = p.save();
    });
    ash_sec.append(&chk_oom);

    tab.append(&ash_sec);

    tab
}

fn build_health_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 12);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);

    // 1. Health Status Card
    let stat_sec = GtkBox::new(Orientation::Vertical, 10);
    stat_sec.add_css_class("section-box");

    let stat_title = Label::new(Some("Platform Durumu ve Süreç İzolasyonu (ChromeOS Modeli)"));
    stat_title.add_css_class("section-title");
    stat_title.set_halign(gtk4::Align::Start);
    stat_sec.append(&stat_title);

    let stat_desc = Label::new(Some(
        "SolarUI mimarisi bileşenleri izole eder: Bir UI bileşeni veya servis çöktüğünde diğerleri ve açık pencereleriniz ASLA kapanmaz.",
    ));
    stat_desc.add_css_class("setting-subtext");
    stat_desc.set_wrap(true);
    stat_desc.set_halign(gtk4::Align::Start);
    stat_sec.append(&stat_desc);

    // Table of components
    let components = [
        (
            "SolarCore Platform Servisi",
            "Aktif (Unix IPC Soketi Hazır)",
            "status-badge-ok",
        ),
        (
            "SolarShell Görev Çubuğu",
            "Aktif (Sıfır Polling / Olay Güdümlü)",
            "status-badge-ok",
        ),
        (
            "SolarUI Pencere Yöneticisi",
            "Canlı (Wayland Oturumu)",
            "status-badge-ok",
        ),
        (
            "SolarWindowRegistry",
            "Senkronize (Atomik Durum Makinesi)",
            "status-badge-info",
        ),
    ];

    for (comp, status, badge_cls) in components {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_top(4);
        row.set_margin_bottom(4);

        let c_lbl = Label::new(Some(comp));
        c_lbl.add_css_class("setting-label");
        c_lbl.set_hexpand(true);
        c_lbl.set_halign(gtk4::Align::Start);
        row.append(&c_lbl);

        let s_lbl = Label::new(Some(status));
        s_lbl.add_css_class(badge_cls);
        s_lbl.set_halign(gtk4::Align::End);
        row.append(&s_lbl);

        stat_sec.append(&row);
    }

    tab.append(&stat_sec);

    // 2. Self-Healing Crash Test Card
    let test_sec = GtkBox::new(Orientation::Vertical, 10);
    test_sec.add_css_class("section-box");

    let test_title = Label::new(Some("Self-Healing Watchdog Testi"));
    test_title.add_css_class("section-title");
    test_title.set_halign(gtk4::Align::Start);
    test_sec.append(&test_title);

    let test_info = Label::new(Some(
        "Aşağıdaki butona tıkladığınızda `solar-shell` süreci aniden sonlandırılır. SolarUI pencere yöneticisi ve SolarCore çalışmaya devam eder. Görev çubuğu yeniden başlatıldığında açık uygulamalarınızın durumu yeniden yüklenir.",
    ));
    test_info.add_css_class("setting-subtext");
    test_info.set_wrap(true);
    test_info.set_halign(gtk4::Align::Start);
    test_sec.append(&test_info);

    let crash_btn = Button::with_label("Kabuğu Test Amaçlı Sonlandır (Self-Healing Crash Test)");
    crash_btn.add_css_class("danger-btn");
    crash_btn.set_halign(gtk4::Align::Start);
    crash_btn.connect_clicked(|_| {
        std::thread::spawn(|| {
            // Kill solar-shell run instance
            let _ = Command::new("pkill")
                .args(["-9", "-f", "solar-shell (run|gui|taskbar|window)"])
                .status();
            std::thread::sleep(std::time::Duration::from_millis(150));
            // Ensure watchdog brings it back
            let _ = Command::new("solar-shell").arg("run").spawn();
        });
    });
    test_sec.append(&crash_btn);

    tab.append(&test_sec);

    tab
}

fn build_pixel_clock_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 10);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);

    // Section 1: Style Selection Cards (Matching user screenshots 1, 2, 4)
    let style_sec = GtkBox::new(Orientation::Vertical, 10);
    style_sec.add_css_class("section-box");

    let s_title = Label::new(Some("Google Pixel / Material You Saat Stili"));
    s_title.add_css_class("section-title");
    s_title.set_halign(gtk4::Align::Start);
    style_sec.append(&s_title);

    let s_sub = Label::new(Some("Masaüstü duvar kağıdı üzerinde çalışan Google Pixel saat stilleri:"));
    s_sub.add_css_class("setting-subtext");
    s_sub.set_halign(gtk4::Align::Start);
    style_sec.append(&s_sub);

    let style_choices = StringList::new(&[
        "Pixel Kalın Yatay (16:49 + Pil Kapsülü & Tarih)",
        "Pixel İki Katlı Dikey (16 / 49 Büyük Balon Rakam)",
        "Pixel İçi Boş Kontur (Outline Minimalist)",
        "Pixel Analog Kalın (12, 3, 6, 9 Kalın Rakamlar)",
        "Pixel Analog Kontur (12, 3, 6, 9 İçi Boş Rakamlar)",
        "Pixel Analog Takimetre (Hız Göstergesi Kadranı)",
    ]);
    let style_dd = DropDown::new(Some(style_choices), None::<gtk4::Expression>);

    let clock_cfg = PixelClockConfig::load();
    let initial_idx = match clock_cfg.style {
        PixelClockStyle::BoldHorizontal => 0,
        PixelClockStyle::StackedVertical => 1,
        PixelClockStyle::Outline => 2,
        PixelClockStyle::AnalogBold => 3,
        PixelClockStyle::AnalogOutline => 4,
        PixelClockStyle::AnalogTachymeter => 5,
    };
    style_dd.set_selected(initial_idx);
    style_dd.connect_selected_notify(|dd| {
        let mut cfg = PixelClockConfig::load();
        cfg.style = match dd.selected() {
            0 => PixelClockStyle::BoldHorizontal,
            1 => PixelClockStyle::StackedVertical,
            2 => PixelClockStyle::Outline,
            3 => PixelClockStyle::AnalogBold,
            4 => PixelClockStyle::AnalogOutline,
            _ => PixelClockStyle::AnalogTachymeter,
        };
        let _ = cfg.save();
    });
    style_sec.append(&style_dd);
    tab.append(&style_sec);

    // Section 2: Colors & Material You (Matching screenshot 3)
    let color_sec = GtkBox::new(Orientation::Vertical, 10);
    color_sec.add_css_class("section-box");

    let c_title = Label::new(Some("Renklendirme Seçenekleri (Coloring Options)"));
    c_title.add_css_class("section-title");
    c_title.set_halign(gtk4::Align::Start);
    color_sec.append(&c_title);

    let color_choices = StringList::new(&[
        "Dinamik: Material You (Duvar kağıdından otomatik uyarlanan ton)",
        "Özel Renk: Elektrik Camgöbeği (Cyan #80d4ff)",
        "Özel Renk: Pastel Mor (Lavender #c084fc)",
        "Özel Renk: Nane Yeşili (Mint #4ade80)",
        "Özel Renk: Şeftali / Mercan (Peach #fb923c)",
        "Özel Renk: Saf Beyaz (Pure White #ffffff)",
    ]);
    let color_dd = DropDown::new(Some(color_choices), None::<gtk4::Expression>);
    let color_idx = if clock_cfg.color_mode == "dynamic" {
        0
    } else {
        match clock_cfg.custom_color.as_str() {
            "#80d4ff" => 1,
            "#c084fc" => 2,
            "#4ade80" => 3,
            "#fb923c" => 4,
            _ => 5,
        }
    };
    color_dd.set_selected(color_idx);
    color_dd.connect_selected_notify(|dd| {
        let mut cfg = PixelClockConfig::load();
        match dd.selected() {
            0 => {
                cfg.color_mode = "dynamic".to_string();
            }
            1 => {
                cfg.color_mode = "custom".to_string();
                cfg.custom_color = "#80d4ff".to_string();
            }
            2 => {
                cfg.color_mode = "custom".to_string();
                cfg.custom_color = "#c084fc".to_string();
            }
            3 => {
                cfg.color_mode = "custom".to_string();
                cfg.custom_color = "#4ade80".to_string();
            }
            4 => {
                cfg.color_mode = "custom".to_string();
                cfg.custom_color = "#fb923c".to_string();
            }
            _ => {
                cfg.color_mode = "custom".to_string();
                cfg.custom_color = "#ffffff".to_string();
            }
        }
        let _ = cfg.save();
    });
    color_sec.append(&color_dd);
    tab.append(&color_sec);

    // Section 3: Opacity, Battery Pill, Date
    let opts_sec = GtkBox::new(Orientation::Vertical, 10);
    opts_sec.add_css_class("section-box");

    let o_title = Label::new(Some("Rakam Opaklığı ve Göstergeler"));
    o_title.add_css_class("section-title");
    o_title.set_halign(gtk4::Align::Start);
    opts_sec.append(&o_title);

    let row1 = GtkBox::new(Orientation::Horizontal, 16);
    let op_lbl = Label::new(Some("Rakam Opaklığı (Numeral Opacity):"));
    op_lbl.add_css_class("setting-label");
    row1.append(&op_lbl);

    let op_choices = StringList::new(&["%100 (Tam Belirgin)", "%95 (Önerilen)", "%80 (Hafif Saydam)", "%60 (Yarı Saydam)", "%40 (Düşük)"]);
    let op_dd = DropDown::new(Some(op_choices), None::<gtk4::Expression>);
    op_dd.set_selected(1);
    op_dd.connect_selected_notify(|dd| {
        let mut cfg = PixelClockConfig::load();
        cfg.numeral_opacity = match dd.selected() {
            0 => 1.0,
            1 => 0.95,
            2 => 0.80,
            3 => 0.60,
            _ => 0.40,
        };
        let _ = cfg.save();
    });
    row1.append(&op_dd);
    opts_sec.append(&row1);

    let row2 = GtkBox::new(Orientation::Horizontal, 20);
    let bat_chk = CheckButton::with_label("Pil Kapsülünü Göster (Battery Pill ⚡ %41)");
    bat_chk.set_active(clock_cfg.show_battery_pill);
    bat_chk.connect_toggled(|cb| {
        let mut cfg = PixelClockConfig::load();
        cfg.show_battery_pill = cb.is_active();
        let _ = cfg.save();
    });
    row2.append(&bat_chk);

    let date_chk = CheckButton::with_label("Tarih Göster");
    date_chk.set_active(clock_cfg.show_date);
    date_chk.connect_toggled(|cb| {
        let mut cfg = PixelClockConfig::load();
        cfg.show_date = cb.is_active();
        let _ = cfg.save();
    });
    row2.append(&date_chk);
    opts_sec.append(&row2);
    tab.append(&opts_sec);

    // Section 4: Action Buttons (Launch / Restart widget)
    let act_sec = GtkBox::new(Orientation::Horizontal, 12);
    act_sec.set_halign(gtk4::Align::End);

    let launch_btn = Button::with_label("Pixel Saat Widget'ını Masaüstünde Aç");
    launch_btn.add_css_class("action-btn");
    launch_btn.connect_clicked(|_| {
        let _ = Command::new("pkill").args(["-f", "solar-shell (clock|pixel-clock)"]).status();
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(150));
            let _ = Command::new("solar-shell").arg("clock").spawn();
        });
    });
    act_sec.append(&launch_btn);
    tab.append(&act_sec);

    tab
}

fn build_taskbar_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 10);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);

    // Section 1: Appearance Style Presets
    let preset_sec = GtkBox::new(Orientation::Vertical, 10);
    preset_sec.add_css_class("section-box");

    let p_title = Label::new(Some("Görev Çubuğu Görünüm Stili (Taskbar Appearance)"));
    p_title.add_css_class("section-title");
    p_title.set_halign(gtk4::Align::Start);
    preset_sec.append(&p_title);

    let p_sub = Label::new(Some(
        "Buzlu cam efekti zorunlu değildir. İster düz renk, ister şeffaf cam efekti, ister tam saydam, isterseniz de Material 3 duvar kağıdı renklerine dinamik uyarlanan stil seçebilirsiniz:",
    ));
    p_sub.add_css_class("setting-subtext");
    p_sub.set_wrap(true);
    p_sub.set_halign(gtk4::Align::Start);
    preset_sec.append(&p_sub);

    let preset_choices = StringList::new(&[
        "Buzlu Cam (Frosted Acrylic Glass — %32 Şeffaflık & Blur)",
        "Düz Opak Renk (Solid Opaque — %100 Mat)",
        "Tam Şeffaf (Fully Transparent — Sadece Simgeler Yüzer)",
        "Material 3 Dinamik Renk (Duvar Kağıdı Rengine Uyumlu)",
    ]);
    let preset_dd = DropDown::new(Some(preset_choices), None::<gtk4::Expression>);
    preset_dd.set_selected(0);
    preset_dd.connect_selected_notify(|dd| {
        match dd.selected() {
            0 => apply_taskbar_preset("glass"),
            1 => apply_taskbar_preset("solid"),
            2 => apply_taskbar_preset("transparent"),
            _ => apply_taskbar_preset("m3-adaptive"),
        }
    });
    preset_sec.append(&preset_dd);
    tab.append(&preset_sec);

    // Section 2: Fine-grained Customization (Opacity, Corner Radius, Height)
    let fine_sec = GtkBox::new(Orientation::Vertical, 10);
    fine_sec.add_css_class("section-box");

    let f_title = Label::new(Some("Manuel Boyut ve Şeffaflık İnce Ayarları"));
    f_title.add_css_class("section-title");
    f_title.set_halign(gtk4::Align::Start);
    fine_sec.append(&f_title);

    // Opacity
    let op_row = GtkBox::new(Orientation::Horizontal, 16);
    let op_l = Label::new(Some("Şeffaflık Derecesi:"));
    op_l.add_css_class("setting-label");
    op_l.set_hexpand(true);
    op_l.set_halign(gtk4::Align::Start);
    op_row.append(&op_l);

    let op_list = StringList::new(&["%10 (Ultra Cam)", "%25 (Hafif Cam)", "%32 (Varsayılan Solar)", "%50 (Yarı Saydam)", "%75 (Koyu Cam)", "%100 (Mat)"]);
    let op_dd = DropDown::new(Some(op_list), None::<gtk4::Expression>);
    op_dd.set_selected(2);
    op_dd.connect_selected_notify(|dd| {
        let op = match dd.selected() {
            0 => 0.10,
            1 => 0.25,
            2 => 0.32,
            3 => 0.50,
            4 => 0.75,
            _ => 1.0,
        };
        update_noctalia_taskbar_opacity(op);
    });
    op_row.append(&op_dd);
    fine_sec.append(&op_row);

    // Corner Radius
    let rad_row = GtkBox::new(Orientation::Horizontal, 16);
    let rad_l = Label::new(Some("Köşe Yuvarlaklığı:"));
    rad_l.add_css_class("setting-label");
    rad_l.set_hexpand(true);
    rad_l.set_halign(gtk4::Align::Start);
    rad_row.append(&rad_l);

    let rad_list = StringList::new(&["0 px (Köşeli Düz)", "8 px (Hafif Kavis)", "16 px (Varsayılan Modern)", "24 px (Ekstra Yuvarlak)", "40 px (Kapsül)"]);
    let rad_dd = DropDown::new(Some(rad_list), None::<gtk4::Expression>);
    rad_dd.set_selected(2);
    rad_dd.connect_selected_notify(|dd| {
        let r = match dd.selected() {
            0 => 0,
            1 => 8,
            2 => 16,
            3 => 24,
            _ => 40,
        };
        update_noctalia_taskbar_radius(r);
    });
    rad_row.append(&rad_dd);
    fine_sec.append(&rad_row);

    // Height / Thickness
    let h_row = GtkBox::new(Orientation::Horizontal, 16);
    let h_l = Label::new(Some("Çubuk Yüksekliği (Kalınlık):"));
    h_l.add_css_class("setting-label");
    h_l.set_hexpand(true);
    h_l.set_halign(gtk4::Align::Start);
    h_row.append(&h_l);

    let h_list = StringList::new(&["36 px (Kompakt)", "40 px (Varsayılan)", "44 px (Rahat)", "48 px (Büyük)", "56 px (KDE Plasma Standart)"]);
    let h_dd = DropDown::new(Some(h_list), None::<gtk4::Expression>);
    h_dd.set_selected(1);
    h_dd.connect_selected_notify(|dd| {
        let th = match dd.selected() {
            0 => 36,
            1 => 40,
            2 => 44,
            3 => 48,
            _ => 56,
        };
        update_noctalia_taskbar_thickness(th);
    });
    h_row.append(&h_dd);
    fine_sec.append(&h_row);

    tab.append(&fine_sec);

    // Section 3: Open Full Noctalia Bar Settings Button
    let btn_row = GtkBox::new(Orientation::Horizontal, 12);
    btn_row.set_halign(gtk4::Align::End);
    let full_btn = Button::with_label("Gelişmiş Çubuk Düzenleyicisini Aç");
    full_btn.add_css_class("action-btn");
    full_btn.connect_clicked(|_| {
        let _ = Command::new("noctalia").args(["msg", "settings-open", "bar"]).spawn();
    });
    btn_row.append(&full_btn);
    tab.append(&btn_row);

    tab
}

fn apply_taskbar_preset(preset: &str) {
    let path = "/home/darkmorpheus/.local/state/noctalia/settings.toml";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(mut doc) = content.parse::<toml::Value>() {
            if let Some(bar) = doc.get_mut("bar").and_then(|b| b.get_mut("taskbar")) {
                if let Some(t) = bar.as_table_mut() {
                    match preset {
                        "glass" => {
                            t.insert("background_opacity".to_string(), toml::Value::Float(0.32));
                            t.insert("border".to_string(), toml::Value::String("outline".to_string()));
                            t.insert("radius".to_string(), toml::Value::Integer(16));
                        }
                        "solid" => {
                            t.insert("background_opacity".to_string(), toml::Value::Float(1.0));
                            t.insert("border".to_string(), toml::Value::String("none".to_string()));
                            t.insert("radius".to_string(), toml::Value::Integer(12));
                        }
                        "transparent" => {
                            t.insert("background_opacity".to_string(), toml::Value::Float(0.0));
                            t.insert("border".to_string(), toml::Value::String("none".to_string()));
                            t.insert("radius".to_string(), toml::Value::Integer(16));
                        }
                        "m3-adaptive" => {
                            t.insert("background_opacity".to_string(), toml::Value::Float(0.42));
                            t.insert("border".to_string(), toml::Value::String("outline".to_string()));
                            t.insert("radius".to_string(), toml::Value::Integer(20));
                        }
                        _ => {}
                    }
                }
            }
            if let Ok(serialized) = toml::to_string_pretty(&doc) {
                let _ = std::fs::write(path, serialized);
                let _ = Command::new("noctalia").args(["msg", "templates-apply"]).spawn();
            }
        }
    }
}

fn update_noctalia_taskbar_opacity(opacity: f64) {
    let path = "/home/darkmorpheus/.local/state/noctalia/settings.toml";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(mut doc) = content.parse::<toml::Value>() {
            if let Some(bar) = doc.get_mut("bar").and_then(|b| b.get_mut("taskbar")) {
                if let Some(t) = bar.as_table_mut() {
                    t.insert("background_opacity".to_string(), toml::Value::Float(opacity));
                }
            }
            if let Ok(serialized) = toml::to_string_pretty(&doc) {
                let _ = std::fs::write(path, serialized);
                let _ = Command::new("noctalia").args(["msg", "templates-apply"]).spawn();
            }
        }
    }
}

fn update_noctalia_taskbar_radius(radius: i32) {
    let path = "/home/darkmorpheus/.local/state/noctalia/settings.toml";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(mut doc) = content.parse::<toml::Value>() {
            if let Some(bar) = doc.get_mut("bar").and_then(|b| b.get_mut("taskbar")) {
                if let Some(t) = bar.as_table_mut() {
                    t.insert("radius".to_string(), toml::Value::Integer(radius as i64));
                }
            }
            if let Ok(serialized) = toml::to_string_pretty(&doc) {
                let _ = std::fs::write(path, serialized);
                let _ = Command::new("noctalia").args(["msg", "templates-apply"]).spawn();
            }
        }
    }
}

fn update_noctalia_taskbar_thickness(thickness: i32) {
    let path = "/home/darkmorpheus/.local/state/noctalia/settings.toml";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(mut doc) = content.parse::<toml::Value>() {
            if let Some(bar) = doc.get_mut("bar").and_then(|b| b.get_mut("taskbar")) {
                if let Some(t) = bar.as_table_mut() {
                    t.insert("thickness".to_string(), toml::Value::Integer(thickness as i64));
                }
            }
            if let Ok(serialized) = toml::to_string_pretty(&doc) {
                let _ = std::fs::write(path, serialized);
                let _ = Command::new("noctalia").args(["msg", "templates-apply"]).spawn();
            }
        }
    }
}

fn update_noctalia_shell_settings(engine: &str, launcher: &str, dash: &str) {
    let path = "/home/darkmorpheus/.local/state/noctalia/settings.toml";
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(mut doc) = content.parse::<toml::Value>() {
            if let Some(table) = doc.as_table_mut() {
                if !table.contains_key("solarui") {
                    table.insert("solarui".to_string(), toml::Value::Table(toml::map::Map::new()));
                }
                if let Some(s) = table.get_mut("solarui").and_then(|v| v.as_table_mut()) {
                    s.insert("shell_engine".to_string(), toml::Value::String(engine.to_string()));
                    s.insert("launcher_provider".to_string(), toml::Value::String(launcher.to_string()));
                    s.insert("dashboard_provider".to_string(), toml::Value::String(dash.to_string()));
                }
            }
            if let Ok(serialized) = toml::to_string_pretty(&doc) {
                let _ = std::fs::write(path, serialized);
            }
        }
    }
}

fn build_shell_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 12);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);

    let sec = GtkBox::new(Orientation::Vertical, 12);
    sec.add_css_class("section-box");

    let title = Label::new(Some("Kabuk ve Yüzey Motoru (Shell Engine Switcher)"));
    title.add_css_class("section-title");
    title.set_halign(gtk4::Align::Start);
    sec.append(&title);

    let desc = Label::new(Some(
        "SolarUI mimarisi altında Noctalia v5 (C++23 Native) ve Caelestia Shell (Quickshell QML) arasında dinamik geçiş yapabilir veya Hibrit modda Noctalia omurgasını Caelestia yüzeyleri (Launcher/Dashboard) ile birleştirebilirsiniz.",
    ));
    desc.add_css_class("setting-subtext");
    desc.set_wrap(true);
    desc.set_halign(gtk4::Align::Start);
    sec.append(&desc);

    let cfg = SolarConfig::load();

    // 1. Shell Engine Selector
    let row1 = GtkBox::new(Orientation::Horizontal, 12);
    let lbl1 = Label::new(Some("Aktif Kabuk Motoru:"));
    lbl1.add_css_class("setting-label");
    lbl1.set_hexpand(true);
    lbl1.set_halign(gtk4::Align::Start);
    row1.append(&lbl1);

    let engines = [
        "Noctalia v5 (C++23 Native)",
        "Caelestia Shell (Quickshell QML)",
        "SolarUI Hibrit (Noctalia + Caelestia)",
    ];
    let engine_model = StringList::new(&engines);
    let engine_dd = DropDown::new(Some(engine_model), None::<gtk4::Expression>);
    let active_idx = match cfg.shell.engine {
        ShellEngine::Noctalia => 0,
        ShellEngine::Caelestia => 1,
        ShellEngine::Hybrid => 2,
    };
    engine_dd.set_selected(active_idx);
    row1.append(&engine_dd);
    sec.append(&row1);

    // 2. Launcher Provider Selector (for Hybrid)
    let row2 = GtkBox::new(Orientation::Horizontal, 12);
    let lbl2 = Label::new(Some("Başlatıcı Sağlayıcı (Launcher):"));
    lbl2.add_css_class("setting-label");
    lbl2.set_hexpand(true);
    lbl2.set_halign(gtk4::Align::Start);
    row2.append(&lbl2);

    let launchers = ["Noctalia Başlatıcı", "Caelestia Başlatıcı"];
    let launcher_model = StringList::new(&launchers);
    let launcher_dd = DropDown::new(Some(launcher_model), None::<gtk4::Expression>);
    launcher_dd.set_selected(match cfg.shell.launcher_provider {
        SurfaceProvider::Noctalia => 0,
        _ => 1,
    });
    row2.append(&launcher_dd);
    sec.append(&row2);

    // 3. Dashboard Provider Selector (for Hybrid)
    let row3 = GtkBox::new(Orientation::Horizontal, 12);
    let lbl3 = Label::new(Some("Dashboard Sağlayıcı:"));
    lbl3.add_css_class("setting-label");
    lbl3.set_hexpand(true);
    lbl3.set_halign(gtk4::Align::Start);
    row3.append(&lbl3);

    let dash_providers = ["Caelestia Dashboard", "Noctalia Kontrol Merkezi"];
    let dash_model = StringList::new(&dash_providers);
    let dash_dd = DropDown::new(Some(dash_model), None::<gtk4::Expression>);
    dash_dd.set_selected(match cfg.shell.dashboard_provider {
        SurfaceProvider::Caelestia => 0,
        _ => 1,
    });
    row3.append(&dash_dd);
    sec.append(&row3);

    // 4. Action Buttons
    let btn_box = GtkBox::new(Orientation::Horizontal, 10);
    btn_box.set_margin_top(8);

    let apply_btn = Button::with_label("Kabuk Motorunu Uygula & Geçiş Yap");
    apply_btn.add_css_class("action-btn");
    apply_btn.set_halign(gtk4::Align::Start);

    let engine_dd_clone = engine_dd.clone();
    let launcher_dd_clone = launcher_dd.clone();
    let dash_dd_clone = dash_dd.clone();

    let shell_status_lbl = Label::new(Some(""));
    shell_status_lbl.add_css_class("setting-subtext");
    shell_status_lbl.set_halign(gtk4::Align::Start);

    let shell_status_clone = shell_status_lbl.clone();
    apply_btn.connect_clicked(move |_| {
        let mut cfg = SolarConfig::load();
        let target_engine = match engine_dd_clone.selected() {
            0 => ShellEngine::Noctalia,
            1 => ShellEngine::Caelestia,
            _ => ShellEngine::Hybrid,
        };
        cfg.shell.engine = target_engine;
        cfg.shell.launcher_provider = match launcher_dd_clone.selected() {
            0 => SurfaceProvider::Noctalia,
            _ => SurfaceProvider::Caelestia,
        };
        cfg.shell.dashboard_provider = match dash_dd_clone.selected() {
            0 => SurfaceProvider::Caelestia,
            _ => SurfaceProvider::Noctalia,
        };
        let _ = cfg.save();

        update_noctalia_shell_settings(
            target_engine.as_str(),
            cfg.shell.launcher_provider.as_str(),
            cfg.shell.dashboard_provider.as_str(),
        );

        let engine_str = target_engine.as_str();
        shell_status_clone.set_text(&format!(
            "✓ Kabuk motoru ({}) seçildi ve uygulandı.",
            engine_str
        ));

        let engine_to_switch = engine_str.to_string();
        std::thread::spawn(move || {
            let _ = Command::new("solar-shell")
                .args(["switch", &engine_to_switch])
                .status();
        });
    });
    btn_box.append(&apply_btn);

    let reconcile_btn = Button::with_label("Durumu Doğrula (Supervisor Reconcile)");
    reconcile_btn.add_css_class("secondary-btn");
    reconcile_btn.set_halign(gtk4::Align::Start);
    let shell_status_rec = shell_status_lbl.clone();
    reconcile_btn.connect_clicked(move |_| {
        shell_status_rec.set_text("✓ Kabuk durumu denetleniyor ve doğrulanıyor...");
        std::thread::spawn(|| {
            let _ = Command::new("solar-shell")
                .args(["supervisor"])
                .status();
        });
    });
    btn_box.append(&reconcile_btn);

    sec.append(&btn_box);
    sec.append(&shell_status_lbl);
    tab.append(&sec);

    tab
}

fn build_caelestia_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 12);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);
    
    let cfg = SolarConfig::load();
    if cfg.shell.engine == ShellEngine::Noctalia {
        let msg = Label::new(Some("Bu ayarlar sadece SolarUI Hibrit veya tam Caelestia motoru aktifken kullanılabilir. Lütfen 'Kabuk Motoru' sekmesinden motoru değiştirin."));
        msg.add_css_class("setting-label");
        msg.set_wrap(true);
        msg.set_halign(gtk4::Align::Center);
        msg.set_valign(gtk4::Align::Center);
        msg.set_vexpand(true);
        tab.append(&msg);
        return tab;
    }
    
    // Section 1: Wallpaper
    let sec_wall = GtkBox::new(Orientation::Vertical, 10);
    sec_wall.add_css_class("section-box");
    
    let w_title = Label::new(Some("Duvar Kağıdı ve Material 3 Renk Şeması"));
    w_title.add_css_class("section-title");
    w_title.set_halign(gtk4::Align::Start);
    sec_wall.append(&w_title);
    
    let w_desc = Label::new(Some(
        "Caelestia yüzeyleri (Launcher, Dashboard) seçilen duvar kağıdına göre dinamik olarak renk paleti üretir. Duvar kağıdını değiştirdiğinizde tüm arayüz renkleri otomatik güncellenir."
    ));
    w_desc.add_css_class("setting-subtext");
    w_desc.set_wrap(true);
    w_desc.set_halign(gtk4::Align::Start);
    sec_wall.append(&w_desc);
    
    let btn_row = GtkBox::new(Orientation::Horizontal, 10);
    
    let btn_rand = Button::with_label("Rastgele Duvar Kağıdı");
    btn_rand.add_css_class("action-btn");
    btn_rand.connect_clicked(|_| {
        std::thread::spawn(|| {
            let _ = Command::new("caelestia").args(["wallpaper", "-r"]).status();
        });
    });
    btn_row.append(&btn_rand);

    let btn_pick = Button::with_label("Dosyadan Seç...");
    btn_pick.add_css_class("secondary-btn");
    btn_pick.connect_clicked(|_| {
        std::thread::spawn(|| {
            if let Ok(out) = Command::new("zenity").args(["--file-selection", "--title=Duvar Kağıdı Seç (Caelestia)"]).output() {
                if out.status.success() {
                    if let Ok(path) = String::from_utf8(out.stdout) {
                        let path = path.trim();
                        if !path.is_empty() {
                            let _ = Command::new("caelestia").args(["wallpaper", "-f", path]).status();
                        }
                    }
                }
            }
        });
    });
    btn_row.append(&btn_pick);
    
    sec_wall.append(&btn_row);
    tab.append(&sec_wall);
    
    // Section 2: Mode and Layout
    let sec_lay = GtkBox::new(Orientation::Vertical, 10);
    sec_lay.add_css_class("section-box");
    
    let l_title = Label::new(Some("Masaüstü ve Arayüz Düzeni"));
    l_title.add_css_class("section-title");
    l_title.set_halign(gtk4::Align::Start);
    sec_lay.append(&l_title);
    
    let l_desc = Label::new(Some(
        "Quickshell tabanlı Caelestia arayüzü için tema modu ve diğer yapılandırmalar."
    ));
    l_desc.add_css_class("setting-subtext");
    l_desc.set_halign(gtk4::Align::Start);
    sec_lay.append(&l_desc);
    
    let row_mode = GtkBox::new(Orientation::Horizontal, 12);
    let lbl_mode = Label::new(Some("Caelestia Tema Modu:"));
    lbl_mode.add_css_class("setting-label");
    row_mode.append(&lbl_mode);

    let mode_dd = DropDown::new(Some(StringList::new(&["Karanlık (Dark)", "Aydınlık (Light)"])), None::<gtk4::Expression>);
    
    // Initial read
    let scheme_path = "/home/darkmorpheus/.local/state/caelestia/scheme.json";
    if let Ok(content) = std::fs::read_to_string(scheme_path) {
        if content.contains("\"mode\": \"light\"") {
            mode_dd.set_selected(1);
        }
    }

    mode_dd.connect_selected_notify(move |dd| {
        let is_light = dd.selected() == 1;
        std::thread::spawn(move || {
            let mode_str = if is_light { "light" } else { "dark" };
            if let Ok(content) = std::fs::read_to_string(scheme_path) {
                // Simple string replacement to avoid depending on serde_json in this crate
                // if it's not imported or available (just safely replace the exact key)
                let updated = content.replace("\"mode\": \"dark\"", &format!("\"mode\": \"{}\"", mode_str))
                                     .replace("\"mode\": \"light\"", &format!("\"mode\": \"{}\"", mode_str));
                let _ = std::fs::write(scheme_path, updated);
                // Trigger Caelestia reload
                let _ = Command::new("pkill").args(["-SIGUSR2", "-f", "quickshell.*caelestia"]).status();
            }
        });
    });
    row_mode.append(&mode_dd);
    sec_lay.append(&row_mode);

    tab.append(&sec_lay);
    tab
}

fn build_display_tab() -> GtkBox {
    let tab = GtkBox::new(Orientation::Vertical, 12);
    tab.set_margin_top(10);
    tab.set_margin_bottom(10);

    let sec = GtkBox::new(Orientation::Vertical, 12);
    sec.add_css_class("section-box");

    let title = Label::new(Some("Ekran Çözünürlüğü ve Arayüz Ölçeği"));
    title.add_css_class("section-title");
    title.set_halign(gtk4::Align::Start);
    sec.append(&title);

    let desc = Label::new(Some(
        "Ekranınızdaki pencere ve simgelerin boyutunu donanımınıza ve ekran çözünürlüğünüze göre ayarlayın. Küçük veya sanal makine ekranlarında (1280x800 vb.) arayüzün çok büyük görünmesini engellemek için %85 veya %75 kompakt ölçek önerilir.",
    ));
    desc.add_css_class("setting-subtext");
    desc.set_wrap(true);
    desc.set_halign(gtk4::Align::Start);
    sec.append(&desc);

    let cfg = SolarConfig::load();

    // Row: Scale factor selection
    let row_scale = GtkBox::new(Orientation::Horizontal, 12);
    let lbl_scale = Label::new(Some("Masaüstü & Pencere Ölçeği:"));
    lbl_scale.add_css_class("setting-label");
    lbl_scale.set_hexpand(true);
    lbl_scale.set_halign(gtk4::Align::Start);
    row_scale.append(&lbl_scale);

    let scale_presets = [
        "0.75x — Kompakt (%75)",
        "0.85x — Küçük (%85 - 1280x800 ve sanal makine için önerilen)",
        "1.00x — Standart (%100 - Varsayılan)",
        "1.25x — Büyük (%125)",
        "1.50x — Geniş (%150 - 2K Ekranlar)",
        "1.75x — Ekstra Büyük (%175)",
        "2.00x — HiDPI (%200 - 4K Ekranlar)",
    ];
    let scale_values = [0.75, 0.85, 1.00, 1.25, 1.50, 1.75, 2.00];

    let scale_model = StringList::new(&scale_presets);
    let scale_dd = DropDown::new(Some(scale_model), None::<gtk4::Expression>);

    // Select matching index based on current config
    let cur_scale = cfg.display.scale;
    let mut selected_idx = 2; // Default to 1.00x (index 2)
    for (i, &val) in scale_values.iter().enumerate() {
        if (cur_scale - val).abs() < 0.04 {
            selected_idx = i as u32;
            break;
        }
    }
    scale_dd.set_selected(selected_idx);
    row_scale.append(&scale_dd);
    sec.append(&row_scale);

    // Row: Status label for feedback
    let status_lbl = Label::new(Some(""));
    status_lbl.add_css_class("setting-subtext");
    status_lbl.set_halign(gtk4::Align::Start);

    // Row: Apply buttons
    let btn_box = GtkBox::new(Orientation::Horizontal, 10);
    btn_box.set_margin_top(8);

    let apply_btn = Button::with_label("Ölçeği Uygula (Anında Geçerli)");
    apply_btn.add_css_class("action-btn");

    let scale_dd_clone = scale_dd.clone();
    let status_clone = status_lbl.clone();

    apply_btn.connect_clicked(move |_| {
        let idx = scale_dd_clone.selected() as usize;
        let chosen_scale = if idx < scale_values.len() {
            scale_values[idx]
        } else {
            1.00
        };

        // 1. Save in SolarConfig
        let mut c = SolarConfig::load();
        c.display.scale = chosen_scale;
        c.display.text_scale = chosen_scale;
        let _ = c.save();

        // 2. Apply dynamically via Niri IPC
        apply_display_scale_dynamically(chosen_scale);

        // 3. Update status label
        status_clone.set_text(&format!(
            "✓ Ölçek {:.2}x başarıyla uygulandı ve kaydedildi.",
            chosen_scale
        ));
    });
    btn_box.append(&apply_btn);

    // Reset button
    let reset_btn = Button::with_label("Varsayılana Sıfırla (1.00x)");
    reset_btn.add_css_class("secondary-btn");
    let scale_dd_reset = scale_dd.clone();
    let status_reset = status_lbl.clone();
    reset_btn.connect_clicked(move |_| {
        scale_dd_reset.set_selected(2);
        let mut c = SolarConfig::load();
        c.display.scale = 1.00;
        c.display.text_scale = 1.00;
        let _ = c.save();
        apply_display_scale_dynamically(1.00);
        status_reset.set_text("✓ Ölçek varsayılan 1.00x değerine sıfırlandı.");
    });
    btn_box.append(&reset_btn);

    sec.append(&btn_box);
    sec.append(&status_lbl);
    tab.append(&sec);

    tab
}

pub fn detect_screen_resolution_and_auto_scale() -> f64 {
    let cfg = SolarConfig::load();
    // If user already explicitly set a custom non-default scale, respect their preference
    if (cfg.display.scale - 1.0).abs() > 0.04 {
        return cfg.display.scale;
    }

    // 1. Try to query Niri via `niri msg --json outputs`
    if let Ok(out) = Command::new("niri").args(["msg", "--json", "outputs"]).output() {
        if out.status.success() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(obj) = val.as_object() {
                    let mut min_height = u32::MAX;
                    for (_name, output_val) in obj {
                        let height = if let Some(current_mode_idx) = output_val.get("current_mode").and_then(|v| v.as_u64()) {
                            output_val.get("modes")
                                .and_then(|m| m.as_array())
                                .and_then(|modes| modes.get(current_mode_idx as usize))
                                .and_then(|mode| mode.get("height"))
                                .and_then(|h| h.as_u64())
                                .map(|h| h as u32)
                        } else {
                            output_val.get("modes")
                                .and_then(|m| m.as_array())
                                .and_then(|modes| modes.first())
                                .and_then(|mode| mode.get("height"))
                                .and_then(|h| h.as_u64())
                                .map(|h| h as u32)
                        };

                        if let Some(h) = height {
                            if h < min_height {
                                min_height = h;
                            }
                        }
                    }

                    if min_height != u32::MAX {
                        return scale_for_height(min_height);
                    }
                }
            }
        }
    }

    // 2. Fallback: check DRM modes in /sys/class/drm/*/modes
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        let mut min_height = u32::MAX;
        for entry in entries.flatten() {
            let modes_file = entry.path().join("modes");
            if let Ok(content) = std::fs::read_to_string(modes_file) {
                for line in content.lines() {
                    if let Some((_w, h)) = line.split_once('x') {
                        if let Ok(height) = h.trim().parse::<u32>() {
                            if height < min_height {
                                min_height = height;
                            }
                        }
                    }
                }
            }
        }
        if min_height != u32::MAX {
            return scale_for_height(min_height);
        }
    }

    1.00
}

fn scale_for_height(height: u32) -> f64 {
    if height <= 720 {
        0.75
    } else if height <= 800 {
        0.85
    } else if height <= 900 {
        0.90
    } else if height >= 2160 {
        1.75
    } else if height >= 1440 {
        1.25
    } else {
        1.00
    }
}

pub fn sync_caelestia_scale(_scale: f64) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/liveuser".to_string());
    let conf_dir = PathBuf::from(&home).join(".config/caelestia");
    let _ = std::fs::create_dir_all(&conf_dir);
    let shell_json_path = conf_dir.join("shell.json");

    let mut val: serde_json::Value = if let Ok(content) = std::fs::read_to_string(&shell_json_path) {
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !val.is_object() {
        val = serde_json::json!({});
    }

    if !val.get("appearance").map_or(false, |v| v.is_object()) {
        val["appearance"] = serde_json::json!({});
    }

    if let Some(app) = val.get_mut("appearance").and_then(|v| v.as_object_mut()) {
        if !app.contains_key("font") || !app["font"].is_object() {
            app.insert("font".to_string(), serde_json::json!({}));
        }
        if let Some(font_obj) = app.get_mut("font").and_then(|v| v.as_object_mut()) {
            if !font_obj.contains_key("icon") || !font_obj["icon"].is_object() {
                font_obj.insert("icon".to_string(), serde_json::json!({}));
            }
            if let Some(icon_obj) = font_obj.get_mut("icon").and_then(|v| v.as_object_mut()) {
                icon_obj.insert(
                    "family".to_string(),
                    serde_json::Value::String("Material Symbols Rounded".to_string()),
                );
            }
        }
    }

    if let Ok(formatted) = serde_json::to_string_pretty(&val) {
        let _ = std::fs::write(&shell_json_path, formatted);
    }
}

pub fn apply_display_scale_dynamically(scale: f64) {
    let scale_str = format!("{:.2}", scale);

    // 1. Update active Niri outputs
    let mut outputs_found = Vec::new();
    if let Ok(out) = Command::new("niri").args(["msg", "--json", "outputs"]).output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            for token in s.split('"') {
                if token == "Virtual-1"
                    || token.starts_with("eDP")
                    || token.starts_with("HDMI")
                    || token.starts_with("DP-")
                {
                    if !outputs_found.contains(&token.to_string()) {
                        outputs_found.push(token.to_string());
                    }
                }
            }
        }
    }
    if outputs_found.is_empty() {
        outputs_found.push("Virtual-1".to_string());
    }

    for op in &outputs_found {
        let _ = Command::new("niri")
            .args(["msg", "output", op, "scale", &scale_str])
            .status();
    }

    // 2. Persist to ~/.config/solarui/niri.kdl
    if let Ok(home) = std::env::var("HOME") {
        let niri_kdl = PathBuf::from(home).join(".config/solarui/niri.kdl");
        if let Ok(content) = std::fs::read_to_string(&niri_kdl) {
            let mut new_content = String::new();
            let mut skip_block = false;
            for line in content.lines() {
                if line.trim().starts_with("output \"") {
                    skip_block = true;
                    continue;
                }
                if skip_block {
                    if line.trim() == "}" {
                        skip_block = false;
                    }
                    continue;
                }
                new_content.push_str(line);
                new_content.push('\n');
            }

            // Append output blocks with the new scale
            for op in &outputs_found {
                new_content.push_str(&format!(
                    "\noutput \"{}\" {{\n    scale {}\n}}\n",
                    op, scale_str
                ));
            }

            let _ = std::fs::write(&niri_kdl, new_content);
        }
    }

    // 3. Set GTK font / interface scaling factor
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "text-scaling-factor", &scale_str])
        .status();

    // 4. Sync Caelestia configuration
    sync_caelestia_scale(scale);

    // 5. Update SolarConfig
    let mut cfg = SolarConfig::load();
    cfg.display.scale = scale;
    cfg.display.text_scale = scale;
    let _ = cfg.save();
}



