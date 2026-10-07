# Blaze SolarEvolution 5.04 — Kurulum ve Günlük Kullanım Test Formu

Bu formu her fiziksel bilgisayar veya sanal makine için ayrı kopyalayın. Her satırı `GEÇTİ`, `KALDI`, `UYGULANAMAZ` olarak işaretleyin; kalan maddelerde ekran görüntüsü ve günlük ekleyin.

## 1. Test kimliği

- Tarih / test eden:
- ISO dosyası: `Blaze-SolarEvolution-5-x86_64.iso`
- ISO SHA-256: `afab6337b8c1826c71003b82d303f25573543a94610966b9e946e2deca769df2`
- Donanım / VM yazılımı:
- Firmware: UEFI / Legacy BIOS
- Secure Boot: Açık / Kapalı
- CPU / RAM / disk:
- GPU ve sürücü:
- Ekranlar, çözünürlük, ölçek:
- Ağ: Ethernet / Wi-Fi / Çevrimdışı
- Sonuç özeti:

## 2. ISO ve açılış

- [ ] SHA-256 eşleşiyor: `sha256sum -c Blaze-SolarEvolution-5-x86_64.iso.sha256`
- [ ] ISO yazdırıldıktan sonra önyükleme menüsündeki medya kontrolü geçiyor.
- [ ] UEFI açılışı çalışıyor; GRUB menüsü, normal başlatma ve sorun giderme girdileri görünüyor.
- [ ] Legacy BIOS açılışı çalışıyor.
- [ ] Secure Boot açık testinin sonucu kaydedildi. Başarısızsa firmware mesajının fotoğrafı eklendi.
- [ ] Canlı masaüstü açılıyor; siyah/gri ekran, imleç kilitlenmesi ve sürekli yeniden başlama yok.
- [ ] Hoş geldiniz penceresi bir kez açılıyor, kapatma düğmesi ve `Esc` çalışıyor.

## 3. Kurulum sihirbazı — masaüstü ve ağ

### Tam çevrimdışı test

- [ ] VM ağ aygıtı çıkarıldı veya fiziksel ağ tamamen kapatıldı.
- [ ] Yalnızca **SolarUI** seçilebilir ve `Çevrimdışı Hazır` rozeti taşır.
- [ ] GNOME, Niri, KDE, COSMIC, Hyprland, Cinnamon, XFCE ve Sway soluktur, radyo düğmeleri devre dışıdır ve `İnternet Gerektirir` yazar.
- [ ] Devre dışı kartlara fare, klavye ve dokunmatik ile basmak seçimi değiştirmez.
- [ ] İleri/geri gidildiğinde seçim SolarUI olarak korunur.

### Bağlantı değişimi testi

- [ ] Ağ kurulum ekranı açıkken bağlandı; seçenekler en geç 20 saniyede etkinleşti.
- [ ] DNS var fakat internet yok senaryosunda seçenekler etkinleşmedi.
- [ ] Captive portal veya Fedora aynasına erişemeyen ağda seçenekler etkinleşmedi.
- [ ] Çevrimiçi bir masaüstü seçildikten sonra ağ kesildi; seçim en geç 20 saniyede SolarUI'ye döndü.
- [ ] Ağ yeniden geldiğinde seçenekler tekrar etkinleşti.

### Seçimlerin kurulu sisteme aktarılması

- [ ] Seçilen değer kurulum sırasında `/mnt/sysroot/etc/blazeos-desktop-choice` içinde görünüyor.
- [ ] Önyükleyici seçimi `/mnt/sysroot/etc/blazeos-bootloader-choice` içinde görünüyor.
- [ ] Kurulum tamamlanırken seçim dosyaları kaybolmuyor veya eski değere dönmüyor.

## 4. Depolama ve hata yolları

- [ ] Otomatik bölümleme boş diskte çalışıyor.
- [ ] Mevcut EFI Sistem Bölümü korunarak yanına kurulum çalışıyor.
- [ ] Tam disk şifreleme testi yapıldı; yanlış ve doğru parola davranışları doğru.
- [ ] Yetersiz disk alanı anlaşılır hata veriyor, diske yarım kurulum bırakmıyor.
- [ ] Kurulum ortasında ağ kesilmesi SolarUI kurulumunu bozmaz.
- [ ] Kurulum iptali ana ekrana güvenli dönüyor; ikinci deneme çalışıyor.
- [ ] Disk çıkarma / salt okunur disk / I/O hatası kullanıcıya açık hata gösteriyor.
- [ ] `gen_grub_cfgstub` hatası artık `Kuruluma devam et` uyarısı üretmiyor.
- [ ] Kurulum bitişi, yeniden başlatma ve ISO'nun çıkarılması doğru çalışıyor.

## 5. Önyükleyici matrisi

### GRUB

- [ ] UEFI ve BIOS kurulumunda GRUB seçilebilir ve sistem açılır.
- [ ] UEFI kurulumundan sonra ISO çıkarıldığında sistem doğrudan GRUB menüsüne gider; çıplak `grub>` istemi görünmez.
- [ ] `/boot/efi/EFI/fedora/grub.cfg` içinde `/root/var/lib/mock`, `image-root` veya başka derleme makinesi yolu yoktur.
- [ ] Fedora çekirdeği güncellendiğinde GRUB girdileri yenilenir.
- [ ] BERP/kurtarma girdisi açılır.

### Limine

- [ ] Limine yalnızca UEFI modunda seçilebilir; BIOS modunda pasiftir.
- [ ] Kurulum ağ olmadan tamamlanır; COPR veya başka üçüncü taraf depo eklenmez.
- [ ] Limine menüsü normal sistem, BERP, tek kullanıcı kurtarma ve GRUB zincir yükleme girdilerini gösterir.
- [ ] `root=` satırı `UUID=...`, `LABEL=...` veya fstab'daki özgün biçimdedir; `UUID=LABEL=...` oluşmaz.
- [ ] Çekirdek güncellemesinden sonra Limine çekirdek/initramfs kopyaları güncellenir.
- [ ] GRUB'a geri dönme çalışır ve EFI yedeği korunur.

## 6. İlk açılış ve masaüstü seçimi

- [ ] İlk açılış yardımcısı kullanıcı, parola, klavye, saat dilimi ve dil işlemlerini tamamlar.
- [ ] Parola ve parola doğrulama alanları ilk açılışta boştur; kaynakta veya arayüzde hazır parola bulunmaz.
- [ ] Otomatik giriş varsayılan olarak kapalıdır; kapalı bırakıldığında her yeniden başlatmada SDDM parola ister.
- [ ] Yardımcı yarıda kapatılıp yeniden başlatıldığında veri bozulmadan devam eder.
- [ ] Tahmin edilebilir `blaze/blaze` hesabı veya parolasız sudo hesabı yoktur.
- [ ] SolarUI ilk oturumda açılır; GNOME kabuğu kendiliğinden başlamaz.
- [ ] Oturum seçicide stok olarak SolarUI görünür; saf Niri ve GNOME gizlidir.
- [ ] OOBE tamamlanınca `BlazeOS Kurulum Asistanı` oturumu, OOBE autologin ayarı ve geçici AccountsService hesabı silinir; yeniden başlatmada geri gelmez.
- [ ] OOBE sonrası ikinci açılışta siyah ekran oluşmaz ve SDDM'nin oturum listesinde yalnız SolarUI görünür.
- [ ] Çevrimiçi seçilen masaüstü resmi Fedora deposundan kurulur ve bir sonraki girişte varsayılan olur.
- [ ] İndirme başarısızsa SolarUI çalışır; `/var/log/blazeos-postinstall.log` açık hata içerir ve sonraki açılışta yeniden dener.
- [ ] Her çevrimiçi seçenek ayrı kurulumda sınandı: GNOME / Niri / KDE / COSMIC / Hyprland / Cinnamon / XFCE / Sway.

Tanılama:

```bash
cat /etc/blazeos-desktop-choice
cat /etc/blazeos-bootloader-choice
systemctl status blazeos-postinstall.service --no-pager
sudo journalctl -b -u blazeos-postinstall.service --no-pager
sudo tail -n 300 /var/log/blazeos-postinstall.log
dnf5 repolist --enabled
```

## 7. Uygulamalar ve işlevsiz düğme avı

- [ ] Uygulama menüsündeki `Blaze Sistem İzleyicisi` Kitty içinde htop açar.
- [ ] `GNOME Sistem Monitörü` menüde ve kurulu paketlerde yoktur: `rpm -q gnome-system-monitor` başarısız olur.
- [ ] htop CPU, RAM, süreç sonlandırma, sıralama ve arama işlevleri çalışır.
- [ ] Nexus, hızlı ayarlar, ağ, Bluetooth, ses, ekran, güç, güncelleme, duvar kâğıdı ve oturum düğmelerinin her biri gerçek işlem yapar.
- [ ] Her düğme fare, klavye ve dokunmatik ile sınandı; tıklayıp hiçbir şey yapmayan öğeler listelendi.
- [ ] Dosya seçicilerde iptal, bozuk dosya, izinsiz dizin ve boş seçim denendi.
- [ ] Türkçe/İngilizce/sistem dili canlı değişiyor ve yeniden girişten sonra korunuyor.

## 8. Performans ve kararlılık

Ölçümleri üç soğuk açılışın ortalaması olarak kaydedin.

- Firmware süresi:
- Kernel süresi:
- Userspace süresi:
- Giriş ekranına toplam süre:
- Paroladan kullanılabilir SolarUI'ye süre:
- Boştaki RAM (5 dakika sonra):
- Boştaki CPU ortalaması:
- 99. yüzdelik kare süresi / gözlenen takılma:

```bash
systemd-analyze
systemd-analyze blame | head -40
systemd-analyze critical-chain
systemctl --failed
free -h
ps -eo pid,comm,rss,%cpu --sort=-rss | head -30
journalctl -b -p warning..alert --no-pager
coredumpctl list --since boot
```

- [ ] 30 pencere aç/kapat, çalışma alanı geçişi ve hızlı kaydırma sırasında fare takılmıyor.
- [ ] Caelestia ve Noctalia art arda 20 yeniden başlatmada zaman aşımına düşmüyor.
- [ ] 8 saat boşta bekleme ve 2 saat yoğun kullanım sonrası RAM sürekli yükselmiyor.
- [ ] 20 askıya al/uyandır döngüsünde panel, Wi-Fi, ses ve imleç geri geliyor.
- [ ] Harici monitör tak/çıkar, HiDPI, döndürme ve yenileme hızı değişimleri çalışıyor.
- [ ] Intel / AMD / NVIDIA test sonucu ve kullanılan sürücü kaydedildi.

## 9. BERP, hardreset ve ölüm ekranı

- [ ] BERP normal sistemden ve önyükleme menüsünden açılıyor.
- [ ] Kurtarma ISO'su seçme, doğrulama ve başlatma gerçek işlem yapıyor.
- [ ] Geçersiz ISO, eksik ISO, bozuk checksum ve salt okunur USB açık hata veriyor.
- [ ] ISO flash hedefi sistem diskini varsayılan olarak seçmiyor; hedef ve veri kaybı uyarısı doğru.
- [ ] Flash sırasında güç/ağ kesilmesi sonrası BERP tekrar açılabiliyor.
- [ ] Hardreset yalnızca simülasyon kapsamındaki dosyaları yok eder; BERP ikilisi, hedefi, önyükleme girdisi ve gerekli EFI dosyaları korunur.
- [ ] Hardreset sonrası özel ölüm ekranı açılıyor ve kurtarma menüsüne geçiş çalışıyor.
- [ ] Kernel panic senaryosunda otomatik yeniden başlatma döngüsü oluşmuyor; ölüm ekranı/kurtarma yolu erişilebilir.
- [ ] Kurtarma günlükleri dış diske aktarılabiliyor; düğmeler sahte başarı mesajı vermiyor.

## 10. Hata kaydı şablonu

- Kimlik:
- Önem: Engelleyici / Yüksek / Orta / Düşük
- Tekrarlanma: Her zaman / Aralıklı / Bir kez
- Ön koşul:
- Adımlar:
- Beklenen:
- Gerçekleşen:
- Ekran görüntüsü / video:
- `journalctl` / coredump:
- Donanım ve oturum:
- Geçici çözüm:
