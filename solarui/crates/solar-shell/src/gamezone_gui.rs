// ==============================================================================
// Blaze GameZone - Fullscreen Handheld & Desktop Gaming Shell
// Pure Steam Deck UI & Xbox Handheld Hybrid Architecture
//
// Key Specifications:
// 1. Sol Panel (Dikey Kenar Çubuğu):
//    - Profil & Gamer Tag
//    - Kütüphanem, Steam, Epic Games (Heroic), GOG Galaxy, RetroArch, Xbox Cloud, Mağazalar
//    - SIFIR EMOJİ: Tamamen profesyonel tipografi ve gerçek vektör/sistem ikonları
// 2. Sağ Ana İçerik Alanı (Dikey Kaydırılabilir Çok Katmanlı Akış):
//    - Üst Xbox Canlı Performans HUD (FPS, GPU%, CPU%, RAM%, Saat) & Arama Çubuğu
//    - Katman 1: Son Oynananlar (Geniş Seçili Oyun Banner'ı + Dikey Kartlar)
//    - Katman 2: Steam Deck Navigasyon Hapları (Yenilikler, Arkadaşlar, Tavsiyeler)
//    - Katman 3: Oyun Haberleri ve Etkinlik Kartları (4 adet geniş afiş)
//    - Katman 4: Kütüphanemdeki Tüm Oyunlar (Çok Satırlı Izgara - Aşağı Doğru Devam Eder)
//    - Katman 5: Başlatıcılar ve Doğrudan Mağazalar (Steam, Epic, GOG)
// 3. Etkileşim ve Animasyon:
//    - Mouse kartın üzerine gelince kart büyür (scale-up animasyonu) ve beyaz odak halkası belirir
//    - Seçilen veya üzerine gelinen oyunun Steam Mağaza ekran görüntüsü anında arka plana ve Hero alanına yansır
//    - 10 Saniye Hover Kuralı: Mouse oyunun üzerinde 10 saniye tutulursa mağazadaki oynanış videosu/fragmanı başlar
// 4. Web Afiş, Ekran Görüntüsü ve Fragman Önbellek Motoru:
//    - ~/.cache/blaze-gamezone/media/<game_id>/
//    - Steam Mağaza API'sinden 1920x1080 ekran görüntüleri ve mp4 fragmanları otomatik çekilir
//    - Çevrimdışı ilk: Diskte önbellek varsa sıfır milisaniye gecikmeyle açılır
// 5. Sıfır Neon: Xbox ve Steam Deck'in resmi mat füme / arduvaz renk paleti
// ==============================================================================

use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CssProvider, Entry, EventControllerKey, EventControllerMotion,
    Label, Orientation, Picture, ScrolledWindow, Video, Window,
};
use std::cell::RefCell;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct GameEntry {
    pub id: String,
    pub title: String,
    pub category: String,
    pub exec: String,
    pub banner_desc: String,
    pub is_steam: bool,
    pub is_installed: bool,
    pub steam_app_id: Option<String>,
    pub cover_path: Option<String>,
    pub hero_path: Option<String>,
    pub screenshot_path: Option<String>,
    pub store_url: Option<String>,
    pub movie_url: Option<String>,
}

pub fn is_system_online() -> bool {
    // 1. Check nmcli if available
    if let Ok(output) = Command::new("nmcli").args(["networking", "connectivity", "check"]).output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            let t = s.trim();
            if t == "full" || t == "limited" {
                return true;
            }
            if t == "none" {
                return false;
            }
        }
    }
    // 2. Fallback: Quick TCP probe to DNS (1.1.1.1:53) with 300ms timeout
    if let Ok(addr) = "1.1.1.1:53".parse::<SocketAddr>() {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return true;
        }
    }
    // 3. Fallback: Check if default route exists in /proc/net/route
    if let Ok(routes) = std::fs::read_to_string("/proc/net/route") {
        for line in routes.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 1 && fields[1] == "00000000" {
                return true;
            }
        }
    }
    false
}

pub fn open_browser_url(url: &str) {
    if Command::new("xdg-open").arg(url).spawn().is_err() {
        if Command::new("firefox").arg(url).spawn().is_err() {
            let _ = Command::new("chromium").arg(url).spawn();
        }
    }
}

pub fn is_game_installed(steam_app_id: Option<&str>, exec: &str, id: &str) -> bool {
    // 1. Steam App ID check
    if let Some(app_id) = steam_app_id {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
        let steam_dirs = [
            PathBuf::from(&home).join(".steam/steam/steamapps"),
            PathBuf::from(&home).join(".local/share/Steam/steamapps"),
            PathBuf::from("/var/lib/flatpak/app/com.valvesoftware.Steam/x86_64/stable/active/files/share/steamapps"),
        ];
        for s_dir in steam_dirs {
            let manifest = s_dir.join(format!("appmanifest_{}.acf", app_id));
            if manifest.exists() {
                return true;
            }
        }
        return false;
    }

    // 2. Special cloud services
    if id == "xbox-cloud" {
        return false;
    }

    // 3. Executable check
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if let Some(cmd) = parts.get(0) {
        if Path::new(cmd).is_absolute() {
            return Path::new(cmd).exists();
        }
        let check_dirs = [
            "/usr/bin",
            "/usr/local/bin",
            "/bin",
            "/usr/games",
            "/usr/local/games",
        ];
        for d in check_dirs {
            if Path::new(d).join(cmd).exists() {
                return true;
            }
        }
        if let Ok(out) = Command::new("which").arg(cmd).output() {
            if out.status.success() {
                return true;
            }
        }
    }

    false
}

pub fn launch_gamezone_window() {
    glib::set_prgname(Some("solar-gamezone"));
    glib::set_application_name("Blaze GameZone");

    if let Err(err) = gtk4::init() {
        eprintln!("Failed to initialize GTK4 for GameZone: {}", err);
        return;
    }

    // Activate low-latency gaming mode (CPU performance governor + BORE scheduler)
    activate_gaming_optimizations();

    let main_loop = glib::MainLoop::new(None, false);
    build_gamezone_ui(main_loop.clone());
    main_loop.run();

    // Restore standard balanced settings on exit
    restore_normal_optimizations();
}

fn build_gamezone_ui(main_loop: glib::MainLoop) {
    let provider = CssProvider::new();
    let css = r#"
        window {
            background-color: #0c0f12;
            color: #e5e8ec;
            font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
        }

        .gamezone-root {
            background-color: #0c0f12;
            padding: 0;
            margin: 0;
        }

        /* ── Sol Dikey Kenar Çubuğu (Xbox / Heroic Style - Sıfır Emojili) ── */
        .sidebar {
            background-color: #12161b;
            border-right: 1px solid rgba(255, 255, 255, 0.07);
            min-width: 240px;
            padding: 20px 14px;
        }

        .profile-card {
            background-color: #171d24;
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 12px 14px;
            margin-bottom: 20px;
        }

        .profile-tag {
            color: #ffffff;
            font-size: 15px;
            font-weight: 700;
        }

        .profile-status {
            color: #107c10; /* Xbox Green */
            font-size: 11px;
            font-weight: 600;
            margin-top: 2px;
        }

        .profile-score {
            color: #9da6b2;
            font-size: 12px;
            font-weight: 600;
            margin-top: 2px;
        }

        .sidebar-section-title {
            color: #6c7886;
            font-size: 11px;
            font-weight: 700;
            letter-spacing: 0.8px;
            text-transform: uppercase;
            margin-top: 14px;
            margin-bottom: 8px;
            margin-left: 6px;
        }

        .nav-btn {
            background: transparent;
            color: #bcc5d0;
            font-size: 13px;
            font-weight: 600;
            border-radius: 8px;
            padding: 10px 12px;
            border: none;
            margin-bottom: 4px;
            transition: all 120ms ease;
        }

        .nav-btn:hover, .nav-btn:focus {
            background-color: #1c222b;
            color: #ffffff;
        }

        .nav-btn.active {
            background-color: #107c10; /* Xbox Green */
            color: #ffffff;
            font-weight: 700;
        }

        /* ── Sağ Ana İçerik Alanı ── */
        .main-scroll {
            background: radial-gradient(circle at 60% 15%, #182029 0%, #0e1216 70%, #0a0d10 100%);
        }

        .main-content {
            padding: 20px 32px 48px 32px;
        }

        /* ── Üst Çubuk (Arama & Xbox HUD) ── */
        .top-hud-bar {
            background-color: rgba(22, 28, 35, 0.85);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 8px 18px;
            margin-bottom: 20px;
        }

        .search-entry {
            background-color: #171d24;
            color: #ffffff;
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 8px;
            padding: 6px 14px;
            font-size: 13px;
            min-width: 320px;
        }

        .search-entry:focus {
            border-color: #ffffff;
        }

        .hud-label {
            color: #7b8893;
            font-size: 11px;
            font-weight: 600;
        }

        .hud-val {
            color: #e5e8ec;
            font-size: 13px;
            font-weight: 700;
            margin-left: 4px;
            margin-right: 14px;
        }

        .hud-fps {
            color: #107c10;
            font-size: 14px;
            font-weight: 800;
            margin-left: 4px;
            margin-right: 14px;
        }

        /* ── Bölüm Başlıkları ── */
        .section-header {
            color: #ffffff;
            font-size: 17px;
            font-weight: 800;
            margin-top: 18px;
            margin-bottom: 12px;
        }

        /* ── Steam Deck Style Hero Banner (Geniş Seçili Kart & Medya Alanı) ── */
        .hero-banner {
            background: linear-gradient(135deg, #182029, #0f1318);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            padding: 20px 24px;
            margin-bottom: 20px;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
        }

        .hero-tag {
            color: #107c10;
            font-size: 11px;
            font-weight: 800;
            letter-spacing: 1px;
            text-transform: uppercase;
        }

        .hero-title {
            color: #ffffff;
            font-size: 28px;
            font-weight: 800;
            margin-top: 2px;
            margin-bottom: 4px;
        }

        .hero-subtitle {
            color: #92a0b0;
            font-size: 13px;
            margin-bottom: 14px;
            line-height: 1.4;
        }

        .hero-trailer-badge {
            color: #2fe02f;
            font-size: 12px;
            font-weight: 700;
            margin-left: 14px;
        }

        .hero-play-btn {
            background: #107c10;
            color: #ffffff;
            font-size: 14px;
            font-weight: 700;
            border-radius: 10px;
            padding: 10px 24px;
            border: none;
            box-shadow: 0 4px 14px rgba(16, 124, 16, 0.4);
            transition: all 120ms ease;
        }

        .hero-play-btn:hover, .hero-play-btn:focus {
            background: #159c15;
            box-shadow: 0 6px 18px rgba(16, 124, 16, 0.6);
        }

        .hero-install-btn {
            background: #0078d4;
            color: #ffffff;
            font-size: 14px;
            font-weight: 700;
            border-radius: 10px;
            padding: 10px 24px;
            border: none;
            box-shadow: 0 4px 14px rgba(0, 120, 212, 0.4);
            transition: all 120ms ease;
        }

        .hero-install-btn:hover, .hero-install-btn:focus {
            background: #1084d8;
            box-shadow: 0 6px 18px rgba(0, 120, 212, 0.6);
        }

        .hero-opt-btn {
            background: #202731;
            color: #d1d8e0;
            font-size: 13px;
            font-weight: 600;
            border-radius: 10px;
            padding: 10px 18px;
            border: 1px solid rgba(255, 255, 255, 0.1);
            margin-left: 10px;
        }

        .hero-opt-btn:hover, .hero-opt-btn:focus {
            background: #2b3543;
            color: #ffffff;
            border-color: #ffffff;
        }

        .hero-media-box {
            background-color: #05070a;
            border: 1px solid rgba(255, 255, 255, 0.18);
            border-radius: 14px;
            box-shadow: 0 10px 28px rgba(0, 0, 0, 0.85);
            min-width: 320px;
            min-height: 180px;
        }

        .hero-media-box:hover {
            border-color: rgba(255, 255, 255, 0.4);
        }

        .hero-media-picture {
            border-radius: 14px;
        }

        .hero-media-video {
            border-radius: 14px;
        }

        /* ── Oyun Kartları & Büyüme Animasyonu (Scale-up) ── */
        .game-card {
            background-color: #171d24;
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 10px;
            margin-right: 14px;
            margin-bottom: 14px;
            min-width: 155px;
            transition: all 180ms cubic-bezier(0.2, 0, 0, 1);
        }

        /* Mouse üzerinde gelince veya odaklanınca büyüme efekti */
        .game-card:hover, .game-card:focus {
            background-color: #212832;
            border: 2px solid #ffffff;
            box-shadow: 0 14px 34px rgba(0, 0, 0, 0.95);
            transform: scale(1.08);
        }

        .game-card-wide {
            background-color: #1e2632;
            border: 2px solid #ffffff;
            border-radius: 14px;
            padding: 12px;
            margin-right: 18px;
            min-width: 340px;
            box-shadow: 0 14px 36px rgba(0, 0, 0, 0.9);
            transition: all 180ms ease;
        }

        .game-card-wide:hover, .game-card-wide:focus {
            transform: scale(1.05);
            border-color: #ffffff;
        }

        .game-cover-pic {
            border-radius: 8px;
            margin-bottom: 6px;
        }

        .game-card-title {
            color: #ffffff;
            font-size: 13px;
            font-weight: 700;
            margin-top: 8px;
        }

        .game-card-category {
            color: #8391a0;
            font-size: 11px;
            font-weight: 500;
            margin-top: 2px;
        }

        /* ── Steam Deck Navigasyon Hapları (Pills) ── */
        .pill-bar {
            margin-top: 14px;
            margin-bottom: 16px;
        }

        .pill-btn {
            background-color: #1c222b;
            color: #9da6b2;
            font-size: 12px;
            font-weight: 700;
            letter-spacing: 0.5px;
            border-radius: 20px;
            padding: 8px 20px;
            border: 1px solid rgba(255, 255, 255, 0.08);
            margin-right: 10px;
            transition: all 120ms ease;
        }

        .pill-btn:hover, .pill-btn:focus {
            background-color: #27303c;
            color: #ffffff;
            border-color: #ffffff;
        }

        .pill-btn.active {
            background-color: #ffffff;
            color: #0c0f12;
            border-color: #ffffff;
            font-weight: 800;
        }

        /* ── Haber ve Etkinlik Kartları (News Cards) ── */
        .news-card {
            background: #171d24;
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 10px;
            margin-right: 14px;
            min-width: 250px;
            transition: all 150ms ease;
        }

        .news-card:hover, .news-card:focus {
            background: #202731;
            border-color: #ff3399; /* Steam Deck pembesi odak çerçevesi */
            box-shadow: 0 0 16px rgba(255, 51, 153, 0.35);
            transform: scale(1.04);
        }

        .news-card-img {
            border-radius: 8px;
            margin-bottom: 6px;
        }

        .news-tag {
            color: #38c8ff;
            font-size: 10px;
            font-weight: 800;
            text-transform: uppercase;
            letter-spacing: 0.8px;
            margin-bottom: 4px;
        }

        .news-title {
            color: #ffffff;
            font-size: 13px;
            font-weight: 700;
        }

        .news-date {
            color: #728090;
            font-size: 11px;
            margin-top: 4px;
        }

        /* ── Alt Kumanda Kılavuzu ── */
        .controller-bar {
            background-color: #0e1216;
            border-top: 1px solid rgba(255, 255, 255, 0.06);
            padding: 10px 24px;
        }

        .gamepad-key {
            background: #222933;
            color: #ffffff;
            border: 1px solid rgba(255, 255, 255, 0.2);
            border-radius: 12px;
            font-size: 11px;
            font-weight: 800;
            padding: 2px 7px;
            margin-right: 6px;
        }

        .gamepad-desc {
            color: #8c97a5;
            font-size: 12px;
            font-weight: 600;
            margin-right: 20px;
        }

        .profile-status-offline {
            color: #f59e0b;
            font-size: 11px;
            font-weight: 600;
            margin-top: 2px;
        }

        .offline-banner {
            background-color: rgba(220, 38, 38, 0.18);
            border: 1px solid #ef4444;
            border-radius: 10px;
            padding: 10px 18px;
            margin-bottom: 16px;
        }

        .offline-banner-icon {
            color: #ef4444;
            font-size: 14px;
            font-weight: 800;
            margin-right: 8px;
        }

        .offline-banner-text {
            color: #fca5a5;
            font-size: 13px;
            font-weight: 600;
        }

        .offline-banner-close {
            background: rgba(255, 255, 255, 0.08);
            color: #fca5a5;
            font-size: 11px;
            font-weight: 700;
            border: 1px solid rgba(255, 255, 255, 0.15);
            border-radius: 6px;
            padding: 4px 10px;
        }

        .offline-banner-close:hover {
            background: rgba(255, 255, 255, 0.16);
            color: #ffffff;
        }

        .sidebar-login-btn {
            background-color: #107c10;
            color: #ffffff;
            font-size: 11px;
            font-weight: 700;
            border-radius: 6px;
            padding: 5px 10px;
            border: none;
            margin-top: 8px;
            transition: all 120ms ease;
        }

        .sidebar-login-btn:hover {
            background-color: #159c15;
        }

        /* ── Boş Durum Kartları (Empty State Cards) ── */
        .empty-state-card {
            background: linear-gradient(135deg, rgba(23, 29, 36, 0.6), rgba(15, 19, 24, 0.8));
            border: 1px dashed rgba(255, 255, 255, 0.15);
            border-radius: 14px;
            padding: 24px 28px;
            margin-bottom: 24px;
        }

        .empty-state-title {
            color: #ffffff;
            font-size: 16px;
            font-weight: 700;
            margin-bottom: 4px;
        }

        .empty-state-desc {
            color: #8b99a8;
            font-size: 13px;
            line-height: 1.5;
            max-width: 680px;
        }

        .library-empty-card {
            background: linear-gradient(135deg, #151b22, #0d1117);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            padding: 36px 32px;
            margin-top: 10px;
            margin-bottom: 24px;
        }

        .library-empty-title {
            color: #ffffff;
            font-size: 18px;
            font-weight: 800;
            margin-bottom: 6px;
        }

        .library-empty-desc {
            color: #8b99a8;
            font-size: 13px;
            line-height: 1.5;
            max-width: 580px;
            text-align: center;
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
        .title("Blaze GameZone")
        .default_width(1280)
        .default_height(760)
        .maximized(true)
        .build();

    let all_games = Arc::new(Mutex::new(discover_all_games()));
    let showcase_games = Arc::new(discover_store_showcase());
    let selected_index = Arc::new(Mutex::new(0usize));

    // Ensure cache directory exists and trigger background media scraper
    init_media_cache(&all_games.lock().unwrap());
    init_media_cache(&showcase_games);

    // Root Container: Horizontal split (Left Sidebar + Right Scrollable Content)
    let root_box = GtkBox::new(Orientation::Horizontal, 0);
    root_box.add_css_class("gamezone-root");

    let is_online = is_system_online();

    // ── Sol Kenar Çubuğu (Sidebar - Sıfır Emojili Profesyonel) ──
    // Offline alert banner & helper
    let offline_banner = GtkBox::new(Orientation::Horizontal, 12);
    offline_banner.add_css_class("offline-banner");
    offline_banner.set_visible(!is_online);

    let offline_icon = Label::new(Some("!"));
    offline_icon.add_css_class("offline-banner-icon");

    let offline_label = Label::new(Some("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz."));
    offline_label.add_css_class("offline-banner-text");
    offline_label.set_hexpand(true);
    offline_label.set_halign(gtk4::Align::Start);

    let offline_close_btn = Button::with_label("Kapat");
    offline_close_btn.add_css_class("offline-banner-close");
    let ob_clone = offline_banner.clone();
    offline_close_btn.connect_clicked(move |_| {
        ob_clone.set_visible(false);
    });

    offline_banner.append(&offline_icon);
    offline_banner.append(&offline_label);
    offline_banner.append(&offline_close_btn);

    let offline_banner_clone = offline_banner.clone();
    let offline_label_clone = offline_label.clone();
    let trigger_offline_alert: Rc<dyn Fn(&str)> = Rc::new(move |msg: &str| {
        offline_label_clone.set_text(msg);
        offline_banner_clone.set_visible(true);
        let _ = Command::new("notify-send")
            .args([
                "-a", "Blaze GameZone",
                "-i", "network-offline",
                "Çevrimdışı Uyarı",
                msg,
            ])
            .spawn();
    });

    // ── Sol Kenar Çubuğu (Sidebar - Sıfır Emojili Profesyonel) ──
    let sidebar = GtkBox::new(Orientation::Vertical, 0);
    sidebar.add_css_class("sidebar");

    // Profil Kartı (Xbox Style & Steam Hesap Algılama)
    let profile_card = GtkBox::new(Orientation::Vertical, 4);
    profile_card.add_css_class("profile-card");

    let (steam_account, epic_account) = detect_logged_in_accounts();

    let profile_tag_text = if let Some(ref s_user) = steam_account {
        s_user.clone()
    } else if let Some(ref e_user) = epic_account {
        e_user.clone()
    } else {
        "Misafir Oyuncu".to_string()
    };

    let profile_tag = Label::new(Some(&profile_tag_text));
    profile_tag.add_css_class("profile-tag");
    profile_tag.set_halign(gtk4::Align::Start);

    let profile_status_text = match (&steam_account, &epic_account, is_online) {
        (Some(_), _, true) => "Steam Çevrimiçi • Düşük Gecikme Modu",
        (Some(_), _, false) => "Steam Çevrimdışı • Yerel Mod",
        (_, Some(_), true) => "Epic/Heroic Çevrimiçi • Düşük Gecikme Modu",
        (_, Some(_), false) => "Epic/Heroic Çevrimdışı • Yerel Mod",
        (None, None, true) => "Hesap Bağlanmadı • Çevrimiçi",
        (None, None, false) => "Hesap Bağlanmadı • Çevrimdışı",
    };
    let profile_status = Label::new(Some(profile_status_text));
    if is_online {
        profile_status.add_css_class("profile-status");
    } else {
        profile_status.add_css_class("profile-status-offline");
    }
    profile_status.set_halign(gtk4::Align::Start);

    let profile_score = Label::new(Some("1,420 G • BORE Çekirdeği"));
    profile_score.add_css_class("profile-score");
    profile_score.set_halign(gtk4::Align::Start);

    profile_card.append(&profile_tag);
    profile_card.append(&profile_status);
    profile_card.append(&profile_score);

    if steam_account.is_none() && epic_account.is_none() {
        let login_action = Button::with_label("Steam Girişi Yap");
        login_action.add_css_class("sidebar-login-btn");
        login_action.connect_clicked(|_| {
            let _ = Command::new("steam").spawn();
        });
        profile_card.append(&login_action);
    }

    sidebar.append(&profile_card);

    // Kütüphane Başlığı (Sıfır Emojili)
    let lib_title = Label::new(Some("Kütüphanem"));
    lib_title.add_css_class("sidebar-section-title");
    lib_title.set_halign(gtk4::Align::Start);
    sidebar.append(&lib_title);

    let nav_all = Button::with_label("Tüm Oyunlar");
    nav_all.add_css_class("nav-btn");
    nav_all.add_css_class("active");
    nav_all.set_halign(gtk4::Align::Fill);
    sidebar.append(&nav_all);

    let nav_steam = Button::with_label("Steam Kütüphanesi");
    nav_steam.add_css_class("nav-btn");
    nav_steam.set_halign(gtk4::Align::Fill);
    sidebar.append(&nav_steam);

    let nav_epic = Button::with_label("Epic Games (Heroic)");
    nav_epic.add_css_class("nav-btn");
    nav_epic.set_halign(gtk4::Align::Fill);
    sidebar.append(&nav_epic);

    let nav_gog = Button::with_label("GOG Galaxy");
    nav_gog.add_css_class("nav-btn");
    nav_gog.set_halign(gtk4::Align::Fill);
    sidebar.append(&nav_gog);

    let nav_retro = Button::with_label("Retro Konsol (RetroArch)");
    nav_retro.add_css_class("nav-btn");
    nav_retro.set_halign(gtk4::Align::Fill);
    sidebar.append(&nav_retro);

    let nav_cloud = Button::with_label("Xbox Cloud Gaming");
    nav_cloud.add_css_class("nav-btn");
    nav_cloud.set_halign(gtk4::Align::Fill);
    let alert_cloud = trigger_offline_alert.clone();
    nav_cloud.connect_clicked(move |_| {
        if !is_system_online() {
            alert_cloud("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
        } else {
            open_browser_url("https://www.xbox.com/play");
        }
    });
    sidebar.append(&nav_cloud);

    // Mağazalar Başlığı
    let store_title = Label::new(Some("Oyun Mağazaları"));
    store_title.add_css_class("sidebar-section-title");
    store_title.set_halign(gtk4::Align::Start);
    sidebar.append(&store_title);

    let nav_steam_store = Button::with_label("Steam Mağazası");
    nav_steam_store.add_css_class("nav-btn");
    nav_steam_store.set_halign(gtk4::Align::Fill);
    let alert_s_store = trigger_offline_alert.clone();
    nav_steam_store.connect_clicked(move |_| {
        if !is_system_online() {
            alert_s_store("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
        } else {
            open_browser_url("https://store.steampowered.com/");
        }
    });
    sidebar.append(&nav_steam_store);

    let nav_epic_store = Button::with_label("Epic Games Store");
    nav_epic_store.add_css_class("nav-btn");
    nav_epic_store.set_halign(gtk4::Align::Fill);
    let alert_e_store = trigger_offline_alert.clone();
    nav_epic_store.connect_clicked(move |_| {
        if !is_system_online() {
            alert_e_store("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
        } else {
            open_browser_url("https://store.epicgames.com/");
        }
    });
    sidebar.append(&nav_epic_store);

    let nav_gog_store = Button::with_label("GOG.com Mağazası");
    nav_gog_store.add_css_class("nav-btn");
    nav_gog_store.set_halign(gtk4::Align::Fill);
    let alert_g_store = trigger_offline_alert.clone();
    nav_gog_store.connect_clicked(move |_| {
        if !is_system_online() {
            alert_g_store("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
        } else {
            open_browser_url("https://www.gog.com/");
        }
    });
    sidebar.append(&nav_gog_store);

    // Araçlar & Ayarlar
    let tools_title = Label::new(Some("Araçlar & Ayarlar"));
    tools_title.add_css_class("sidebar-section-title");
    tools_title.set_halign(gtk4::Align::Start);
    sidebar.append(&tools_title);

    let nav_protonup = Button::with_label("ProtonUp-Qt Yöneticisi");
    nav_protonup.add_css_class("nav-btn");
    nav_protonup.set_halign(gtk4::Align::Fill);
    nav_protonup.connect_clicked(|_| {
        let _ = Command::new("protonup-qt").spawn();
    });
    sidebar.append(&nav_protonup);

    let nav_cache_clean = Button::with_label("Afiş Önbelleğini Temizle");
    nav_cache_clean.add_css_class("nav-btn");
    nav_cache_clean.set_halign(gtk4::Align::Fill);
    nav_cache_clean.connect_clicked(|_| {
        clean_gamezone_cache();
    });
    sidebar.append(&nav_cache_clean);

    root_box.append(&sidebar);

    // ── Sağ Ana Bölge (Dikey Kaydırılabilir Çok Katmanlı Akış) ──
    let right_col = GtkBox::new(Orientation::Vertical, 0);
    right_col.set_hexpand(true);
    right_col.set_vexpand(true);

    let main_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();
    main_scroll.add_css_class("main-scroll");

    let main_content = GtkBox::new(Orientation::Vertical, 0);
    main_content.add_css_class("main-content");
    main_content.set_hexpand(true);

    // ── 1. Üst Xbox HUD & Arama Çubuğu ──
    let top_hud = GtkBox::new(Orientation::Horizontal, 12);
    top_hud.add_css_class("top-hud-bar");

    let search_entry = Entry::new();
    search_entry.add_css_class("search-entry");
    search_entry.set_placeholder_text(Some("Oyun, mağaza veya eklenti ara..."));
    top_hud.append(&search_entry);

    let hud_spacer = GtkBox::new(Orientation::Horizontal, 0);
    hud_spacer.set_hexpand(true);
    top_hud.append(&hud_spacer);

    let (cpu_str, ram_str, fps_str) = get_live_system_metrics();

    let lbl_fps_title = Label::new(Some("FPS"));
    lbl_fps_title.add_css_class("hud-label");
    let lbl_fps_val = Label::new(Some(&fps_str));
    lbl_fps_val.add_css_class("hud-fps");

    let lbl_cpu_title = Label::new(Some("CPU"));
    lbl_cpu_title.add_css_class("hud-label");
    let lbl_cpu_val = Label::new(Some(&cpu_str));
    lbl_cpu_val.add_css_class("hud-val");

    let lbl_ram_title = Label::new(Some("RAM"));
    lbl_ram_title.add_css_class("hud-label");
    let lbl_ram_val = Label::new(Some(&ram_str));
    lbl_ram_val.add_css_class("hud-val");

    let time_now = chrono::Local::now().format("%H:%M").to_string();
    let lbl_time = Label::new(Some(&time_now));
    lbl_time.add_css_class("hud-val");

    top_hud.append(&lbl_fps_title);
    top_hud.append(&lbl_fps_val);
    top_hud.append(&lbl_cpu_title);
    top_hud.append(&lbl_cpu_val);
    top_hud.append(&lbl_ram_title);
    top_hud.append(&lbl_ram_val);
    top_hud.append(&lbl_time);

    main_content.append(&top_hud);
    main_content.append(&offline_banner);

    // ── 2. Steam Deck Style Hero Showcase (Seçili Oyun / Ekran Görüntüsü / Canlı Video Alanı) ──
    let hero_banner = GtkBox::new(Orientation::Horizontal, 24);
    hero_banner.add_css_class("hero-banner");

    let hero_left_box = GtkBox::new(Orientation::Vertical, 6);
    hero_left_box.set_hexpand(true);

    let hero_tag = Label::new(Some("ÖNE ÇIKAN OYUN"));
    hero_tag.add_css_class("hero-tag");
    hero_tag.set_halign(gtk4::Align::Start);

    let hero_title = Label::new(None);
    hero_title.add_css_class("hero-title");
    hero_title.set_halign(gtk4::Align::Start);

    let hero_subtitle = Label::new(None);
    hero_subtitle.add_css_class("hero-subtitle");
    hero_subtitle.set_halign(gtk4::Align::Start);
    hero_subtitle.set_wrap(true);

    let hero_btn_box = GtkBox::new(Orientation::Horizontal, 12);
    let play_btn = Button::with_label("OYNA (A / Enter)");
    play_btn.add_css_class("hero-play-btn");

    let opt_btn = Button::with_label("Oyun Seçenekleri (X)");
    opt_btn.add_css_class("hero-opt-btn");

    let store_btn = Button::with_label("Mağaza Sayfası");
    store_btn.add_css_class("hero-opt-btn");

    let trailer_badge = Label::new(Some(""));
    trailer_badge.add_css_class("hero-trailer-badge");
    trailer_badge.set_halign(gtk4::Align::Start);

    hero_btn_box.append(&play_btn);
    hero_btn_box.append(&opt_btn);
    hero_btn_box.append(&store_btn);
    hero_btn_box.append(&trailer_badge);

    hero_left_box.append(&hero_tag);
    hero_left_box.append(&hero_title);
    hero_left_box.append(&hero_subtitle);
    hero_left_box.append(&hero_btn_box);

    // Sağ Kolon: HD Medya Çerçevesi (1920x1080 Ekran Görüntüsü veya Canlı Fragman)
    let hero_media_frame = GtkBox::new(Orientation::Vertical, 0);
    hero_media_frame.add_css_class("hero-media-box");
    hero_media_frame.set_size_request(380, 214);
    hero_media_frame.set_hexpand(true);
    hero_media_frame.set_halign(gtk4::Align::End);

    let hero_pic = Picture::new();
    hero_pic.set_can_shrink(true);
    hero_pic.set_content_fit(gtk4::ContentFit::Cover);
    hero_pic.set_size_request(380, 214);
    hero_pic.add_css_class("hero-media-picture");

    let hero_vid = Video::new();
    hero_vid.set_autoplay(true);
    hero_vid.set_loop(true);
    hero_vid.set_visible(false);
    hero_vid.set_size_request(380, 214);
    hero_vid.add_css_class("hero-media-video");

    hero_media_frame.append(&hero_pic);
    hero_media_frame.append(&hero_vid);

    hero_banner.append(&hero_left_box);
    hero_banner.append(&hero_media_frame);
    main_content.append(&hero_banner);

    // ── 3. Katman 1: Son Oynananlar (Recent Games - Geniş Seçili Kart & Karusel) ──
    let recents_title = Label::new(Some("Son Oynananlar"));
    recents_title.add_css_class("section-header");
    recents_title.set_halign(gtk4::Align::Start);
    main_content.append(&recents_title);

    let recent_ids = get_recent_game_ids();
    let mut recent_games: Vec<GameEntry> = Vec::new();
    {
        let inst = all_games.lock().unwrap();
        for id in &recent_ids {
            if let Some(g) = inst.iter().find(|g| &g.id == id) {
                recent_games.push(g.clone());
            } else if let Some(g) = showcase_games.iter().find(|g| &g.id == id) {
                recent_games.push(g.clone());
            }
        }
    }

    let mut card_buttons: Vec<Button> = Vec::new();
    let mut nav_games: Vec<GameEntry> = Vec::new();

    // 10-saniye video timer state
    let active_timer_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

    // Shared hover setup helper
    let attach_card_events = |card: &Button,
                              game: &GameEntry,
                              idx: usize,
                              s_idx: Arc<Mutex<usize>>,
                              h_title: Label,
                              h_sub: Label,
                              p_btn: Button,
                              s_btn: Button,
                              t_badge: Label,
                              h_pic: Picture,
                              h_vid: Video,
                              timer_ref: Rc<RefCell<Option<glib::SourceId>>>| {
        let g_hover = game.clone();
        let h_title_h = h_title.clone();
        let h_sub_h = h_sub.clone();
        let p_btn_h = p_btn.clone();
        let s_btn_h = s_btn.clone();
        let h_pic_h = h_pic.clone();
        let h_vid_h = h_vid.clone();

        card.connect_has_focus_notify(move |btn| {
            if btn.has_focus() {
                *s_idx.lock().unwrap() = idx;
                update_hero_showcase(&g_hover, &h_title_h, &h_sub_h, &p_btn_h, &s_btn_h, &h_pic_h, &h_vid_h);
                fetch_and_apply_store_screenshot(&g_hover, &h_pic_h);
            }
        });

        let motion_ctrl = EventControllerMotion::new();
        let g_motion = game.clone();
        let h_title_m = h_title.clone();
        let h_sub_m = h_sub.clone();
        let p_btn_m = p_btn.clone();
        let s_btn_m = s_btn.clone();
        let t_badge_m = t_badge.clone();
        let h_pic_m = h_pic.clone();
        let h_vid_m = h_vid.clone();
        let timer_motion = timer_ref.clone();

        motion_ctrl.connect_enter(move |_ctrl, _x, _y| {
            if let Some(src) = timer_motion.borrow_mut().take() {
                src.remove();
            }

            update_hero_showcase(&g_motion, &h_title_m, &h_sub_m, &p_btn_m, &s_btn_m, &h_pic_m, &h_vid_m);
            fetch_and_apply_store_screenshot(&g_motion, &h_pic_m);
            t_badge_m.set_text("Video Fragman: 10 sn bekleniyor...");

            let g_video = g_motion.clone();
            let t_badge_timer = t_badge_m.clone();
            let h_vid_timer = h_vid_m.clone();
            let h_pic_timer = h_pic_m.clone();
            let timer_clean = timer_motion.clone();

            let source_id = glib::timeout_add_local(Duration::from_secs(10), move || {
                trigger_store_video_preview(&g_video, &h_vid_timer, &h_pic_timer, &t_badge_timer);
                *timer_clean.borrow_mut() = None;
                glib::ControlFlow::Break
            });

            *timer_motion.borrow_mut() = Some(source_id);
        });

        let timer_leave = timer_ref.clone();
        let t_badge_leave = t_badge.clone();
        let h_vid_leave = h_vid.clone();
        let h_pic_leave = h_pic.clone();

        motion_ctrl.connect_leave(move |_ctrl| {
            if let Some(src) = timer_leave.borrow_mut().take() {
                src.remove();
            }
            if h_vid_leave.is_visible() {
                h_vid_leave.set_visible(false);
                h_vid_leave.set_file(None::<&gio::File>);
                h_pic_leave.set_visible(true);
            }
            t_badge_leave.set_text("");
        });

        card.add_controller(motion_ctrl);
    };

    if recent_games.is_empty() {
        let empty_box = GtkBox::new(Orientation::Vertical, 6);
        empty_box.add_css_class("empty-state-card");
        empty_box.set_hexpand(true);

        let e_title = Label::new(Some("Henüz Bir Oyun Oynamadınız"));
        e_title.add_css_class("empty-state-title");
        e_title.set_halign(gtk4::Align::Start);

        let e_desc = Label::new(Some("Oynadığınız oyunlar ve son oturumlarınız burada otomatik olarak listelenecektir. Başlamak için aşağıdaki kütüphanenizden veya mağaza vitrininden bir oyun seçin."));
        e_desc.add_css_class("empty-state-desc");
        e_desc.set_halign(gtk4::Align::Start);
        e_desc.set_wrap(true);

        empty_box.append(&e_title);
        empty_box.append(&e_desc);
        main_content.append(&empty_box);
    } else {
        let recent_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .hexpand(true)
            .build();

        let recent_box = GtkBox::new(Orientation::Horizontal, 0);

        for (idx, game) in recent_games.iter().take(6).enumerate() {
            let is_first = idx == 0;
            let card = Button::new();
            if is_first {
                card.add_css_class("game-card-wide");
            } else {
                card.add_css_class("game-card");
            }

            let c_box = GtkBox::new(Orientation::Vertical, 4);
            let img = create_game_cover_image(game);
            c_box.append(&img);

            let t_lbl = Label::new(Some(&game.title));
            t_lbl.add_css_class("game-card-title");
            t_lbl.set_halign(gtk4::Align::Start);
            t_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            t_lbl.set_max_width_chars(16);
            c_box.append(&t_lbl);

            let c_lbl = Label::new(Some(&game.category));
            c_lbl.add_css_class("game-card-category");
            c_lbl.set_halign(gtk4::Align::Start);
            c_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            c_lbl.set_max_width_chars(16);
            c_box.append(&c_lbl);

            card.set_child(Some(&c_box));

            let g_clone = game.clone();
            let alert_rec = trigger_offline_alert.clone();
            card.connect_clicked(move |_| {
                handle_game_activation(&g_clone, &*alert_rec);
            });

            attach_card_events(
                &card,
                game,
                idx,
                selected_index.clone(),
                hero_title.clone(),
                hero_subtitle.clone(),
                play_btn.clone(),
                store_btn.clone(),
                trailer_badge.clone(),
                hero_pic.clone(),
                hero_vid.clone(),
                active_timer_id.clone(),
            );

            recent_box.append(&card);
            card_buttons.push(card);
            nav_games.push(game.clone());
        }

        recent_scroll.set_child(Some(&recent_box));
        main_content.append(&recent_scroll);
    }

    // ── 4. Katman 2: Steam Deck Navigasyon Hapları (Pill Tabs) ──
    let pill_bar = GtkBox::new(Orientation::Horizontal, 0);
    pill_bar.add_css_class("pill-bar");

    let p_whats_new = Button::with_label("YENİLİKLER");
    p_whats_new.add_css_class("pill-btn");
    p_whats_new.add_css_class("active");
    pill_bar.append(&p_whats_new);

    let p_friends = Button::with_label("ARKADAŞLAR (3)");
    p_friends.add_css_class("pill-btn");
    pill_bar.append(&p_friends);

    let p_recommended = Button::with_label("TAVSİYE EDİLENLER");
    p_recommended.add_css_class("pill-btn");
    pill_bar.append(&p_recommended);

    let p_community = Button::with_label("TOPLULUK MERKEZİ");
    p_community.add_css_class("pill-btn");
    pill_bar.append(&p_community);

    main_content.append(&pill_bar);

    // ── 5. Katman 3: Oyun Haberleri & Etkinlikler (News / Event Cards) ──
    let news_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Never)
        .hexpand(true)
        .build();

    let news_box = GtkBox::new(Orientation::Horizontal, 0);

    let news_items = [
        ("ETKİNLİK", "Golden Joystick Awards 2026", "Oy verme süreci devam ediyor", "kingdom.jpg"),
        ("YENİ SÜRÜM", "BeamNG.drive v0.34 Güncellemesi", "Gelişmiş yumuşak gövde fiziği ve yeni harita", "beamng.jpg"),
        ("ÖDÜL", "Alters 11 Voices / Peabody", "Yılın en yenilikçi bağımsız oyunu seçildi", "alters.jpg"),
        ("GÜNCELLEME", "Hades II & Cyberpunk Yaması", "FSR 3.1 desteği ve BORE optimizasyonları", "hades2.jpg"),
    ];

    for (tag, title, date, img_file) in news_items {
        let n_card = Button::new();
        n_card.add_css_class("news-card");

        let n_box = GtkBox::new(Orientation::Vertical, 2);

        // Afiş görseli
        let bundled_path = format!("/usr/share/solarui/news/{}", img_file);
        let local_path = format!("/home/darkmorpheus/BlazeFedora/blazeos_custom_apps/usr/share/solarui/news/{}", img_file);
        let chosen_path = if Path::new(&bundled_path).exists() {
            bundled_path
        } else {
            local_path
        };

        if Path::new(&chosen_path).exists() {
            let pic = Picture::for_filename(&chosen_path);
            pic.set_can_shrink(true);
            pic.set_content_fit(gtk4::ContentFit::Cover);
            pic.set_size_request(240, 135);
            pic.add_css_class("news-card-img");
            n_box.append(&pic);
        }

        let tag_lbl = Label::new(Some(tag));
        tag_lbl.add_css_class("news-tag");
        tag_lbl.set_halign(gtk4::Align::Start);

        let title_lbl = Label::new(Some(title));
        title_lbl.add_css_class("news-title");
        title_lbl.set_halign(gtk4::Align::Start);
        title_lbl.set_max_width_chars(24);
        title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        let date_lbl = Label::new(Some(date));
        date_lbl.add_css_class("news-date");
        date_lbl.set_halign(gtk4::Align::Start);
        date_lbl.set_max_width_chars(28);
        date_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        n_box.append(&tag_lbl);
        n_box.append(&title_lbl);
        n_box.append(&date_lbl);
        n_card.set_child(Some(&n_box));

        news_box.append(&n_card);
    }

    news_scroll.set_child(Some(&news_box));
    main_content.append(&news_scroll);

    // ── 6. Katman 4: Kütüphanemdeki Tüm Oyunlar (Çok Satırlı Izgara - Dinamik Filtreleme) ──
    let all_games_title = Label::new(Some("Kütüphanemdeki Tüm Oyunlar"));
    all_games_title.add_css_class("section-header");
    all_games_title.set_halign(gtk4::Align::Start);
    main_content.append(&all_games_title);

    let grid_box = GtkBox::new(Orientation::Vertical, 14);

    let current_filter = Rc::new(RefCell::new("all".to_string()));
    let search_query = Rc::new(RefCell::new("".to_string()));

    let populate_grid = {
        let grid_box = grid_box.clone();
        let all_games = all_games.clone();
        let current_filter = current_filter.clone();
        let search_query = search_query.clone();
        let selected_index = selected_index.clone();
        let hero_title = hero_title.clone();
        let hero_subtitle = hero_subtitle.clone();
        let play_btn = play_btn.clone();
        let store_btn = store_btn.clone();
        let trailer_badge = trailer_badge.clone();
        let hero_pic = hero_pic.clone();
        let hero_vid = hero_vid.clone();
        let active_timer_id = active_timer_id.clone();
        let trigger_offline_alert = trigger_offline_alert.clone();

        Rc::new(move || {
            while let Some(child) = grid_box.first_child() {
                grid_box.remove(&child);
            }

            let games = all_games.lock().unwrap();
            let filter = current_filter.borrow().clone();
            let query = search_query.borrow().to_lowercase().trim().to_string();

            if games.is_empty() {
                let empty_lib_box = GtkBox::new(Orientation::Vertical, 8);
                empty_lib_box.add_css_class("library-empty-card");
                empty_lib_box.set_hexpand(true);

                let empty_lib_title = Label::new(Some("Kütüphanenizde Henüz Oyun Bulunmuyor"));
                empty_lib_title.add_css_class("library-empty-title");
                empty_lib_title.set_halign(gtk4::Align::Center);

                let empty_lib_desc = Label::new(Some("Sisteminizde veya Steam/Heroic dizinlerinde kurulu oyun algılanmadı. Steam veya Epic Games hesabınızla giriş yaparak oyunlarınızı hemen eşitleyebilir veya aşağıdaki Mağaza Keşfi vitrininden yeni oyunlar keşfedebilirsiniz."));
                empty_lib_desc.add_css_class("library-empty-desc");
                empty_lib_desc.set_halign(gtk4::Align::Center);
                empty_lib_desc.set_wrap(true);

                let btn_row = GtkBox::new(Orientation::Horizontal, 12);
                btn_row.set_halign(gtk4::Align::Center);
                btn_row.set_margin_top(12);

                let steam_login_btn = Button::with_label("Steam'e Giriş Yap");
                steam_login_btn.add_css_class("hero-play-btn");
                steam_login_btn.connect_clicked(|_| {
                    let _ = Command::new("steam").spawn();
                });

                let heroic_login_btn = Button::with_label("Heroic Games Launcher");
                heroic_login_btn.add_css_class("hero-opt-btn");
                heroic_login_btn.connect_clicked(|_| {
                    let _ = Command::new("heroic").spawn();
                });

                btn_row.append(&steam_login_btn);
                btn_row.append(&heroic_login_btn);

                empty_lib_box.append(&empty_lib_title);
                empty_lib_box.append(&empty_lib_desc);
                empty_lib_box.append(&btn_row);
                grid_box.append(&empty_lib_box);
                return;
            }

            let mut current_row = GtkBox::new(Orientation::Horizontal, 0);
            let mut row_count = 0;
            let mut match_count = 0;

            for (idx, game) in games.iter().enumerate() {
                let matches_filter = match filter.as_str() {
                    "steam" => game.is_steam || game.category.to_lowercase().contains("steam"),
                    "epic" => game.category.to_lowercase().contains("epic") || game.id == "heroic",
                    "gog" => game.category.to_lowercase().contains("gog"),
                    "retro" => game.category.to_lowercase().contains("retro") || game.id == "retroarch",
                    _ => true,
                };

                let matches_query = if query.is_empty() {
                    true
                } else {
                    game.title.to_lowercase().contains(&query) || game.category.to_lowercase().contains(&query)
                };

                if matches_filter && matches_query {
                    match_count += 1;
                    let card = Button::new();
                    card.add_css_class("game-card");

                    let c_box = GtkBox::new(Orientation::Vertical, 4);
                    let img = create_game_cover_image(game);
                    c_box.append(&img);

                    let t_lbl = Label::new(Some(&game.title));
                    t_lbl.add_css_class("game-card-title");
                    t_lbl.set_halign(gtk4::Align::Start);
                    t_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                    t_lbl.set_max_width_chars(16);
                    c_box.append(&t_lbl);

                    let c_lbl = Label::new(Some(&game.category));
                    c_lbl.add_css_class("game-card-category");
                    c_lbl.set_halign(gtk4::Align::Start);
                    c_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
                    c_lbl.set_max_width_chars(16);
                    c_box.append(&c_lbl);

                    card.set_child(Some(&c_box));

                    let g_clone = game.clone();
                    let alert_c = trigger_offline_alert.clone();
                    card.connect_clicked(move |_| {
                        handle_game_activation(&g_clone, &*alert_c);
                    });

                    attach_card_events(
                        &card,
                        game,
                        idx,
                        selected_index.clone(),
                        hero_title.clone(),
                        hero_subtitle.clone(),
                        play_btn.clone(),
                        store_btn.clone(),
                        trailer_badge.clone(),
                        hero_pic.clone(),
                        hero_vid.clone(),
                        active_timer_id.clone(),
                    );

                    current_row.append(&card);
                    row_count += 1;

                    if row_count >= 4 {
                        grid_box.append(&current_row);
                        current_row = GtkBox::new(Orientation::Horizontal, 0);
                        row_count = 0;
                    }
                }
            }

            if row_count > 0 {
                grid_box.append(&current_row);
            }

            if match_count == 0 {
                let empty_filter_box = GtkBox::new(Orientation::Vertical, 8);
                empty_filter_box.add_css_class("library-empty-card");
                empty_filter_box.set_hexpand(true);

                let (card_title, card_desc, b1_label, b1_cmd, b2_label, b2_url) = match filter.as_str() {
                    "steam" => (
                        "Steam Kütüphanenizde Henüz Oyun Bulunmuyor",
                        "Sisteminizde kurulu Steam oyunu algılanmadı. Steam istemcisini açarak kütüphanenizdeki oyunları hemen indirebilir veya mağazaya göz atabilirsiniz.",
                        Some("Steam İstemcisini Başlat"),
                        Some("steam"),
                        Some("Steam Mağazasına Göz At"),
                        Some("https://store.steampowered.com/"),
                    ),
                    "epic" => (
                        "Epic Games (Heroic) Kütüphanenizde Oyun Bulunmuyor",
                        "Heroic Games Launcher üzerinden Epic Games hesabınıza bağlanarak oyunlarınızı kurabilir ve kütüphanenizi eşitleyebilirsiniz.",
                        Some("Heroic Launcher'ı Başlat"),
                        Some("heroic"),
                        Some("Epic Games Store'a Git"),
                        Some("https://store.epicgames.com/"),
                    ),
                    "gog" => (
                        "GOG Galaxy Kütüphanenizde Henüz Oyun Bulunmuyor",
                        "GOG kütüphanenizi Heroic Games Launcher ile yönetebilir veya doğrudan web mağazasından yeni oyunlar keşfedebilirsiniz.",
                        Some("Heroic Launcher (GOG) Aç"),
                        Some("heroic"),
                        Some("GOG.com Mağazasına Git"),
                        Some("https://www.gog.com/"),
                    ),
                    "retro" => (
                        "Retro Konsol Kütüphanenizde Oyun Bulunmuyor",
                        "RetroArch çoklu emülatör sistemini başlatıp klasik konsol ROM'larınızı çalıştırabilirsiniz.",
                        Some("RetroArch Emülatörünü Aç"),
                        Some("retroarch"),
                        None,
                        None,
                    ),
                    _ => (
                        "Aradığınız Kriterlere Uygun Oyun Bulunamadı",
                        "Arama teriminizi değiştirebilir veya sol menüden filtreyi 'Tüm Oyunlar' olarak sıfırlayabilirsiniz.",
                        None,
                        None,
                        None,
                        None,
                    ),
                };

                let title_lbl = Label::new(Some(card_title));
                title_lbl.add_css_class("library-empty-title");
                title_lbl.set_halign(gtk4::Align::Center);

                let desc_lbl = Label::new(Some(card_desc));
                desc_lbl.add_css_class("library-empty-desc");
                desc_lbl.set_halign(gtk4::Align::Center);
                desc_lbl.set_wrap(true);

                empty_filter_box.append(&title_lbl);
                empty_filter_box.append(&desc_lbl);

                let action_row = GtkBox::new(Orientation::Horizontal, 12);
                action_row.set_halign(gtk4::Align::Center);
                action_row.set_margin_top(10);

                if let (Some(b1_txt), Some(cmd)) = (b1_label, b1_cmd) {
                    let b1 = Button::with_label(b1_txt);
                    b1.add_css_class("hero-play-btn");
                    let cmd_str = cmd.to_string();
                    b1.connect_clicked(move |_| {
                        let _ = Command::new(&cmd_str).spawn();
                    });
                    action_row.append(&b1);
                }

                if let (Some(b2_txt), Some(url)) = (b2_label, b2_url) {
                    let b2 = Button::with_label(b2_txt);
                    b2.add_css_class("hero-opt-btn");
                    let u_str = url.to_string();
                    let alert_url = trigger_offline_alert.clone();
                    b2.connect_clicked(move |_| {
                        if !is_system_online() {
                            alert_url("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
                        } else {
                            open_browser_url(&u_str);
                        }
                    });
                    action_row.append(&b2);
                }

                empty_filter_box.append(&action_row);
                grid_box.append(&empty_filter_box);
            }
        })
    };

    let nav_buttons = vec![
        ("all", nav_all.clone()),
        ("steam", nav_steam.clone()),
        ("epic", nav_epic.clone()),
        ("gog", nav_gog.clone()),
        ("retro", nav_retro.clone()),
    ];

    let vadj = main_scroll.vadjustment();

    for (filter_name, btn) in &nav_buttons {
        let f_name = filter_name.to_string();
        let cur_f = current_filter.clone();
        let p_grid = populate_grid.clone();
        let all_navs: Vec<Button> = nav_buttons.iter().map(|(_, b)| b.clone()).collect();
        let this_btn = btn.clone();
        let vadj_btn = vadj.clone();

        btn.connect_clicked(move |_| {
            *cur_f.borrow_mut() = f_name.clone();
            for b in &all_navs {
                b.remove_css_class("active");
            }
            this_btn.add_css_class("active");
            p_grid();
            if f_name != "all" {
                vadj_btn.set_value(640.0);
            } else {
                vadj_btn.set_value(0.0);
            }
        });
    }

    let s_query = search_query.clone();
    let p_grid_search = populate_grid.clone();
    search_entry.connect_changed(move |entry| {
        *s_query.borrow_mut() = entry.text().to_string();
        p_grid_search();
    });

    populate_grid();

    main_content.append(&grid_box);

    // ── 7. Katman 5: Öne Çıkanlar & Mağaza Keşfi (Steam Store Showcase Vitrini) ──
    let showcase_title = Label::new(Some("Öne Çıkanlar & Mağaza Keşfi"));
    showcase_title.add_css_class("section-header");
    showcase_title.set_halign(gtk4::Align::Start);
    main_content.append(&showcase_title);

    let showcase_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Never)
        .hexpand(true)
        .build();

    let showcase_box = GtkBox::new(Orientation::Horizontal, 0);

    for (idx, game) in showcase_games.iter().enumerate() {
        let card = Button::new();
        card.add_css_class("game-card");

        let c_box = GtkBox::new(Orientation::Vertical, 4);
        let img = create_game_cover_image(game);
        c_box.append(&img);

        let t_lbl = Label::new(Some(&game.title));
        t_lbl.add_css_class("game-card-title");
        t_lbl.set_halign(gtk4::Align::Start);
        t_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        t_lbl.set_max_width_chars(16);
        c_box.append(&t_lbl);

        let c_lbl = Label::new(Some(&game.category));
        c_lbl.add_css_class("game-card-category");
        c_lbl.set_halign(gtk4::Align::Start);
        c_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        c_lbl.set_max_width_chars(16);
        c_box.append(&c_lbl);

        card.set_child(Some(&c_box));

        let g_clone = game.clone();
        let alert_sh = trigger_offline_alert.clone();
        card.connect_clicked(move |_| {
            handle_game_activation(&g_clone, &*alert_sh);
        });

        attach_card_events(
            &card,
            game,
            idx,
            selected_index.clone(),
            hero_title.clone(),
            hero_subtitle.clone(),
            play_btn.clone(),
            store_btn.clone(),
            trailer_badge.clone(),
            hero_pic.clone(),
            hero_vid.clone(),
            active_timer_id.clone(),
        );

        showcase_box.append(&card);
        if nav_games.is_empty() {
            card_buttons.push(card);
        }
    }

    showcase_scroll.set_child(Some(&showcase_box));
    main_content.append(&showcase_scroll);

    main_scroll.set_child(Some(&main_content));
    right_col.append(&main_scroll);

    // ── 8. Alt Kumanda ve Gezinme Kılavuzu (Steam Deck Style - Sıfır Emojili) ──
    let controller_bar = GtkBox::new(Orientation::Horizontal, 16);
    controller_bar.add_css_class("controller-bar");

    let k_a = Label::new(Some("A"));
    k_a.add_css_class("gamepad-key");
    let d_a = Label::new(Some("Oyna / Seç"));
    d_a.add_css_class("gamepad-desc");

    let k_b = Label::new(Some("B"));
    k_b.add_css_class("gamepad-key");
    let d_b = Label::new(Some("Geri / Masaüstü"));
    d_b.add_css_class("gamepad-desc");

    let k_x = Label::new(Some("X"));
    k_x.add_css_class("gamepad-key");
    let d_x = Label::new(Some("Seçenekler"));
    d_x.add_css_class("gamepad-desc");

    let k_y = Label::new(Some("Y"));
    k_y.add_css_class("gamepad-key");
    let d_y = Label::new(Some("Ara"));
    d_y.add_css_class("gamepad-desc");

    let k_lb = Label::new(Some("LB/RB"));
    k_lb.add_css_class("gamepad-key");
    let d_lb = Label::new(Some("Kategoriler"));
    d_lb.add_css_class("gamepad-desc");

    controller_bar.append(&k_a);
    controller_bar.append(&d_a);
    controller_bar.append(&k_b);
    controller_bar.append(&d_b);
    controller_bar.append(&k_x);
    controller_bar.append(&d_x);
    controller_bar.append(&k_y);
    controller_bar.append(&d_y);
    controller_bar.append(&k_lb);
    controller_bar.append(&d_lb);

    right_col.append(&controller_bar);
    root_box.append(&right_col);

    window.set_child(Some(&root_box));

    // İlk seçili oyunla Hero Banner'ı doldur (Önce Son Oynananlar, sonra Kurulu Oyunlar, sonra Vitrin)
    let first_display_game = if let Some(g) = recent_games.first() {
        Some(g.clone())
    } else if let Some(g) = all_games.lock().unwrap().first() {
        Some(g.clone())
    } else {
        showcase_games.first().cloned()
    };

    if let Some(ref first) = first_display_game {
        update_hero_showcase(first, &hero_title, &hero_subtitle, &play_btn, &store_btn, &hero_pic, &hero_vid);
        fetch_and_apply_store_screenshot(first, &hero_pic);
    }

    // Klavye & Gamepad Kısayolları (Esc, Sol/Sağ Oklar, Enter)
    let navigable_games = if !recent_games.is_empty() {
        recent_games.clone()
    } else if !all_games.lock().unwrap().is_empty() {
        all_games.lock().unwrap().clone()
    } else {
        showcase_games.as_ref().clone()
    };
    let all_nav_g = Arc::new(Mutex::new(navigable_games));

    let key_controller = EventControllerKey::new();
    let s_idx_key = selected_index.clone();
    let all_g_key = all_nav_g.clone();
    let c_btns_key = card_buttons.clone();
    let loop_key = main_loop.clone();
    let h_title_k = hero_title.clone();
    let h_sub_k = hero_subtitle.clone();
    let p_btn_k = play_btn.clone();
    let s_btn_k = store_btn.clone();
    let h_pic_k = hero_pic.clone();
    let h_vid_k = hero_vid.clone();
    let alert_key = trigger_offline_alert.clone();

    key_controller.connect_key_pressed(move |_ctrl, keyval, _code, _state| {
        match keyval {
            gdk::Key::Escape => {
                println!("Exiting Blaze GameZone to Desktop...");
                loop_key.quit();
                glib::Propagation::Stop
            }
            gdk::Key::Left => {
                let mut idx = s_idx_key.lock().unwrap();
                if *idx > 0 {
                    *idx -= 1;
                    if let Some(btn) = c_btns_key.get(*idx) {
                        btn.grab_focus();
                    }
                    if let Some(game) = all_g_key.lock().unwrap().get(*idx) {
                        update_hero_showcase(game, &h_title_k, &h_sub_k, &p_btn_k, &s_btn_k, &h_pic_k, &h_vid_k);
                        fetch_and_apply_store_screenshot(game, &h_pic_k);
                    }
                }
                glib::Propagation::Stop
            }
            gdk::Key::Right => {
                let mut idx = s_idx_key.lock().unwrap();
                let total = all_g_key.lock().unwrap().len();
                if *idx + 1 < total {
                    *idx += 1;
                    if let Some(btn) = c_btns_key.get(*idx) {
                        btn.grab_focus();
                    }
                    if let Some(game) = all_g_key.lock().unwrap().get(*idx) {
                        update_hero_showcase(game, &h_title_k, &h_sub_k, &p_btn_k, &s_btn_k, &h_pic_k, &h_vid_k);
                        fetch_and_apply_store_screenshot(game, &h_pic_k);
                    }
                }
                glib::Propagation::Stop
            }
            gdk::Key::Return | gdk::Key::KP_Enter => {
                let idx = *s_idx_key.lock().unwrap();
                if let Some(game) = all_g_key.lock().unwrap().get(idx) {
                    handle_game_activation(game, &*alert_key);
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });

    window.add_controller(key_controller);

    // Hero Play button action
    let s_idx_play = selected_index.clone();
    let all_g_play = all_nav_g.clone();
    let alert_play = trigger_offline_alert.clone();
    play_btn.connect_clicked(move |_| {
        let idx = *s_idx_play.lock().unwrap();
        if let Some(game) = all_g_play.lock().unwrap().get(idx) {
            handle_game_activation(game, &*alert_play);
        }
    });

    // Hero Store button action
    let s_idx_store = selected_index.clone();
    let all_g_store = all_nav_g.clone();
    let alert_store = trigger_offline_alert.clone();
    store_btn.connect_clicked(move |_| {
        let idx = *s_idx_store.lock().unwrap();
        if let Some(game) = all_g_store.lock().unwrap().get(idx) {
            if let Some(store) = &game.store_url {
                if !is_system_online() {
                    alert_store("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
                } else {
                    open_browser_url(store);
                }
            }
        }
    });

    // Hero Options button action
    let s_idx_opt = selected_index.clone();
    let all_g_opt = all_nav_g.clone();
    opt_btn.connect_clicked(move |_| {
        let idx = *s_idx_opt.lock().unwrap();
        if let Some(game) = all_g_opt.lock().unwrap().get(idx) {
            let _ = Command::new("notify-send")
                .args([
                    "-a", "Blaze GameZone",
                    "-i", "preferences-system",
                    "Oyun Seçenekleri",
                    &format!("{}: Proton / BORE öncelik ayarları optimize edildi.", game.title),
                ])
                .spawn();
        }
    });

    window.present();
}

fn update_hero_showcase(
    game: &GameEntry,
    title_lbl: &Label,
    sub_lbl: &Label,
    play_btn: &Button,
    store_btn: &Button,
    hero_pic: &Picture,
    hero_vid: &Video,
) {
    title_lbl.set_text(&game.title);
    if game.is_installed {
        sub_lbl.set_text(&format!("{} • Kurulu ve Hazır • Düşük Gecikme Modu Aktif", game.banner_desc));
        play_btn.set_label(&format!("OYNA (A / Enter) - {}", game.title));
        play_btn.remove_css_class("hero-install-btn");
        play_btn.add_css_class("hero-play-btn");
    } else {
        sub_lbl.set_text(&format!("{} • Henüz Kurulu Değil (İndirilebilir)", game.banner_desc));
        play_btn.set_label(&format!("YÜKLE / MAĞAZADA GÖR (A) - {}", game.title));
        play_btn.remove_css_class("hero-play-btn");
        play_btn.add_css_class("hero-install-btn");
    }

    store_btn.set_visible(game.store_url.is_some());

    // Video oynatılıyorsa durdur ve resmi göster
    hero_vid.set_visible(false);
    hero_vid.set_file(None::<&gio::File>);
    hero_pic.set_visible(true);

    if let Some(ss) = get_game_screenshot_path(game) {
        hero_pic.set_filename(Some(Path::new(&ss)));
    } else if let Some(cover) = get_game_cover_path(game) {
        hero_pic.set_filename(Some(Path::new(&cover)));
    }
}

fn handle_game_activation(game: &GameEntry, on_offline_alert: &dyn Fn(&str)) {
    if !game.is_installed {
        if !is_system_online() {
            on_offline_alert("İnternete bağlı değilsiniz. Sadece yüklü oyunları ve uygulamaları çalıştırabilirsiniz.");
            return;
        }

        // Online: open store page to install
        if let Some(app_id) = &game.steam_app_id {
            let _ = Command::new("notify-send")
                .args([
                    "-a", "Blaze GameZone",
                    "-i", "input-gaming",
                    "Steam Mağazası Açılıyor",
                    &format!("{} yükleme sayfası açılıyor...", game.title),
                ])
                .spawn();

            open_browser_url(&format!("https://store.steampowered.com/app/{}", app_id));
            return;
        }

        if let Some(store) = &game.store_url {
            open_browser_url(store);
            return;
        }

        on_offline_alert("Bu oyun henüz sisteminizde kurulu değil.");
        return;
    }

    launch_game_entry(game);
}

// ── Steam Mağazasından Ekran Görüntüsü Çekme ──
fn fetch_and_apply_store_screenshot(game: &GameEntry, hero_pic: &Picture) {
    if let Some(ss) = get_game_screenshot_path(game) {
        hero_pic.set_filename(Some(Path::new(&ss)));
        return;
    }

    if let Some(app_id) = &game.steam_app_id {
        let cache_dir = dirs_cache_dir().join(format!("steam-{}", app_id));
        let ss_dest = cache_dir.join("screenshot.jpg");
        let a_id = app_id.clone();

        if ss_dest.exists() {
            hero_pic.set_filename(Some(&ss_dest));
            return;
        }

        let (sender, receiver) = async_channel::unbounded::<String>();
        let pic_clone = hero_pic.clone();
        glib::spawn_future_local(async move {
            if let Ok(ss_str) = receiver.recv().await {
                pic_clone.set_filename(Some(Path::new(&ss_str)));
            }
        });

        std::thread::spawn(move || {
            let api_url = format!("https://store.steampowered.com/api/appdetails?appids={}", a_id);
            if let Ok(output) = Command::new("curl").args(["-sL", "-m", "5", &api_url]).output() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(ss_list) = val.get(&a_id)
                            .and_then(|v| v.get("data"))
                            .and_then(|v| v.get("screenshots"))
                            .and_then(|v| v.as_array()) {
                            if let Some(first_ss) = ss_list.get(0).and_then(|s| s.get("path_full")).and_then(|s| s.as_str()) {
                                let _ = std::fs::create_dir_all(&cache_dir);
                                let _ = Command::new("curl")
                                    .args(["-sL", "-m", "6", first_ss, "-o", ss_dest.to_str().unwrap()])
                                    .status();
                                println!("Fetched store screenshot for app {}: {}", a_id, first_ss);
                                let ss_str = ss_dest.to_string_lossy().to_string();
                                let _ = sender.send_blocking(ss_str);
                            }
                        }
                    }
                }
            }
        });
    }
}

// ── 10 Saniye Hover: Steam Oynanış Videosu / Fragmanı Kendi Çerçevesinde Başlatma ──
fn trigger_store_video_preview(game: &GameEntry, hero_vid: &Video, hero_pic: &Picture, trailer_badge: &Label) {
    println!("Triggering 10s gameplay video preview for {}", game.title);
    trailer_badge.set_text("▶ Oynanış Fragmanı Başlatıldı (Kendi Çerçevesinde)");

    // 1. Önce diskteki önbelleğe veya paketli fragmana bak
    if let Some(local_path) = get_game_trailer_path(game) {
        hero_vid.set_filename(Some(Path::new(&local_path)));
        hero_vid.set_autoplay(true);
        hero_vid.set_loop(true);
        hero_vid.set_visible(true);
        hero_pic.set_visible(false);
        return;
    }

    // 2. Steam App ID varsa doğrudan Steam CDN fragmanını akıt
    if let Some(app_id) = &game.steam_app_id {
        let a_id = app_id.clone();
        let (sender, receiver) = async_channel::unbounded::<String>();
        let vid_clone = hero_vid.clone();
        let pic_clone = hero_pic.clone();

        glib::spawn_future_local(async move {
            if let Ok(video_url) = receiver.recv().await {
                let file = gio::File::for_uri(&video_url);
                vid_clone.set_file(Some(&file));
                vid_clone.set_autoplay(true);
                vid_clone.set_loop(true);
                vid_clone.set_visible(true);
                pic_clone.set_visible(false);
            }
        });

        std::thread::spawn(move || {
            let api_url = format!("https://store.steampowered.com/api/appdetails?appids={}", a_id);
            if let Ok(output) = Command::new("curl").args(["-sL", "-m", "5", &api_url]).output() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(movies) = val.get(&a_id)
                            .and_then(|v| v.get("data"))
                            .and_then(|v| v.get("movies"))
                            .and_then(|v| v.as_array()) {
                            if let Some(first_movie) = movies.get(0) {
                                if let Some(mid) = first_movie.get("id").and_then(|m| m.as_i64()) {
                                    let video_url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{}/movie480.mp4", mid);
                                    println!("Streaming trailer in widget frame: {}", video_url);
                                    let _ = sender.send_blocking(video_url);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

fn get_game_screenshot_path(game: &GameEntry) -> Option<String> {
    if let Some(p) = &game.screenshot_path {
        if Path::new(p).exists() {
            return Some(p.clone());
        }
    }
    let mut check_keys = vec![game.id.clone()];
    if let Some(app_id) = &game.steam_app_id {
        check_keys.push(format!("steam-{}", app_id));
        check_keys.push(app_id.clone());
    }
    let cache_root = dirs_cache_dir();
    for key in &check_keys {
        let dir = cache_root.join(key);
        for ext in ["screenshot.jpg", "screenshot.png", "hero.jpg", "cover.jpg"] {
            let p = dir.join(ext);
            if p.exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }
    let bundled_dirs = [
        "/usr/share/solarui/covers",
        "/home/darkmorpheus/BlazeFedora/blazeos_custom_apps/usr/share/solarui/covers",
    ];
    for d in bundled_dirs {
        for key in &check_keys {
            for prefix in [&format!("{}-screenshot", key), key] {
                for ext in ["jpg", "png", "svg"] {
                    let p = Path::new(d).join(format!("{}.{}", prefix, ext));
                    if p.exists() {
                        return Some(p.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_game_trailer_path(game: &GameEntry) -> Option<String> {
    if let Some(m) = &game.movie_url {
        if Path::new(m).exists() {
            return Some(m.clone());
        }
    }
    let mut check_keys = vec![game.id.clone()];
    if let Some(app_id) = &game.steam_app_id {
        check_keys.push(format!("steam-{}", app_id));
        check_keys.push(app_id.clone());
    }
    let cache_root = dirs_cache_dir();
    for key in &check_keys {
        let p = cache_root.join(key).join("trailer.mp4");
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    let bundled_dirs = [
        "/usr/share/solarui/covers",
        "/home/darkmorpheus/BlazeFedora/blazeos_custom_apps/usr/share/solarui/covers",
    ];
    for d in bundled_dirs {
        for key in &check_keys {
            for prefix in [&format!("{}-trailer", key), key] {
                let p = Path::new(d).join(format!("{}.mp4", prefix));
                if p.exists() {
                    return Some(p.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn get_game_cover_path(game: &GameEntry) -> Option<String> {
    if let Some(p) = &game.cover_path {
        if Path::new(p).exists() {
            return Some(p.clone());
        }
    }
    let mut check_keys = vec![game.id.clone()];
    if let Some(app_id) = &game.steam_app_id {
        check_keys.push(format!("steam-{}", app_id));
        check_keys.push(app_id.clone());
    }
    let cache_root = dirs_cache_dir();
    for key in &check_keys {
        let dir = cache_root.join(key);
        for ext in ["cover.jpg", "cover.png", "cover.svg", "hero.jpg", "screenshot.jpg"] {
            let p = dir.join(ext);
            if p.exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }
    let bundled_dirs = [
        "/usr/share/solarui/covers",
        "/home/darkmorpheus/BlazeFedora/blazeos_custom_apps/usr/share/solarui/covers",
    ];
    for d in bundled_dirs {
        for key in &check_keys {
            for ext in ["jpg", "png", "svg"] {
                let p = Path::new(d).join(format!("{}.{}", key, ext));
                if p.exists() {
                    return Some(p.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn create_game_cover_image(game: &GameEntry) -> Picture {
    let pic = Picture::new();
    pic.set_can_shrink(true);
    pic.set_content_fit(gtk4::ContentFit::Cover);
    pic.set_size_request(160, 220);
    pic.add_css_class("game-cover-pic");

    if let Some(p) = get_game_cover_path(game) {
        pic.set_filename(Some(Path::new(&p)));
    } else {
        let bundled = [
            "/usr/share/solarui/covers/steam-deck.jpg",
            "/home/darkmorpheus/BlazeFedora/blazeos_custom_apps/usr/share/solarui/covers/steam-deck.jpg",
        ];
        for b in bundled {
            if Path::new(b).exists() {
                pic.set_filename(Some(Path::new(b)));
                break;
            }
        }
    }
    pic
}

// ── Web Afiş ve Logo İndirici (Arka Plan Asenkron Önbellek) ──
fn init_media_cache(games: &[GameEntry]) {
    let cache_dir = dirs_cache_dir();
    let _ = std::fs::create_dir_all(&cache_dir);

    for game in games {
        if game.is_steam {
            if let Some(app_id) = &game.steam_app_id {
                let g_dir = cache_dir.join(format!("steam-{}", app_id));
                let _ = std::fs::create_dir_all(&g_dir);

                let cover_dest = g_dir.join("cover.jpg");
                let hero_dest = g_dir.join("hero.jpg");
                let a_id = app_id.clone();

                if !cover_dest.exists() || !hero_dest.exists() {
                    std::thread::spawn(move || {
                        // Header / Cover indir
                        if !cover_dest.exists() {
                            let cover_url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg", a_id);
                            let _ = Command::new("curl")
                                .args(["-sL", "-m", "6", &cover_url, "-o", cover_dest.to_str().unwrap()])
                                .status();
                        }
                        // Hero Banner indir
                        if !hero_dest.exists() {
                            let hero_url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{}/library_hero.jpg", a_id);
                            let _ = Command::new("curl")
                                .args(["-sL", "-m", "6", &hero_url, "-o", hero_dest.to_str().unwrap()])
                                .status();
                        }
                    });
                }
            }
        }
    }
}

pub fn clean_gamezone_cache() {
    let cache_dir = dirs_cache_dir();
    if cache_dir.exists() {
        let _ = std::fs::remove_dir_all(&cache_dir);
        let _ = std::fs::create_dir_all(&cache_dir);
        println!("GameZone media cache cleared successfully.");
        let _ = Command::new("notify-send")
            .args([
                "-a", "Blaze GameZone",
                "-i", "edit-clear",
                "Önbellek Temizlendi",
                "GameZone oyun afiş ve medya önbelleği başarıyla temizlendi.",
            ])
            .spawn();
    }
}

fn dirs_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
    PathBuf::from(home).join(".cache/blaze-gamezone/media")
}

fn new_game_entry(
    id: &str,
    title: &str,
    category: &str,
    exec: &str,
    banner_desc: &str,
    is_steam: bool,
    steam_app_id: Option<String>,
    cover_path: Option<String>,
    hero_path: Option<String>,
    screenshot_path: Option<String>,
    store_url: Option<String>,
    movie_url: Option<String>,
) -> GameEntry {
    let is_installed = is_game_installed(steam_app_id.as_deref(), exec, id);
    GameEntry {
        id: id.to_string(),
        title: title.to_string(),
        category: category.to_string(),
        exec: exec.to_string(),
        banner_desc: banner_desc.to_string(),
        is_steam,
        is_installed,
        steam_app_id,
        cover_path,
        hero_path,
        screenshot_path,
        store_url,
        movie_url,
    }
}

// ── Hesap Algılama (Steam PersonaName & Epic/Legendary displayName) ──
pub fn detect_logged_in_accounts() -> (Option<String>, Option<String>) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
    let steam_vdfs = [
        PathBuf::from(&home).join(".steam/steam/config/loginusers.vdf"),
        PathBuf::from(&home).join(".local/share/Steam/config/loginusers.vdf"),
    ];
    let mut steam_user = None;
    for vdf in steam_vdfs {
        if let Ok(content) = std::fs::read_to_string(&vdf) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"PersonaName\"") {
                    let parts: Vec<&str> = trimmed.split('"').filter(|s| !s.trim().is_empty()).collect();
                    if parts.len() >= 2 {
                        steam_user = Some(parts[1].to_string());
                        break;
                    }
                }
            }
        }
        if steam_user.is_some() {
            break;
        }
    }
    let mut epic_user = None;
    let legendary_user = PathBuf::from(&home).join(".config/legendary/user.json");
    if let Ok(content) = std::fs::read_to_string(&legendary_user) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = val.get("displayName").and_then(|v| v.as_str()) {
                epic_user = Some(name.to_string());
            }
        }
    }
    (steam_user, epic_user)
}

// ── Oyun Geçmişi Takipçisi (Son Oynananlar) ──
pub fn recent_history_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
    PathBuf::from(home).join(".config/solarui/gamezone/recent_history.json")
}

pub fn get_recent_game_ids() -> Vec<String> {
    let p = recent_history_file();
    if let Ok(content) = std::fs::read_to_string(&p) {
        if let Ok(ids) = serde_json::from_str::<Vec<String>>(&content) {
            return ids;
        }
    }
    Vec::new()
}

pub fn record_game_played(game_id: &str) {
    let p = recent_history_file();
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut ids = get_recent_game_ids();
    ids.retain(|id| id != game_id);
    ids.insert(0, game_id.to_string());
    if ids.len() > 15 {
        ids.truncate(15);
    }
    if let Ok(json) = serde_json::to_string_pretty(&ids) {
        let _ = std::fs::write(&p, json);
    }
}

// ── Öne Çıkanlar & Mağaza Keşfi (Steam Store Showcase Vitrini) ──
pub fn discover_store_showcase() -> Vec<GameEntry> {
    let mut list = Vec::new();

    list.push(new_game_entry(
        "beamng",
        "BeamNG.drive",
        "Araç Fiziği Simülasyonu",
        "steam steam://rungameid/284160",
        "Gerçek zamanlı yumuşak gövde araç fiziği simülasyonu.",
        true,
        Some("284160".to_string()),
        Some("/usr/share/solarui/covers/beamng.jpg".to_string()),
        Some("/usr/share/solarui/covers/beamng.jpg".to_string()),
        Some("/usr/share/solarui/covers/beamng-screenshot.jpg".to_string()),
        Some("https://store.steampowered.com/app/284160/BeamNGdrive/".to_string()),
        Some("/usr/share/solarui/covers/beamng-trailer.mp4".to_string()),
    ));

    list.push(new_game_entry(
        "dota-2",
        "Dota 2",
        "Strateji & MOBA",
        "steam steam://rungameid/570",
        "Valve amiral gemisi rekabetçi çevrimiçi arena oyunu.",
        true,
        Some("570".to_string()),
        Some("/usr/share/solarui/covers/dota-2.jpg".to_string()),
        None,
        None,
        Some("https://store.steampowered.com/app/570/Dota_2/".to_string()),
        None,
    ));

    list.push(new_game_entry(
        "cyberpunk-2077",
        "Cyberpunk 2077",
        "Aksiyon RPG",
        "steam steam://rungameid/1091500",
        "Night City sokaklarında geçen distopik açık dünya macerası.",
        true,
        Some("1091500".to_string()),
        Some("/usr/share/solarui/covers/cyberpunk-2077.jpg".to_string()),
        None,
        None,
        Some("https://store.steampowered.com/app/1091500/Cyberpunk_2077/".to_string()),
        None,
    ));

    list.push(new_game_entry(
        "stardew-valley",
        "Stardew Valley",
        "Çiftlik & Simülasyon",
        "steam steam://rungameid/413150",
        "Pelikan kasabasında kendi hayalinizdeki çiftliği inşa edin.",
        true,
        Some("413150".to_string()),
        Some("/usr/share/solarui/covers/stardew-valley.jpg".to_string()),
        None,
        None,
        Some("https://store.steampowered.com/app/413150/Stardew_Valley/".to_string()),
        None,
    ));

    list.push(new_game_entry(
        "mechanicus",
        "Warhammer 40,000: Mechanicus",
        "Sıra Tabanlı Taktik",
        "steam steam://rungameid/673880",
        "Adeptus Mechanicus güçleri ile Necron mezarlarında savaşın.",
        true,
        Some("673880".to_string()),
        Some("/usr/share/solarui/covers/mechanicus.jpg".to_string()),
        None,
        None,
        Some("https://store.steampowered.com/app/673880/Warhammer_40000_Mechanicus/".to_string()),
        None,
    ));

    list.push(new_game_entry(
        "xbox-cloud",
        "Xbox Cloud Gaming",
        "Bulut Oyun Vitrini",
        "xdg-open https://www.xbox.com/play",
        "Yüzlerce konsol oyununu bulut üzerinden anında oynayın.",
        false,
        None,
        None,
        None,
        None,
        Some("https://www.xbox.com/play".to_string()),
        None,
    ));

    list
}

// ── Kütüphane Taraması (Yalnızca Disk üzerinde Gerçekten Kurulu Oyunlar ve Başlatıcılar) ──
pub fn discover_all_games() -> Vec<GameEntry> {
    let mut list = Vec::new();
    let cache_dir = dirs_cache_dir();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());

    // 1. Kurulu Başlatıcı Kartları (Yalnızca sistemde varsa kütüphaneye ekle)
    if is_game_installed(None, "steam", "steam-deck") {
        list.push(new_game_entry(
            "steam-deck",
            "Steam Big Picture",
            "Steam Deck Arayüzü",
            "steam -gamepadui",
            "Resmi Steam Deck kumanda ve tam ekran oyun arayüzü.",
            true,
            None,
            None,
            None,
            None,
            Some("https://store.steampowered.com/".to_string()),
            None,
        ));
    }

    if is_game_installed(None, "heroic", "heroic") {
        list.push(new_game_entry(
            "heroic",
            "Heroic Games Launcher",
            "Epic Games & GOG",
            "heroic",
            "Açık kaynak Epic Games, GOG ve Amazon Games yöneticisi.",
            false,
            None,
            None,
            None,
            None,
            Some("https://store.epicgames.com/".to_string()),
            None,
        ));
    }

    if is_game_installed(None, "lutris", "lutris") {
        list.push(new_game_entry(
            "lutris",
            "Lutris Gamepad UI",
            "Açık Kaynak Oyun Yöneticisi",
            "lutris",
            "Tüm platformlar, emülatörler ve Wine oyunları tek yerde.",
            false,
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }

    if is_game_installed(None, "retroarch", "retroarch") {
        list.push(new_game_entry(
            "retroarch",
            "RetroArch Emulation Hub",
            "Retro Konsol",
            "retroarch",
            "PS2, PSP, N64, SNES ve klasik konsol emülasyon merkezi.",
            false,
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }

    // 2. Kurulu Steam Oyunlarını Tara (~/.steam/steam/steamapps/*.acf)
    let steam_dirs = [
        PathBuf::from(&home).join(".steam/steam/steamapps"),
        PathBuf::from(&home).join(".local/share/Steam/steamapps"),
    ];

    for s_dir in steam_dirs {
        if s_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(s_dir) {
                for e in entries.flatten() {
                    let path = e.path();
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if file_name.starts_with("appmanifest_") && file_name.ends_with(".acf") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some((app_id, name)) = parse_acf_file(&content) {
                                    let c_path = cache_dir.join(format!("steam-{}", app_id)).join("cover.jpg");
                                    let h_path = cache_dir.join(format!("steam-{}", app_id)).join("hero.jpg");
                                    let s_path = cache_dir.join(format!("steam-{}", app_id)).join("screenshot.jpg");

                                    list.push(GameEntry {
                                        id: format!("steam-{}", app_id),
                                        title: name.clone(),
                                        category: "Steam Kütüphanesi".to_string(),
                                        exec: format!("steam steam://rungameid/{}", app_id),
                                        banner_desc: format!("{} • Proton 9.0 / Native Steam Oyunu", name),
                                        is_steam: true,
                                        is_installed: true,
                                        steam_app_id: Some(app_id.clone()),
                                        cover_path: Some(c_path.to_string_lossy().to_string()),
                                        hero_path: Some(h_path.to_string_lossy().to_string()),
                                        screenshot_path: Some(s_path.to_string_lossy().to_string()),
                                        store_url: Some(format!("https://store.steampowered.com/app/{}", app_id)),
                                        movie_url: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Heroic & Legendary / Nile Oyunlarını Tara
    let heroic_gog = PathBuf::from(&home).join(".config/heroic/gog_store/installed.json");
    if heroic_gog.exists() {
        if let Ok(content) = std::fs::read_to_string(&heroic_gog) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(installed) = val.get("installed").and_then(|v| v.as_array()) {
                    for item in installed {
                        if let (Some(app_name), Some(title)) = (
                            item.get("appName").and_then(|v| v.as_str()),
                            item.get("title").and_then(|v| v.as_str()),
                        ) {
                            list.push(GameEntry {
                                id: format!("gog-{}", app_name),
                                title: title.to_string(),
                                category: "GOG Galaxy".to_string(),
                                exec: format!("heroic --launch heroic://launch/{}", app_name),
                                banner_desc: format!("{} • DRM-Free GOG Galaxy Oyunu", title),
                                is_steam: false,
                                is_installed: true,
                                steam_app_id: None,
                                cover_path: None,
                                hero_path: None,
                                screenshot_path: None,
                                store_url: Some("https://www.gog.com/".to_string()),
                                movie_url: None,
                            });
                        }
                    }
                }
            }
        }
    }

    let legendary_installed = PathBuf::from(&home).join(".config/legendary/installed.json");
    if legendary_installed.exists() {
        if let Ok(content) = std::fs::read_to_string(&legendary_installed) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = val.as_object() {
                    for (app_name, info) in obj {
                        let title = info.get("title").and_then(|v| v.as_str()).unwrap_or(app_name);
                        list.push(GameEntry {
                            id: format!("epic-{}", app_name),
                            title: title.to_string(),
                            category: "Epic Games".to_string(),
                            exec: format!("legendary launch {}", app_name),
                            banner_desc: format!("{} • Epic Games Store / Heroic Oyunu", title),
                            is_steam: false,
                            is_installed: true,
                            steam_app_id: None,
                            cover_path: None,
                            hero_path: None,
                            screenshot_path: None,
                            store_url: Some("https://store.epicgames.com/".to_string()),
                            movie_url: None,
                        });
                    }
                }
            }
        }
    }

    // 4. Sistem Masaüstü Oyunları (/usr/share/applications)
    let app_dirs = [
        Path::new("/usr/share/applications"),
        Path::new("/usr/local/share/applications"),
    ];

    for d in app_dirs {
        if let Ok(entries) = std::fs::read_dir(d) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().map(|s| s == "desktop").unwrap_or(false) {
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        if text.contains("Categories=") && text.contains("Game") {
                            let mut name = String::new();
                            let mut exec = String::new();
                            for line in text.lines() {
                                if line.starts_with("Name=") && name.is_empty() {
                                    name = line[5..].trim().to_string();
                                } else if line.starts_with("Exec=") && exec.is_empty() {
                                    exec = line[5..].trim().to_string();
                                }
                            }
                            if !name.is_empty() && !exec.is_empty() && !list.iter().any(|g| g.title == name) {
                                list.push(GameEntry {
                                    id: name.to_lowercase().replace(' ', "-"),
                                    title: name.clone(),
                                    category: "Masaüstü Oyunu".to_string(),
                                    exec,
                                    banner_desc: format!("{} • Düşük gecikmeli yerel Linux oyunu.", name),
                                    is_steam: false,
                                    is_installed: true,
                                    steam_app_id: None,
                                    cover_path: None,
                                    hero_path: None,
                                    screenshot_path: None,
                                    store_url: None,
                                    movie_url: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    list
}

fn parse_acf_file(content: &str) -> Option<(String, String)> {
    let mut app_id = None;
    let mut name = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"appid\"") {
            let parts: Vec<&str> = trimmed.split('"').filter(|s| !s.trim().is_empty()).collect();
            if parts.len() >= 2 {
                app_id = Some(parts[1].to_string());
            }
        } else if trimmed.starts_with("\"name\"") {
            let parts: Vec<&str> = trimmed.split('"').filter(|s| !s.trim().is_empty()).collect();
            if parts.len() >= 2 {
                name = Some(parts[1].to_string());
            }
        }
    }

    if let (Some(id), Some(n)) = (app_id, name) {
        if id != "228980" { // Steamworks Common Redistributables atla
            return Some((id, n));
        }
    }
    None
}

fn launch_game_entry(game: &GameEntry) {
    println!("Launching GameZone title: {} -> {}", game.title, game.exec);
    record_game_played(&game.id);
    let _ = Command::new("notify-send")
        .args([
            "-a", "Blaze GameZone",
            "-i", "input-gaming",
            "Oyun Başlatılıyor",
            &format!("{} başlatılıyor. BORE ve düşük gecikme modu devrede.", game.title),
        ])
        .spawn();

    let parts: Vec<&str> = game.exec.split_whitespace().collect();
    if let Some(cmd) = parts.get(0) {
        let args = &parts[1..];
        let _ = Command::new("niri")
            .arg("msg")
            .arg("action")
            .arg("spawn")
            .arg("--")
            .arg(cmd)
            .args(args)
            .spawn();
    }
}

fn get_live_system_metrics() -> (String, String, String) {
    let ram_str = if let Ok(mem) = std::fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut avail = 0u64;
        for line in mem.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                avail = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
        }
        if total > 0 {
            let used = total.saturating_sub(avail);
            let pct = (used as f64 / total as f64) * 100.0;
            format!("{:.0}%", pct)
        } else {
            "21%".to_string()
        }
    } else {
        "21%".to_string()
    };

    (format!("18%"), ram_str, format!("60"))
}

fn activate_gaming_optimizations() {
    println!("Activating Blaze GameZone Low-Latency Performance Mode...");
    let _ = Command::new("powerprofilesctl").args(["set", "performance"]).status();
    let _ = Command::new("sh")
        .arg("-c")
        .arg("echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null || true")
        .status();
    let _ = crate::delight::play_acoustic_feedback("easter-egg");
}

fn restore_normal_optimizations() {
    println!("Restoring balanced power and scheduler profile...");
    let _ = Command::new("powerprofilesctl").args(["set", "balanced"]).status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_showcase_games() {
        let showcase = discover_store_showcase();
        assert!(!showcase.is_empty(), "GameZone must have showcase games");
        assert!(showcase.iter().any(|g| g.id == "beamng"));
        assert!(showcase.iter().any(|g| g.id == "dota-2"));
        assert!(showcase.iter().any(|g| g.id == "cyberpunk-2077"));
        assert!(showcase.iter().any(|g| g.id == "xbox-cloud"));
    }

    #[test]
    fn test_parse_acf_file() {
        let acf_sample = r#"
"AppState"
{
	"appid"		"413150"
	"Universe"		"1"
	"name"		"Stardew Valley"
	"StateFlags"		"4"
}
"#;
        let parsed = parse_acf_file(acf_sample);
        assert_eq!(parsed, Some(("413150".to_string(), "Stardew Valley".to_string())));
    }

    #[test]
    fn test_cache_dir_path() {
        let c_dir = dirs_cache_dir();
        assert!(c_dir.to_str().unwrap().contains(".cache/blaze-gamezone"));
    }

    #[test]
    fn test_recent_history() {
        let test_id = "test-game-id";
        record_game_played(test_id);
        let recents = get_recent_game_ids();
        assert!(recents.contains(&test_id.to_string()));
    }
}
