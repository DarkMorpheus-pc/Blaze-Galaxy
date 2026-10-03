# Blaze SolarEvolution 5 — teknik denetim ve ürün yol haritası

28 Eylül 2026. Bu çalışma bir sürüm sertifikasyonu değildir. Doğrulanmış kaynak bulguları, ISO gözlemleri ve henüz uygulanmamış tasarım önerileri aşağıda ayrıdır. Üçüncü taraf 1,8 GB vendor ağacının, Niri'nin ve tüm Fedora paketlerinin her satırının elle incelendiği iddia edilmiyor. Blaze House bu ağaçta ELF ikilisi; kaynak kodu görülmeden onun iç mantığı için satır denetimi yapılamaz. FireHub başlatıcısı incelendi; Electron uygulamasının tamamı denetlenmedi.

## Sonuç

Öncelik daha fazla efekt eklemekten önce güvenilir kurulum/oturum, tekrar üretilebilir paketleme ve hatalı güncellemeden kurtarmadır. Rust kullanımı tek başına bellek sızıntısını, görev birikmesini, sürücü hatalarını veya grafik oturumu çökmesini engellemez. Mevcut SolarUI kaynaklarında sınırlı IPC kuyrukları, mesaj boyutu sınırları, atomik ayar kayıtları, zaman aşımları ve artan aralıkla yeniden başlatma gibi iyi temeller zaten var.

## Test edilen ISO ve görülen durum

- En yeni dosya: `Blaze-SolarEvolution-5-x86_64.iso`; dosya değiştirilme zamanı 28 Eylül 2026 19:38, 3.800.956.928 bayt. Dosya zamanı ISO içindeki bir derleme kimliğinin yerine geçmez.
- QEMU: q35, TCG, 2 vCPU, 4 GiB RAM, virtio VGA, ağ yok, fiziksel disk yok. `/dev/kvm` yok. Bu koşullardan gerçek CPU/GPU performans sonucu çıkarılamaz.
- Açılış logosundan sonra **kullanıcı listesi boş GDM ekranına** ulaşıldı. SolarUI masaüstünün çalıştığı doğrulanamadı. Bu bir GNOME masaüstü oturumu gözlemi değildir; GDM giriş arayüzüdür.
- ISO SHA256: `7889f5878f20984974959648d0bc94b18eb2d4bc27b12916fadb620cf6494351`.
- Kanıt: `audit/2026-09-28/vm-status.png`. İlk açılış logosu `vm.png`.
- ISO'dan SquashFS çıkarılarak `etc/sysconfig/livesys`, SolarUI desktop girişi ve postinstall servis birimi okundu. SolarUI seçimi var; postinstall için canlı ortam koşulu yok.
- Karşılaştırma: aynı ISO içinden çıkarılan kernel/initrd ile `systemd.mask=blazeos-postinstall.service` eklenerek ikinci TCG açılışı yapıldı. Bu denemede de boş GDM ekranı görüldü (`vm-masked.png`); dolayısıyla postinstall hatasının giderilmesi **tek başına oturum sorununun çözüldüğünü kanıtlamıyor**. İkinci denemede seri konsol parametreleri de eklendi; tam kontrollü donanım karşılaştırması değildir.
- Maskeli denemede TTY üzerinden `liveuser` ile giriş başarılı oldu (`vm-tty.png`). Hesap mevcutken grafik oturum hâlâ açılmıyor; GDM/SolarUI günlüğü bir sonraki teşhis adımı. Günlük alınamadan QEMU, host Codex sürecinin SIGTERM sinyaliyle sonlandı; bu guest işletim sistemi çökmesi olarak yorumlanmamalı.
- Mevcut ISO değiştirilmedi. Kaynak düzeltmeleri yeni ISO olarak derlenip açılış/kurulum testinden geçirilmedi.

## Kritik bulgular ve yapılan düzeltmeler

| Öncelik | Kanıt / dosya | Etki | Bu çalışmadaki durum |
|---|---|---|---|
| P0 | `usr/local/bin/blazeos-postinstall`, rootfs içindeki aynı adlı servis | Canlı ISO'da da çalışan servis `liveuser` hesabını siliyor, GDM autologin ayarını kaldırıyor. livesys ile sıralama donanım hızına göre değişebilir. | Canlı kernel parametresi ve mount dizini kontrolüyle erken çıkış eklendi. |
| P1 | `blazeos-postinstall` | Bütün normal kullanıcılar wheel/input/video/audio/render grubuna ekleniyor; kurulumdaki standart kullanıcı kararı bozuluyor. | Otomatik grup yükseltmesi kaldırıldı. |
| P1 | `blazeos-postinstall` | GNOME/Niri seçimi GDM'yi değiştirirken AccountsService SolarUI kalabiliyor; XFCE türü Wayland kalabiliyor. | Seçilen gerçek desktop dosyasına göre Session, XSession, SessionType eşitlendi. Oturum dosyası yoksa hata. |
| P1 | `blazeos-nvidia-setup` | Modül bulunamasa da 300 saniye sonrasında nouveau kapatılıyor ve tamamlandı işareti yazılıyor. | Modül yoksa hata; sonraki boot değişikliklerine geçilmiyor. |
| P1 | aynı NVIDIA betiği | Secure Boot'ta modül dosyasının varlığı anahtarın güvenilir olduğunu kanıtlamıyor. | Secure Boot açık ve NVIDIA yüklü değilse duruyor. Tam MOK kayıt sihirbazı henüz yok. |
| P1 | iki ISO build betiği | Takip edilen overlay hiç kopyalanmıyor; kaynak betiği düzelse de eski rootfs paketleniyor. | Overlay geçici rootfs'ye uygulanıyor. Rust/üçüncü taraf ikililerini kaynaklardan yeniden üretmek hâlâ ayrı iş. |
| P1 | iki build betiği | SELinux `setfiles` hatası yutuluyor. | Etiketleme hatası artık derlemeyi durduruyor. |
| P1 | offline build | Masaüstü paketleri kurulamazsa bile “offline tamamlandı” deniyor. | Paket kurulum hatası artık derlemeyi durduruyor. |
| P1 | `solar-lock` | Noctalia kilit isteği başarısız olsa da başarıyla çıkıyor; başka kullanıcının sürecini de eşleştirebiliyor. | Süreç mevcut kullanıcıyla sınırlı; 5 saniyede başarısız istek bağımsız kilitleyiciye geçiyor. Gerçek kilit/uyku testi hâlâ gerekli. |
| P2 | `blazeos-control` | Log TextBuffer ve GLib callback kuyruğu sınırsız büyüyebiliyor; paralel paket işlemleri başlatılabiliyor. | 256 parçalık sınırlı kuyruk, 4096 karakterlik okuma, 200.000 karakter görünür günlük ve tek işlem koruması. |
| P2 | aynı Control aracı | Yetkili shell komutunda USER ortam değişkeninden dosya yolu üretiliyor. | Gerçek UID'nin passwd kaydı kullanılıyor ve yol shell için tırnaklanıyor. |
| P1 | final build | `chvt` dosyasına elle setuid verilmesi gereksiz ayrıcalık oluşturuyor. | Bu setuid ataması kaldırıldı. |

Bu düzeltmeler mevcut ISO'nun otomatik olarak düzeldiği anlamına gelmez. Özellikle rootfs'nin sahiplik/veri kaynağı sorunu çözülmeden üretim ISO'su yayımlanmamalı.

## SolarUI yerine GNOME/GDM: teşhis sırası

1. Önce GDM giriş ekranı ile gerçek GNOME masaüstünü ayır. Postinstall/livesys çakışması kaynakta doğrulanmış bir hata; ancak servisin maskelendiği karşılaştırma da boş GDM ekranına geldi. Başka bir oturum/GPU/PAM/izin arızası araştırılmalı.
2. Canlı oturumdaysa `livesys.service` ve `blazeos-postinstall.service` günlüklerini karşılaştır. Kurulu sistemdeyse kullanıcının AccountsService Session/XSession/SessionType kaydını incele.
3. `solarui.desktop` Exec ve TryExec hedeflerini doğrula. Kaynak `solarui/data/solarui.desktop` doğrudan solar-session, overlay sürümü wrapper kullanıyor; paketleme bunları tek kaynağa indirmeli.
4. `~/.local/share/solarui/session.log`, `journalctl -b -u gdm`, kernel DRM ve SELinux AVC kayıtlarına bak. Oturum günlüğü sınırsız büyüyor; journald veya rotasyon gerekli.
5. GPU/driver/GBM/KMS, Secure Boot, hibrit GPU ve `nomodeset` durumlarını incele. “NVIDIA Mode” nouveau'yu daha boot sırasında blacklist ediyor; kullanılabilir NVIDIA modülü olmayan canlı ortamda grafik aygıtı kaybolabilir.
6. Wrapper'daki WLR değişkenlerinin Niri/Smithay için etkili olduğu varsayılmamalı. `WLR_RENDERER=pixman` güvenilir bir Niri kurtarma modu olarak sunulmamalı. Global `LIBGL_ALWAYS_SOFTWARE` CPU yükünü yükseltir.
7. Gerçek donanım için yerel, salt okunur `blazeos-session-diagnose` eklendi. Oturumdan çalıştırıp çıktıyı dosyaya kaydet; paylaşmadan önce kullanıcı adı/dizin/log içeriklerini gözden geçir. Araç hiçbir yere yükleme yapmaz.

NVIDIA için Niri upstream GBM uyumlu sürücü ve modeset gereksinimini anlatıyor: https://niri-wm.github.io/niri/Getting-Started.html . VM başarısı fiziksel GPU uyumluluğunu kanıtlamaz.

## Henüz düzeltilmemiş önemli teknik borçlar

### Derleme ve dağıtım

- `chown -R 0:0` tüm rootfs'de servis kullanıcı/grup sahipliklerini yok ediyor. Sonraki birkaç chmod bunları veya capabilities'i geri getirmez. Mevcut çıkarılmış ağaçta sahiplik zaten geliştirici UID'sine dönüşmüş görünüyor. Çözüm: güvenilir taban imajdan ayrıcalıklı, numeric owner/xattr koruyan çıkarma; RPM metadata doğrulaması; özel dosyaları RPM olarak paketleme. Yalnızca chown satırını silmek mevcut bozuk ağacı onarmaz.
- Elle verilen SUID/SGID listesi paket metadata'sının yerine geçemez. Grup root olunca unix_chkpwd/crontab gibi araçların davranışı değişebilir.
- Derleme önceden hazırlanmış ve Git'te bulunmayan rootfs/initrd/grub ağaçlarına bağlı. README'deki klonla-derle akışı tek başına tekrar üretilebilir değil.
- `Cargo.lock` ignore edilmiş. Uygulama workspace'i için lockfile sürüm kontrolüne alınmalı; `--locked` kullanılmalı. Paket kaynak sürümleri, hash'leri, derleyici ve taban ISO SHA256 kaydedilmeli.
- Kitty indirmesi hash/imza doğrulaması olmadan curl→tar. Özel Quickshell kütüphaneleri ve Python 3.15 ABI uzantıları için kaynağa bağlı derleme tarifi gerekli.
- tmpfs 28/40 GiB; 8/16 GiB makinelerde bellek baskısı/başarısız build riski. Disk tabanlı staging varsayılan, tmpfs açık seçenek olmalı; yeterli alan önkontrolü ve hata cleanup trap'i eklenmeli.
- Online/offline aynı squashfs çıktı yolunu kullanıyor. Eşzamanlı build tehlikeli. Ayrı build kimliği, staging dizini ve atomik çıktı yayını gerekli.
- MD5 iç medya kontrolüdür; dağıtım doğrulaması için SHA256 ve imza gerekir. Son çalışan ISO'yu baştan silmek yerine geçici dosyada oluşturup doğrulayınca yeniden adlandır.
- Beta taban ISO kullanılıyor. Test kanalı ile kararlı kanal ayrılmalı. Tarihe bakarak güvenilirlik iddiası yerine paket manifesti ve desteklenen donanım listesi yayımlanmalı.

### Oturum, IPC ve uygulamalar

- `solar-session/src/main.rs`: ölçek uygulamak için açılan Rust thread hemen ardından `exec(niri)` ile yok olur. Ayrıca eski süreç yeni compositor'un NIRI_SOCKET ortamını devralamaz. Ekran ayarını Niri hazır olduktan sonra başlayan ayrı kullanıcı servisinde veya doğrudan KDL output ayarında uygula.
- SolarCore elle spawn ediliyor; systemd birimleri ve Niri spawn-at-startup da var. Tek yaşam döngüsü sahibi seç. Core ve shell oturum hedefiyle birlikte başlamalı/bitmeli; kalan süreçler takip edilmeli.
- Portallar compositor hazır olmadan başlatılıp restart ediliyor. Ortam aktarımı Niri hazır olayı sonrasında bir kez yapılmalı; ekran paylaşımı gerçek uygulamalarla test edilmeli.
- `/usr/bin/true` çalıştıran oneshot birime Nice/OOMScoreAdjust vermek compositor'u korumaz. Negatif değerler user service yetkilerinde ayrıca başarısız olabilir.
- `hardware.rs` içindeki watchdog/idle/OOM seçenekleri ayarlarda yazılıyor; incelenen Rust kaynaklarında bunları davranışa bağlayan okuma bulunmadı. Etkisiz “koruma açık” anahtarları kaldırılmalı veya uygulanmalı.
- `sysmon.rs`: `/proc/loadavg * 25` CPU kullanım yüzdesi değildir. `/proc/stat` ardışık farklarından busy/total hesapla; ilk örnek “ölçülüyor” olsun.
- `privacy.rs`: mikrofon daima false, kamera `/proc/self/fd/0` üzerinden yer tutucu. Kamera aygıtının varlığı kullanım kanıtı değildir. PipeWire akışları + portal bilgisiyle uygulama atfı yap; doğrudan V4L2 erişiminin kapsam sınırını açıkla. Yanlış güven veren yeşil gösterge sunma.
- `LaunchApp` whitespace bölmesi tırnaklı yolları/argümanları bozar. `.desktop` başlatmayı GIO DesktopAppInfo üzerinden yap; `%U/%F`, Terminal, DBusActivatable ve yerelleştirmeyi doğru uygula.
- `apps.rs` yalnız NoDisplay kontrol ediyor; Hidden, OnlyShowIn/NotShowIn, TryExec ve kullanıcı override öncelikleri için test gerekir.
- Bazı GUI işlemleri senkron subprocess çağırıyor; sürücü/IPC takılması UI'yi dondurabilir. İptal edilebilir, süre sınırlı worker işlerine taşı.
- `HardwareTuningProfile::save` doğrudan fs::write; elektrik kesintisine karşı ortak atomik persistence API'sini kullan.
- `remove-gnome` yardımcı eylemi yeni display manager kurulumu başarısızken de GNOME kaldırmaya devam ediyor. İşlem zinciri başarı koşulları ve Wayland uyumlu kurtarma yolu olmadan kullanıma sunulmamalı.
- GNOME uzantı sürüm kontrolünü kapatmak uyumluluk sağlamaz. Uyumsuz eklentiyi kapatıp desteklenen sürümü paketle.
- FireHub wrapper VM'de GPU sandbox'ını, root'ta tüm sandbox'ı kapatıyor. Root GUI yerine yetkili dar backend; Chromium sandbox sorununu paket izinleri/user namespace düzeyinde çöz.

## Telemetri ve servisler

Noctalia C++ kaynağında gerçek telemetri var: `telemetry_service.cpp`, `https://api.noctalia.dev/ping`. Etkinleştirilirse kalıcı instance ID, sürüm, compositor/OS, RAM kapasitesi ve monitör boyut/ölçek bilgileri gönderiliyor. `config_types.h` varsayılanı **false**; görülen rootfs Noctalia settings.toml bunu true yapmıyor. Bu kaynak/config bulgusudur; tüm ISO için paket yakalamayla doğrulanmış “hiç trafik yok” sonucu değildir. Caelestia ve custom overlay taramasında aynı anahtarlarla başka açık gönderici bulunmaması yokluk kanıtı sayılmaz.

Fedora NetworkManager bağlantı kontrolü açık: `http://fedoraproject.org/static/hotspot.txt`. Captive portal tespiti içindir. Ayarlarda açıklanıp kapatılabilmeli; kapatılırsa otel/kafe Wi-Fi giriş sayfası algılaması etkilenebilir. Hava durumu, güncelleme, DNS, Flatpak ve saat eşitlemesini telemetriyle karıştırma.

Rootfs'de etkin bağlantıları görülenler:

| Bileşen | Öneri |
|---|---|
| tuned + tuned-ppd | Güç profillerinin tek sahibi olarak kullan. `blaze-optimize` doğrudan governor/EPP/GPU ayarlarıyla yarışmasın. |
| systemd-oomd | Kapatma; cgroup yerleşimi ve ZRAM ile baskı altında test et. Bütün oturumu değil sorunlu uygulamayı hedefleme davranışını ölç. |
| virtqemud + çok sayıda libvirt socket | Sanallaştırma özelliği kurulunca etkinleştirmeyi değerlendir. Socket etkinliği sürekli ağır CPU kullanımı demek değildir. |
| ModemManager | Hücresel modem kullananlar için tut; minimal profilde isteğe bağlı. |
| cups / avahi | Yazıcı ve ağ keşfi deneyiminin parçası; topluca kapatmak kullanıcı işlevlerini bozar. |
| localsearch / plocate | Büyük dosya ağaçlarında I/O ölç; ilk indekslemeyi pilde ertele ve görünür durum sun. |
| iSCSI / RAID birimleri | Donanım ve boot ihtiyacına göre koşullu; sadece adlarına bakarak silme. |
| VM guest agents | İlgili sanallaştırma ortamında koşullu çalıştır. Fiziksel makinelerde yüklenmemelerini doğrula. |

ABRT config dizini incelenen rootfs'de yoktu. Bu tüm hata raporlama bileşenlerinin bulunmadığını kanıtlamaz. `systemd-report-*` adını tek başına uzaktan telemetri kanıtı sayma; unit içeriği ve ağ davranışı gerekir.

## Bellek, hız ve çökme dayanıklılığı planı

1. **Ölçüm tabanı:** SolarUI+Noctalia ve SolarUI+Caelestia için ayrı 10 dakika boşta PSS/RSS, CPU, wakeup, GPU VRAM ve güç örnekleri. Ardından 8 saat çalışma; aynı pencere setine dönüşte bellek eğrisini karşılaştır. RSS'nin artması tek başına leak değildir; allocator/cache ayrımı yap.
2. **Yük senaryosu:** 100 aç/kapat, 100 shell geçişi, 50 monitör hotplug/ölçek değişimi, 30 suspend/resume; aynı adımlar sonrası kalan process/fd/GLib source sayısı. Heaptrack/Valgrind C++ tarafında, uygun test derlemelerinde sanitizers; Rust'ta Arc döngüleri, task yaşam süreleri ve kanal kapatılmasını incele.
3. **Render:** görünmeyen yüzeyleri çizme; ekran kapalıyken animasyon/spektrum/duvar kâğıdı güncellemelerini durdur. Blur/damage bölgesi ve aşırı çizimi profille; p95/p99 frame-time ölç. 60 Hz için 16,7 ms kare bütçesi bir hedef, mevcut ölçüm değil.
4. **Gözetim:** shell çökünce sadece shell yeniden başlasın; compositor çökünce mevcut Wayland istemcilerinin sorunsuz yaşayacağını vaat etme. Son çalışan ayar + sınırlı restart + açıklamalı kurtarma oturumu.
5. **OOM:** uygulamaları ayrı app.slice scope'larına koy. systemd-oomd PSI davranışını test et; her uygulamaya OOMScoreAdjust=-1000 vermek sistemi kurtarmaz. Shell'e kör MemoryMax koymak masaüstünü kendi kendine öldürebilir.
6. **ZRAM:** mevcut min(RAM,32GiB)/zstd ve swappiness=180 otomatik olarak yanlış değil; 4/8/16/32 GiB profillerde sıkıştırma CPU maliyeti, swap ve gecikme ölç. ZRAM leak düzeltmez. Kullanım sırasında swap/ZRAM servisini restart etme.
7. **Güç:** varsayılan balanced; oyun süresince GameMode, sonra önceki profile dönüş. Sabit AMD high ve global performance fan/pil/ısınma bedeli getirir. Dizüstü bataryadayken efektleri azalt ama kullanıcı tercihini koru.
8. **Ağ:** BBR'yi bütün oyun pinglerini düşüren sihirli ayar gibi sunma; TCP davranışı ile UDP oyun trafiğini ayır. Fedora varsayılanına karşı ölçmeden toplu sysctl dağıtma.
9. **Güncelleme:** tek koordinatör, bir işlem kuyruğu, yeniden başlatma gereksinimi, sürücü/kernel eşleşmesi, kesilen indirmeyi sürdürme. Çalışan paket işlemlerini “kilit temizleme” için zorla durdurma.
10. **Kurtarma:** son çalışan kernel, boot menüsünde recovery, açık disk şifreleme kurtarma akışı. Snapshot başarıyla geri dönülüp doğrulanmadan “tek tık kurtarma” rozeti gösterme.

systemd-oomd davranışı: https://www.freedesktop.org/software/systemd/man/org.freedesktop.oom1.html ve https://fedoraproject.org/wiki/Changes/EnableSystemdOomd .

## Amiral gemisi ürün deneyimi: somut tasarım

### Tek seferlik Merhaba / ilk kurulum

Önerilen uygulama: mevcut Rust/GTK4/libadwaita altyapısında `blaze-first-run`; Noctalia/Caelestia'dan bağımsız çalışır. Root olarak grafik uygulama çalıştırılmaz. Ayrı ve dar Polkit işlemleri sistem ayarlarını yapar.

Akış: `new → greeting → language → keyboard → network → privacy → appearance → ready`. Her tamamlanan adım atomik olarak kullanıcı state dosyasına kaydedilir. `/var/lib/blazeos` yalnız makine hazırlık işlerini; `$XDG_STATE_HOME/blazeos/first-run-v1.json` kullanıcı deneyimini tutar. Live oturumda kurulu sistem sihirbazı çalışmaz. Çökme/reboot sonrasında son tamamlanmış adımdan devam eder. Yeni kullanıcı kendi akışını görür; sürüm güncellemesi her kullanıcıya tekrar tam kurulum yaptırmaz.

- Siyah/açık fon üzerinde özgün Blaze işareti; ardından “Merhaba / Hello / Hola / Bonjour / こんにちは / مرحبًا”. Tipografiyi kendi karakterine göre tasarla; Apple animasyonunu birebir kopyalama.
- El yazısı etkisini normal fonta harf harf metin basmakla değil, özgün vektör stroke yollarının ilerleyen maskesiyle yap. Metnin erişilebilir karşılığı ayrı label olsun. Arapça RTL ve CJK font fallback hazır bulunsun.
- İlk ekran 3–5 saniyelik isteğe bağlı gösterim; ilk tıklama/Enter ile atlanır. “Hareketi azalt”, ekran okuyucu, klavye ve yüksek kontrast ilk ekrandan erişilebilir.
- “Sisteminizi hazırlıyoruz” yalnız gerçekten çalışan işlerin önünde görünür: kullanıcı dizinleri, varsayılan uygulamalar, tema/portal hazırlığı. Ağ yoksa çevrimdışı hazır hale gel; güncellemeyi zorunlu bekletme.
- Sahte yüzde yerine tamamlanan gerçek işler; aşama takılırsa zaman aşımı, anlaşılır hata ve “Masaüstüne devam et”. NVIDIA/MOK gibi işlem kullanıcıya açıkça anlatılır.
- Klavye seçiminde test kutusu; Türkçe Q/F, ABD, CapsLock göstergesi; parolayı yanlış yerleşimde oluşturmayı engelle.
- Masaüstü seçimi: SolarUI Noctalia / SolarUI Caelestia için canlı küçük önizleme, aynı kısayollar ve aynı sistem ayarları. Motor değişince paneller/duvar kâğıdı tutarlı kalır.
- Görünüm: açık/koyu/otomatik, vurgu rengi, ölçek, animasyon azaltma. Donanım önerisi kullanıcı ayarını ezmez.
- Gizlilik: telemetri kapalı, gönderilecek alanları gör, tek seferlik destek paketi önizle, asla otomatik paylaşma.
- Son ekran: “Hazırsınız.” Üç eylem: uygulamalar, dosyaları taşı, kısa tur. Varsayılan tarayıcı, dosya yöneticisi, mağaza hemen bulunabilir.

### Günlük kullanımda fark yaratacak ayrıntılar

- Wi-Fi/Bluetooth bağlantısında bekleniyor/bağlandı/başarısız durumları; görünür retry. Şifreyi yanlışsa doğru hata; sonsuz spinner yok.
- Mikrofon/kamera göstergesinde uygulama adı, erişimi durdurma, geçmişi yerelde tutma seçeneği.
- Ses değişiminde isteğe bağlı kısa geri bildirim; rahatsız etmeyin modunda sessizlik; maksimum ses üstü için açık uyarı.
- Bildirim gruplama, eylem düğmeleri, toplantıda sessiz mod; aynı bildirimi on defa üretmeme.
- 125/150/175% ölçek ve farklı DPI'lı çift monitör; pencere geri getirme, dock çıkarma, kapak kapatma, projektör modu.
- Kilit ekranı klavye dili/CapsLock, ekran okuyucu, güvenilir suspend öncesi kilitleme. Kilit isteğinin başarılı olması ile ekranın gerçekten kilitlenmesini ayrı test et.
- Her önemli ayar için geri al; monitör düzeninde 15 saniyelik otomatik geri dönüş; shell temasında son çalışan yapılandırma.
- İmleç, font, ikon, pencere köşe/dolgu, odak halkası ve animasyon süreleri GTK/Qt/Electron arasında ortak tasarım token'larıyla tutarlı.
- Türkçe kalite: noktalı/noktasız I araması, tarih/saat, çoğul ekleri, taşan metin; İngilizceye geçince artık Türkçe dizelerin kalmaması. Evrensel ürün dili, isteyen için Türkiye kökenini anlatan Hakkında bölümü.
- Disk azaldığında güvenli temizlik önerisi; kullanıcı verisini ve tarayıcı oturumunu izinsiz silmeme.
- Ekran görüntüsü/alan kaydı, ekran paylaşımı, gece ışığı, pil tahmini, touchpad hareketleri, oyun kumandası ve yazıcı kurulumunu uçtan uca tamamla.

### Blaze uygulamalarının görev dağılımı

**Blaze House:** karşılama, sistem sağlığı, kılavuzlar, sürüm notları, destek paketi. “Sağlıklı” ancak ölçülmüş servis/sürücü/alan durumundan hesaplanmalı. Kaynak kodu ayrıca denetlenmeli.

**BlazeOS Control:** tek sistem ayarları merkezi; sürücü durumu, güç profili, kurtarma, varsayılanlar. Yetkili işlemler açık eylem kimlikli backend'e taşınmalı. Her işlemin gerçek sonucu ve iptal edilebilir aşamaları olmalı.

**FireHub:** RPM/Flatpak kaynak etiketi, izin özeti, kurulu boyut, güncelleme kuyruğu, ağ yoksa cache, tekrar deneme, uygulama kaldırırken veri tut/sil seçimi. Aynı uygulamanın RPM ve Flatpak sürümünü açıkça ayır. Mağaza ve Control iki ayrı güncelleme motoru çalıştırmasın.

**SolarUI:** compositor/shell/oturum sınırları net; Caelestia ve Noctalia görünüm seçenekleri. İki tam kabuğu gereksiz yere aynı anda sürekli çalıştırma. Ürün özelliği henüz yoksa etkin anahtar gösterme.

## Diğer sistemlerden alınabilecek fikirler

- **Bazzite:** görüntü tabanlı güncelleme ve önceki sürüme dönüş, sürücüye göre test matrisi. Mevcut DNF tabanına birkaç betik eklemek atomik sistem yapmaz; bootc/OSTree ayrı mimari kararıdır. https://docs.bazzite.gg/Installing_and_Managing_Software/Updates_Rollbacks_and_Rebasing/
- **Windows:** aşamalı ilk kurulum, anlaşılır aygıt sorun giderme ve kurtarma ekranlarının kullanıcı akışı.
- **macOS:** tutarlı tipografi, kısa geçişler, cihaz ilk açılışındaki görsel özen; erişilebilir ve atlanabilir özgün uyarlama.
- **ChromeOS:** kullanıcıdan bakım ayrıntılarını saklayan, arızadan geri dönebilen durum makinesi fikri; güncelleme arızasını boot edilemez sisteme dönüştürmeme.
- **Ubuntu / Debian:** az sürprizli varsayılanlar, paket sahipliği, bakım politikası ve güçlü belgeler.
- **CachyOS:** donanıma göre profil ve ölçüme dayalı seçenek yaklaşımı. Her CPU'ya aynı flags/sysctl kopyalamak yerine x86-64 taban uyumluluğunu koru.

Bunlar tasarım önerileridir; bu sistemlerin güncel tüm özelliklerinin karşılaştırmalı ölçümü yapılmadı. Açık kaynak kod alınacaksa her bileşenin lisansı, kaynak sürümü ve değişiklikleri kaydedilmeli; üçüncü taraf copyright/lisans bildirimleri korunmalı. Windows/macOS kodunun açık kaynak olduğu varsayılmamalı. Somut lisans incelemesi ayrıca yapılmalı.

## Yayın kapıları ve uygulanabilir sıra

**Aşama 1 — açılır/kurulur:** canlı postinstall ayrımı, kullanıcı/oturum kayıtları, NVIDIA/MOK ve driver fallback, dosya sahiplikleri, paketli overlay. BIOS/UEFI, Intel/AMD/NVIDIA ve hibrit GPU'da canlı oturum + diske kurulum + ikinci reboot testi.

**Aşama 2 — sağlam temel:** tekrarlanabilir RPM'ler, kaynak hash'leri, SBOM, imzalı repo/ISO; systemd tek oturum sahibi; çalışan kilit ve portal; elektrik/ağ kesilmesinden devam eden güncelleme. Son çalışan kernel ile kurtarma testi.

**Aşama 3 — ölçülen kalite:** 4/8/16 GiB testleri, düşük disk alanı, bellek baskısı, 8 saat soak, 30 uyku döngüsü, çift monitör ve Unicode. Ölçüm sonuçları olmadan “%30 hızlı / sıfır sızıntı” yazma.

**Aşama 4 — kimlik:** Merhaba akışı, özgün ses/animasyon dili, tutarlı Blaze House/Control/FireHub, oyun/geliştirici/içerik üretici profilleri. İlk kurulum offline tamamlanabilmeli; sonradan ek masaüstleri isteğe bağlı indirilmeli.

Her aşamanın kabulü: başarısızlık kullanıcı verisi kaybettirmiyor, anlaşılır hata veriyor, geri dönüş denenmiş, kaynak sürümüne kadar izlenebiliyor. Desteklenmeyen donanım açık belirtiliyor. Donanımda hiç test edilmemiş ISO “kararlı” diye etiketlenmemeli.

## Doğrulama kaydı

- `cargo test --workspace --offline`: **40 test geçti**. İlk sandbox çalıştırması Unix socket açma izninde hata verdi; aynı suite izinli çalıştırmada geçti. Ayrıntı: `audit/2026-09-28/cargo-test.log`.
- Yeni izole Python regresyonları: **4 test geçti** — live erken çıkış, NVIDIA modülü yokken boot değişikliği yapmama, log sınırı, GDM/AccountsService eşleştirmesi. Sahte komut ve geçici dosyalar kullanılır.
- Bash syntax, Python AST parse, `git diff --check`: geçti.
- Değiştirilen shell betiklerinde `shellcheck -S warning`: geçti; `shellcheck-changed.txt`.
- ShellCheck raporu `audit/2026-09-28/shellcheck.txt`; uyarısız kod iddiası yok. Bu ilk tarama kaydıdır.
- Envanter `source-inventory.json` metin/çeviri verisini de içerir; satır sayısı elle incelenmiş kod satırı sayısı değildir.
- Yeni kaynaklardan release derlemesi, düzeltilmiş ISO remaster, kurulu sistem testi, Secure Boot/hardware GPU testi ve bellek profili yapılmadı. Gerçek bilgisayardaki GNOME davranışının kesin kök nedeni henüz günlükle doğrulanmadı.
