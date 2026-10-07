# Blaze SolarEvolution 5.03 — 7 Ekim 2026 Doğrulama Raporu

## Üretilen ISO

- Dosya: `Blaze-SolarEvolution-5-x86_64.iso`
- Boyut: `3,883,008,000` bayt
- SHA-256: `aaa9cd9c9d814ac7032d2e666de910bd800c3b17080f486f43457344b133851a`
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

## Bu çalışmada yakalanan son hata

`livesys-solarui` içinde Ethernet bağlantısını hazırlayan arka plan alt kabuğunun kapanışı eksikti. Arayüz çalışsa bile masaüstü ve önyükleyici seçimlerini `/mnt/sysroot` içine taşıyan döngü güvenilir değildi. Eksik `) &` eklendi ve derleme öncesi zorunlu sözdizimi kontrolü ile aynı hata sınıfının tekrar ISO'ya girmesi engellendi.

Kurulumdan sonra çıplak `grub>` istemine düşme de yeniden üretildi. Kurulu ESP'deki `/EFI/fedora/grub.cfg`, canlı imajın `/root/var/lib/mock/.../image-root/boot/grub2` yolunu taşıyordu. Gerçek `/boot/grub2/grub.cfg` sağlamdı ve elle `configfile (hd0,gpt2)/grub2/grub.cfg` çalıştırıldığında sistem açıldı. ISO'ya ayrı `/boot` bölümündeki gerçek yapılandırmayı bulan taşınabilir bir EFI güvenlik stub'ı eklendi; `gen_grub_cfgstub` için UUID bulunamadığında çalışan geri dönüş oluşturuldu ve postinstall'ın EFI stub'ını tam `grub2-mkconfig` çıktısıyla ezmesi engellendi.

## Fiziksel sistemde bekleyen kontroller

- Gerçek diske tam Anaconda kurulumu ve yeniden başlatma.
- Her çevrimiçi masaüstünün paket indirme ve oturum açma testi.
- Legacy BIOS, Secure Boot, gerçek NVIDIA/AMD/Intel GPU ve Wi-Fi donanım matrisi.
- Limine ile gerçek EFI NVRAM girdisi, çekirdek güncellemesi ve GRUB'a geri dönüş.
- BERP, hardreset, ISO doğrulama/flash ve özel ölüm ekranının yıkıcı hata enjeksiyonu testleri.
- Uzun süreli RAM, askıya al/uyandır ve kare süresi ölçümü.

Bu kontroller için `TEST-CHECKLIST-5.03.md` dosyası kullanılmalıdır.
