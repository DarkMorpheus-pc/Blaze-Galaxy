# SolarUI kararlılık çalışması — 16 Eylül 2026

Bu çalışma mevcut çalışma ağacının üzerine uygulandı; kullanıcının önceki değişiklikleri korunmuştur. Canlı masaüstü oturumu yeniden başlatılmadı ve sistem kurulumu yapılmadı.

## Araştırılan kaynaklar ve uygulanan kararlar

Resmi belgelerin erişim tarihi: 16 Eylül 2026. `latest` belgelerindeki API'lerin projede kilitlenmiş bağımlılıklarla derlenmesi ayrıca doğrulandı; toplu bağımlılık sürümü yükseltmesi yapılmadı.

- [Tokio broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html): yavaş alıcılar mesaj kaybedebilir. Core güncellemeleri sınırlı kapasiteli `mpsc` üzerinden tek durum işleyicisine gidiyor; broadcast yalnızca durum güncellendikten sonra istemcilere yayın yapıyor. Yavaş IPC istemcileri tam durum görüntüsüyle yeniden eşitleniyor. Görüntü alınırken alıcı kuyruğu aynı durum kilidi altında sıfırlanıyor; eski olaylar yeni görüntünün üzerine uygulanmıyor.
- [Tokio AsyncBufReadExt](https://docs.rs/tokio/latest/tokio/io/trait.AsyncBufReadExt.html): `select!` içinde iptal edilen `read_line` kısmi veriyi kaybedebilir. Ortak `JsonLines` okuyucusu iptal sırasında parçaları koruyor ve mesaj boyutunu sınırlıyor. Core komutları 64 KiB; shell/Niri durum mesajları 8 MiB ile sınırlı. Core aynı anda en fazla 64 istemci kabul ediyor.
- [Niri WorkspaceReferenceArg](https://docs.rs/niri-ipc/latest/niri_ipc/enum.WorkspaceReferenceArg.html) ve [Workspace](https://docs.rs/niri-ipc/latest/niri_ipc/struct.Workspace.html): sabit `Id` ile monitördeki değişebilen `Index` ayrıldı. Geri yükleme ve çalışma alanı seçimi ID kullanıyor. Orijinal alan silinmişse odaklı alana geri dönülüyor. Çoklu monitörde `is_focused` ve `WorkspaceActivated.focused` dikkate alınıyor.
- [Rust File kilitleri](https://doc.rust-lang.org/std/fs/struct.File.html): core örnekleri, shell geçişleri ve küçültme işlemleri süreçler arası dosya kilitleriyle korunuyor. Kilit dosyaları silinmiyor; kilit sahibi dosya tanıtıcısı bırakılınca kilit çözülüyor. Kullanılan API Rust 1.89'da kararlı hale geldiği için workspace paketlerinde bu alt sınır belirtildi.
- [tempfile NamedTempFile](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html): kayıtlar hedefle aynı dizindeki geçici dosyaya yazılıyor; dosya senkronizasyonu, atomik değiştirme ve üst dizin senkronizasyonu uygulanıyor. Bozuk ayar dosyası watchdog tarafından varsayılan motora geçme talebi sayılmıyor ve kayıt sırasında sessizce ezilmiyor.
- [Tokio process Command](https://docs.rs/tokio/latest/tokio/process/struct.Command.html): süre sınırlı shell IPC komutlarında `kill_on_drop` kullanılıyor, çıkış durumu denetleniyor. Niri istekleri 3 saniye, shell sağlık/IPC komutları 2 saniyeyle sınırlı. Watchdog başarısızlıklarda 3–60 saniye aralığında artan bekleme kullanıyor.
- [Pango Layout.set_text](https://docs.gtk.org/Pango/method.Layout.set_text.html): uzunluk verilmiş metnin NUL ile sonlanması gerekmiyor. Opsiyonel C kancasında uzunluğa saygılı karşılaştırma ve `pthread_once` ile sembol çözümleme kullanılıyor.

Noctalia ve Caelestia sağlık kontrolü komutları ayrıca bu depodaki sağlayıcı kaynaklarına karşı doğrulandı:

- `vendor/noctalia/src/app/application_ipc.cpp`: `msg status` JSON yanıtı.
- `vendor/caelestia-shell/modules/Shortcuts.qml`: `drawers isOpen bar` için `0` / `1` yanıtı.

## Davranış değişiklikleri

- Başarısız shell geçişi başarı olarak kaydedilmiyor. Önceden sağlıklı motor geri getirilmeye çalışılıyor; geri dönüş de başarısızsa bu açıkça hata olarak bildiriliyor. Eşzamanlı geçiş reddediliyor; watchdog eski bir ayarı kilit beklemeden sonra uygulamıyor.
- Noctalia/Caelestia süreç varlığı tek başına sağlık sayılmıyor. IPC yanıtı da doğrulanıyor. Sağlıksız süreç yeniden başlatılmadan önce durduruluyor. Süreç arama/sonlandırma mevcut kullanıcıyla sınırlandırıldı.
- CLI ve core aynı küçültme/geri yükleme uygulamasını kullanıyor. İşlem boyunca kayıt kilidi tutuluyor. Geri yükleme tamamlanmadan kurtarma kaydı silinmiyor. Pencerenin gerçek park alanı, kayıt silinmeden önce gelen geri yükleme olaylarının doğru gösterilmesini sağlıyor.
- Sabit 1000 piksel kaydırma kaldırıldı. Yeni küçültmeler pencereyi `background` çalışma alanına park ediyor ve mevcut floating/tiling modunu koruyor. Eski kayıtların kaydırma davranışı için uyumluluk yolu bulunuyor.
- Niri olayları yerel pencere modeline uygulanıyor; her odak olayında bütün pencere listesini yeniden sorgulayan bağlantılar kaldırıldı. İkincil monitördeki aktivasyon global odağı değiştirmiyor ve overview durumu korunuyor.
- Shell arayüzünün olay kuyruğu sınırlı. Başlangıç durumu core'dan geliyor; geç gelen bağımsız Niri sorgusu yeni durumu ezemiyor. Ham pencere listesi ve ardından registry olayı nedeniyle yapılan çift çizim azaltıldı; değişmeyen registry çizimi atlanıyor.
- `RequestStateSync` gerçekten snapshot gönderiyor. Komut hataları istemciye `CommandFailed` olayıyla iletiliyor ve shell bunları logluyor. Bluetooth komutunun argüman ayrımı düzeltildi. IPC arızası daemon tarafından başarılı çıkışa çevrilmiyor.
- UPower/NetworkManager dinleyicileri sonlanırsa yeniden başlatılıyor; başlangıç ve yeniden abonelikte durum okunuyor. Ağ sorgusundaki bloklayan `iwgetid` çağrısı süre sınırlı asenkron çağrı oldu.
- Oturum genelindeki otomatik `LD_PRELOAD` kaldırıldı. GTK pencere nesnesini yok sayıp odaktaki başka pencereyi küçültebilen kancalar kaldırıldı. Uygulamanın kendi başlık çubuğundaki küçültme artık doğal GTK/compositor davranışına bağlıdır; SolarUI görev çubuğu ve kısayol üzerinden küçültme devam eder. Niri'nin doğal küçültme desteğinin yerine uygulama içine kanca enjekte edilmiyor.
- Yerel kurulum betiği eski binary bulduğu için derlemeyi atlamıyor; kurulumdan önce güncel kaynaklar derleniyor. Betik bu çalışma sırasında çalıştırılmadı.

## Doğrulama

- `cargo test --workspace --offline`: 40 test geçti. Başlangıçta 18 test vardı; iki kurulu Noctalia gerektiren test, izole sahte sağlayıcı testleriyle değiştirildi ve yeni regresyon kapsamı eklendi.
- Testler: IPC parçalanması/iptali, bozuk ve büyük mesajlar, broadcast gerisinde kalan istemci, snapshot sıralaması, ikinci daemon, eski canlı/stale socket, atomik dosya değiştirme, kilit çakışması, bozuk ayar/kayıt, 255'ten büyük çalışma alanı ID'si, başarısız restore, kaybolan çalışma alanı, yanıtsız compositor, odak olayları, shell geçişi/rollback ve başarısız IPC komutları.
- Unix soketi testleri sandbox içinde `Operation not permitted` aldığı için izinli sandbox dışı çalıştırmada doğrulandı. Bu testler geçici dizinlerdeki sahte soketleri kullanır; gerçek masaüstü süreçlerini durdurmaz.
- `cargo build --workspace --offline`: debug executable derlemesi.
- `cargo clippy --workspace --all-targets --offline`: hata yok; mevcut türetilebilir `Default`, uzun parametre listeleri ve benzeri stil uyarıları sürüyor. `-D warnings` temizliği iddia edilmiyor.
- `gcc -Wall -Wextra -Werror -pthread -o /tmp/solar-brand-hook-test tests/brand_hook_test.c -ldl` ve test executable'ı: geçti. Sonlandırılmamış metnin arkasındaki bellek sayfası erişime kapatılarak sınır dışı okuma regresyonu sınandı.
- Opsiyonel C kancası `-Wall -Wextra -Werror -O2 -fPIC -shared -pthread` ile derlendi.
- `bash -n scripts/install-local.sh`, `git diff --check`: geçti.

## Sınırlar ve sonraki doğrulama

Bu çalışma ChromeOS/Ash düzeyinde bütün bir masaüstü sertifikasyonu veya ölçülmüş performans sonucu değildir. Gerçek oturumda kilit ekranı/uyku, ekran çıkarma-takma, farklı DPI, ekran paylaşımı/portal ve GPU sürücüsü testleri yapılmadı. Boşta CPU, enerji tüketimi, bellek ve kare süresi yüzdelikleri ölçülmedi. Görev çubuğu hâlâ değişen pencere kümesini yeniden kuruyor; tamamen anahtarlı artımlı widget güncellemesi ileride ölçümle değerlendirilmeli.

Oturum/systemd yaşam döngüsünün tek sahipli hale getirilmesi, aynı kullanıcının birden fazla grafik oturumu için sağlayıcı süreçlerinin ayrıca ayrılması, compositor yeniden başladıktan sonra eski küçültme kayıtlarının oturum kimliğiyle geçersizleştirilmesi ve bütün ayar ekranlarının alan bazlı `SolarConfig::update` kullanmasına geçirilmesi sonraki çalışmalardır. Atomik kayıt, eski bir arayüz kopyasının yeni bir ayarı ezmesini tek başına önlemez.

Debug çıktıları çalışma alanında hazırdır. Sistemdeki kurulu binary'ler ve mevcut oturum bu değişikliklere geçirilmedi; dağıtım için güncel kaynaklardan kurulum ve kontrollü yeni oturum testi gerekir.
