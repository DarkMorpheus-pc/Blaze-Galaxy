# BlazeOS Kurtarma ve Acil Durum Mimarisi (BERP & BERE)

Bu doküman, BlazeOS bünyesinde geliştirilen acil durum kurtarma ekosisteminin (BERP - Blaze Emergency Recovery Platform ve BERE - Blaze Emergency Recovery Environment) mimarisini, dosya sistemi izolasyonunu, arayüz çalışma mantığını ve felaket kurtarma senaryolarını detaylandırır.

---

## 1. Mimari Genel Bakis

BlazeOS acil durum yapısı iki temel bileşenden meydana gelir:

1. **BERP (Blaze Emergency Recovery Platform):**
   - Çekirdek (kernel), initramfs ve acil durum servislerinin entegre olduğu altyapı katmanıdır.
   - Sistem açılışında GRUB menüsünden ("BlazeOS Emergency Recovery Environment (BERE)") veya sistemin kritik bir çökme (kernel panic, rootfs bozulması, acil durum modu) yaşaması durumunda otomatik olarak devreye girer.

2. **BERE (Blaze Emergency Recovery Environment - OrangeFox Tarzi GUI):**
   - Bağımsız bir Cage/Weston Wayland kompozitörü üzerinde koşan, tam donanım hızlandırmalı ve dokunmatik/klavye/fare destekli modern kurtarma arayüzüdür.
   - Akıllı telefon dünyasındaki popüler OrangeFox Recovery felsefesinden esinlenilmiştir; yanlışlıkla veri silmeyi önleyen kaydırmalı onay sürgüleri (swipe-to-confirm) ve modüler kart tasarımına sahiptir.
   - Tamamen çift dil (Türkçe ve İngilizce) desteği barındırır.
   - Sistemde hiçbir sahte buton bulunmaz; her kontrol doğrudan Python backend API (`berp-recovery-gui`) aracılığıyla Linux çekirdeği ve sistem yardımcı programları (btrfs, parted, efibootmgr, dracut, grub2) ile haberleşir.

---

## 2. İzolasyon ve Kök Dizin Ayrımı (Isolation Architecture)

En önemli tasarım gereksinimlerinden biri, kök dosya sistemi (`/`) tamamen silinse dahi kurtarma ortamının ve acil durum ekranının hayatta kalmasıdır.

### Ayrım Mekanizması:
- Standart BlazeOS kurulumunda `/boot` ve `/boot/efi` dizinleri kök dosya sisteminden (`/dev/mapper/bs-root` veya Btrfs subvolume) bağımsız ayrı fiziksel bölümlerde (örneğin `/dev/vda2` ve `/dev/vda1`) yer alır.
- BERE çalışma ikilileri ve arayüz dosyaları, yıkıcı bir sıfırlama (Hard Reset / Wipe) öncesinde otomatik olarak `/boot/bere/` dizinine senkronize edilir:
  - `/boot/bere/bin/berp-recovery-gui`
  - `/boot/bere/bin/blaze-death-screen`
  - `/boot/bere/share/berp-recovery/index.html`
  - `/boot/bere/share/berp-recovery/style.css`
  - `/boot/bere/share/berp-recovery/app.js`
- Böylece kullanıcı `sudo rm -rf /*` (Hard Reset) çalıştırsa veya kök dosya sistemi geri dönülemez şekilde hasar alsa bile, `/boot` altındaki BERE imajı ve kurtarma çekirdeği bozulmadan korunur.

---

## 3. Özel Ölüm Ekranı (Blaze Emergency Guard / Custom Death Screen)

Linux sistemlerinde meydana gelen standart "Kernel Panic" veya sistemd `emergency.target` / `rescue.target` metinleri kullanıcı dostu değildir ve sistemi kilitlenmiş hissettirir. BlazeOS bu durumu ortadan kaldırmak için özel bir TTY Acil Durum Ekranı (`blaze-death-screen`) içerir:

### Çalışma Şekli:
- `emergency.service` ve `rescue.service` systemd birimleri `override.conf` dosyalarıyla yamalanmıştır.
- Sistem boot edemediğinde veya dosya sistemi mount hatası aldığında standart bash komut satırı yerine ANSI formatlı BlazeOS Death Screen ekrana gelir.
- Kullanıcıya sunulan seçenekler:
  1. **[ENTER] BERE Görsel Kurtarma Ortamını Başlat:** Doğrudan grafik arayüze (`systemctl isolate blaze-recovery-gui.target`) geçiş yapar.
  2. **[R] Sistemi Yeniden Başlat:** Sistemi güvenli şekilde yeniden başlatır.
  3. **[S] Acil Durum Kök Terminali:** Konsol seviyesinde kurtarma yapmak isteyen ileri düzey kullanıcılar için root shell açar.

---

## 4. BERE Arayüz Sekmeleri ve Yetenekleri

BERE arayüzü 8 ana modülden oluşur:

### 1. Yükle (ISO Kurulumu - Flash/Direct Boot)
- Sistem disklerinde, USB sürücülerde veya `Downloads` klasöründe bulunan `.iso` dosyalarını otomatik olarak tarar.
- USB belleğe ihtiyaç duymadan, seçilen ISO kalıbını loopback yöntemiyle GRUB yapılandırmasına ekler ve doğrudan ISO üzerinden kurulum/başlatma imkanı sunar.
- İşlem OrangeFox onay sürgüsü ile tetiklenir.

### 2. Yedekle (Btrfs Snapshots)
- Btrfs Copy-on-Write (CoW) altyapısını kullanarak milisaniyeler içerisinde sistemin kök (`/`) ve kullanıcı (`/home`) anlık görüntülerini (snapshot) alır.
- Disk alanı tüketimi sıfıra yakındır.

### 3. Geri Yükle (System Rollback)
- Alınmış olan Btrfs kurtarma noktalarını listeler.
- Sistem hatalı bir güncelleme aldığında veya bozulduğunda kullanıcı tek bir kaydırma hareketiyle sistemi eski sağlıklı durumuna geri döndürebilir.

### 4. Sıfırla (Wipe & Factory Reset & Hard Reset)
Bu sekmede 5 farklı temizleme seviyesi bulunur:
- **Sistem ve Paket Önbellekleri (`/var/cache`):** DNF, Flatpak ve geçici sistem artıklarını temizler. Kullanıcı dosyalarına dokunmaz.
- **Kullanıcı Önbellekleri (`~/.cache`):** Tarayıcı ve masaüstü önbelleklerini sıfırlar. Belgeleri ve şifreleri korur.
- **Kullanıcı Hesaplarını ve Verilerini Sıfırla (`/home`):** Kullanıcı hesaplarını siler ve sistem açılışını ilk karşılama sihirbazına (OOBE - "Merhaba") yönlendirir.
- **Tam Fabrika Sıfırlaması (Format Data / Root Subvolumes):** Sistem kökünü ve ayarları tamamen sıfırlar, logları ve geçici dosyaları temizler.
- **Tam Sistem Temizliği (Hard Reset / Wipe System Data - `sudo rm -rf /*`):**
  - Bu seçenek seçildiğinde doğrudan aktifleşmez; özel bir güvenlik penceresi (modal) açılır.
  - Modal üzerinde *"Bu işlem geri alınamaz emin misiniz?"* uyarısı yer alır.
  - Kullanıcı kutucuğa tam olarak **"YES"** yazmak ve yetkili/root şifresini girmek zorundadır.
  - Onay verilmeden kaydırma sürgüsü açılmaz.
  - İşlem başladığında `/boot/bere` dizini korunarak sistem temizliği icra edilir ve ardından sistem kurtarma moduna yeniden başlatılır.

### 5. Diskler ve Bölümler (Partitions & SMART Health)
- Sistemdeki tüm blok aygıtlarını (NVMe, SSD, HDD, USB) ve dosya sistemlerini listeler.
- Tek tıkla Bağlama (Mount) ve Ayırma (Unmount) işlemleri yapılabilir.
- NVMe/SMART sağlık telemetrisi ile disk ömrü ve sektör sağlığı denetlenir.

### 6. Önyükleme Onarımı (Boot Repair)
- **GRUB / Limine Onarımı:** `/etc/grub.d` yapılandırmasını yeniden tarar ve `grub2-mkconfig` çalıştırır.
- **EFI NVRAM Onarımı:** Anakart UEFI NVRAM tablosundaki bozuk veya kayıp boot girişlerini `efibootmgr` ile onarır.
- **Dracut Initramfs Yeniden Derleme:** Sistemdeki tüm kurulu çekirdekler için initramfs imajlarını baştan üretir.

### 7. Acil Durum Terminali (Root Emergency Shell)
- Grafik arayüz içerisine gömülü, doğrudan root yetkilerine sahip interaktif konsoldur.
- Kullanıcı herhangi bir Linux komutunu çalıştırabilir ve çıktısını anında görebilir.

### 8. Yeniden Başlat ve Güç (Reboot & Power)
- Sistemi Normal Modda Başlat
- Kurtarmayı Yeniden Başlat
- Doğrudan UEFI / BIOS Ayarlarına Git
- Bilgisayarı Kapat

---

## 5. Çift Dil (TR | EN) Mimarisi

BERE arayüzünün sağ üst durum çubuğunda `TR | EN` geçiş butonu yer alır:
- Kullanıcı dilediği an arayüz dilini Türkçeden İngilizceye veya İngilizceden Türkçeye dönüştürebilir.
- Dil tercihi yerel depolamada (`localStorage`) saklanır ve oturumlar arasında korunur.
- Sekme başlıkları, açıklamalar, kart metinleri, modal uyarıları, butonlar ve sürgü etiketleri dahil tüm metinler dinamik çeviri sözlüğü üzerinden anında güncellenir.

---

## 6. Servisler ve Dosya Haritası

| Dosya / Dizin | Görevi |
|---|---|
| `/usr/bin/berp-recovery-gui` | BERE arka plan REST API sunucusu ve sistem işlemlerini icra eden Python çekirdeği |
| `/usr/bin/blaze-death-screen` | Acil durum/çökme anında TTY1'de beliren özel BlazeOS Ölüm Ekranı |
| `/usr/share/berp-recovery/index.html` | OrangeFox tarzı kurtarma arayüzünün ana şablonu |
| `/usr/share/berp-recovery/style.css` | Koyu tema, modern kartlar ve kaydırmalı sürgü stilleri |
| `/usr/share/berp-recovery/app.js` | REST API istemcisi, çoklu dil yöneticisi ve modal kontrol mantığı |
| `/etc/systemd/system/emergency.service.d/override.conf` | Çekirdek acil durumunda `blaze-death-screen`i başlatan systemd yaması |
| `/etc/systemd/system/rescue.service.d/override.conf` | Kurtarma modunda `blaze-death-screen`i başlatan systemd yaması |
| `/etc/systemd/system/blaze-recovery-gui.service` | BERE GUI sunucusunu başlatan systemd servisi |
| `/usr/bin/blaze-recovery-gui-launcher` | seatd/DRM denetimli güvenilir Wayland grafik başlatıcısı |
| `/etc/systemd/system/blaze-recovery-gui.target` | BERE grafik ortamını izole eden systemd hedefi |
| `/boot/bere/` | Kök dosya sistemi silinse dahi kurtarmanın çalışmasını sağlayan izole koruma alanı |

---

## 7. OOBE (İlk Karşılama Sihirbazı) Entegrasyonu

Wipe sekmesinde kullanıcı verileri veya fabrika ayarları sıfırlandığında:
1. Mevcut tüm insan kullanıcı hesapları `/etc/passwd` üzerinden güvenli şekilde kaldırılır (`userdel -r -f`).
2. `/home/*` dizini temizlenir.
3. Kurulum tamamlandı bayrağı (`/etc/blazeos-setup-completed`) ve otomatik giriş dosyası (`/etc/sddm.conf.d/autologin.conf`) silinir.
4. `/etc/sddm.conf.d/00-blaze-oobe.conf` dosyası oluşturularak SDDM'in açılışta doğrudan `blaze-setup` kullanıcısıyla el yazısı "Merhaba" animasyonlu kurulum sihirbazını başlatması garanti edilir.
