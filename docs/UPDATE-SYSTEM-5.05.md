# Blaze SolarEvolution 5.05 Güncelleme Mimarisi

## Kullanıcı akışı

Blaze House, Fedora tabanında güncellemeyi açık masaüstü oturumunda kurmaz.

1. `dnf5 check-upgrade` ile paketler listelenir.
2. **İndir ve hazırla**, sabit komutlu `/usr/libexec/blazeos-update prepare`
   yardımcısı üzerinden `dnf5 --refresh upgrade --offline -y` çalıştırır.
3. **Şimdi yeniden başlat**, DNF5 çevrimdışı işlemini başlatır.
4. `dnf5-offline-transaction.service` aynı DNF işlemini
   `/usr/libexec/blazeos-offline-update` sarmalayıcısıyla yürütür.
5. DNF'nin `[işlenen/toplam]` çıktısı Plymouth'a gerçek ve geriye gitmeyen
   bir yüzde olarak iletilir.
6. Hata kodu korunur; başarısız işlem başarılıymış gibi gösterilmez.

Yetki penceresi SolarUI Wayland ortamı hazır olduktan sonra başlatılan
`mate-polkit` aracısı tarafından gösterilir. Blaze House serbest metinli bir
root kabuğu çalıştırmaz; yardımcının kabul ettiği eylemler yalnızca `prepare`,
`reboot` ve `cancel` seçenekleridir.

## Yayın kaynağı

ISO güncelleme kaynağı değildir. ISO kurulum ve kurtarma medyası olarak kalır.
Archive.org yalnızca tarihî ISO aynası için uygundur.

Üretim kanalı şu yapıda olmalıdır:

- Fedora paketleri: Fedora'nın imzalı resmi depoları.
- BlazeOS paketleri: BlazeOS'a ait GPG imzalı RPM deposu.
- Depo barındırma: GitHub Pages, GitHub Release varlıkları önünde statik CDN
  veya bağımsız sunucu olabilir; DNF'nin tükettiği şey `repodata/` içeren RPM
  deposudur.
- Kaynak kodu arşivi ve `.iso` dosyası paket güncellemesi olarak kurulmaz.

Blaze bileşenleri ayrı RPM'lere ayrılmalıdır: `blazeos-core`, `blaze-house`,
`solarui`, `blaze-recovery`, `blaze-branding`. Her RPM Fedora anahtarı yerine
BlazeOS yayın anahtarıyla imzalanır. Depo ayarında hem `gpgcheck=1` hem
`repo_gpgcheck=1` zorunlu tutulur. Yayın anahtarının özel kısmı geliştirici
makinesinde veya çevrimdışı imza ortamında kalır; ISO yalnızca genel anahtarı
taşır.

Bu paket deposu yayımlandığında mevcut Blaze House akışı hiçbir arayüz
değişikliği gerektirmeden Fedora ve BlazeOS paketlerini aynı çevrimdışı DNF
işleminde kurar.

## Görsel durum

Plymouth güncelleme modu şunları gösterir:

- `Evren güncelleniyor, daha da genişliyor…`
- `Lütfen bilgisayarınızı kapatmayın`
- gerçek paket işlem yüzdesi ve ilerleme çubuğu
- ekranın altında `Secured by Blaze`

Normal açılış görünümü korunur; bu düzen yalnızca `updates` veya
`system-upgrade` Plymouth modunda etkinleşir.
