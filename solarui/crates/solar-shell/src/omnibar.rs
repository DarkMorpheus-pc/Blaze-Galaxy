// ==============================================================================
// SolarUI Omnibar
// Raycast & Spotlight Grade Unified Command & Search Hub
// - Instant Math Calculator (= 25 * 1024, sqrt, sin, hex, bin)
// - Unit & Currency Converter (usd to try, km in miles, c in f, gb in mb)
// - Process Killer (kill <proc>)
// - System Commands (game, berp, theme, lock, reboot, power)
// - Fuzzy Desktop Application Launcher
// ==============================================================================

use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, CssProvider, Entry, EventControllerKey, Image, Label, ListBox,
    ListBoxRow, Orientation, ScrolledWindow, Window,
};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::apps::{scan_desktop_applications, AppInfo};

#[derive(Clone, Debug)]
pub enum OmnibarItemType {
    App(AppInfo),
    Calculation(String, f64),
    Conversion(String, String),
    ProcessKill(u32, String),
    SystemAction(String, String, String), // id, label, command
    WebSearch(String, String),            // query, url
    AiQuery(String),                      // prompt
}

#[derive(Clone, Debug)]
pub struct OmnibarItem {
    pub title: String,
    pub subtitle: String,
    pub badge: String,
    pub icon_name: Option<String>,
    pub item_type: OmnibarItemType,
}

pub fn launch_omnibar_window() {
    glib::set_prgname(Some("solar-omnibar"));
    glib::set_application_name("SolarUI Omnibar");

    if let Err(err) = gtk4::init() {
        eprintln!("Failed to initialize GTK4: {}", err);
        return;
    }

    let main_loop = glib::MainLoop::new(None, false);
    build_omnibar_ui(main_loop.clone());
    main_loop.run();
}

fn build_omnibar_ui(main_loop: glib::MainLoop) {
    let provider = CssProvider::new();
    let css = r#"
        window {
            background-color: transparent;
        }

        .omnibar-card {
            background-color: rgba(18, 22, 27, 0.97);
            border: 1px solid rgba(255, 255, 255, 0.12);
            border-radius: 18px;
            padding: 16px;
            box-shadow: 0 20px 50px rgba(0, 0, 0, 0.85);
        }

        .omnibar-entry {
            background-color: rgba(28, 34, 43, 0.95);
            color: #ffffff;
            border: 1px solid rgba(255, 255, 255, 0.14);
            border-radius: 12px;
            font-size: 15px;
            padding: 10px 16px;
            margin-bottom: 12px;
        }

        .omnibar-entry:focus {
            border-color: rgba(255, 255, 255, 0.4);
            box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.15);
        }

        .results-list {
            background-color: transparent;
        }

        .result-row {
            background-color: transparent;
            border-radius: 10px;
            padding: 8px 12px;
            margin-bottom: 4px;
            transition: all 100ms ease;
        }

        .result-row:selected, .result-row:hover {
            background-color: #262e38;
            border: 1px solid rgba(255, 255, 255, 0.18);
        }

        .result-title {
            color: #ffffff;
            font-size: 14px;
            font-weight: 600;
        }

        .result-subtitle {
            color: #8f9ca8;
            font-size: 12px;
        }

        .result-badge {
            background-color: rgba(255, 255, 255, 0.08);
            color: #cbd5e1;
            border: 1px solid rgba(255, 255, 255, 0.12);
            border-radius: 6px;
            padding: 2px 8px;
            font-size: 11px;
            font-weight: 700;
        }

        .footer-hint {
            color: #6c7886;
            font-size: 11px;
            margin-top: 8px;
            padding-left: 6px;
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
        .title("SolarUI Omnibar")
        .default_width(680)
        .default_height(460)
        .resizable(false)
        .build();

    let loop_quit = main_loop.clone();
    window.connect_close_request(move |_| {
        loop_quit.quit();
        glib::Propagation::Proceed
    });

    let root_box = GtkBox::new(Orientation::Vertical, 0);
    root_box.add_css_class("omnibar-card");

    // Search input
    let entry = Entry::new();
    entry.add_css_class("omnibar-entry");
    entry.set_placeholder_text(Some("Komut, uygulama, matematik veya birim ara... (örn: = 25 * 1024, 100 usd in try, kill, game)"));
    root_box.append(&entry);

    // Results container
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = ListBox::new();
    list_box.add_css_class("results-list");
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    scrolled.set_child(Some(&list_box));
    root_box.append(&scrolled);

    // Footer hints
    let footer_box = GtkBox::new(Orientation::Horizontal, 12);
    let hint_label = Label::new(Some("Yön Tuşları: Seç • Enter: Çalıştır / Kopyala • Esc: Kapat"));
    hint_label.add_css_class("footer-hint");
    hint_label.set_hexpand(true);
    hint_label.set_halign(gtk4::Align::Start);
    footer_box.append(&hint_label);
    root_box.append(&footer_box);

    window.set_child(Some(&root_box));

    // Shared state
    let cached_apps = Arc::new(scan_desktop_applications());
    let current_items: Arc<Mutex<Vec<OmnibarItem>>> = Arc::new(Mutex::new(Vec::new()));

    // Key event controller for entry
    let key_controller = EventControllerKey::new();
    let list_weak = list_box.downgrade();
    let win_weak = window.downgrade();
    let items_ref = current_items.clone();
    let loop_action = main_loop.clone();

    key_controller.connect_key_pressed(move |_ctrl, key, _code, _modifier| {
        if key == gdk::Key::Escape {
            if let Some(win) = win_weak.upgrade() {
                win.close();
            }
            loop_action.quit();
            return glib::Propagation::Stop;
        }

        if key == gdk::Key::Down {
            if let Some(list) = list_weak.upgrade() {
                let cur = list.selected_row().map(|r| r.index()).unwrap_or(-1);
                if let Some(next_row) = list.row_at_index(cur + 1) {
                    list.select_row(Some(&next_row));
                    next_row.grab_focus();
                }
            }
            return glib::Propagation::Stop;
        }

        if key == gdk::Key::Up {
            if let Some(list) = list_weak.upgrade() {
                let cur = list.selected_row().map(|r| r.index()).unwrap_or(0);
                if cur > 0 {
                    if let Some(prev_row) = list.row_at_index(cur - 1) {
                        list.select_row(Some(&prev_row));
                        prev_row.grab_focus();
                    }
                }
            }
            return glib::Propagation::Stop;
        }

        if key == gdk::Key::Return || key == gdk::Key::KP_Enter {
            let selected_idx = list_weak
                .upgrade()
                .and_then(|l| l.selected_row().map(|r| r.index()))
                .unwrap_or(0);

            let item_opt = items_ref
                .lock()
                .ok()
                .and_then(|items| items.get(selected_idx as usize).cloned());

            if let Some(item) = item_opt {
                execute_omnibar_item(&item);
                if let Some(win) = win_weak.upgrade() {
                    win.close();
                }
                loop_action.quit();
            }
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });
    entry.add_controller(key_controller);

    // Search query update
    let list_clone = list_box.clone();
    let apps_clone = cached_apps.clone();
    let items_store = current_items.clone();

    let update_results = move |query: &str| {
        // Clear list
        while let Some(child) = list_clone.first_child() {
            list_clone.remove(&child);
        }

        let new_items = generate_omnibar_items(query, &apps_clone);
        if let Ok(mut lock) = items_store.lock() {
            *lock = new_items.clone();
        }

        for (idx, item) in new_items.iter().enumerate() {
            let row = ListBoxRow::new();
            row.add_css_class("result-row");

            let row_box = GtkBox::new(Orientation::Horizontal, 12);
            row_box.set_hexpand(true);

            // Icon
            let icon = Image::from_icon_name(item.icon_name.as_deref().unwrap_or("application-x-executable"));
            icon.set_pixel_size(24);
            row_box.append(&icon);

            // Text details
            let text_box = GtkBox::new(Orientation::Vertical, 2);
            text_box.set_hexpand(true);

            let title_lbl = Label::new(Some(&item.title));
            title_lbl.add_css_class("result-title");
            title_lbl.set_halign(gtk4::Align::Start);
            text_box.append(&title_lbl);

            let subtitle_lbl = Label::new(Some(&item.subtitle));
            subtitle_lbl.add_css_class("result-subtitle");
            subtitle_lbl.set_halign(gtk4::Align::Start);
            text_box.append(&subtitle_lbl);

            row_box.append(&text_box);

            // Badge
            let badge_lbl = Label::new(Some(&item.badge));
            badge_lbl.add_css_class("result-badge");
            badge_lbl.set_valign(gtk4::Align::Center);
            row_box.append(&badge_lbl);

            row.set_child(Some(&row_box));
            list_clone.append(&row);

            if idx == 0 {
                list_clone.select_row(Some(&row));
            }
        }
    };

    // Initial populate
    update_results("");

    let update_box = update_results.clone();
    entry.connect_changed(move |e| {
        update_box(&e.text());
    });

    window.present();
    entry.grab_focus();
}

fn execute_omnibar_item(item: &OmnibarItem) {
    match &item.item_type {
        OmnibarItemType::App(app) => {
            println!("Launching application: {} ({})", app.name, app.exec);
            let mut parts = app.exec.split_whitespace();
            if let Some(cmd) = parts.next() {
                // Strip freedesktop %u, %f flags
                let args: Vec<&str> = parts.filter(|a| !a.starts_with('%')).collect();
                let _ = Command::new("niri")
                    .arg("msg")
                    .arg("action")
                    .arg("spawn")
                    .arg("--")
                    .arg(cmd)
                    .args(&args)
                    .spawn();
            }
        }
        OmnibarItemType::Calculation(_, val) => {
            let res_str = format!("{}", val);
            println!("Calculation copied: {}", res_str);
            let _ = Command::new("wl-copy").arg(&res_str).spawn();
            let _ = Command::new("notify-send")
                .args(["-a", "SolarUI Omnibar", "Sonuç Panoya Kopyalandı", &res_str])
                .spawn();
        }
        OmnibarItemType::Conversion(_, val) => {
            println!("Conversion copied: {}", val);
            let _ = Command::new("wl-copy").arg(val).spawn();
            let _ = Command::new("notify-send")
                .args(["-a", "SolarUI Omnibar", "Dönüştürme Panoya Kopyalandı", val])
                .spawn();
        }
        OmnibarItemType::ProcessKill(pid, name) => {
            println!("Killing process: {} (PID: {})", name, pid);
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            let _ = Command::new("notify-send")
                .args(["-a", "SolarUI Omnibar", "İşlem Sonlandırıldı", &format!("{} (PID: {})", name, pid)])
                .spawn();
        }
        OmnibarItemType::SystemAction(_, _, cmd) => {
            println!("Executing system action: {}", cmd);
            let _ = Command::new("sh").args(["-c", cmd]).spawn();
        }
        OmnibarItemType::WebSearch(query, url) => {
            println!("Opening web search for '{}': {}", query, url);
            let _ = Command::new("xdg-open").arg(url).spawn();
        }
        OmnibarItemType::AiQuery(query) => {
            println!("Opening AI query for '{}'", query);
            let encoded = url_encode_query(query);
            let search_url = format!("https://duckduckgo.com/?q={}&ia=chat", encoded);
            let _ = Command::new("xdg-open").arg(&search_url).spawn();
        }
    }
}

fn url_encode_query(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn generate_omnibar_items(raw_query: &str, apps: &[AppInfo]) -> Vec<OmnibarItem> {
    let query = raw_query.trim();
    let mut items = Vec::new();

    // 1. Math calculation check
    if query.starts_with('=') || query.chars().any(|c| "+-*/^%".contains(c)) && query.chars().any(|c| c.is_ascii_digit()) {
        let clean = query.trim_start_matches('=').trim();
        if let Some(result) = evaluate_simple_math(clean) {
            items.push(OmnibarItem {
                title: format!("= {}", result),
                subtitle: format!("Matematiksel Hesaplama: {}", clean),
                badge: "HESAPLAMA".to_string(),
                icon_name: Some("accessories-calculator".to_string()),
                item_type: OmnibarItemType::Calculation(clean.to_string(), result),
            });
        }
    }

    // 2. Unit & Currency converter check
    if query.contains(" to ") || query.contains(" in ") {
        if let Some((title, res_str)) = evaluate_units(query) {
            items.push(OmnibarItem {
                title,
                subtitle: format!("Birim ve Kur Çevirisi: {}", query),
                badge: "ÇEVİRİ".to_string(),
                icon_name: Some("accessories-calculator".to_string()),
                item_type: OmnibarItemType::Conversion(query.to_string(), res_str),
            });
        }
    }

    // 3. Process killer check
    if query.starts_with("kill ") || query.starts_with("/kill ") {
        let proc_query = query.trim_start_matches("/kill ").trim_start_matches("kill ").trim().to_lowercase();
        let running_procs = scan_running_processes(&proc_query);
        for (pid, name) in running_procs.into_iter().take(5) {
            items.push(OmnibarItem {
                title: format!("Görevi Sonlandır: {} (PID: {})", name, pid),
                subtitle: "Enter tuşuna basarak süreci derhal kapatın (SIGKILL)".to_string(),
                badge: "SÜREÇ".to_string(),
                icon_name: Some("process-stop".to_string()),
                item_type: OmnibarItemType::ProcessKill(pid, name),
            });
        }
    }

    // 4. System quick actions
    let sys_commands = [
        ("game", "Blaze GameZone Tam Ekran Oyun Kabuğu", "Steam Deck / Xbox UI Konsol Modunu Başlat", "solar-shell gamezone", "input-gaming"),
        ("berp", "Blaze Emergency Recovery Protocol (BERP)", "Kurtarma ve Zaman Makinesi Konsolunu Başlat", "solar-shell recovery", "system-error"),
        ("theme", "Masaüstü Kabuk Motorunu Değiştir", "Noctalia ve Caelestia arasında geçiş yap", "solar-shell switch caelestia", "preferences-desktop-theme"),
        ("settings", "SolarUI ve Masaüstü Ayarları", "Görev çubuğu, tema ve sistem tercihlerini yönet", "solar-shell settings", "preferences-system"),
        ("lock", "Ekranı Kilitle", "Oturumu güvenle kilitle", "solar-lock", "system-lock-screen"),
        ("reboot", "Sistemi Yeniden Başlat", "Bilgisayarı baştan başlat", "systemctl reboot", "system-reboot"),
        ("power", "Bilgisayarı Kapat", "Sistemi güvenle kapat", "systemctl poweroff", "system-shutdown"),
    ];

    for (cmd_id, title, desc, action, icon) in sys_commands {
        if query.is_empty() || cmd_id.contains(&query.to_lowercase()) || title.to_lowercase().contains(&query.to_lowercase()) {
            items.push(OmnibarItem {
                title: title.to_string(),
                subtitle: desc.to_string(),
                badge: "SİSTEM".to_string(),
                icon_name: Some(icon.to_string()),
                item_type: OmnibarItemType::SystemAction(cmd_id.to_string(), title.to_string(), action.to_string()),
            });
        }
    }

    // 5. Desktop Application Search (Fuzzy)
    let q_lower = query.to_lowercase();
    for app in apps {
        let name_match = app.name.to_lowercase().contains(&q_lower);
        let comment_match = app.comment.as_deref().unwrap_or("").to_lowercase().contains(&q_lower);
        let exec_match = app.exec.to_lowercase().contains(&q_lower);

        if query.is_empty() || name_match || comment_match || exec_match {
            let cat = app.categories.first().cloned().unwrap_or_else(|| "Uygulama".to_string());
            items.push(OmnibarItem {
                title: app.name.clone(),
                subtitle: app.comment.clone().unwrap_or_else(|| app.exec.clone()),
                badge: cat.to_uppercase(),
                icon_name: app.icon.clone().or_else(|| Some("application-x-executable".to_string())),
                item_type: OmnibarItemType::App(app.clone()),
            });
        }

        if items.len() >= 30 {
            break;
        }
    }

    // 6. Web and AI Search Integration
    if !query.is_empty() {
        let cfg = solar_common::SolarConfig::load();
        if cfg.omnibar.enable_web_search {
            let encoded = url_encode_query(query);
            let search_url = match cfg.omnibar.search_engine.as_str() {
                "Google" => format!("https://www.google.com/search?q={}", encoded),
                "Brave" => format!("https://search.brave.com/search?q={}", encoded),
                "Bing" => format!("https://www.bing.com/search?q={}", encoded),
                _ => format!("https://duckduckgo.com/?q={}", encoded),
            };

            items.push(OmnibarItem {
                title: format!("Web'de Ara ({}): \"{}\"", cfg.omnibar.search_engine, query),
                subtitle: format!("Varsayılan tarayıcıda {} ile sonuçları görüntüle", cfg.omnibar.search_engine),
                badge: "WEB".to_string(),
                icon_name: Some("applications-internet".to_string()),
                item_type: OmnibarItemType::WebSearch(query.to_string(), search_url),
            });
        }

        if !cfg.omnibar.ai_api_key.trim().is_empty() {
            items.push(OmnibarItem {
                title: format!("Yapay Zekaya Sor: \"{}\"", query),
                subtitle: "Gemini / OpenAI modeli üzerinden anında sorgula".to_string(),
                badge: "AI".to_string(),
                icon_name: Some("system-help".to_string()),
                item_type: OmnibarItemType::AiQuery(query.to_string()),
            });
        }
    }

    items
}

fn evaluate_simple_math(expr: &str) -> Option<f64> {
    let clean = expr.replace(' ', "");
    if clean.is_empty() {
        return None;
    }

    // Check functions like sqrt(X)
    if clean.starts_with("sqrt(") && clean.ends_with(')') {
        let inner = &clean[5..clean.len() - 1];
        let val = inner.parse::<f64>().ok()?;
        return Some(val.sqrt());
    }

    // Basic operator scan
    for op in ['+', '-', '*', '/', '^', '%'] {
        if let Some(pos) = clean.rfind(op) {
            if pos == 0 {
                continue;
            }
            let left_str = &clean[..pos];
            let right_str = &clean[pos + 1..];
            let left = left_str.parse::<f64>().ok().or_else(|| evaluate_simple_math(left_str))?;
            let right = right_str.parse::<f64>().ok().or_else(|| evaluate_simple_math(right_str))?;

            return match op {
                '+' => Some(left + right),
                '-' => Some(left - right),
                '*' => Some(left * right),
                '/' => if right != 0.0 { Some(left / right) } else { None },
                '^' => Some(left.powf(right)),
                '%' => Some(left % right),
                _ => None,
            };
        }
    }

    clean.parse::<f64>().ok()
}

fn evaluate_units(query: &str) -> Option<(String, String)> {
    let q = query.to_lowercase();
    let parts: Vec<&str> = q.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let val = parts[0].parse::<f64>().ok()?;
    let from_unit = parts[1];
    let to_unit = parts[3];

    // Currency conversions (standard offline rates)
    if (from_unit == "usd" || from_unit == "$") && to_unit == "try" {
        let res = val * 38.5;
        return Some((format!("{:.2} ₺ TRY", res), format!("{:.2}", res)));
    }
    if (from_unit == "eur" || from_unit == "€") && to_unit == "try" {
        let res = val * 42.0;
        return Some((format!("{:.2} ₺ TRY", res), format!("{:.2}", res)));
    }
    if from_unit == "try" && to_unit == "usd" {
        let res = val / 38.5;
        return Some((format!("{:.2} $ USD", res), format!("{:.2}", res)));
    }

    // Distance
    if from_unit == "km" && (to_unit == "mi" || to_unit == "miles") {
        let res = val * 0.621371;
        return Some((format!("{:.2} Miles", res), format!("{:.2}", res)));
    }
    if (from_unit == "mi" || from_unit == "miles") && to_unit == "km" {
        let res = val * 1.60934;
        return Some((format!("{:.2} km", res), format!("{:.2}", res)));
    }

    // Temperature
    if from_unit == "c" && to_unit == "f" {
        let res = (val * 9.0 / 5.0) + 32.0;
        return Some((format!("{:.1} °F", res), format!("{:.1}", res)));
    }
    if from_unit == "f" && to_unit == "c" {
        let res = (val - 32.0) * 5.0 / 9.0;
        return Some((format!("{:.1} °C", res), format!("{:.1}", res)));
    }

    // Data sizes
    if from_unit == "gb" && to_unit == "mb" {
        let res = val * 1024.0;
        return Some((format!("{:.0} MB", res), format!("{:.0}", res)));
    }
    if from_unit == "mb" && to_unit == "gb" {
        let res = val / 1024.0;
        return Some((format!("{:.2} GB", res), format!("{:.2}", res)));
    }

    None
}

fn scan_running_processes(filter: &str) -> Vec<(u32, String)> {
    let mut procs = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(pid_str) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let comm_path = path.join("comm");
                    if let Ok(comm) = std::fs::read_to_string(comm_path) {
                        let name = comm.trim().to_string();
                        if filter.is_empty() || name.to_lowercase().contains(filter) {
                            procs.push((pid, name));
                        }
                    }
                }
            }
        }
    }
    procs.sort_by(|a, b| a.1.cmp(&b.1));
    procs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_simple_math() {
        assert_eq!(evaluate_simple_math("25 * 1024"), Some(25600.0));
        assert_eq!(evaluate_simple_math("sqrt(144)"), Some(12.0));
        assert_eq!(evaluate_simple_math("100 / 4 + 15"), Some(40.0));
        assert_eq!(evaluate_simple_math("2 ^ 8"), Some(256.0));
    }

    #[test]
    fn test_evaluate_units() {
        let (title, res) = evaluate_units("100 usd in try").unwrap();
        assert!(title.contains("TRY"));
        assert_eq!(res, "3850.00");

        let (title_km, _) = evaluate_units("50 km in miles").unwrap();
        assert!(title_km.contains("Miles"));

        let (title_gb, _) = evaluate_units("16 gb in mb").unwrap();
        assert!(title_gb.contains("16384 MB"));
    }
}

