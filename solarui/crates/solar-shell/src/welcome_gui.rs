use gdk_pixbuf::PixbufLoader;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, CssProvider, Grid, Image, Label,
    Orientation, Window,
};
use solar_common::SolarConfig;

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

pub fn launch_welcome_window() {
    glib::set_prgname(Some("solar-welcome"));
    glib::set_application_name("SolarUI Welcome");

    if let Err(err) = gtk4::init() {
        eprintln!("Failed to initialize GTK4: {}", err);
        return;
    }

    let main_loop = glib::MainLoop::new(None, false);
    build_welcome_ui(main_loop.clone());
    main_loop.run();
}

fn build_welcome_ui(main_loop: glib::MainLoop) {
    let provider = CssProvider::new();
    let css = r#"
        window {
            background-color: transparent;
        }

        .welcome-card {
            background-color: alpha(#16191c, 0.94);
            border: 1px solid alpha(#0095c7, 0.55);
            border-radius: 18px;
            padding: 24px;
            box-shadow: 0 12px 36px rgba(0, 0, 0, 0.7);
        }

        .welcome-title {
            color: #ffffff;
            font-size: 24px;
            font-weight: 700;
            margin-top: 6px;
        }

        .welcome-subtitle {
            color: #38c8ff;
            font-size: 13px;
            font-weight: 500;
            margin-bottom: 16px;
        }

        .shortcut-grid {
            background-color: alpha(#23272a, 0.65);
            border: 1px solid alpha(#525c62, 0.35);
            border-radius: 12px;
            padding: 14px 18px;
            margin-bottom: 18px;
        }

        .keycap {
            background: linear-gradient(180deg, #32383e, #23272b);
            color: #c4e7ff;
            border: 1px solid alpha(#616c72, 0.6);
            border-radius: 6px;
            padding: 3px 8px;
            font-size: 12px;
            font-weight: 600;
            box-shadow: 0 2px 4px rgba(0, 0, 0, 0.35);
        }

        .shortcut-desc {
            color: #e1e3e5;
            font-size: 13px;
            margin-left: 12px;
        }

        .start-btn {
            background: linear-gradient(135deg, #007ba4, #00b4e6);
            color: #ffffff;
            font-size: 14px;
            font-weight: 600;
            border-radius: 8px;
            padding: 8px 24px;
            border: none;
            box-shadow: 0 4px 12px rgba(0, 149, 199, 0.4);
            transition: all 120ms ease;
        }

        .start-btn:hover {
            background: linear-gradient(135deg, #008ebf, #1cc6f9);
        }

        .install-btn {
            background: linear-gradient(135deg, #2e7d32, #4caf50);
            color: #ffffff;
            font-size: 14px;
            font-weight: 600;
            border-radius: 8px;
            padding: 8px 20px;
            border: none;
            box-shadow: 0 4px 12px rgba(46, 125, 50, 0.4);
            transition: all 120ms ease;
        }

        .install-btn:hover {
            background: linear-gradient(135deg, #388e3c, #66bb6a);
        }

        .startup-checkbox {
            color: #8a9297;
            font-size: 12px;
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
        .title("SolarUI Desktop - Karşılama ve Kısayollar")
        .default_width(640)
        .default_height(480)
        .resizable(false)
        .build();

    let loop_quit = main_loop.clone();
    window.connect_close_request(move |_| {
        loop_quit.quit();
        glib::Propagation::Proceed
    });

    let root_box = GtkBox::new(Orientation::Vertical, 0);
    root_box.add_css_class("welcome-card");

    // Logo & Header
    let logo = create_logo_image(64);
    logo.set_halign(gtk4::Align::Center);
    root_box.append(&logo);

    let title = Label::new(Some("SolarUI Masaüstüne Hoş Geldiniz"));
    title.add_css_class("welcome-title");
    title.set_halign(gtk4::Align::Center);
    root_box.append(&title);

    let subtitle = Label::new(Some("SolarUI • Masaüstünüz, sizin düzeniniz"));
    subtitle.add_css_class("welcome-subtitle");
    subtitle.set_halign(gtk4::Align::Center);
    root_box.append(&subtitle);

    // Grid of Essential Shortcuts
    let grid = Grid::new();
    grid.add_css_class("shortcut-grid");
    grid.set_column_spacing(24);
    grid.set_row_spacing(10);

    let shortcuts = [
        ("Mod + Space", "SolarUI Uygulama Menüsü"),
        ("Mod + T / F", "KDE Serbest Kayan Pencere (Floating Window)"),
        ("Mod + Sol Tık Sürükle", "Pencereyi Ekranda İstediğin Yere Taşı"),
        ("Mod + Sağ Tık Sürükle", "Pencereyi İstenilen Boyuta Getir"),
        ("Mod + Y / Ctrl+Aşağı", "Pencereyi Arka Plana At (Minimize)"),
        ("Alt + F4 / Mod + Q", "Pencereyi Kapat"),
        ("Mod + S", "Kontrol Merkezi & Ses / Ağ Paneli"),
        ("Mod + Shift + S", "SolarUI Masaüstü ve Görev Çubuğu Ayarları"),
        ("Mod + Enter", "Uçbirim (Terminal: Alacritty)"),
        ("Mod + Shift + Q", "Oturumu Kapat / Güç Menüsü"),
    ];

    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        let row = i as i32;
        let key_label = Label::new(Some(key));
        key_label.add_css_class("keycap");
        key_label.set_halign(gtk4::Align::Start);

        let desc_label = Label::new(Some(desc));
        desc_label.add_css_class("shortcut-desc");
        desc_label.set_halign(gtk4::Align::Start);

        grid.attach(&key_label, 0, row, 1, 1);
        grid.attach(&desc_label, 1, row, 1, 1);
    }
    root_box.append(&grid);

    // Bottom Action Bar
    let bottom_box = GtkBox::new(Orientation::Horizontal, 12);
    bottom_box.set_halign(gtk4::Align::Fill);

    let cfg = SolarConfig::load();
    let check = CheckButton::with_label("Açılışta bu kısayol ekranını göster");
    check.add_css_class("startup-checkbox");
    check.set_active(cfg.shortcuts_hud.show_at_startup);
    check.set_hexpand(true);
    check.connect_toggled(move |c| {
        let mut updated = SolarConfig::load();
        updated.shortcuts_hud.show_at_startup = c.is_active();
        let _ = updated.save();
    });
    bottom_box.append(&check);

    let is_live_installer = std::path::Path::new("/usr/bin/liveinst").exists()
        || std::path::Path::new("/run/initramfs/live").exists()
        || std::path::Path::new("/dev/mapper/live-base").exists();

    if is_live_installer {
        let install_btn = Button::with_label("💿 Sabit Diske Kur (Anaconda)");
        install_btn.add_css_class("install-btn");
        let win_weak_inst = window.downgrade();
        let loop_inst = main_loop.clone();
        install_btn.connect_clicked(move |_| {
            if let Some(win) = win_weak_inst.upgrade() {
                win.close();
            }
            let _ = std::process::Command::new("niri")
                .args(["msg", "action", "spawn", "--", "/usr/bin/liveinst"])
                .spawn();
            loop_inst.quit();
        });
        bottom_box.append(&install_btn);
    }

    let start_btn = Button::with_label("Masaüstüne Başla");
    start_btn.add_css_class("start-btn");
    let win_weak = window.downgrade();
    let loop_clone = main_loop.clone();
    start_btn.connect_clicked(move |_| {
        if let Some(win) = win_weak.upgrade() {
            win.close();
        }
        loop_clone.quit();
    });
    bottom_box.append(&start_btn);

    root_box.append(&bottom_box);

    window.set_child(Some(&root_box));
    window.present();
}
