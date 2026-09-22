use cairo::Context;
use chrono::Local;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{CssProvider, DrawingArea, Window};
use pangocairo::pango::FontDescription;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PixelClockStyle {
    BoldHorizontal,
    StackedVertical,
    Outline,
    AnalogBold,
    AnalogOutline,
    AnalogTachymeter,
}

impl Default for PixelClockStyle {
    fn default() -> Self {
        Self::BoldHorizontal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelClockConfig {
    pub enabled: bool,
    pub style: PixelClockStyle,
    pub display_style: String,    // "horizontal" or "vertical"
    pub color_mode: String,       // "dynamic" or "custom"
    pub custom_color: String,     // "#80d4ff"
    pub numeral_opacity: f64,     // 0.2 .. 1.0
    pub background_spacing: i32,  // 10 .. 40
    pub show_battery_pill: bool,
    pub show_date: bool,
    pub show_seconds: bool,
}

impl Default for PixelClockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            style: PixelClockStyle::BoldHorizontal,
            display_style: "horizontal".to_string(),
            color_mode: "dynamic".to_string(),
            custom_color: "#80d4ff".to_string(),
            numeral_opacity: 0.95,
            background_spacing: 16,
            show_battery_pill: true,
            show_date: true,
            show_seconds: false,
        }
    }
}

impl PixelClockConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/darkmorpheus".to_string());
        PathBuf::from(home).join(".config/solarui/pixel_clock.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }
        let def = Self::default();
        let _ = def.save();
        def
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

fn get_battery_info() -> Option<(u32, bool)> {
    let cap_str = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .or_else(|_| std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
        .ok()?;
    let capacity: u32 = cap_str.trim().parse().ok()?;
    let status_str = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .or_else(|_| std::fs::read_to_string("/sys/class/power_supply/BAT1/status"))
        .unwrap_or_default();
    let is_charging = status_str.to_lowercase().contains("charging");
    Some((capacity, is_charging))
}

fn parse_hex_color(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(212) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f64 / 255.0;
        (r, g, b)
    } else {
        (0.5, 0.83, 1.0)
    }
}

fn get_accent_color(cfg: &PixelClockConfig) -> (f64, f64, f64) {
    if cfg.color_mode == "custom" {
        parse_hex_color(&cfg.custom_color)
    } else {
        let theme = solar_common::M3Theme::default();
        parse_hex_color(&theme.primary)
    }
}

pub fn launch_pixel_clock() {
    glib::set_prgname(Some("solar-clock"));
    glib::set_application_name("SolarUI Google Pixel Clock");

    if let Err(err) = gtk4::init() {
        eprintln!("Failed to initialize GTK4 for Pixel Clock: {}", err);
        return;
    }

    let config = Arc::new(Mutex::new(PixelClockConfig::load()));
    let window = Window::new();
    window.set_title(Some("SolarUI Pixel Clock"));
    window.set_default_size(440, 260);
    window.set_decorated(false);

    // Apply transparent / frameless CSS
    let provider = CssProvider::new();
    provider.load_from_string(
        "window, window.background, .background, drawingarea {
            background-color: transparent;
            background-image: none;
            box-shadow: none;
            border: none;
         }",
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("No display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );

    let drawing_area = DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    let click = gtk4::GestureClick::new();
    click.set_button(0);
    click.connect_pressed(|gesture, n_press, _x, _y| {
        if gesture.current_button() == 3 || n_press == 2 {
            let _ = std::process::Command::new("solar-shell").arg("settings").spawn();
        }
    });
    drawing_area.add_controller(click);

    let cfg_clone = config.clone();
    drawing_area.set_draw_func(move |_area, cr, width, height| {
        cr.set_operator(cairo::Operator::Clear);
        let _ = cr.paint();
        cr.set_operator(cairo::Operator::Over);

        let cfg = {
            let guard = cfg_clone.lock().unwrap();
            guard.clone()
        };
        draw_clock(&cfg, cr, width as f64, height as f64);
    });

    // Update once per second
    let area_weak = drawing_area.downgrade();
    let cfg_reload = config.clone();
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        if let Some(area) = area_weak.upgrade() {
            // Hot-reload config if changed on disk
            let on_disk = PixelClockConfig::load();
            {
                let mut guard = cfg_reload.lock().unwrap();
                *guard = on_disk;
            }
            area.queue_draw();
            glib::ControlFlow::Continue
        } else {
            glib::ControlFlow::Break
        }
    });

    window.set_child(Some(&drawing_area));
    window.present();

    let main_loop = glib::MainLoop::new(None, false);
    main_loop.run();
}

fn draw_clock(cfg: &PixelClockConfig, cr: &Context, w: f64, h: f64) {
    if !cfg.enabled {
        return;
    }

    let now = Local::now();
    let (r, g, b) = get_accent_color(cfg);
    let alpha = cfg.numeral_opacity.clamp(0.1, 1.0);

    match cfg.style {
        PixelClockStyle::BoldHorizontal => {
            draw_bold_horizontal(cfg, cr, w, h, &now, r, g, b, alpha);
        }
        PixelClockStyle::StackedVertical => {
            draw_stacked_vertical(cfg, cr, w, h, &now, r, g, b, alpha);
        }
        PixelClockStyle::Outline => {
            draw_outline(cfg, cr, w, h, &now, r, g, b, alpha);
        }
        PixelClockStyle::AnalogBold => {
            draw_analog(cfg, cr, w, h, &now, r, g, b, alpha, false, false);
        }
        PixelClockStyle::AnalogOutline => {
            draw_analog(cfg, cr, w, h, &now, r, g, b, alpha, true, false);
        }
        PixelClockStyle::AnalogTachymeter => {
            draw_analog(cfg, cr, w, h, &now, r, g, b, alpha, false, true);
        }
    }
}

fn draw_bold_horizontal(
    cfg: &PixelClockConfig,
    cr: &Context,
    w: f64,
    h: f64,
    now: &chrono::DateTime<Local>,
    r: f64,
    g: f64,
    b: f64,
    alpha: f64,
) {
    let time_str = now.format("%H:%M").to_string();

    let layout = pangocairo::functions::create_layout(cr);
    let mut font = FontDescription::from_string("Fredoka Bold 64");
    layout.set_font_description(Some(&mut font));
    layout.set_text(&time_str);

    let (tw, th) = layout.pixel_size();
    let x = (w - tw as f64) / 2.0;
    let y = (h - th as f64) / 2.0 - 18.0;

    // Soft drop shadow
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.35);
    cr.move_to(x + 2.0, y + 2.0);
    pangocairo::functions::show_layout(cr, &layout);

    // Main playful digits
    cr.set_source_rgba(r, g, b, alpha);
    cr.move_to(x, y);
    pangocairo::functions::show_layout(cr, &layout);

    // Sub-row: Battery Pill & Date
    let sub_y = y + th as f64 + 6.0;

    let bat_info = if cfg.show_battery_pill {
        get_battery_info()
    } else {
        None
    };

    let date_str = if cfg.show_date {
        now.format("%A, %d %B").to_string()
    } else {
        String::new()
    };

    let mut start_x = x;
    if let Some((cap, charging)) = bat_info {
        let icon = if charging { "⚡" } else { "🔋" };
        let pill_text = format!("{} {}%", icon, cap);

        let pill_layout = pangocairo::functions::create_layout(cr);
        let mut pill_font = FontDescription::from_string("Fredoka SemiBold 13");
        pill_layout.set_font_description(Some(&mut pill_font));
        pill_layout.set_text(&pill_text);

        let (pw, ph) = pill_layout.pixel_size();
        let pill_w = pw as f64 + 18.0;
        let pill_h = ph as f64 + 8.0;

        // Draw pill capsule
        draw_rounded_rect(cr, start_x, sub_y, pill_w, pill_h, pill_h / 2.0);
        cr.set_source_rgba(r, g, b, 0.22);
        let _ = cr.fill_preserve();
        cr.set_source_rgba(r, g, b, 0.5);
        cr.set_line_width(1.2);
        let _ = cr.stroke();

        // Text inside pill
        cr.set_source_rgba(r, g, b, 0.95);
        cr.move_to(start_x + 9.0, sub_y + 4.0);
        pangocairo::functions::show_layout(cr, &pill_layout);

        start_x += pill_w + 12.0;
    }

    if !date_str.is_empty() {
        let date_layout = pangocairo::functions::create_layout(cr);
        let mut date_font = FontDescription::from_string("Fredoka Medium 14");
        date_layout.set_font_description(Some(&mut date_font));
        date_layout.set_text(&date_str);

        cr.set_source_rgba(r, g, b, alpha * 0.85);
        cr.move_to(start_x, sub_y + 3.0);
        pangocairo::functions::show_layout(cr, &date_layout);
    }
}

fn draw_stacked_vertical(
    cfg: &PixelClockConfig,
    cr: &Context,
    w: f64,
    h: f64,
    now: &chrono::DateTime<Local>,
    r: f64,
    g: f64,
    b: f64,
    alpha: f64,
) {
    let hour_str = now.format("%H").to_string();
    let min_str = now.format("%M").to_string();

    let layout_h = pangocairo::functions::create_layout(cr);
    let mut font_h = FontDescription::from_string("Fredoka Bold 58");
    layout_h.set_font_description(Some(&mut font_h));
    layout_h.set_text(&hour_str);

    let layout_m = pangocairo::functions::create_layout(cr);
    let mut font_m = FontDescription::from_string("Fredoka Bold 58");
    layout_m.set_font_description(Some(&mut font_m));
    layout_m.set_text(&min_str);

    let (tw_h, th_h) = layout_h.pixel_size();
    let (tw_m, th_m) = layout_m.pixel_size();

    let max_tw = (tw_h as f64).max(tw_m as f64);
    let total_th = th_h as f64 + th_m as f64 - 16.0;

    let x = (w - max_tw) / 2.0;
    let y_h = (h - total_th) / 2.0 - 10.0;
    let y_m = y_h + th_h as f64 - 16.0;

    // Hour
    cr.set_source_rgba(r, g, b, alpha);
    cr.move_to(x + (max_tw - tw_h as f64) / 2.0, y_h);
    pangocairo::functions::show_layout(cr, &layout_h);

    // Minute
    cr.set_source_rgba(r, g, b, alpha * 0.88);
    cr.move_to(x + (max_tw - tw_m as f64) / 2.0, y_m);
    pangocairo::functions::show_layout(cr, &layout_m);

    // Battery pill alongside
    if cfg.show_battery_pill {
        if let Some((cap, charging)) = get_battery_info() {
            let icon = if charging { "⚡" } else { "🔋" };
            let pill_text = format!("{} {}%", icon, cap);

            let pill_layout = pangocairo::functions::create_layout(cr);
            let mut pill_font = FontDescription::from_string("Fredoka SemiBold 12");
            pill_layout.set_font_description(Some(&mut pill_font));
            pill_layout.set_text(&pill_text);

            let (pw, ph) = pill_layout.pixel_size();
            let pill_w = pw as f64 + 14.0;
            let pill_h = ph as f64 + 6.0;

            let pill_x = x + max_tw + 16.0;
            let pill_y = y_h + 8.0;

            draw_rounded_rect(cr, pill_x, pill_y, pill_w, pill_h, pill_h / 2.0);
            cr.set_source_rgba(r, g, b, 0.22);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(r, g, b, 0.5);
            cr.set_line_width(1.0);
            let _ = cr.stroke();

            cr.set_source_rgba(r, g, b, 0.95);
            cr.move_to(pill_x + 7.0, pill_y + 3.0);
            pangocairo::functions::show_layout(cr, &pill_layout);
        }
    }
}

fn draw_outline(
    _cfg: &PixelClockConfig,
    cr: &Context,
    w: f64,
    h: f64,
    now: &chrono::DateTime<Local>,
    r: f64,
    g: f64,
    b: f64,
    alpha: f64,
) {
    let time_str = now.format("%H:%M").to_string();

    let layout = pangocairo::functions::create_layout(cr);
    let mut font = FontDescription::from_string("Fredoka Bold 64");
    layout.set_font_description(Some(&mut font));
    layout.set_text(&time_str);

    let (tw, th) = layout.pixel_size();
    let x = (w - tw as f64) / 2.0;
    let y = (h - th as f64) / 2.0 - 10.0;

    cr.move_to(x, y);
    pangocairo::functions::layout_path(cr, &layout);

    cr.set_source_rgba(r, g, b, alpha);
    cr.set_line_width(2.8);
    let _ = cr.stroke();
}

fn draw_analog(
    _cfg: &PixelClockConfig,
    cr: &Context,
    w: f64,
    h: f64,
    now: &chrono::DateTime<Local>,
    r: f64,
    g: f64,
    b: f64,
    alpha: f64,
    outline_mode: bool,
    tachymeter_mode: bool,
) {
    let cx = w / 2.0;
    let cy = h / 2.0;
    let radius = (w.min(h) / 2.0) - 16.0;

    if !tachymeter_mode {
        draw_organic_blob(cr, cx, cy, radius, 12, 0.08);
        cr.set_source_rgba(r, g, b, 0.12);
        let _ = cr.fill_preserve();
        cr.set_source_rgba(r, g, b, if outline_mode { 0.5 } else { 0.25 });
        cr.set_line_width(1.8);
        let _ = cr.stroke();
    } else {
        cr.arc(cx, cy, radius, 0.0, 2.0 * PI);
        cr.set_source_rgba(r, g, b, 0.15);
        let _ = cr.fill_preserve();
        cr.set_source_rgba(r, g, b, 0.6);
        cr.set_line_width(2.2);
        let _ = cr.stroke();

        for i in 0..60 {
            let angle = (i as f64) * (2.0 * PI / 60.0) - PI / 2.0;
            let is_major = i % 5 == 0;
            let tick_len = if is_major { 10.0 } else { 5.0 };
            let r_outer = radius - 3.0;
            let r_inner = r_outer - tick_len;

            let x1 = cx + r_outer * angle.cos();
            let y1 = cy + r_outer * angle.sin();
            let x2 = cx + r_inner * angle.cos();
            let y2 = cy + r_inner * angle.sin();

            cr.move_to(x1, y1);
            cr.line_to(x2, y2);
            cr.set_source_rgba(r, g, b, if is_major { 0.8 } else { 0.35 });
            cr.set_line_width(if is_major { 1.8 } else { 1.0 });
            let _ = cr.stroke();
        }
    }

    let numerals = [("12", 0.0), ("3", PI / 2.0), ("6", PI), ("9", 3.0 * PI / 2.0)];
    let num_dist = radius * 0.68;

    for (num_str, angle) in numerals {
        let nx = cx + num_dist * angle.sin();
        let ny = cy - num_dist * angle.cos();

        let layout = pangocairo::functions::create_layout(cr);
        let mut font = FontDescription::from_string("Fredoka Bold 22");
        layout.set_font_description(Some(&mut font));
        layout.set_text(num_str);

        let (tw, th) = layout.pixel_size();
        let tx = nx - tw as f64 / 2.0;
        let ty = ny - th as f64 / 2.0;

        if outline_mode {
            cr.move_to(tx, ty);
            pangocairo::functions::layout_path(cr, &layout);
            cr.set_source_rgba(r, g, b, alpha);
            cr.set_line_width(1.8);
            let _ = cr.stroke();
        } else {
            cr.set_source_rgba(r, g, b, alpha);
            cr.move_to(tx, ty);
            pangocairo::functions::show_layout(cr, &layout);
        }
    }

    let sec = now.timestamp_subsec_millis() as f64 / 1000.0 + now.format("%S").to_string().parse::<f64>().unwrap_or(0.0);
    let min = now.format("%M").to_string().parse::<f64>().unwrap_or(0.0) + sec / 60.0;
    let hour = (now.format("%I").to_string().parse::<f64>().unwrap_or(0.0) % 12.0) + min / 60.0;

    let hour_angle = hour * (2.0 * PI / 12.0) - PI / 2.0;
    let min_angle = min * (2.0 * PI / 60.0) - PI / 2.0;
    let sec_angle = sec * (2.0 * PI / 60.0) - PI / 2.0;

    let hour_len = radius * 0.48;
    cr.set_line_cap(cairo::LineCap::Round);
    cr.move_to(cx, cy);
    cr.line_to(cx + hour_len * hour_angle.cos(), cy + hour_len * hour_angle.sin());
    cr.set_source_rgba(r, g, b, 0.95);
    cr.set_line_width(8.0);
    let _ = cr.stroke();

    let min_len = radius * 0.72;
    cr.move_to(cx, cy);
    cr.line_to(cx + min_len * min_angle.cos(), cy + min_len * min_angle.sin());
    cr.set_source_rgba(r, g, b, 0.85);
    cr.set_line_width(5.5);
    let _ = cr.stroke();

    let sec_len = radius * 0.82;
    cr.move_to(cx, cy);
    cr.line_to(cx + sec_len * sec_angle.cos(), cy + sec_len * sec_angle.sin());
    cr.set_source_rgba(1.0, 0.45, 0.6, 0.9);
    cr.set_line_width(2.0);
    let _ = cr.stroke();

    cr.arc(cx, cy, 6.0, 0.0, 2.0 * PI);
    cr.set_source_rgba(r, g, b, 1.0);
    let _ = cr.fill();
    cr.arc(cx, cy, 2.5, 0.0, 2.0 * PI);
    cr.set_source_rgba(0.1, 0.12, 0.14, 1.0);
    let _ = cr.fill();
}

fn draw_rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    cr.close_path();
}

fn draw_organic_blob(cr: &Context, cx: f64, cy: f64, base_r: f64, lobes: usize, amount: f64) {
    let steps = 120;
    cr.new_sub_path();
    for i in 0..=steps {
        let angle = (i as f64) * (2.0 * PI / steps as f64);
        let wave = (angle * lobes as f64).cos();
        let r = base_r * (1.0 + amount * wave);
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        if i == 0 {
            cr.move_to(x, y);
        } else {
            cr.line_to(x, y);
        }
    }
    cr.close_path();
}
