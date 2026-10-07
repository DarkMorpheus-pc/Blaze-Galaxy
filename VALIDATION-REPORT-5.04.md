# Blaze SolarEvolution 5.04 — 7 Ekim 2026 Doğrulama Raporu

## Üretilen ISO

- Dosya: `Blaze-SolarEvolution-5-x86_64.iso`
- Boyut: `3,883,008,000` bayt
- SHA-256: `afab6337b8c1826c71003b82d303f25573543a94610966b9e946e2deca769df2`
- `checkisomd5`: `PASS`

## Otomatik kontroller

- Canlı oturum, ilk açılış, önyükleyici ve sistem izleyicisi betikleri `bash -n` kontrolünden geçti.
- Anaconda WebUI JavaScript'i `node --check` kontrolünden geçti.
- `git diff --check` boş sonuç verdi.
- Squashfs xattr/SELinux etiketleri korunarak oluşturuldu.
- Squashfs içinden çıkarılan sıkıştırılmış `index.js.gz` güncel kaynakla eşleşti ve JavaScript kontrolünden geçti.
- Squashfs içinden çıkarılan `livesys-solarui` betiği tekrar `bash -n` ile doğrulandı.
- Squashfs içinde `htop`, `blaze-system-monitor`, Limine ikilisi ve Limine UEFI dosyası doğrulandı.
- GNOME Sistem Monitörü uygulama girdisi squashfs içinde bulunmadı.

## Sanal makine kontrolleri

Ortam: QEMU/KVM, 4 vCPU, 6 GiB RAM, 40 GiB disk, UEFI, Secure Boot kapalı.

| Senaryo | Sonuç |
|---|---|
| Son ISO'dan canlı SolarUI açılışı | GEÇTİ |
| Anaconda WebUI'nin yüklenmesi | GEÇTİ |
| Ağ aygıtı yokken yalnızca SolarUI'nin etkin olması | GEÇTİ |
| GNOME, Niri, KDE, COSMIC, Hyprland, Cinnamon, XFCE ve Sway'in çevrimdışı kilitlenmesi | GEÇTİ |
| Kullanıcı ağı eklenince çevrimiçi seçeneklerin etkinleşmesi | GEÇTİ |
| Çevrimiçi GNOME kartının seçilebilmesi | GEÇTİ |
| GNOME seçiliyken ağ düşürülünce seçimin SolarUI'ye dönmesi | GEÇTİ |
| Ağ düşünce çevrimiçi seçeneklerin yeniden kilitlenmesi | GEÇTİ |
| Boş 36 GiB diske tam Anaconda kurulumu | GEÇTİ |
| ISO çıkarıldıktan sonra yalnızca kurulu diskten UEFI açılışı | GEÇTİ |
| GRUB menüsünün `grub>` istemine düşmeden yüklenmesi | GEÇTİ |
| Kurulu sistemde SolarUI ilk açılış yardımcısının gelmesi | GEÇTİ |
| Kurulu ESP'de taşınabilir, mock yolu içermeyen GRUB stub'ı | GEÇTİ |
| OOBE parola alanlarının boş ve otomatik giriş seçiminin kapalı gelmesi | GEÇTİ |
| OOBE bitince kurulum asistanı oturumunun ve geçici hesabın silinmesi | GEÇTİ |
| SDDM oturum listesinde yalnız SolarUI görünmesi | GEÇTİ |
| Otomatik giriş kapalıyken ikinci açılışın parola ekranında kalması | GEÇTİ |
| Zorunlu Weston greeter ayarı kaldırılınca ikinci açılışta siyah ekran oluşmaması | GEÇTİ |

## Bu çalışmada yakalanan son hata

`livesys-solarui` içinde Ethernet bağlantısını hazırlayan arka plan alt kabuğunun kapanışı eksikti. Arayüz çalışsa bile masaüstü ve önyükleyici seçimlerini `/mnt/sysroot` içine taşıyan döngü güvenilir değildi. Eksik `) &` eklendi ve derleme öncesi zorunlu sözdizimi kontrolü ile aynı hata sınıfının tekrar ISO'ya girmesi engellendi.

Kurulumdan sonra çıplak `grub>` istemine düşme de yeniden üretildi. Kurulu ESP'deki `/EFI/fedora/grub.cfg`, canlı imajın `/root/var/lib/mock/.../image-root/boot/grub2` yolunu taşıyordu. Gerçek `/boot/grub2/grub.cfg` sağlamdı ve elle `configfile (hd0,gpt2)/grub2/grub.cfg` çalıştırıldığında sistem açıldı. ISO'ya ayrı `/boot` bölümündeki gerçek yapılandırmayı bulan taşınabilir bir EFI güvenlik stub'ı eklendi; `gen_grub_cfgstub` için UUID bulunamadığında çalışan geri dönüş oluşturuldu ve postinstall'ın EFI stub'ını tam `grub2-mkconfig` çıktısıyla ezmesi engellendi.

OOBE tamamlandıktan sonra kurulum yardımcısının SDDM oturum listesinde kalması da düzeltildi. Tamamlama işlemi oturum dosyasını, geçici otomatik giriş yapılandırmasını ve `blaze-setup` AccountsService kaydını siliyor; açılış öncesi çalışan hijyen servisi yarım kalmış temizliği tekrar deniyor. Hazır `2121` parolası kaldırıldı ve postinstall artık kullanıcının kapalı bıraktığı otomatik giriş seçimini zorla açmıyor. İkinci açılışta yakalanan siyah ekranın nedeni olan zorunlu `SDDM + Weston kiosk` greeter ayarı kurulu sistemden kaldırıldı; paketlenmiş SDDM görüntü yolu korunuyor.

## Fiziksel sistemde bekleyen kontroller

- Gerçek diske tam Anaconda kurulumu ve yeniden başlatma.
- Her çevrimiçi masaüstünün paket indirme ve oturum açma testi.
- Legacy BIOS, Secure Boot, gerçek NVIDIA/AMD/Intel GPU ve Wi-Fi donanım matrisi.
- Limine ile gerçek EFI NVRAM girdisi, çekirdek güncellemesi ve GRUB'a geri dönüş.
- BERP, hardreset, ISO doğrulama/flash ve özel ölüm ekranının yıkıcı hata enjeksiyonu testleri.
- Uzun süreli RAM, askıya al/uyandır ve kare süresi ölçümü.

Bu kontroller için `TEST-CHECKLIST-5.04.md` dosyası kullanılmalıdır.
