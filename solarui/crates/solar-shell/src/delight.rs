// ==============================================================================
// SolarUI Delight & Atmosphere Engine
// Acoustic Feedback, Easter Eggs (Konami Code, CRT Mode) & Seasonal Dynamics
// ==============================================================================

use anyhow::Result;
use chrono::{Datelike, Local};
use std::process::Command;

pub fn trigger_easter_egg(egg_type: &str) -> Result<()> {
    match egg_type {
        "konami" | "crt" => {
            println!("[KONAMI] [KONAMI CODE DETECTED] Blaze SolarEvolution Retro Mode Activated!");

            // 1. Play retro acoustic chime
            let _ = play_acoustic_feedback("easter-egg");

            // 2. Send desktop notification
            let _ = Command::new("notify-send")
                .args([
                    "-a", "SolarUI Delight",
                    "-i", "input-gaming",
                    "-u", "critical",
                    "[KONAMI] Konami Kodu Devrede!",
                    "Tebrikler! Blaze SolarEvolution 80'ler Retro CRT & Matrix Modunu Keşfettiniz.\n(Yukari Yukari Asagi Asagi Sol Sag Sol Sag B A)",
                ])
                .status();

            // 3. If in terminal or GUI, display retro ASCII banner
            println!(r#"
  ____  _        _     ____________ ___  ____  
 | __ )| |      / \   |__  / ____/ / _ \/ ___| 
 |  _ \| |     / _ \    / /|  _|  | | | \___ \ 
 | |_) | |___ / ___ \  / /_| |___ | |_| |___) |
 |____/|_____/_/   \_\/____|_____(_)___/|____/ 
                                               
 >>> RETRO SCANLINE & SOLAR GRAVITY UNLOCKED <<<
"#);
        }
        "gravity" => {
            let _ = Command::new("notify-send")
                .args([
                    "-a", "SolarUI Physics",
                    "-i", "preferences-desktop-display",
                    "[GRAVITY] Solar Gravity Modu",
                    "Pencere yay fiziği ve elastik lastikleme (rubber-banding) maksimum ivmeye ayarlandı!",
                ])
                .status();
            println!("Solar Gravity & Dynamic Elasticity active.");
        }
        other => {
            println!("Bilinmeyen sürpriz yumurta: '{}'. Geçerli olanlar: konami, crt, gravity", other);
        }
    }
    Ok(())
}

pub fn play_acoustic_feedback(event: &str) -> Result<()> {
    // Check if system sound effects are enabled or use lightweight system sounds
    let sound_candidates = match event {
        "click" => vec!["/usr/share/sounds/freedesktop/stereo/button-pressed.oga", "/usr/share/sounds/gnome/default/alerts/glass.ogg"],
        "slide" => vec!["/usr/share/sounds/freedesktop/stereo/dialog-information.oga"],
        "screenshot" => vec!["/usr/share/sounds/freedesktop/stereo/camera-shutter.oga"],
        "alert" | "easter-egg" => vec!["/usr/share/sounds/freedesktop/stereo/bell.oga", "/usr/share/sounds/gnome/default/alerts/bark.ogg"],
        _ => vec!["/usr/share/sounds/freedesktop/stereo/button-pressed.oga"],
    };

    for snd in sound_candidates {
        if std::path::Path::new(snd).exists() {
            if Command::new("pw-play").arg(snd).spawn().is_ok() {
                return Ok(());
            } else if Command::new("paplay").arg(snd).spawn().is_ok() {
                return Ok(());
            }
        }
    }

    Ok(())
}

pub fn get_seasonal_atmosphere() -> (&'static str, &'static str) {
    let now = Local::now();
    let month = now.month();
    let day = now.day();

    // Check specific astronomical events
    if month == 3 && (day >= 20 && day <= 22) {
        ("Bahar Ekinoksu (Spring Equinox)", "#00e676") // Emerald warmth
    } else if month == 6 && (day >= 20 && day <= 22) {
        ("Yaz Gündönümü (Summer Solstice)", "#ffab00") // Solar Gold
    } else if month == 9 && (day >= 22 && day <= 24) {
        ("Sonbahar Ekinoksu (Autumn Equinox)", "#ff6d00") // Amber Sunset
    } else if month == 12 && (day >= 21 && day <= 23) {
        ("Kış Gündönümü (Winter Solstice)", "#00b0ff") // Arctic Blue
    } else {
        match month {
            12 | 1 | 2 => ("Solar Kış Atmosferi", "#29b6f6"),
            3 | 4 | 5 => ("Solar Bahar Tazeliği", "#66bb6a"),
            6 | 7 | 8 => ("Solar Yaz Sıcaklığı", "#ffa726"),
            _ => ("Solar Sonbahar Zarafeti", "#ab47bc"),
        }
    }
}
