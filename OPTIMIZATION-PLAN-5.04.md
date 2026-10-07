# Blaze SolarEvolution 5.04 Performans ve Açılış Optimizasyon Planı

## 1. Hedefler ve başarı ölçütleri

Optimizasyonlar hissiyata göre değil, aynı donanım ve aynı VM profili üzerinde en az beş soğuk açılışın medyanı ve en kötü değeri ile doğrulanacaktır.

| Ölçüm | Gerçek donanım hedefi | KVM/virtio-gpu hedefi |
|---|---:|---:|
| Kernel başlangıcından SDDM hazır olana kadar | <= 4 sn | <= 7 sn |
| Oturum açmadan ilk kullanılabilir panel karesine kadar | <= 0,8 sn | <= 1,5 sn |
| Caelestia/Noctalia tamamen hazır | <= 1,5 sn | <= 2,5 sn |
| 60 Hz normal kullanım kare süresi p95 | <= 16,7 ms | <= 16,7 ms |
| Normal kullanımda 50 ms üstü takılma | 0 | en fazla 1/10 dk |
| Kabuk boşta toplam CPU | <= %1 | <= %2 |
| Watchdog boşta process spawn | 0 | 0 |
| Kabuk çökmesinden otomatik dönüş | <= 2 sn | <= 3 sn |

Her değişiklikten önce ve sonra şu veriler saklanacaktır:

- `systemd-analyze time`, `critical-chain`, `blame` ve SVG boot plot.
- `systemd-analyze --user critical-chain` ve kullanıcı journal zaman çizelgesi.
- Niri başlangıcı, ilk Wayland surface, ilk panel karesi ve shell-ready zaman damgaları.
- `systemd-cgtop`, `pidstat`, PSI, RSS/PSS, page-fault ve I/O değerleri.
- Sysprof/perf ile 30 saniyelik giriş, pencere taşıma, panel açma ve duvar kâğıdı değiştirme kaydı.
- `glxinfo -B`, `vulkaninfo --summary`, DRM sürücüsü ve gerçek renderer adı. `llvmpipe` sonuçları donanım hızlandırmalı sonuçlardan ayrı tutulacaktır.

## 2. P0: Takılmaya doğrudan neden olan render ayarları

### 2.1 Donanım imlecini geri aç

`usr/bin/solar-session-wrapper` şu anda bütün cihazlarda `WLR_NO_HARDWARE_CURSORS=1` ayarlıyor. Bu, imleci her karede GPU/CPU kompozisyonuna sokarak özellikle VM ve yüksek yenileme hızlı ekranlarda mouse takılmasına yol açabilir.

- Değişken varsayılan olarak kaldırılacak.
- Sadece doğrulanmış bozuk GPU/sürücü kimlikleri için `/usr/lib/solarui/gpu-quirks.d/` kuralı uygulanacak.
- Yazılım imleci kurtarma modu yalnızca Niri ilk başlatması başarısız olursa kullanılacak.

### 2.2 DMA-BUF yolunu varsayılan yap

`WEBKIT_DISABLE_DMABUF_RENDERER=1` bütün oturuma veriliyor. Bu ayar WebKit tabanlı uygulamalarda kopyalama maliyetini ve CPU kullanımını artırabilir.

- Genel değişken kaldırılacak.
- Sorunlu NVIDIA/VM sürücüsü için dar kapsamlı uygulama override'ı kullanılacak.
- Firefox, WebKit ve GTK uygulamalarında zero-copy yolu renderer bazında test edilecek.

### 2.3 VM renderer seçimini düzelt

Mevcut kod her sanal makinede `QSG_RENDER_LOOP=basic` kullanıyor. Virtio-GPU/VirGL/Venus çalışan bir VM de bu yüzden yavaş yola düşüyor.

Üç profil üretilecek:

1. **Native GPU:** threaded render loop, hardware cursor, DMA-BUF açık.
2. **Accelerated VM:** virtio-gpu + VirGL/Venus doğrulanırsa native profile yakın ayarlar.
3. **Software fallback:** yalnızca renderer `llvmpipe`, `softpipe` veya Niri/EGL başlatması başarısızsa `basic`, pixman ve software cursor.

Profil kararı `systemd-detect-virt` ile tek başına verilmeyecek; DRM driver, render node ve renderer adı birlikte kullanılacak. Seçim journal'a tek satır olarak yazılacak.

Libvirt test profili virtio video, 3D acceleration, host render node ve yeterli video RAM kullanacak. QXL veya saf llvmpipe üzerinde 60 FPS garantisi verilmeyecek.

## 3. P0: Boot kritik zincirini kısalt

### 3.1 Postinstall işini gerçek tek seferlik servise dönüştür

`blazeos-postinstall.service`, `Before=display-manager.service` ile her kurulu sistem açılışında display manager'ı bekletiyor. Betik kullanıcıları tarıyor, dosya ve display manager yapılandırmalarını tekrar yazıyor ve seçilen masaüstüne göre paket kurulumu bile yapabiliyor.

- OOBE hazırlığı ve kurulum finalizasyonu iki ayrı servise bölünecek.
- Finalizasyon başarılı olduğunda atomik marker yazacak ve kendini devre dışı bırakacak.
- Ağ/paket kurulumu boot yolundan tamamen çıkarılacak.
- Günlük açılışta bu betik hiç çalışmayacak.

### 3.2 `blaze-optimization` engelini kaldır

Servis display manager'dan önce ZRAM servisini yeniden başlatıyor. Sysctl ve ZRAM ayarları zaten deklaratif systemd mekanizmalarıyla uygulanabilir.

- Sysctl değerleri yalnızca `/etc/sysctl.d` üzerinden yüklenecek.
- ZRAM yalnızca `zram-generator` ile kurulacak; her bootta restart yapılmayacak.
- Donanım profili değişiklikleri `tuned-ppd` API'si üzerinden olay bazlı uygulanacak.
- `blaze-optimization.service` boot kritik zincirinden kaldırılacak.

### 3.3 Güç yöneticisi çakışmasını gider

Rootfs içinde hem `tuned.service` hem `tuned-ppd.service` etkin. Tek sahip modeli kullanılacak.

- Masaüstü arayüzü için `tuned-ppd` korunacak.
- Aynı ayarları doğrudan sysfs'e yazan Blaze kodu kaldırılacak veya yalnızca kurtarma aracı olacak.
- Profil değişimi tek DBus işlemi olacak; CPU başına dosya yazan döngüler UI thread'inden çağrılmayacak.

### 3.4 Ağ ve sürücü kurulumunu boot yolundan çıkar

- `NetworkManager-wait-online.service`, boot için gerçekten ağ isteyen servis yoksa devre dışı bırakılacak.
- `blazeos-nvidia-firstboot.service` display açılmadan önce 600 saniyeye kadar çevrimiçi paket kurulumu yapmayacak.
- Gerekli NVIDIA paketleri ISO'da bulunacak veya masaüstü açıldıktan sonra kullanıcıya ilerleme ve geri alma sunan görev olarak çalışacak.
- Güncelleme, COPR, Flatpak repo ekleme ve ek masaüstü kurulumu boot sırasında yapılmayacak.

### 3.5 Donanıma göre servis etkinleştir

`vboxservice`, `vmtoolsd` ve `qemu-guest-agent` aynı imajda bulunabilir fakat yalnızca eşleşen hypervisor üzerinde başlamalıdır. `ModemManager`, `smartd`, `mdmonitor`, NFS, Avahi, CUPS ve `systemd-homed` için de kullanım/cihaz koşulları belirlenecek.

- VM ajanları `ConditionVirtualization=` ve udev koşullarıyla ayrılacak.
- Yazdırma socket/path activation olarak kalacak.
- ModemManager yalnızca WWAN donanımı varsa çalışacak.
- NFS ve RAID servisleri kullanılan özellik yoksa boot hedefinden çıkarılacak.
- Güvenlik için gerekli `firewalld`, `systemd-oomd` ve çözümleyici ölçüm yapılmadan kapatılmayacak.

## 4. P0: SolarUI oturum başlangıcını yeniden düzenle

### 4.1 Statik systemd user servisleri kullan

`solar-shell autostart`, her girişte portal dosyası ve `solarui-session.service` oluşturuyor, ardından `daemon-reload`, environment import ve DBus environment update komutları çalıştırıyor.

- Portal ve servis dosyaları ISO içinde hazır sevk edilecek.
- `niri-session.target`, `solarui-shell.target`, `noctalia.service`, `caelestia.service`, `solar-clipboard.service` ve `solar-welcome.service` oluşturulacak.
- Kabuklar foreground çalışacak ve systemd tarafından `Restart=on-failure` ile yönetilecek.
- Wayland ortamı bir kez Niri session başlangıcında içe aktarılacak.
- Her girişte `daemon-reload` ve dosya üretimi yapılmayacak.

### 4.2 Fork tabanlı watchdog'u kaldır

Mevcut watchdog üç saniyede bir health check yapıyor; kontroller `pgrep`, CLI ve başka alt süreçler doğuruyor. Switcher bazı geçişlerde Noctalia ve iki Caelestia provider'ını tekrar tekrar sorguluyor.

- Süreç yaşam döngüsü systemd user service ve pidfd ile izlenecek.
- Sağlık durumu DBus/Unix socket heartbeat üzerinden olay bazlı tutulacak.
- Watchdog yalnızca state değiştiğinde çalışacak; boşta sıfır subprocess üretecek.
- Restart storm için `StartLimitIntervalSec`, `StartLimitBurst` ve exponential backoff uygulanacak.
- Üç ardışık çökmeden sonra güvenli shell profiline geçilecek ve kullanıcıya journal bağlantısı verilecek.

### 4.3 İlk kullanılabilir kareyi öne al

Başlangıç iki aşamaya ayrılacak:

1. Niri + basit panel/skeleton + input, clipboard ve bildirimler.
2. Hava durumu, takvim, plugin kataloğu, medya taraması, duvar kâğıdı indeksleme ve gelişmiş paneller.

İkinci aşama ilk frame sunulduktan sonra idle callback ile başlayacak. Welcome HUD mevcut iki saniyelik sabit uyku yerine shell-ready sinyalini bekleyecek ve kurulu sistemde yalnızca ilk girişte açılacak. Live installer ve Welcome aynı anda GPU/CPU için yarışmayacak.

## 5. P1: Caelestia optimizasyonu

- QML dosyaları build sırasında `qmlcachegen/qmlsc` ile derlenecek; ilk açılışta yorumlama ve disk taraması azaltılacak.
- Nexus, dashboard, hava durumu ve ağır popout'lar `Loader` ile ilk açılışa kadar oluşturulmayacak.
- Gizli yüzeylerde animasyon, shader ve timer çalışmayacak.
- Weather'ın bir saniyelik timer'ı yalnızca aktif isteğin zaman aşımı sırasında çalışacak; normal durumda saatlik timer yeterli olacak.
- `SysInfo` polling görünürlük ve güç profiline göre 15/30/60 saniye olarak uyarlanacak.
- Aynı DBus bilgisini isteyen QML servisleri tek cache/service üzerinden beslenecek.
- Blur, gölge ve wallpaper shader'ları çözünürlük ve renderer kabiliyetine göre kalite kademesi kullanacak.

## 6. P1: Noctalia optimizasyonu

Noctalia yerel C++ ve sınırlı allocator arena kullanımıyla iyi bir temele sahip. Kazanç başlangıç sıralamasından alınacak.

- Bar ve temel input yüzeyleri ilk olarak oluşturulacak.
- Weather, calendar, plugin catalog, remote asset ve wallpaper indeksleme ilk frame sonrasına taşınacak.
- Varsayılan duvar kâğıdı build sırasında uygun çözünürlüklerde önceden decode/cache edilecek.
- VM profilinde `fade + zoom, 1000 ms` başlangıç geçişi yerine kısa fade veya geçişsiz başlangıç kullanılacak.
- Sysmon ve desktop widget güncellemeleri görünmezken duracak; frame tick'e bağlı widget'lara ayrı düşük frekans bütçesi verilecek.
- Ağ istekleri kullanıcı özelliği etkinleştirilmeden başlamayacak; konum/hava durumu ilk kullanımda açık izin isteyecek.

## 7. P1: Niri ve kare zamanlaması

- Mevcut spring ayarlarındaki çok küçük `epsilon=0.0001` kare kuyruğu ve animasyon kuyruğu açısından ölçülecek; gerekirse `0.001-0.01` aralığı kullanılacak.
- 60/90/120/144 Hz ekranlarda ayrı kare zamanı testi yapılacak.
- VRR/tearing politikası global ortam değişkeni yerine output ve uygulama bazlı olacak.
- Çoklu monitör, fractional scale, NVIDIA explicit sync ve Intel/AMD direct scanout senaryoları ayrı test edilecek.
- Pointer latency testi hardware cursor açık/kapalı A/B karşılaştırmasıyla yapılacak.

## 8. P1: Bellek, OOM ve çökme dayanıklılığı

- Noctalia, Quickshell ve SolarUI ayrı systemd user cgroup'larında tutulacak.
- `MemoryHigh` gözleme dayalı yumuşak eşik olacak; yanlış bir sabit `MemoryMax` ile shell öldürülmeyecek.
- `ManagedOOMMemoryPressure=kill` yalnızca yeniden başlatılabilir kullanıcı uygulamalarına uygulanacak; compositor son kurban olacak.
- ZRAM boyutu RAM'e göre kademeli seçilecek ve `zstd/lz4` karşılaştırılacak.
- Uzun süreli test: 8 saat pencere aç/kapat, shell switch, wallpaper, bildirim ve uyku/uyanma döngüsü. PSS, fd, Wayland buffer ve GPU memory eğrileri kaydedilecek.
- Crash sonrası core dump, pstore ve son 200 journal satırı BERP tanı paketine eklenecek.

## 9. Uygulama sırası

### Sprint A — Ölçüm ve düşük riskli düzeltmeler

1. Benchmark toplayıcı ve boot/session timestamp altyapısı.
2. Hardware cursor ve DMA-BUF global engellerini kaldırma.
3. Native/accelerated-VM/software renderer profil seçici.
4. Postinstall ve optimization servislerini kritik zincirden çıkarma.
5. `tuned`/`tuned-ppd` tek sahip düzeltmesi.

Beklenen sonuç: mouse takılmasının büyük ölçüde kaybolması ve display manager'a ulaşma süresinde belirgin düşüş.

### Sprint B — systemd user oturum mimarisi

1. Statik user unit ve target dosyaları.
2. Noctalia/Caelestia foreground servisleri.
3. Fork tabanlı polling watchdog'un kaldırılması.
4. Shell-ready protokolü ve doğru welcome/installer sıralaması.

Beklenen sonuç: panelin tutarlı ve hızlı açılması, yarış koşullarının ve bazen geç açılma sorununun kaldırılması.

### Sprint C — UI ve render bütçesi

1. Caelestia QML cache ve lazy loading.
2. Noctalia staged initialization.
3. Görünmez timer/widget durdurma.
4. Niri animasyon ve VM kalite profili A/B testleri.

Beklenen sonuç: düşük donanım ve VM'de daha dengeli kare süresi.

### Sprint D — dayanıklılık ve donanım matrisi

Intel, AMD, NVIDIA proprietary, KVM/Virtio, VirtualBox ve VMware üzerinde cold boot, suspend/resume, hotplug, multi-monitor ve 8 saat soak testi yapılacak. Her sınıf için başarısız renderer otomatik fallback testi zorunlu olacak.

## 10. Değişiklik kabul kuralı

Bir ayar yalnızca ortalama hızlandığı için kabul edilmeyecek. Boot medyanını veya kare süresini iyileştirirken en kötü değer, pil tüketimi, görüntü doğruluğu ya da çökme oranını bozuyorsa geri alınacak. Kernel/sysctl “gaming tweak” listeleri ölçümsüz eklenmeyecek. Her commit tek hipotezi değiştirecek ve önce/sonra ölçümünü taşıyacak.
