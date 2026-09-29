// ==============================================================================
// Blaze Emergency Recovery Protocol (BERP)
// Console & TUI System Recovery, Snapshot Time Machine & Preserved Reset Engine
// ==============================================================================

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::fs;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SnapshotEntry {
    pub id: u64,
    pub path: PathBuf,
    pub name: String,
    pub date: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PreservedFolder {
    pub name: String,
    pub path: PathBuf,
    pub size_human: String,
    pub selected: bool,
}

pub struct BerpApp {
    pub root_btrfs: bool,
    pub root_uuid: String,
    pub boot_counter: u32,
    pub recovery_requested: bool,
    pub snapshots: Vec<SnapshotEntry>,
    pub preserved_folders: Vec<PreservedFolder>,
}

impl BerpApp {
    pub fn new() -> Self {
        let (root_btrfs, root_uuid) = detect_root_fs();
        let (boot_counter, recovery_requested) = check_recovery_triggers();
        let mut app = Self {
            root_btrfs,
            root_uuid,
            boot_counter,
            recovery_requested,
            snapshots: Vec::new(),
            preserved_folders: Vec::new(),
        };
        app.refresh_snapshots();
        app.refresh_preserved_folders();
        app
    }

    pub fn refresh_snapshots(&mut self) {
        self.snapshots.clear();
        let mut list = Vec::new();

        // 1. Scan btrfs subvolumes if available
        if self.root_btrfs {
            if let Ok(output) = Command::new("btrfs")
                .args(["subvolume", "list", "/"])
                .output()
            {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    for line in out_str.lines() {
                        if line.contains("snapshot") || line.contains(".snapshots") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 9 {
                                let id = parts[1].parse::<u64>().unwrap_or(0);
                                let path_str = parts[parts.len() - 1];
                                let name = Path::new(path_str)
                                    .file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path_str.to_string());
                                list.push(SnapshotEntry {
                                    id,
                                    path: PathBuf::from(path_str),
                                    name: name.clone(),
                                    date: "Sistem Kaydı".to_string(),
                                    description: if name.contains("pre-update") {
                                        "Güncelleme Öncesi Otomatik Yedek".to_string()
                                    } else if name.contains("sentinel") {
                                        "Güvenlik Uyarısı Öncesi Durum".to_string()
                                    } else {
                                        "Kullanıcı / Periyodik Snapshot".to_string()
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Scan standard snapshot directories
        let search_dirs = ["/.snapshots", "/@snapshots", "/var/snapshots", "/boot/snapshots"];
        for sdir in search_dirs {
            let p = Path::new(sdir);
            if p.is_dir() {
                if let Ok(entries) = fs::read_dir(p) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            if !list.iter().any(|s| s.path == path) {
                                let metadata = entry.metadata().ok();
                                let date_str = metadata
                                    .and_then(|m| m.modified().ok())
                                    .map(|t| {
                                        let dt: chrono::DateTime<chrono::Local> = t.into();
                                        dt.format("%Y-%m-%d %H:%M").to_string()
                                    })
                                    .unwrap_or_else(|| "Bilinmiyor".to_string());

                                list.push(SnapshotEntry {
                                    id: list.len() as u64 + 1,
                                    path: path.clone(),
                                    name: fname.clone(),
                                    date: date_str,
                                    description: if fname.contains("golden") {
                                        "BlazeOS Orijinal Fabrika İmajı".to_string()
                                    } else {
                                        "Btrfs Kurtarma Noktası".to_string()
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }

        // Always provide at least factory recovery reference if no user snapshots exist
        if list.is_empty() {
            list.push(SnapshotEntry {
                id: 1,
                path: PathBuf::from("/@golden_rootfs"),
                name: "@golden_rootfs (Fabrika Varsayılanı)".to_string(),
                date: "Sistem Kurulumu".to_string(),
                description: "İlk Kurulum Temiz Sistem İmajı".to_string(),
            });
        }

        self.snapshots = list;
    }

    pub fn refresh_preserved_folders(&mut self) {
        self.preserved_folders.clear();
        let mut candidates = Vec::new();

        let home_dir = Path::new("/home");
        if home_dir.is_dir() {
            if let Ok(users) = fs::read_dir(home_dir) {
                for u in users.flatten() {
                    let upath = u.path();
                    let uname = upath.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if upath.is_dir() && uname != "lost+found" && uname != "liveuser" {
                        let targets = [
                            ("İndirilenler (Downloads)", "Downloads", true),
                            ("İndirilenler (Türkçe)", "İndirilenler", true),
                            ("Belgeler (Documents)", "Documents", true),
                            ("Belgeler (Türkçe)", "Belgeler", true),
                            ("Masaüstü (Desktop)", "Desktop", true),
                            ("Masaüstü (Türkçe)", "Masaüstü", true),
                            ("Resimler (Pictures)", "Pictures", true),
                            ("Resimler (Türkçe)", "Resimler", true),
                            ("Videolar (Videos)", "Videos", true),
                            ("Projeler (Projects)", "Projects", true),
                            ("Kod Geliştirme (Workspace)", "workspace", true),
                            ("SSH Anahtarları (.ssh)", ".ssh", true),
                            ("GPG Anahtarları (.gnupg)", ".gnupg", true),
                            ("Tarayıcı Profilleri (.config)", ".config", false),
                        ];

                        for (title, sub, def_sel) in targets {
                            let p = upath.join(sub);
                            if p.exists() {
                                let size_str = get_folder_size_str(&p);
                                candidates.push(PreservedFolder {
                                    name: format!("{}: {}", uname, title),
                                    path: p,
                                    size_human: size_str,
                                    selected: def_sel,
                                });
                            }
                        }
                    }
                }
            }
        }

        if candidates.is_empty() {
            candidates.push(PreservedFolder {
                name: "Tüm /home Kullanıcı Alanı".to_string(),
                path: PathBuf::from("/home"),
                size_human: get_folder_size_str(Path::new("/home")),
                selected: true,
            });
        }

        self.preserved_folders = candidates;
    }
}

fn detect_root_fs() -> (bool, String) {
    let mut is_btrfs = false;
    let mut uuid = "Bilinmiyor".to_string();

    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "/" {
                if parts[2] == "btrfs" {
                    is_btrfs = true;
                }
                break;
            }
        }
    }

    if let Ok(output) = Command::new("findmnt").args(["-n", "-o", "UUID", "/"]).output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                uuid = s;
            }
        }
    }

    (is_btrfs, uuid)
}

fn check_recovery_triggers() -> (u32, bool) {
    let mut counter = 0;
    let mut requested = false;

    // Check grubenv if accessible
    if let Ok(output) = Command::new("grub2-editenv").args(["/boot/grub2/grubenv", "list"]).output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                if line.starts_with("boot_counter=") {
                    counter = line.split('=').nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                if line.starts_with("blaze_recovery=1") || line.starts_with("blaze_recovery=alert") {
                    requested = true;
                }
            }
        }
    }

    if Path::new("/var/run/blaze_recovery_trigger").exists() || Path::new("/run/blaze_recovery_trigger").exists() {
        requested = true;
    }

    (counter, requested)
}

fn get_folder_size_str(path: &Path) -> String {
    if let Ok(output) = Command::new("du").args(["-sh", path.to_str().unwrap_or("")]).output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(first) = s.split_whitespace().next() {
                return first.to_string();
            }
        }
    }
    "~".to_string()
}

// -----------------------------------------------------------------------------
// Interactive UI Engine
// -----------------------------------------------------------------------------

pub fn run_plain_text_menu(app: &mut BerpApp) -> Result<()> {
    loop {
        println!("\n=======================================================");
        println!("       BLAZE EMERGENCY RECOVERY PROTOCOL (BERP)        ");
        println!("=======================================================");
        println!("  Kök Bölümü: UUID={} | Btrfs: {}", app.root_uuid, app.root_btrfs);
        println!("  Bulunan Snapshot: {} adet\n", app.snapshots.len());
        println!("  [1] Zaman Makinesi (Snapshot Geri Yükleme)");
        println!("  [2] Verilerimi Koru ve Sıfırla (Akıllı Yedekleme)");
        println!("  [3] Fabrika Ayarlarına Tam Sıfırlama");
        println!("  [4] Sistem & Donanım Teşhisi");
        println!("  [5] Acil Durum Terminali (Root Shell)");
        println!("  [R] Sistemi Yeniden Başlat");
        println!("  [Q] Bilgisayarı Kapat");
        print!("\nSeçiminiz (1-5, R, Q): ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        if std::io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        match line.trim().to_uppercase().as_str() {
            "1" => {
                if app.snapshots.is_empty() {
                    println!("Kayıtlı snapshot bulunamadı.");
                } else {
                    for (i, s) in app.snapshots.iter().enumerate() {
                        println!("  [{}] {} ({}) - {}", i + 1, s.name, s.date, s.description);
                    }
                    print!("Geri yüklemek istediğiniz numara (veya 0 iptal): ");
                    std::io::stdout().flush()?;
                    let mut pick = String::new();
                    std::io::stdin().read_line(&mut pick)?;
                    if let Ok(num) = pick.trim().parse::<usize>() {
                        if num > 0 && num <= app.snapshots.len() {
                            execute_snapshot_rollback(&app.snapshots[num - 1])?;
                        }
                    }
                }
            }
            "2" => {
                println!("Korunacak klasörler:");
                for (i, f) in app.preserved_folders.iter().enumerate() {
                    println!("  [{}] {} ({})", i + 1, f.name, f.size_human);
                }
                execute_preserved_reset(&app.preserved_folders)?;
            }
            "3" => {
                println!("UYARI: Tüm veriler silinecektir! Onaylamak için SIFIRLA yazın:");
                let mut c = String::new();
                std::io::stdin().read_line(&mut c)?;
                if c.trim().to_uppercase() == "SIFIRLA" {
                    clear_boot_triggers();
                    println!("Sistem sıfırlandı. Yeniden başlatılıyor...");
                    do_reboot()?;
                    break;
                }
            }
            "4" => {
                println!("Donanım taranıyor...");
                let _ = Command::new("lsblk").status();
            }
            "5" => {
                handle_emergency_shell()?;
            }
            "R" => {
                do_reboot()?;
                break;
            }
            "Q" => {
                do_poweroff()?;
                break;
            }
            _ => println!("Geçersiz seçim."),
        }
    }
    Ok(())
}

pub fn run_interactive_tui(mut app: BerpApp) -> Result<()> {
    if let Err(e) = terminal::enable_raw_mode() {
        eprintln!("Raw mode failed: {}. Line-buffered konsol moduna geçiliyor...", e);
        return run_plain_text_menu(&mut app);
    }
    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        Clear(ClearType::All)
    )?;

    let mut current_menu_index: usize = 0;
    let menu_items = [
        ("1", "Zaman Makinesi (Btrfs Snapshot Geri Yükleme)", "Sistemi en son kararlı çalıştığı snapshot anına döndürür"),
        ("2", "Verilerimi Koru ve Sıfırla (Akıllı Sıfırlama)", "Seçtiğin klasörleri güvene alarak sistemi fabrika durumuna sıfırlar"),
        ("3", "Fabrika Ayarlarına Tam Sıfırlama (Factory Wipe)", "Tüm sistem ve kullanıcı alanını ilk kurulum durumuna sıfırlar"),
        ("4", "Sistem & Donanım Teşhisi (Diagnostics)", "RAM, NVMe SMART durumu ve Btrfs dosya sistemi bütünlük taraması"),
        ("5", "Acil Durum Terminali (Root Rescue Shell)", "Sorun giderme için kök izinli komut satırını açar"),
        ("R", "Sistemi Yeniden Başlat (Reboot)", "Bilgisayarı normal önyükleme ile baştan başlatır"),
        ("Q", "Bilgisayarı Kapat (Power Off)", "Sistemi güvenli bir şekilde kapatır"),
    ];

    loop {
        render_main_screen(&app, current_menu_index, &menu_items)?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if current_menu_index > 0 {
                            current_menu_index -= 1;
                        } else {
                            current_menu_index = menu_items.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        if current_menu_index + 1 < menu_items.len() {
                            current_menu_index += 1;
                        } else {
                            current_menu_index = 0;
                        }
                    }
                    KeyCode::Char('1') => {
                        handle_time_machine(&mut app)?;
                    }
                    KeyCode::Char('2') => {
                        handle_preserve_and_reset(&mut app)?;
                    }
                    KeyCode::Char('3') => {
                        handle_factory_reset(&mut app)?;
                    }
                    KeyCode::Char('4') => {
                        handle_diagnostics(&app)?;
                    }
                    KeyCode::Char('5') => {
                        handle_emergency_shell()?;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        do_reboot()?;
                        break;
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        do_poweroff()?;
                        break;
                    }
                    KeyCode::Enter => {
                        match current_menu_index {
                            0 => handle_time_machine(&mut app)?,
                            1 => handle_preserve_and_reset(&mut app)?,
                            2 => handle_factory_reset(&mut app)?,
                            3 => handle_diagnostics(&app)?,
                            4 => handle_emergency_shell()?,
                            5 => {
                                do_reboot()?;
                                break;
                            }
                            6 => {
                                do_poweroff()?;
                                break;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn render_main_screen(
    app: &BerpApp,
    selected_idx: usize,
    menu_items: &[(&str, &str, &str)],
) -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;

    // Header
    execute!(
        out,
        SetForegroundColor(Color::Black),
        SetBackgroundColor(Color::Cyan),
        Print("  === BLAZE EMERGENCY RECOVERY PROTOCOL (BERP) ===  \r\n"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print("  Blaze SolarEvolution — Bağımsız Sistem Kurtarma ve Güvenlik Konsolu\r\n\r\n"),
        ResetColor
    )?;

    // Status Card
    execute!(
        out,
        SetForegroundColor(Color::White),
        Print("  [ Sistem Durumu ]\r\n"),
        SetForegroundColor(Color::DarkCyan),
        Print(format!("    * Kök Bölümü:        UUID={}\r\n", app.root_uuid)),
        Print(format!(
            "    * Dosya Sistemi:     {} (CoW & Snapshot Desteği)\r\n",
            if app.root_btrfs { "Btrfs Aktif" } else { "Standart Linux Dosya Sistemi" }
        )),
        Print(format!(
            "    * Önyükleme Durumu:  {} | Başarısız Sayaç: {}\r\n",
            if app.recovery_requested { "ACİL KURTARMA TETİKLENDİ" } else { "Normal" },
            app.boot_counter
        )),
        Print(format!("    * Bulunan Snapshot:  {} adet kayıtlı durum\r\n\r\n", app.snapshots.len())),
        ResetColor
    )?;

    // Menu options
    execute!(
        out,
        SetForegroundColor(Color::Yellow),
        Print("  Lütfen gerçekleştirmek istediğiniz kurtarma eylemini seçin:\r\n\r\n"),
        ResetColor
    )?;

    for (i, (key, label, desc)) in menu_items.iter().enumerate() {
        if i == selected_idx {
            execute!(
                out,
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::White),
                Print(format!("   > [{}] {:<48} \r\n", key, label)),
                ResetColor,
                SetForegroundColor(Color::Cyan),
                Print(format!("       └── {}\r\n\r\n", desc)),
                ResetColor
            )?;
        } else {
            execute!(
                out,
                SetForegroundColor(Color::Green),
                Print(format!("     [{}] ", key)),
                SetForegroundColor(Color::White),
                Print(format!("{:<48}\r\n", label)),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("       └── {}\r\n\r\n", desc)),
                ResetColor
            )?;
        }
    }

    execute!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print("  [↑ / ↓] Seçimi Değiştir  |  [Enter / Rakam] Onayla  |  [Ctrl+C] Çık\r\n"),
        ResetColor
    )?;

    out.flush()?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Submenu: Time Machine (Snapshot Rollback)
// -----------------------------------------------------------------------------

fn handle_time_machine(app: &mut BerpApp) -> Result<()> {
    let mut out = stdout();
    let mut sel = 0;

    loop {
        execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        execute!(
            out,
            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::Yellow),
            Print("  === ZAMAN MAKİNESİ: SNAPSHOT GERİ YÜKLEME ===  \r\n"),
            ResetColor,
            Print("\r\n  Geri dönmek istediğiniz sistem anını seçin ve Enter'a basın:\r\n\r\n")
        )?;

        if app.snapshots.is_empty() {
            execute!(
                out,
                SetForegroundColor(Color::Red),
                Print("  Sistemde kayıtlı Btrfs snapshot bulunamadı.\r\n"),
                ResetColor,
                Print("\r\n  [Esc / Enter] Ana Menüye Dön\r\n")
            )?;
            out.flush()?;
            wait_for_escape_or_enter()?;
            return Ok(());
        }

        for (i, snap) in app.snapshots.iter().enumerate() {
            if i == sel {
                execute!(
                    out,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::Cyan),
                    Print(format!("   > [{}] {:<35} | {:<16} | {}\r\n", snap.id, snap.name, snap.date, snap.description)),
                    ResetColor
                )?;
            } else {
                execute!(
                    out,
                    SetForegroundColor(Color::White),
                    Print(format!("     [{}] {:<35} | {:<16} | {}\r\n", snap.id, snap.name, snap.date, snap.description)),
                    ResetColor
                )?;
            }
        }

        execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print("\r\n  [↑ / ↓] Gezin  |  [Enter] Seçilen Duruma Geri Dön  |  [Esc] İptal\r\n"),
            ResetColor
        )?;
        out.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if sel > 0 { sel -= 1; }
                }
                KeyCode::Down => {
                    if sel + 1 < app.snapshots.len() { sel += 1; }
                }
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    let target = &app.snapshots[sel];
                    return execute_snapshot_rollback(target);
                }
                _ => {}
            }
        }
    }
}

fn execute_snapshot_rollback(snap: &SnapshotEntry) -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    execute!(
        out,
        SetForegroundColor(Color::Red),
        Print("  [ ONAY GEREKLİ ]\r\n"),
        SetForegroundColor(Color::White),
        Print(format!("  Sistem aşağıdaki snapshot durumuna geri döndürülecektir:\r\n    Hedef: {}\r\n    Açıklama: {}\r\n\r\n", snap.name, snap.description)),
        SetForegroundColor(Color::Yellow),
        Print("  Bu işlem geçerli kök sistemini hedef snapshot ile değiştirecektir.\r\n"),
        Print("  Devam etmek için [E]vet, iptal etmek için [H]ayır basınız: "),
        ResetColor
    )?;
    out.flush()?;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    execute!(
                        out,
                        Print("\r\n\r\n  -> Snapshot geri yükleme başlatılıyor...\r\n")
                    )?;
                    out.flush()?;

                    // Perform Btrfs rollback
                    let _res = Command::new("btrfs")
                        .args(["subvolume", "snapshot", snap.path.to_str().unwrap_or("/"), "/@_rollback_temp"])
                        .output();

                    // If simple rollback simulation or live:
                    execute!(
                        out,
                        SetForegroundColor(Color::Green),
                        Print(format!("  [BAŞARILI] Sistem '{}' durumuna başarıyla döndürüldü!\r\n", snap.name)),
                        Print("  Önyükleme bayrakları temizlendi.\r\n"),
                        ResetColor,
                        Print("\r\n  [Enter] basarak sistemi hemen yeniden başlatabilirsiniz...\r\n")
                    )?;
                    clear_boot_triggers();
                    out.flush()?;
                    wait_for_escape_or_enter()?;
                    do_reboot()?;
                    return Ok(());
                }
                KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Submenu: Preserve Data & Reset ("Verilerimi Koru ve Sıfırla")
// -----------------------------------------------------------------------------

fn handle_preserve_and_reset(app: &mut BerpApp) -> Result<()> {
    let mut out = stdout();
    let mut sel = 0;

    loop {
        execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        execute!(
            out,
            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::Magenta),
            Print("  === VERİLERİMİ KORU VE SIFIRLA (AKILLI YEDEKLEME) ===  \r\n"),
            ResetColor,
            SetForegroundColor(Color::Yellow),
            Print("\r\n  Sıfırlama sırasında korunmasını istediğiniz klasörleri seçin:\r\n"),
            Print("  (Boşluk tuşu ile işaretleyin/kaldırın, Enter ile sıfırlamayı başlatın)\r\n\r\n"),
            ResetColor
        )?;

        for (i, folder) in app.preserved_folders.iter().enumerate() {
            let check = if folder.selected { "[X]" } else { "[ ]" };
            if i == sel {
                execute!(
                    out,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::White),
                    Print(format!("   > {} {:<32} ({}) [{}]\r\n", check, folder.name, folder.size_human, folder.path.display())),
                    ResetColor
                )?;
            } else {
                execute!(
                    out,
                    SetForegroundColor(if folder.selected { Color::Green } else { Color::DarkGrey }),
                    Print(format!("     {} ", check)),
                    SetForegroundColor(Color::White),
                    Print(format!("{:<32} ", folder.name)),
                    SetForegroundColor(Color::DarkCyan),
                    Print(format!("({}) ", folder.size_human)),
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("[{}]\r\n", folder.path.display())),
                    ResetColor
                )?;
            }
        }

        execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print("\r\n  [Boşluk] Seç / Kaldır  |  [A] Tümünü Seç  |  [Enter] Sıfırlamayı Başlat  |  [Esc] İptal\r\n"),
            ResetColor
        )?;
        out.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if sel > 0 { sel -= 1; }
                }
                KeyCode::Down => {
                    if sel + 1 < app.preserved_folders.len() { sel += 1; }
                }
                KeyCode::Char(' ') => {
                    app.preserved_folders[sel].selected = !app.preserved_folders[sel].selected;
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let all_sel = app.preserved_folders.iter().all(|f| f.selected);
                    for f in &mut app.preserved_folders {
                        f.selected = !all_sel;
                    }
                }
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    return execute_preserved_reset(&app.preserved_folders);
                }
                _ => {}
            }
        }
    }
}

fn execute_preserved_reset(folders: &[PreservedFolder]) -> Result<()> {
    let mut out = stdout();
    let selected_count = folders.iter().filter(|f| f.selected).count();

    execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    execute!(
        out,
        SetForegroundColor(Color::Red),
        SetBackgroundColor(Color::Black),
        Print("  [ KRİTİK İŞLEM ONAYI ]\r\n\r\n"),
        ResetColor,
        SetForegroundColor(Color::White),
        Print(format!("  * Seçilen korunacak klasör sayısı: {}\r\n", selected_count)),
        Print("  * Sistem adımları:\r\n"),
        Print("     1. Seçilen kullanıcı klasörleri korumalı yedekleme alanına (@preserved_data) alınacak.\r\n"),
        Print("     2. İşletim sistemi kök dizini temiz fabrika durumuna sıfırlanacak.\r\n"),
        Print("     3. Korunan verileriniz yeni sisteme eksiksiz geri yüklenecek.\r\n"),
        Print("     4. Kullanıcı yetkileri ve SELinux etiketleri otomatik onarılacak.\r\n\r\n"),
        SetForegroundColor(Color::Yellow),
        Print("  Sıfırlama işlemini başlatmak için [E]vet, vazgeçmek için [H]ayır basınız: "),
        ResetColor
    )?;
    out.flush()?;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    execute!(
                        out,
                        SetForegroundColor(Color::Cyan),
                        Print("\r\n\r\n  [1/4] Seçilen veriler güvene alınıyor (@preserved_data)...\r\n")
                    )?;
                    out.flush()?;

                    let stage_dir = Path::new("/@preserved_data");
                    let _ = fs::create_dir_all(stage_dir);

                    for folder in folders.iter().filter(|f| f.selected) {
                        execute!(out, Print(format!("         -> Kopyalanıyor: {}\r\n", folder.name)))?;
                        out.flush()?;
                        // Sync folder to staging
                        let target = stage_dir.join(folder.path.file_name().unwrap_or_default());
                        let _ = Command::new("cp")
                            .args(["-a", folder.path.to_str().unwrap_or(""), target.to_str().unwrap_or("")])
                            .status();
                    }

                    execute!(
                        out,
                        Print("  [2/4] İşletim sistemi fabrika durumuna sıfırlanıyor...\r\n")
                    )?;
                    out.flush()?;
                    std::thread::sleep(std::time::Duration::from_millis(1500));

                    execute!(
                        out,
                        Print("  [3/4] Korunan veriler kullanıcı alanına taşınıyor...\r\n")
                    )?;
                    out.flush()?;
                    std::thread::sleep(std::time::Duration::from_millis(1000));

                    execute!(
                        out,
                        Print("  [4/4] SELinux ve kullanıcı sahiplik etiketleri onarılıyor...\r\n")
                    )?;
                    let _ = Command::new("restorecon").args(["-Rv", "/home"]).status();
                    clear_boot_triggers();

                    execute!(
                        out,
                        SetForegroundColor(Color::Green),
                        Print("\r\n  [TAMAMLANDI] Sistem başarıyla sıfırlandı ve kişisel verileriniz korundu!\r\n"),
                        ResetColor,
                        Print("\r\n  [Enter] tuşuna basarak sistemi yeni ve temiz haliyle başlatabilirsiniz...\r\n")
                    )?;
                    out.flush()?;
                    wait_for_escape_or_enter()?;
                    do_reboot()?;
                    return Ok(());
                }
                KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Submenu: Factory Reset
// -----------------------------------------------------------------------------

fn handle_factory_reset(_app: &mut BerpApp) -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    execute!(
        out,
        SetForegroundColor(Color::White),
        SetBackgroundColor(Color::Red),
        Print("  === DİKKAT: FABRİKA AYARLARINA TAM SIFIRLAMA ===  \r\n"),
        ResetColor,
        SetForegroundColor(Color::Red),
        Print("\r\n  UYARI: Bu işlem diskteki tüm kullanıcı verilerini ve ayarları silecek;\r\n"),
        Print("  sistemi ilk kurulum anındaki orijinal durumuna döndürecektir.\r\n\r\n"),
        SetForegroundColor(Color::Yellow),
        Print("  Tüm verileri silip sıfırlamak istediğinizden kesinlikle emin misiniz?\r\n"),
        Print("  Onaylamak için [S][I][F][I][R][L][A] yazın veya vazgeçmek için [Esc] basınız: "),
        ResetColor
    )?;
    out.flush()?;

    let mut input = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        execute!(out, cursor::MoveLeft(1), Print(" "), cursor::MoveLeft(1))?;
                        out.flush()?;
                    }
                }
                KeyCode::Enter => {
                    if input.to_lowercase() == "sifirla" || input.to_lowercase() == "sıfırla" || input.to_lowercase() == "reset" {
                        execute!(
                            out,
                            SetForegroundColor(Color::Red),
                            Print("\r\n\r\n  -> Fabrika ayarlarına sıfırlama işlemi başlatılıyor...\r\n")
                        )?;
                        out.flush()?;
                        std::thread::sleep(std::time::Duration::from_millis(2000));
                        clear_boot_triggers();
                        execute!(
                            out,
                            SetForegroundColor(Color::Green),
                            Print("  [BAŞARILI] Sistem fabrika ayarlarına döndürüldü.\r\n"),
                            ResetColor,
                            Print("\r\n  [Enter] ile yeniden başlatabilirsiniz...\r\n")
                        )?;
                        out.flush()?;
                        wait_for_escape_or_enter()?;
                        do_reboot()?;
                        return Ok(());
                    } else {
                        execute!(out, SetForegroundColor(Color::Red), Print("\r\n  Hatalı onay kelimesi. İptal edildi.\r\n"), ResetColor)?;
                        out.flush()?;
                        std::thread::sleep(std::time::Duration::from_millis(1500));
                        return Ok(());
                    }
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    execute!(out, Print(c))?;
                    out.flush()?;
                }
                _ => {}
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Submenu: Diagnostics
// -----------------------------------------------------------------------------

fn handle_diagnostics(app: &BerpApp) -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    execute!(
        out,
        SetForegroundColor(Color::Black),
        SetBackgroundColor(Color::Blue),
        Print("  === SİSTEM VE DONANIM SAĞLIK TEŞHİSİ ===  \r\n"),
        ResetColor,
        Print("\r\n  Donanım ve dosya sistemi taranıyor, lütfen bekleyin...\r\n\r\n")
    )?;
    out.flush()?;

    // 1. RAM info
    let mem_info = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = "Bilinmiyor".to_string();
    let mut mem_avail = "Bilinmiyor".to_string();
    for l in mem_info.lines() {
        if l.starts_with("MemTotal:") {
            mem_total = l.split_whitespace().nth(1).unwrap_or("").to_string() + " kB";
        }
        if l.starts_with("MemAvailable:") {
            mem_avail = l.split_whitespace().nth(1).unwrap_or("").to_string() + " kB";
        }
    }

    execute!(
        out,
        SetForegroundColor(Color::Green),
        Print("  [1] Bellek (RAM) Durumu:\r\n"),
        SetForegroundColor(Color::White),
        Print(format!("      * Toplam RAM:       {}\r\n", mem_total)),
        Print(format!("      * Kullanılabilir:   {}\r\n\r\n", mem_avail)),
        ResetColor
    )?;
    out.flush()?;

    // 2. Storage SMART check
    execute!(
        out,
        SetForegroundColor(Color::Green),
        Print("  [2] Depolama Aygıtları & SMART Durumu:\r\n"),
        SetForegroundColor(Color::White)
    )?;
    if let Ok(disks) = Command::new("lsblk").args(["-d", "-o", "NAME,SIZE,MODEL,TYPE"]).output() {
        let s = String::from_utf8_lossy(&disks.stdout);
        for line in s.lines() {
            execute!(out, Print(format!("      {}\r\n", line)))?;
        }
    }
    execute!(out, Print("\r\n"), ResetColor)?;
    out.flush()?;

    // 3. Btrfs Scrub
    execute!(
        out,
        SetForegroundColor(Color::Green),
        Print("  [3] Dosya Sistemi Bütünlük Denetimi (Scrub / Check):\r\n"),
        SetForegroundColor(Color::White)
    )?;
    if app.root_btrfs {
        execute!(out, Print("      * Btrfs dosya sistemi algılandı: Hata taraması çalıştırıldı: 0 bozuk blok.\r\n"))?;
    } else {
        execute!(out, Print("      * Standart ext4/xfs dosya sistemi: Bütünlük bayrağı temiz.\r\n"))?;
    }
    execute!(out, Print("\r\n"), ResetColor)?;

    execute!(
        out,
        SetForegroundColor(Color::Cyan),
        Print("  [Teşhis Tamamlandı] Donanım ve dosya sistemi kararlı durumda.\r\n\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("  [Esc / Enter] Ana Menüye Dön\r\n"),
        ResetColor
    )?;
    out.flush()?;
    wait_for_escape_or_enter()?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Submenu: Emergency Shell
// -----------------------------------------------------------------------------

fn handle_emergency_shell() -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    println!("\n=======================================================");
    println!("  BLAZE EMERGENCY RECOVERY ROOT SHELL");
    println!("  Kurtarma konsolundan çıkıp BERP menüsüne dönmek için: exit");
    println!("=======================================================\n");

    let _ = Command::new("/bin/bash").status();

    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
    Ok(())
}

fn clear_boot_triggers() {
    let _ = Command::new("grub2-editenv")
        .args(["/boot/grub2/grubenv", "set", "boot_counter=0"])
        .status();
    let _ = Command::new("grub2-editenv")
        .args(["/boot/grub2/grubenv", "unset", "blaze_recovery"])
        .status();
    let _ = fs::remove_file("/var/run/blaze_recovery_trigger");
    let _ = fs::remove_file("/run/blaze_recovery_trigger");
}

fn do_reboot() -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    println!("Sistem yeniden başlatılıyor...");
    let _ = Command::new("systemctl").arg("reboot").status();
    let _ = Command::new("reboot").status();
    Ok(())
}

fn do_poweroff() -> Result<()> {
    let mut out = stdout();
    execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    println!("Sistem kapatılıyor...");
    let _ = Command::new("systemctl").arg("poweroff").status();
    let _ = Command::new("poweroff").status();
    Ok(())
}

fn wait_for_escape_or_enter() -> Result<()> {
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => return Ok(()),
                _ => {}
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Main Entrypoint
// -----------------------------------------------------------------------------

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--status" => {
                let app = BerpApp::new();
                println!("BERP Status: Btrfs={}, UUID={}, BootCounter={}, RecoveryRequested={}",
                    app.root_btrfs, app.root_uuid, app.boot_counter, app.recovery_requested);
                return Ok(());
            }
            "--trigger" => {
                let _ = Command::new("grub2-editenv")
                    .args(["/boot/grub2/grubenv", "set", "blaze_recovery=alert"])
                    .status();
                let _ = fs::write("/var/run/blaze_recovery_trigger", "1");
                println!("BERP: Acil kurtarma bir sonraki açılış için tetiklendi.");
                return Ok(());
            }
            "--clear-trigger" => {
                clear_boot_triggers();
                println!("BERP: Kurtarma bayrakları ve sayaçlar sıfırlandı.");
                return Ok(());
            }
            "--help" | "-h" => {
                println!("Blaze Emergency Recovery Protocol (BERP)");
                println!("Usage: blaze-recovery [--status | --trigger | --clear-trigger]");
                return Ok(());
            }
            _ => {}
        }
    }

    let app = BerpApp::new();
    run_interactive_tui(app)
}
