# end4-pC → SolarUI Tam Entegrasyon Planı

## Amaç ve kapsam

Bu çalışma, [end4-pC](https://github.com/pctrade/end4-pC) kabuğunu adını ve proje kimliğini koruyarak SolarUI içinde Noctalia ve Caelestia'nın yanında seçilebilen üçüncü bir tam kabuk motoruna dönüştürmeyi amaçlar. Hedef compositor Niri'dir. Kullanıcı `solar-shell switch end4-pc` komutuyla veya SolarUI Ayarları içinden canlı geçiş yapabilmeli; başarısız başlangıçta önceki çalışan kabuğa otomatik dönülmelidir.

## İlerleme durumu — 2026-10-16

- [x] Hybrid motoru ve parçalı yüzey seçimleri kaldırıldı; eski Hybrid ayarları Noctalia'ya göç ediyor.
- [x] end4-pC kaynağı upstream commit ve GPL-3.0 kayıtlarıyla vendor dizinine alındı.
- [x] Niri altında Hyprland autostart ve kabuk içi conflict-killer engellendi.
- [x] Bilinmeyen compositor için tehlikeli Hyprland geri dönüşü kaldırıldı.
- [x] Supervisor için gerçek QML/IPC hazır olma probu eklendi.
- [x] İzole PID sahipliği, start/stop, IPC dispatch ve sağlık kontrolü olan `End4PcProvider` eklendi.
- [x] Fedora DNF/RPM güncelleme ve paket sayımı uyarlandı.
- [x] Zorunlu QtPositioning, KDE SyntaxHighlighting ve Kirigami yükleme bağımlılıkları kaldırıldı.
- [x] Niri üzerinde QML yükleme ve IPC smoke testi geçti (`solarui.ping = ready`).
- [x] Görünür Niri akışlarındaki Hyprland komutları dönüştürüldü; Hyprland'a özgü oyun modu ve anti-flashbang kontrolleri Niri'de gizlendi.
- [x] `ShellEngine::End4Pc`, switcher/supervisor/router geçiş ve rollback entegrasyonu tamamlandı.
- [x] SolarUI Ayarları ve OOBE seçeneği, IPC ayar açıcı ve kalıcı yapılandırma desteği eklendi.
- [x] ISO/RPM kaynak, lisans ve upstream kayıtlarının çevrimdışı paketlenmesi eklendi.
- [x] QML başlatma, 480 IPC eylemi, süreç sahipliği ve bellek plato testi tamamlandı.
- [ ] Bildirim, polkit, tray ve clipboard sahipliği temiz bir VM oturumunda doğrulanacak.
- [ ] Çoklu monitör, askıya alma ve gerçek donanım/GPU kabul matrisi uygulanacak.

Bu belge kaynak incelemesine dayanır:

- end4-pC kaynak sürümü: `8fbb6716cba628a13103e665e9538021683a7250`
- SolarUI karşılaştırma sürümü: `40c0ca2a057e06f6dbfd5c9ca1a2e2ecadcb8a4d`
- end4-pC lisansı: GPL-3.0
- İncelenen end4-pC kapsamı: 865 dosya, 633 QML dosyası

## Kaynak incelemesinin sonucu

end4-pC yalnız Hyprland üzerinde çalışabilen bir kabuk değildir. Kaynakta `NiriBackend.qml`, `NiriConfig.qml`, `NiriXkb.qml`, Niri overview bileşeni ve Niri arka plan katmanı bulunur. Pencere, çalışma alanı ve çıktı bilgileri `niri msg -j` ile okunmakta; odaklama, taşıma, kapatma ve çalışma alanı eylemlerinin önemli bölümü uygulanmaktadır. Türkçe çeviri kataloğu da mevcuttur.

Bununla birlikte mevcut depo doğrudan SolarUI'a kopyalanacak durumda değildir:

1. `shell.qml`, `Quickshell.Hyprland` modülünü doğrudan içe aktarır ve Hyprland autostart betiğini hazırlar.
2. Genel kısayol köprüsü yalnız Hyprland aktifken çalışır. Niri kısayolları end4-pC IPC uçlarına henüz bağlanmamıştır.
3. Bilinmeyen compositor algılandığında Hyprland'a geri düşülür. Bu, Niri oturumunda yanlış pozitif davranış üretme riski taşır.
4. Duvar kâğıdı, ekran kaydı ve imleç ayarının bazı yollarında doğrudan `hyprctl` çağrıları vardır.
5. Güncelleme, paket sayımı ve varsayılan güncelleme komutları Arch/pacman varsayımları içerir.
6. Ayarlar içinde Hyprland'a özgü sayfalar ve seçenekler Niri altında görünmeye devam edebilir.
7. `ConflictKiller.qml`, `kded6`, `mako` ve `dunst` süreçlerini öldürebilir. Bu davranış SolarUI sağlayıcı yaşam döngüsüne bağlanmadan etkinleştirilmemelidir.
8. Üst kaynak README'si, çalışan bir illogical-impulse kurulumunu ön koşul kabul eder. BlazeOS paketi bu örtük bağımlılığı ortadan kaldırmalı ve gereken her dosyayı açıkça sağlamalıdır.

Kaynak taramasında Hyprland adını, modülünü veya komutlarını kullanan 90 QML/betik dosyası bulundu. Bunların tamamı hata anlamına gelmez; her biri `Niri karşılığı`, `compositor bağımsız`, `Hyprland'a özel ve gizlenecek` olarak sınıflandırılmadan özellik tamamlanmış sayılmayacaktır.

## Hedef mimari

### Kabuk motoru modeli

end4-pC ilk sürümde bağımsız bir tam kabuk motoru olacaktır:

```text
SolarUI / Niri
├── Noctalia      (tam kabuk)
├── Caelestia     (tam kabuk)
└── end4-pC       (tam kabuk)
```

Hybrid motor kaldırılmıştır. Her kabuk kendi bar, launcher, dashboard, bildirim, OSD, kilit ve oturum yüzeylerini tek başına yönetir. Böylece iki kabuğun katman yüzeyleri, bildirim daemon'ları ve giriş bölgeleri birbirine karışmaz.

### Dosya yerleşimi

- Sürüm kontrollü kaynak: `solarui/vendor/end4-pC/`
- Sistem kurulumu: `/usr/share/quickshell/end4-pC/`
- Kullanıcıya ait ayarlar: `~/.config/end4-pC/` ve `~/.local/state/end4-pC/`
- Önbellek: `~/.cache/end4-pC/`
- Kaynak sürüm kaydı: `/usr/share/doc/solarui/end4-pC-UPSTREAM`
- Lisans: `/usr/share/licenses/solarui/end4-pC/LICENSE`

Program kodu kullanıcı dizinine kopyalanıp sessizce değiştirilmeyecektir. Yazılabilir ayar ve durum dosyaları program dosyalarından ayrılacaktır. Quickshell çalıştırma yöntemi kurulu salt okunur yolu desteklemiyorsa, kullanıcı yapılandırma dizinine yalnız sürümlü bir bağ/başlatıcı yerleştirilecek; kaynak ağdan çalışma anında çekilmeyecektir.

## Uygulama aşamaları

### 1. Kaynağı sabitleme ve lisans uyumu

- end4-pC deposunu yukarıdaki commit'e sabitleyen vendor kaydı ekle.
- Orijinal `LICENSE`, README ve telif bildirimlerini koru.
- `UPSTREAM.md` içinde pctrade/end4-pC, end-4/dots-hyprland ve kullanılan commit'i belirt.
- SolarUI değişikliklerini ayrı `BLAZEOS-CHANGES.md` dosyasında tarih ve kapsamla kaydet.
- Ağdan otomatik `git pull` yapan davranışı paket sürüm yöneticisinden ayır; güncelleme SolarUI/BlazeOS paketleriyle gelsin.

### 2. Niri uyumluluk katmanı

- `WM.qml` için desteklenmeyen compositor durumunu açık hata haline getir; Hyprland'a örtük geri dönüşü kaldır.
- `Quickshell.Hyprland` kullanan nesneleri ayrı QML dosyalarına taşı ve yalnız Hyprland üzerinde `LazyLoader` ile oluştur.
- `shell.qml` içindeki Hyprland autostart yolunu Niri altında kesin olarak devre dışı bırak.
- `hyprctl monitors -j` kullanan duvar kâğıdı ve kayıt kodunu `niri msg -j outputs` veya Quickshell ekran modeliyle değiştir.
- `hyprctl setcursor` yolunu portal/GTK ayarı ve Niri uyumlu imleç yapılandırmasıyla değiştir.
- Overview, pencere odaklama, pencere taşıma, çalışma alanı, çoklu monitör ve XKB eylemlerini gerçek Niri olay akışıyla doğrula.
- Niri'de karşılığı olmayan bir özellik varsa butonu çalışır gibi göstermeden pasifleştir; yanında nedenini ve gereken paketi göster.

### 3. SolarUI Rust sağlayıcısı

`solarui/crates/solar-shell/src/providers/` içine `End4PcProvider` ekle:

- `name()` → `end4-pC`
- `is_installed()` → QML giriş noktası, Quickshell ve zorunlu çalışma zamanı modüllerini doğrular
- `start()` → ayrı process group içinde sabit sistem yolundan Quickshell başlatır
- `stop()` → yalnız bu sağlayıcının PID/process group'unu sonlandırır; genel `killall qs` kullanmaz
- `health_check()` → PID kontrolüne ek olarak IPC ping ve zorunlu yüzeylerin hazır olmasını denetler
- `dispatch()` → SolarUI eylemlerini end4-pC IPC çağrılarına çevirir

Ardından şu tipler ve geçiş tabloları genişletilir:

- `solar-common/src/config.rs`: `ShellEngine::End4Pc` ve gerekirse `SurfaceProvider::End4Pc`
- `solar-shell/src/engine.rs`: görünen ad ve açıklama
- `solar-shell/src/supervisor.rs`: `ActualEngine::End4Pc`, watchdog ve yeniden başlatma sınırı
- `solar-shell/src/switcher.rs`: geçiş, beş saniyelik sağlık doğrulaması ve önceki motora rollback
- `solar-shell/src/router.rs`: eylem yönlendirme matrisi
- `solar-shell/src/main.rs`: CLI ayrıştırma, durum ve oyun modu durdur/geri yükle yolları

Geçiş sırası atomik olacaktır: hedefi ön doğrula → hedefi başlat → IPC hazır olmasını bekle → yeni yüzeylerin hazır olduğunu doğrula → eski kabuğu durdur → tercihi diske yaz. Hedef başarısızsa eski kabuk kesintisiz kalır.

### 4. Eylem ve kısayol köprüsü

end4-pC halihazırda launcher, settings, session, lock, sidebar, wallpaper, overview, media, OSD, clipboard ve başka yüzeyler için Quickshell `IpcHandler` uçları sunar. SolarUI yönlendirmesi şu davranışları kapsamalıdır:

| SolarUI eylemi | end4-pC hedefi | Kabul koşulu |
|---|---|---|
| Launcher | end4-pC launcher IPC | Aç/kapat, Escape ve klavye odağı çalışır |
| Settings | `settings` IPC | Aynı kısayolla aç/kapat çalışır |
| Dashboard / Control Center | ilgili sidebar/dashboard IPC | Tek örnek ve doğru monitör |
| Session | `session` IPC | Kilitle, çıkış, askıya al, yeniden başlat, kapat |
| Lock | `lock.activate` | Parola doğrulaması ve çoklu monitör |
| Wallpaper | wallpaper selector / `wallpapers.apply` | PNG/JPEG, çoklu monitör, hata bildirimi |
| Media | `mediaControls` / `mpris` | oynat/duraklat/önceki/sonraki |
| OSD | ses ve parlaklık IPC | tuşla tetiklenme ve doğru ekran |

`solarui/data/niri.kdl` kısayolları doğrudan end4-pC komutlarına gömülmeyecek; mevcut `solar-shell route ...` sözleşmesini kullanacaktır. Böylece kabuk değiştiğinde Niri yeniden başlatılmaz.

### 5. Fedora 45/BlazeOS uyarlaması

- pacman/yay/paru komutlarını dağıtım algılayan bağdaştırıcıya taşı; BlazeOS'ta DNF5/RPM sorgularını kullan.
- Güncelleme sayacı ile güncelleme eylemi aynı kaynağı kullanmalı; yanlış sayım veya işlevsiz düğme kalmamalı.
- Paket bağımlılıklarını `zorunlu`, `önerilen`, `isteğe bağlı özellik` olarak ayır.
- Temel paket kümesi: Quickshell, Qt6 QML modülleri, PipeWire/WirePlumber, NetworkManager, BlueZ, UPower, polkit agent, `grim`, `slurp`, `wl-clipboard`, `cliphist`, `cava`, `jq`, `curl`, `python3`, `playerctl`, `lm_sensors` ve kullanılan tema araçları.
- `ydotool`, `tesseract`, `songrec`, `satty/swappy`, EasyEffects ve GitHub CLI gibi seçenekler yoksa ilgili UI açıklamalı şekilde devre dışı kalmalı.
- `matugen`, fontlar, sesler, shader'lar ve çeviriler ISO'ya sürüm kontrollü olarak eklenmeli.
- Varsayılan terminal, dosya yöneticisi, görev yöneticisi ve parola değiştirme komutları BlazeOS uygulamalarına/kurulu alternatiflere bağlanmalı.

### 6. Çakışma ve güvenlik sertleştirmesi

- `ConflictKiller` kendi başına süreç öldürmeyecek. Çakışma çözümü supervisor tarafından, açık PID sahipliği ve sağlayıcı bilgisiyle yapılacak.
- `kded6` gibi başka masaüstlerine ait genel süreçler koşulsuz öldürülmeyecek.
- Bildirim daemon geçişinde önce yeni sağlayıcı hazır olacak, sonra yalnız eski sağlayıcıya ait daemon kapatılacak.
- QML içindeki `bash -c` komutları kullanıcı kontrollü değerler bakımından taranacak; yollar argv ile geçirilecek veya güvenli tek tırnak kaçışı uygulanacak.
- Preset içe aktarma; yol aşımı, sembolik bağ, komut alanı, kişisel anahtar ve mutlak yol kontrollerinden geçecek.
- Çevrimiçi preset/GitHub/AI/hava özellikleri ilk kullanımda açık ağ davranışını belirtmeli; anahtarlar loglara veya paylaşılan presetlere girmemeli.
- Kilit ekranı, gerçek oturum kilidi oluşmadan yalnız görsel katman açmış sayılmayacak.

### 7. SolarUI Ayarları ve OOBE

SolarUI Ayarları > Kabuk Motoru listesine şu seçenek eklenir:

```text
end4-pC — Material 3 Expressive Quickshell
```

Kart üzerinde kurulum durumu, zorunlu bağımlılık sonucu, son çökme nedeni ve “end4-pC Ayarlarını Aç” eylemi gösterilir. Seçim sırasında ilerleme ve rollback sonucu kullanıcıya bildirilir.

OOBE'ye eklenmesi ayrı bir ürün kararıdır. ISO içinde eksiksiz paketlenecekse seçilebilir olabilir; ağdan indirilmesi gerekiyorsa çevrimdışı durumda seçilebilir görünmemelidir. OOBE, yalnız sağlık ön kontrolü başarılı sağlayıcıları etkin seçenek olarak göstermelidir.

### 8. Ayar ve özellik bütünlüğü

Her ayar satırı aşağıdaki sınıflardan birine alınır:

- Çalışıyor ve otomatik test ediliyor.
- İsteğe bağlı bağımlılık eksik; açıklamalı olarak pasif.
- Yalnız Hyprland'a özgü; Niri oturumunda gizli.
- Niri karşılığı geliştirilmiş ve doğrulanmış.

Özellikle ekran kaydı, bölge seçimi, ekran çevirisi, renk seçici, gece ışığı, oyun modu, sanal klavye, çoklu monitör duvar kâğıdı, online preset, AI, hava, müzik tanıma ve firmware oturum eylemleri uçtan uca denenmeden “tam port” etiketi kullanılmayacaktır.

### 9. README ve teşekkürler

Hem Blaze-Galaxy hem SolarUI README'sine, mevcut görseller korunarak “Teşekkürler ve Katkılar” bölümü eklenir:

- [Noctalia](https://github.com/noctalia-dev/noctalia)
- [Caelestia Shell](https://github.com/caelestia-dots/shell)
- [end4-pC](https://github.com/pctrade/end4-pC)
- [illogical-impulse / dots-hyprland](https://github.com/end-4/dots-hyprland)

Metin, SolarUI'ın bu projelerin çalışmalarından faydalandığını açıkça söyleyecek; özgün proje adları ve geliştirici atıfları korunacaktır. end4-pC arayüzünde Hakkında/Lisans alanından da upstream kaynak ve lisans görülebilecektir.

## Doğrulama matrisi

### Otomatik testler

- Rust enum/serde geriye uyumluluğu: eski `config.toml` dosyaları açılmalı.
- Sağlayıcı başlangıç, durdurma, çökme, timeout ve rollback birim testleri.
- Router: her SolarUI eyleminin end4-pC IPC eşlemesi.
- Aynı anda iki tam kabuğun bildirim/bar yüzeylerinin açık kalmadığını doğrulayan süreç testi.
- QML statik kontrolü; Niri yolunda zorunlu Hyprland import'u bulunmaması.
- Fedora bağımlılık ve RPM içerik testi.
- GPL lisans/atıf dosyalarının paket içinde bulunduğunu doğrulayan paket testi.
- `cargo test --workspace --release`, `cargo clippy --workspace --all-targets -- -D warnings` ve RPM kurulum testi.

### Sanal makine testleri

- Temiz kullanıcı ve yükseltilmiş eski kullanıcı profili.
- Wayland sanal GPU ile 1 ve 2 monitör; çözünürlük/ölçek/hotplug.
- Noctalia → end4-pC → Caelestia → end4-pC canlı geçişi.
- Hedef kabuğun kasıtlı öldürülmesi ve otomatik toparlanma.
- Eksik isteğe bağlı paketlerde sahte/tepkisiz buton bulunmaması.
- Ağ yok, yavaş ağ, DNS hatası ve captive portal durumları.
- Askıya al/uyandır, ekran kilidi, kullanıcı değişimi ve oturum kapatma.
- Bildirim, pano, ses, mikrofon, parlaklık, Wi-Fi, Bluetooth, medya ve sistem tepsisi.

### Gerçek donanım testleri

- Intel, AMD ve NVIDIA GPU; dokunmatik yüzey ve fare.
- 60/120/144 Hz, HiDPI ve kesirli ölçekleme.
- Dizüstü kapak, pil, güç profili, harici ekran ve ses aygıtı hotplug.
- 8 saat normal kullanım ve 500 kabuk geçişi/IPC eylemi dayanıklılık testi.
- Boşta RAM, geçiş sonrası kalan süreç, GPU kullanımı, QML uyarı sayısı ve coredump kontrolü.

## Yayına kabul ölçütleri

end4-pC seçeneği ancak aşağıdakilerin tümü sağlandığında varsayılan ISO'ya alınır:

1. Niri oturumunda kritik özelliklerde doğrudan `hyprctl` bağımlılığı kalmaz.
2. Ayarlarda görünen her düğme çalışır veya bağımlılık/neden bilgisiyle pasiftir.
3. Canlı geçiş başarısız olduğunda çalışan kabuk ve kullanıcı ayarı kaybolmaz.
4. Kabuk çökmesi Niri oturumunu veya SolarUI supervisor'ını sonlandırmaz.
5. Bildirim, polkit, tray, pano ve kilit hizmetlerinde çift sahiplik oluşmaz.
6. Çevrimdışı ilk açılış çalışır; çalışma anında GitHub klonu gerekmez.
7. Türkçe ve İngilizce temel akışlarda taşan/kodlanmış metin kalmaz.
8. Lisans ve upstream atıfları hem pakette hem README'de görünür.
9. Sanal makine ve en az iki farklı gerçek GPU ailesindeki test matrisi geçer.
10. Bellek kullanımı ve açılış süresi için ölçülen regresyon sınırı belirlenir; sınırı aşan sürüm yayınlanmaz.

## 2026-10-16 yerel doğrulama sonucu

- Rust çalışma alanı: 48 test geçti, 0 başarısız.
- Release derleme: başarılı.
- end4-pC QML başlangıcı: ilk sağlık sorgusunda `ready`.
- IPC dayanıklılığı: launcher, sağ panel, ayarlar ve oturum yüzeylerinde toplam 480 çağrı; 0 hata.
- Kritik QML yükleme/çökme satırı: 0.
- Bellek: bütün ağır yüzeyler ilk kez açıldıktan sonra RSS 933–949 MiB aralığında plato yaptı ve son örnekte 874 MiB'ye geriledi; tepe 980 MiB civarıdır. Bu değer düşük bellek hedefi için hâlâ yüksektir ve VM kabul testinde bütçe olarak ayrıca izlenmelidir.
- Süreç sonlandırma: yalnız test edilen end4-pC PID'si SIGTERM ile temiz biçimde kapandı.

## Önerilen çalışma sırası

1. Vendor + lisans + Fedora bağımlılık manifesti
2. Hyprland bağımlılık envanteri ve Niri uyumluluk yamaları
3. `End4PcProvider`, supervisor, switcher ve rollback
4. IPC/router ve Niri kısayolları
5. SolarUI Ayarları kartı ve end4-pC ayar açıcı
6. Sahte buton/eksik bağımlılık denetimi
7. RPM/ISO paketleme
8. VM testleri, gerçek donanım testi ve performans ölçümü
9. README teşekkürler, lisans ekranı ve yayımlama notları

Bu sıra, görünür seçim kutusunu en sona yakın etkinleştirir. Böylece kullanıcı arayüzünde henüz güvenilir biçimde başlatılamayan bir motor sunulmaz.
