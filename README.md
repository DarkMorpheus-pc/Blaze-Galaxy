# Blaze SolarEvolution 5.04 (Blaze-Galaxy)

[**Upgrade to BlazeOS (ISO)**](https://archive.org/details/blaze-solar-evolution-5-test)

<h3 align="center">Next-Generation Hybrid Desktop Ecosystem & High-Performance Linux Distribution</h3>

<img width="1209" height="752" alt="image" src="https://github.com/user-attachments/assets/e888deac-5b91-4ea9-98e2-8026f75c3bcf" />
<img width="1208" height="757" alt="image" src="https://github.com/user-attachments/assets/f79b5aea-0fb7-4933-a5ca-b52321aa7ab0" />
<img width="1208" height="754" alt="image" src="https://github.com/user-attachments/assets/7b3e685d-8caf-4270-a9d6-da6324638f0b" />

<p align="center">
  <b>"From evolution to expansion — welcome to the galaxy."</b>
</p>

---

## 📋 Sürüm Notları (Release Notes) — Blaze SolarEvolution 5.04

Bu sürüm (`5.04`), 5.03 masaüstü ve Türkçe dil altyapısının üzerine güvenilir kurulum, kalıcı UEFI önyükleme, tek seferlik OOBE ve ikinci açılış kararlılığı ekler:

### 🇹🇷 1. Caelestia Tam Türkçe Yama & Canlı Dil Seçimi
- **%100 Türkçe Yerelleştirme:** Caelestia kaynak kodundaki 670+ arayüz dizesi (Nexus Ayarları, Yan Panel/Sidebar, Sistem Tepsisi, Bildirimler, Kilit Ekranı, Medya Oynatıcı, Hızlı Anahtarlar, Saat, Ağ ve Güç Menüleri) Türkçeleştirildi.
- **Yerel Qt Çeviri Motoru Entegrasyonu:** C++ `CaelestiaQTranslator` geliştirilerek `QCoreApplication::installTranslator` seviyesine kancalandı. Tüm `qsTr(...)` çağrıları çalışma anında O(1) bellek erişimiyle yerel Türkçe olarak çözülüyor.
- **Nexus Ayarlarında Canlı Dil Seçici:** Nexus Ayarları (`Dil ve Bölge` / `Language & region`) sayfasına interaktif dil değiştirme özelliği eklendi. Kullanıcılar **"Sistem Dili (Otomatik)"**, **"Türkçe"** ve **"English"** seçenekleri arasında anında geçiş yapabilir; arayüz yeniden başlatmaya gerek kalmadan canlı olarak güncellenir.
- **Başlangıç Entegrasyonu:** `ServiceLoader.qml` içerisine `Tr` başlatıcısı bağlanarak kabuğun ilk açılışından itibaren Türkçe çeviri motorunun devrede olması sağlandı.

### ☀️ 2. SolarUI Derin Entegrasyonu & Markalama
- Sistem genelinde masaüstü ortamı ve pencere yöneticisi referansları kökten **SolarUI** olarak yapılandırıldı.
- `libsolar_brand.so` kancası ile sistem araçlarında, oturum yöneticisinde ve sistem kimliğinde saf SolarUI deneyimi tescillendi.

### 🏠 3. Blaze House Arayüz Sadeleştirmesi ve İşlev Tamamlama
- Blaze House penceresinin sol üstündeki `V5` etiketi ve sağ üstündeki `Arch/Cachy` rozeti kaldırıldı.
- Uygulama içi tüm butonlar tek tek incelenerek eksik/işlevsiz olanlar tespit edildi ve doğrudan Fedora sistem araçlarına bağlanan kodları inşa edildi.
- Sistem güncelleme mekanizması tamamen Fedora / DNF5 altyapısına uyarlandı.

### 🎨 4. Tema, Görsel ve Duvar Kağıdı Düzeltmeleri
- **Aydınlık Mod Entegrasyonu:** Caelestia çubuğu, paneller ve bileşenlerin aydınlık modda kusursuz açık renge geçmesi sağlandı.
- **İkon Düzeltmesi:** Dosya seçicilerde ve arayüzde oluşan mor/siyah kutu ikon hataları giderildi.
- **Genişletilmiş Duvar Kağıdı Formatları:** Caelestia'nın yalnızca tek bir formatı görme sorunu çözüldü; hem JPEG hem PNG duvar kağıtları sisteme entegre edildi (`/usr/share/backgrounds/blazeos`).
- **Sanal Masaüstü Çubuğu:** Sağ üstte yer alan sanal masaüstü (workspace) geçiş göstergesindeki bulanıklık ve etkileşim engeli giderildi.

### 💻 5. Çift Terminal & Fastfetch
- Mevcut terminalin yanına bağımsız, yüksek performanslı **Kitty** terminali eklendi.
- Fastfetch çıktıları SolarUI / BlazeOS logosu ve özel donanım/yazılım metrikleriyle optimize edildi.

### ⚡ 6. Caelestia Shell Geçiş & Dinamik Bağlayıcı Onarımı
- **Geçiş Çökmesi ve Geri Düşme (Rollback) Çözüldü:** `solar-shell switch caelestia` komutunda Caelestia'nın başlatılamayıp 6 saniye sonra otomatik olarak Noctalia'ya geri dönmesine yol açan runtime yükleme hatası tespit edildi.
- **Kritik Sembol Hatası Giderildi (`TemperatureUnit::staticMetaObject`):** Caelestia'nın `Units.qml` dosyasının ihtiyaç duyduğu `caelestia::config::TemperatureUnit` sembolünün, eski `libcaelestia-config.so` sürümünde tanımlı olmaması nedeniyle Quickshell plugin yüklemesi başarısız oluyordu. Caelestia'nın tüm 9 C++/QML eklenti modülü (`core`, `config`, `settings`, `components`, `models`, `services`, `blobs`, `images`, `i18n`) aynı araç zinciriyle baştan derlendi ve hem kök dizine hem de `blazeos_custom_apps` katmanına işlendi.
- **Qt6 Özel Sembol (Qt_6_PRIVATE_API) İzolasyonu:** `caelestia-i18n` modülündeki AOT (Ahead-of-Time) derlemesi saf çalışma zamanı QML yorumlamasına dönüştürülerek ABI uyuşmazlığı giderildi.
- **Sistem Dinamik Bağlayıcı İzolasyonu (`ldconfig`):** Caelestia'nın özel Quickshell kütüphaneleri sistem genelindeki dinamik bağlayıcı (`ld.so.conf`) yerine `/usr/bin/quickshell` sarmalayıcısı ve `solar-shell` sürecine özel `LD_LIBRARY_PATH` ile izole edildi. Bu sayede Fedora'nın yerel Qt6 kütüphaneleriyle SDDM arasındaki ELF sembol çakışmaları (`GNU_PROPERTY_1_NEEDED_INDIRECT_EXTERN_ACCESS`) kökten engellendi.

### 🛡️ 7. Arka Plan Servisleri, Telemetri ve Bloat Arındırma
- **NetworkManager Telemetri Engelleme:** Fedora'nın varsayılan olarak her 300 saniyede bir `hotspot.txt` pingi atarak ağ gecikmesi oluşturan ve telemetri toplayan captive portal sorgusu `/etc/NetworkManager/conf.d/99-disable-telemetry.conf` ile kapatıldı.
- **Gereksiz Arka Plan Servisleri Devre Dışı:** Masaüstü performansını ve disk G/Ç yanıt hızını düşüren `auditd` (güvenlik denetim izleme), `sssd` (kurumsal LDAP), `stratisd` (depolama yöneticisi), `mcelog`, `rsyslog` (çift günlükleme), `ModemManager` (seri port tarama) ve 27 adet `virt*.socket` servisi kapatıldı.
- **DNF Otomatik Önbellek Zamanlayıcısı:** `dnf-makecache.timer` devreden çıkarılarak oyun veya çalışma esnasında beklenmedik disk/CPU kilitlenmeleri önlendi.
- **Live ISO Otomatik Giriş Koruması:** `blazeos-postinstall.service` servisinin Live ISO oturumu sırasında çalışıp `liveuser` hesabını silmesi ve GDM autologin'i bozması engellendi.

### 🖥️ 8. SDDM Varsayılan Giriş Yöneticisi & GNOME Bileşenlerinden Arındırma
- **SDDM Varsayılan Olarak Etkinleştirildi:** GDM3 ve ağır GNOME arka plan servisleri devreden çıkarıldı. SDDM 0.21 (Qt6) hafif oturum yöneticisi sisteme entegre edildi (`/etc/systemd/system/display-manager.service -> sddm.service`). Sistem açılışında doğrudan ~350-400 MB RAM tasarrufu sağlandı.
- **GNOME Ayarlar ve Çakışmalar Kaldırıldı:** Sistemde açılmayan, çöken ve arayüzü kalabalıklaştıran `gnome-control-center`, `gnome-shell`, `gnome-tour`, `gnome-initial-setup` ve GNOME kabuk eklentileri paket seviyesinde temizlendi. Uygulama menüsündeki tüm ayar talepleri doğrudan yerel **SolarUI Ayarları** (`solar-settings`) aracına yönlendirildi.
- **Sistem İzleyicisi:** GNOME Sistem Monitörü ISO'dan çıkarıldı. Yerine resmî Fedora 45 `htop` paketi ve SolarUI/Kitty ile açılan `Blaze Sistem İzleyicisi` masaüstü girdisi eklendi.
- **UEFI GRUB Onarımı:** Kurulumdan sonra çıplak `grub>` istemine düşmeye neden olan mock derleme yolu temizlendi; taşınabilir EFI GRUB stub'ı ve kurulum sonrası otomatik onarım eklendi.
- **SDDM Canlı Açılış ve Weston Kiosk Entegrasyonu:** Weston kiosk bileşimi yalnız canlı ISO yolunda kullanılıyor; kurulu sistem paketlenmiş, donanım uyumlu SDDM greeter yolunu koruyor. `sddm.service.d/10-livesys.conf` ile canlı oturum başlangıcı `livesys.service` arkasına güvenle sıralandı.
- **Tek Seferlik OOBE ve Oturum Temizliği:** İlk kurulum tamamlanınca `BlazeOS Kurulum Asistanı` oturumu, geçici autologin ayarı ve AccountsService kaydı atomik olarak kaldırılıyor. Açılış öncesi hijyen servisi yarım kalan temizliği tamamlıyor; SDDM'de stok olarak yalnız SolarUI kalıyor.
- **Kullanıcı Kontrollü Otomatik Giriş:** Live ISO'nun `liveuser` girişi ayrı tutuluyor. Kurulu sistemde otomatik giriş yalnız OOBE'de kullanıcı açıkça seçerse oluşturuluyor; parola alanları hazır değer içermiyor.

### 🎯 9. Performans GiB Gösterimi, Bellenim, Çeviriler ve Ekran Köşeleri Sınırlandırması
- **Performans Çekmecesi GiB Gösterimi:** `MemoryCard.qml` ve `StorageCard.qml` içindeki tanımsız fonksiyon referansı giderilerek `Units.formatKibUsage(...)` bağlandı. Bellek ve disk kartlarında yüzdelik bilginin altında dinamik `14.4 / 62.4 GiB` ve `675 / 937.9 GiB` değerlerinin kusursuz görüntülenmesi sağlandı.
- **Bellenim (Firmware) Düzeltmesi:** Sanal makine ve SeaBIOS kaynaklı `Arch Linux 1.17.0-2-2` dizesi `SysInfo.qml` içinde arındırılarak temiz ve standart `UEFI / BIOS` bilgisine dönüştürüldü.
- **Hava Durumu Tam Türkçe Desteği:** Hava durumu çekmecesindeki "Sunrise", "Sunset", "Humidity", "Feels Like", "Wind", "No weather", 7 günlük tahmin ve 26 farklı hava durumu durum kodu Türkçe gettext kataloğuna eklendi, `tr.mo` olarak derlenip tüm sistem katmanlarına dağıtıldı.
- **Ekran Köşeleri & Kör Nokta Sınırlandırması:**
  - **Niri Güvenli Kenar Sınırları (`layout.struts`):** `top 10`, `left 10`, `right 10`, `bottom 52` marjları tanımlandı. Google Chrome sekmeleri ve pencere başlıkları ekranın en tepesindeki çekmece tetikleyicilerinden bağımsız, rahatça tıklanabilir konuma çekildi.
  - **Anaconda Kurulum Sihirbazı Kuralı:** Kurulum penceresi ekranda tam ekran yayılmak yerine %88 genişlik/yükseklikte ortalanmış ve yüzen (`open-floating`) pencere olarak yapılandırıldı. Sağ-alt köşedeki "İleri" butonunun köşe tetikleyicileriyle çakışması kökten önlendi.
  - **Caelestia Çekmece Maskesi Daraltması:** Çekmeceler kapalıyken ekran kenarında oluşan 10–35 piksellik gereksiz girdi maskesi maksimum 2–3 piksele sıkıştırıldı; pencerelerin kenar ve köşe tıklama alanları tamamen serbest bırakıldı.

### 🧰 10. Kurulum, OOBE ve Önyükleme Dayanıklılığı
- **Kalıcı UEFI GRUB Düzeltmesi:** Kurulum ortamına ait `/root/var/lib/mock/.../image-root` yolu ESP'ye sızdırılmıyor. Taşınabilir EFI güvenlik stub'ı gerçek `/grub2/grub.cfg` veya `/boot/grub2/grub.cfg` dosyasını bularak sistemi çıplak `grub>` istemine düşmeden açıyor.
- **Tek Seferlik Kurulum Asistanı:** OOBE tamamlandığında `BlazeOS Kurulum Asistanı` oturumu, geçici autologin yapılandırması ve `blaze-setup` AccountsService kaydı kaldırılıyor. Açılış öncesi hijyen servisi yarıda kalan temizliği güvenle tamamlıyor.
- **Güvenli Hesap Oluşturma:** Parola alanları boş başlıyor; ISO içinde tahmin edilebilir hazır parola bulunmuyor. Otomatik giriş yalnız kullanıcı açıkça seçerse etkinleşiyor.
- **İkinci Açılış Siyah Ekran Düzeltmesi:** Kurulu sistemi Weston kiosk greeter yoluna zorlayan SDDM ayarı kaldırıldı. Donanımla uyumlu paket varsayılanı korunuyor ve oturum listesinde stok olarak yalnız SolarUI gösteriliyor.
- **Derleme Güvenlik Kapıları:** ISO üretimi; derleme makinesi yolu taşıyan GRUB dosyasını, hazır OOBE parolasını, bozuk OOBE servis bağlantısını veya sorunlu SDDM greeter zorlamasını algılarsa duruyor.

---

## 🌟 About (Hakkında)

**Blaze SolarEvolution 5.04** is a high-performance Linux operating system ecosystem built on Fedora Workstation, specifically optimized for gamers, developers, and content creators.

The system ships **SolarUI (Niri Wayland + Noctalia + Caelestia)** as its only offline stock desktop. Eight optional sessions can be selected when the installer verifies access to the official Fedora repositories. GRUB2 and the bundled upstream Limine release are available as bootloader choices on supported firmware.

---

## 🔥 Key Features (Temel Özellikler)

- **SolarUI Desktop Ecosystem:** A clean, fluid desktop experience with glassmorphism support, built on the Niri Wayland window manager, the fast C++ Noctalia shell, and the elegant Caelestia shell.
<p align="center">
  <img width="1205" height="753" alt="image" src="https://github.com/user-attachments/assets/beac43d9-0943-44c9-b951-712629f23456" />
</p>

- **SolarUI plus 8 Online Desktop Options:**
  - **SolarUI:** The default fluid Hybrid desktop (Noctalia + Caelestia, Available Offline)
  - **GNOME:** Installed on demand from the official Fedora repositories
  - **Niri (Standalone):** An optional standalone session using SolarUI's bundled Niri compositor
  - **KDE Plasma:** Highly customizable Plasma 6 on Wayland
  - **COSMIC:** An independent, modern, Rust-based desktop by System76
  - **Hyprland:** A dynamic tiling Wayland environment with fluid animations
  - **Cinnamon:** A classic and practical desktop layout
  - **XFCE:** An extremely lightweight and fast X11 environment
  - **Sway:** An i3-compatible, keyboard-driven tiling environment
- **Limine & GRUB2 Bootloader Support:** GRUB2 remains the Secure Boot compatible default. UEFI installations can select the bundled official Limine 12.9.0 release; no third-party COPR is enabled during installation.
- **BlazeOS Control Center (`blazeos-control`):** A system control center built using GTK4 / Libadwaita, featuring 5 tabs: Updates, Desktop, Performance, Tools, and About.
- **Low Latency & ZRAM Improvements:** DNF5 configurations for faster downloads, default ZRAM optimizations, and CachyOS kernel/animation tuning settings.
- **Online & Offline ISO Architecture:** Build support for a lightweight 3.5 GB Online ISO and a fully bundled 10 GB+ Offline ISO.

---

## 🛠️ Architecture (Sistem Yapısı)

```text
Blaze-Galaxy / Blaze SolarEvolution 5.04
├── build_f45_final.sh          # Online ISO build script (Squashfs + xorriso + SELinux)
├── build_f45_offline.sh        # Offline ISO build script
├── blazeos_custom_apps/        # Custom-developed system tools and desktop configurations
│   ├── usr/local/bin/
│   │   ├── blazeos-control     # GTK4 Control Center
│   │   ├── blazeos-welcome     # Welcome and initial setup wizard
│   │   ├── blaze-bootloader    # Limine / GRUB2 switching tool
│   │   ├── blazeos-postinstall # Post-installation automation service
│   │   └── blaze-optimize      # ZRAM & kernel optimizations
│   ├── usr/bin/firehub         # FireHub smart wrapper
│   ├── usr/share/caelestia/    # Caelestia translations (tr.mo)
│   └── usr/lib64/quickshell/   # Quickshell Caelestia I18n and plugin modules
├── solarui/                    # SolarUI components and source code
│   └── vendor/caelestia-shell/ # Caelestia shell QML and C++ I18n plugin code
├── work_f45/rootfs/            # Live system root filesystem
└── README.md                   # Release documentation (5.04)
```

---

## 📦 Build Requirements (Derleme Gereksinimleri)

To build the ISO image locally, the following packages and tools must be installed on your system:

- **Operating System:** Fedora 40+, Arch Linux, CachyOS, or a RHEL-based 64-bit Linux distribution
- **Disk Space:** At least 30 GB of free tmpfs / disk space
- **Required Tools:**
  ```bash
  # On Fedora / RHEL:
  sudo dnf install -y xorriso isolinux squashfs-tools isomd5sum libattr-devel
  
  # On Arch Linux / CachyOS:
  sudo pacman -S --needed xorriso squashfs-tools attr
  ```

---

## 🏗️ How to Build (ISO Derleme Adımları)

### 1. Clone the Repository
```bash
git clone https://github.com/DarkMorpheus-pc/Blaze-Galaxy.git
cd Blaze-Galaxy
```

### 2. Prepare the Base ISO Image
Place the Fedora Workstation 45 / 44 Live ISO image in the `f45_base/` directory.

### 3. Run the ISO Build Script

- **Online ISO Build (3.5 GB):**
  ```bash
  chmod +x build_f45_final.sh
  sudo ./build_f45_final.sh
  ```

- **Full Offline ISO Build (10 GB+):**
  ```bash
  chmod +x build_f45_offline.sh
  sudo ./build_f45_offline.sh
  ```

Once the build is complete, your ISO image will be available in the repository root directory (`Blaze-SolarEvolution-5-x86_64.iso`) and will pass the MD5 verification check (`checkisomd5 PASS`).

---

## ⌨️ SolarUI Keyboard Shortcuts (Kısayol Tuşları)

| Shortcut | Action (İşlev) |
| :--- | :--- |
| `Mod + Return` | Terminal (Kitty / Ptyxis / Alacritty) Aç |
| `Mod + Space` | Uygulama Başlatıcı (Launcher) |
| `Mod + S` | SolarUI Kontrol Merkezi |
| `Mod + I` | Masaüstü Ayarları (Nexus) |
| `Mod + Alt + R` | Masaüstü Kabuğunu Yeniden Başlat (Supervisor) |
| `Mod + Alt + S` | QuickShell / SolarUI Hızlı Ayarlar |
| `Mod + Q` / `Alt + F4` | Aktif Pencereyi Kapat |
| `Mod + Shift + E` | Oturumu Kapat / Çıkış Menüsü |

---

## 📄 License & Contributing (Lisans ve Katkı)

This project is released under the **GPL-3.0** license. You can contribute by opening a pull request or reporting an issue.

Designed & Crafted for **BlazeOS & CachyOS** by **DarkMorpheus**.
