# Blaze'e Özgü Katman / What Makes BlazeOS More Than Fedora

Bu belge yalnızca **depodaki kodla doğrulanmış** özellikleri listeler. Deneysel olanlar açıkça işaretlidir.
This document lists only features **verified against code in the repositories**. Experimental items are labelled.

Durum / Status: `Stable` kararlı · `Beta` kullanılabilir, kenarları pürüzlü · `Experimental` araştırma/prototip

## 1. Kendi yazdığımız yazılımlar / Software we wrote

| Bileşen / Component | Dil / Lang | Ne yapar / What it does | Durum | Kaynak / Source |
|---|---|---|---|---|
| **SolarUI** (`solar-core`, `solar-shell`, `solar-session`) | Rust + QML/C++ | Niri tabanlı oturum yöneticisi; Noctalia ve Caelestia kabuk motorları arasında canlı geçiş (`solar-shell switch`), kabuk çökerse otomatik gözetim | Stable | [SolarUI](https://github.com/DarkMorpheus-pc/SolarUI) |
| **BlazeHouse** | Rust (Iced) | 18 sayfalı sistem merkezi: Ekranlar, Depolama, Optimizasyon, ZRAM, Pil, Ağ, Bluetooth, Ses, Güvenlik, Giriş, Kullanıcılar, Erişilebilirlik, Dil/Bölge vb.; DNF5 çevrimdışı güncelleme | Stable | Yerel depo (`blaze-house`) |
| **FireHub** | Electron/TS | Flatpak + RPM + DEB + Pisi için tek arayüz; Güncelleme Merkezi; çevrimdışı ikon paketi | Stable | [Blaze-Fire-Hub](https://github.com/DarkMorpheus-pc/Blaze-Fire-Hub) |
| **BERP / BERE** (`blaze-recovery`, `berp-recovery-gui`) | Rust + Python | Kök dizin silinse bile `/boot/bere` üzerinden açılan, kaydırarak onaylamalı kurtarma ortamı (btrfs, parted, efibootmgr, dracut, grub2 araçlarına bağlı) | Beta | [docs/recovery.md](recovery.md) |
| **Blaze Sentinel** v0.3 | Rust | Güvenlik duruşu denetçisi: SELinux/AppArmor, Secure Boot, kernel lockdown, güvenlik duvarı, sysctl sertleştirme; `--json` ve `verify` çıkış kodları | Beta | [Blaze-Universe-](https://github.com/DarkMorpheus-pc/Blaze-Universe-) |
| **Blaze Continuum** v0.1 | Rust | CRIU tabanlı **oturum ışınlama**: odaklı pencereyi dondurup `.blz-state` kapsülü yapar, LAN üzerinden başka makinede (BCTP/1) canlandırır; Ghost Overlay ile ışınlanırken hayalet yüzey gösterir | **Experimental** | [Blaze-Universe-](https://github.com/DarkMorpheus-pc/Blaze-Universe-) |
| **Blaze GameZone** | `solar-shell` alt komutu (`gamezone`) | Tam ekran konsol tarzı oyun kabuğu/başlatıcı (masaüstü girdisi: Oyun/Konsol/Emülatör kategorileri) | Beta | SolarUI |
| **Blaze Death Screen** | Python | Kritik hatada terminal tabanlı acil ekran; kısayolla BERE kurtarma ortamına geçiş | Beta | `usr/bin/blaze-death-screen` |
| **solar-torture** | Bash | Pencere açma/kapama fırtınası, olay seli, hata enjeksiyonu; **Vanilla Niri ile A/B kıyas** (RSS/PSS/FD/thread telemetrisi) | Stable | `usr/local/bin/solar-torture` |
| **Kurulum akışı** (`blaze-setup`, `blazeos-desktop-provision`, `blazeos-apply-install-choices`) | Python/Bash | Anaconda sonrası: seçilen DE'leri arka planda kurar, SDDM'i yeniler, OOBE'yi tek seferlik temizler | Stable | `usr/local/bin` |
| **Tema merkezi** (`blaze-style-center`) | Python/GTK | GNOME/KDE hazır düzenleri, her değişiklikten önce yedek + tek tıkla geri yükleme | Beta | `usr/bin/blaze-style-center` |
| **blazeos-nvidia-setup** | Bash | İlk açılışta NVIDIA donanımı algılanırsa sürücüyü kurar (`blazeos-nvidia-firstboot.service`) | Beta | `usr/local/bin` |
| **blaze-bootloader** | Bash | GRUB2 ↔ Limine geçişi (Secure Boot uyumlu varsayılan GRUB2) | Beta | `usr/local/bin` |

## 2. Düz Fedora ile farklar / Differences from stock Fedora

| Konu / Topic | Düz Fedora Workstation | BlazeOS SolarEvolution 5.05 |
|---|---|---|
| Varsayılan masaüstü | GNOME | SolarUI (Niri + Noctalia/Caelestia), 7 isteğe bağlı DE |
| Oturum ışınlama | Yok | Blaze Continuum (deneysel) |
| Kurtarma | Anaconda rescue / metin | Grafik BERE, `/boot` ayrımıyla kök silinse bile açılır |
| Güvenlik duruşu raporu | Yok (elle `sestatus`, `mokutil`…) | `blaze-sentinel status/verify --json` |
| Oyun kabuğu | Yok | GameZone |
| Güncelleme arayüzü | GNOME Yazılım | BlazeHouse + FireHub, DNF5 çevrimdışı + Plymouth ilerleme |
| Dayanıklılık testi | — | `solar-torture` (Vanilla Niri A/B) |
| Arka plan telemetri | `hotspot` captive-portal ping aktif | NetworkManager captive-portal kontrolü kapalı |
| Türkçe | Genel çeviri | Caelestia'da 670+ dize yerel Qt çevirisi, canlı dil seçici |

> Bu tabloya yalnızca doğrulanmış satırlar girer. Performans karşılaştırmaları `docs/BENCHMARKS.md` içinde ölçümle birlikte yayımlanacaktır.

## 3. Bilinen sınırlar / Known limitations

- **Blaze Continuum deneyseldir** (v0.1): CRIU ve Niri sürümüne bağımlıdır, her uygulama dondurulamaz.
- **Sentinel şu an bir denetçidir**, politika *uygulayıcı* değildir; politikayı SELinux/AppArmor uygular.
- Snapshot/rollback henüz yoktur (planlanıyor: btrfs).
- Özel Rust ikili dosyalarının Fedora içinde yeniden üretilebilir derlenmesi için RPM paketleme çalışması sürmektedir (bkz. `SOURCES.md`).
