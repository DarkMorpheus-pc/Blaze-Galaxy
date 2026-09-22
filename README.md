# BlazeOS SolarEvolution 5 (Blaze-Galaxy)

<p align="center">
  <img src="https://raw.githubusercontent.com/DarkMorpheus-pc/Blaze-Galaxy/main/blazeos_custom_apps/usr/share/icons/hicolor/256x256/apps/solarui.png" alt="BlazeOS Logo" width="140" />
</p>

<h3 align="center">Yeni Nesil, Hibrit Masaüstü Ekosistemi & Yüksek Performanslı Linux Dağıtımı</h3>

<p align="center">
  <b>"From evolution to expansion — welcome to the galaxy."</b>
</p>

---

## 🌟 Hakkında (About)

**BlazeOS SolarEvolution 5**, Fedora Workstation temeli üzerine inşa edilmiş; oyuncular, geliştiriciler ve içerik üreticileri için özel olarak optimize edilmiş yüksek performanslı bir Linux işletim sistemi ekosistemidir. 

Sistem, yenilikçi **SolarUI (Niri Wayland + Noctalia)** masaüstü kabuğunu, 9 farklı masaüstü ortamı seçeneğini ve çift önyükleyici (GRUB2 & Limine) yönetimini tek bir yapıda birleştirir.

---

## 🔥 Öne Çıkan Özellikler (Key Features)

- 🚀 **SolarUI Desktop Ecosystem:** Niri Wayland pencere yöneticisi ve Noctalia hızlı kabuğu üzerine kurulu saf, akıcı ve cam efekti (glassmorphism) destekli masaüstü deneyimi.
- 🎨 **9 Farklı Masaüstü Ortamı Seçeneği:**
  - **SolarUI:** Varsayılan akıcı Niri + Noctalia masaüstü (Çevrimdışı Hazır)
  - **GNOME:** Modern ve kararlı GNOME 50 (Çevrimdışı Hazır)
  - **Niri (Saf):** Sonsuz kaydırmalı saf pencere yöneticisi (Çevrimdışı Hazır)
  - **KDE Plasma:** İleri düzey özelleştirilebilir Plasma 6 Wayland
  - **COSMIC:** System76 Rust tabanlı bağımsız modern masaüstü
  - **Hyprland:** Akıcı animasyonlu dinamik döşemeli Wayland ortamı
  - **Cinnamon:** Klasik ve pratik masaüstü düzeni
  - **XFCE:** Son derece hafif ve hızlı X11 ortamı
  - **Sway:** i3 uyumlu klavye odaklı döşemeli ortam
- ⚡ **Limine & GRUB2 Önyükleyici Desteği:** Yıldırım hızında açılış sağlayan Limine önyükleyicisi ile Secure Boot destekli standart GRUB2 arasında tek komutla geçiş (`blaze-bootloader`).
- 🛠️ **BlazeOS Control Center (`blazeos-control`):** GTK4 / Libadwaita standartlarında geliştirilmiş 5 sekmeli (Güncellemeler, Masaüstü, Performans, Araçlar, Bilgi) sistem kontrol merkezi.
- ⚡ **Düşük Gecikme & ZRAM İyileştirmeleri:** DNF5 hızlı indirme yapılandırmaları, ZRAM varsayılan optimizasyonları ve CachyOS kernel/animasyon tuning ayarları.
- 🌐 **Çevrim İçi & Çevrim Dışı ISO Mimarisi:** 3.5GB hafif Online ISO ve 10GB+ tam paketli Offline ISO derleme desteği.

---

## 🛠️ Sistem Yapısı ve Bileşenler (Architecture)

```
Blaze-Galaxy / BlazeOS SolarEvolution 5
├── build_f45_final.sh          # 3.5GB Online ISO derleme betiği
├── build_f45_offline.sh        # 10GB+ Offline ISO derleme betiği
├── blazeos_custom_apps/        # Özel geliştirilmiş sistem araçları ve masaüstü yapılandırmaları
│   ├── usr/local/bin/
│   │   ├── blazeos-control     # GTK4 Control Center
│   │   ├── blazeos-welcome     # Karşılama ve ilk kurulum sihirbazı
│   │   ├── blaze-bootloader    # Limine / GRUB2 geçiş aracı
│   │   ├── blazeos-postinstall # Kurulum sonrası otomasyon servisi
│   │   └── blaze-optimize      # ZRAM & kernel optimizasyonları
│   └── usr/bin/firehub         # FireHub akıllı sarmalayıcı
├── solarui/                    # SolarUI Rust bileşenleri ve kaynak kodları
├── niri-src/                   # Niri Wayland compositor özelleştirmeleri
└── README.md
```

---

## 📦 Derleme Gereksinimleri (Build Requirements)

ISO imajını yerel ortamınızda derlemek için aşağıdaki paket ve araçların sisteminizde kurulu olması gereklidir:

- **İşletim Sistemi:** Fedora 40+, Arch Linux, CachyOS veya RHEL tabanlı 64-bit Linux
- **Disk Alanı:** En az 30 GB boş tmpfs / disk alanı
- **Gerekli Araçlar:**
  ```bash
  # Fedora / RHEL üzerinde:
  sudo dnf install -y xorriso isolinux squashfs-tools isomd5sum libattr-devel
  
  # Arch Linux / CachyOS üzerinde:
  sudo pacman -S --needed xorriso squashfs-tools attr
  ```

---

## 🏗️ ISO Derleme Adımları (How to Build)

### 1. Depoyu Klonlayın
```bash
git clone https://github.com/DarkMorpheus-pc/Blaze-Galaxy.git
cd Blaze-Galaxy
```

### 2. Taban ISO İmajını Hazırlayın
Fedora Workstation 45 / 44 Live ISO imajını `f45_base/` klasörü altına yerleştirin.

### 3. ISO Derleme Betiğini Çalıştırın

- **Çevrim İçi (Online) ISO Derlemesi (3.5 GB):**
  ```bash
  chmod +x build_f45_final.sh
  sudo ./build_f45_final.sh
  ```

- **Tam Çevrim Dışı (Offline) ISO Derlemesi (10 GB+):**
  ```bash
  chmod +x build_f45_offline.sh
  sudo ./build_f45_offline.sh
  ```

Derleme tamamlandığında ISO imajınız kök dizinde (`Blaze-SolarEvolution-5-x86_64.iso`) hazır olacak ve MD5 doğrulama kontrolünden (`checkisomd5 PASS`) geçecektir.

---

## ⌨️ Kısayol Tuşları (SolarUI Keyboard Shortcuts)

| Kısayol | İşlev |
| :--- | :--- |
| `Mod + Return` | Terminal (Alacritty / Ptyxis) Aç |
| `Mod + Space` | Uygulama Başlatıcı (Launcher) |
| `Mod + S` | SolarUI Kontrol Merkezi |
| `Mod + I` | Masaüstü Ayarları |
| `Mod + Alt + R` | Masaüstü Kabuğunu Yeniden Başlat (Supervisor) |
| `Mod + Alt + S` | QuickShell / SolarUI Hızlı Ayarlar |
| `Mod + Q` / `Alt + F4` | Aktif Pencereyi Kapat |
| `Mod + Shift + E` | Oturumu Kapat / Çıkış Menüsü |

---

## 📄 Lisans ve Katkı (License & Contributing)

Bu proje **GPL-3.0** lisansı altında sunulmaktadır. Katkıda bulunmak için pull request açabilir veya sorun bildiriminde (issue) bulunabilirsiniz.

Designed & Crafted for **BlazeOS & CachyOS** by **DarkMorpheus**.
