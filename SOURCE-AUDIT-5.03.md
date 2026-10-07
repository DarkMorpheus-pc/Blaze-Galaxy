# Blaze SolarEvolution 5.03 — Masaüstü ve Önyükleyici Kaynak Denetimi

## Masaüstü kaynakları

Kurulum ekranındaki çevrimiçi seçenekler `blazeos-postinstall` tarafından DNF5 ile kurulur. Etkin temel depolar Fedora'nın resmî metalink hizmetini kullanır ve RPM imza doğrulaması açıktır:

- `fedora`: `https://mirrors.fedoraproject.org/metalink?repo=fedora-$releasever&arch=$basearch`
- `updates`: `https://mirrors.fedoraproject.org/metalink?repo=updates-released-f$releasever&arch=$basearch`
- `updates-testing`: Fedora 45 Beta tabanında etkin; kararlı Fedora 45 tabanına geçildiğinde kapatılmalıdır.
- Google Chrome, RPM Fusion NVIDIA ve RPM Fusion Steam depo dosyaları ISO'da bulunur fakat varsayılan olarak `enabled=0` durumundadır.

Kurulum eşlemesi:

| Seçenek | DNF5 işlemi | Stok ISO durumu |
|---|---|---|
| SolarUI | İndirme yok | Tek çevrimdışı stok masaüstü |
| Saf Niri | Yeni paket yok; gömülü Niri oturumu görünür yapılır | Çevrimiçi seçim kapısının arkasında |
| GNOME | `gnome-shell`, `gnome-session`, `gnome-control-center`, `gnome-settings-daemon`, `nautilus`, `ptyxis` | ISO'da GNOME Shell yok |
| KDE Plasma | `dnf5 environment install kde-desktop-environment` | Çevrimiçi |
| COSMIC | `dnf5 environment install cosmic-desktop-environment` | Çevrimiçi |
| Hyprland | `hyprland foot wofi waybar` | Çevrimiçi |
| Cinnamon | `dnf5 environment install cinnamon-desktop-environment` | Çevrimiçi |
| XFCE | `dnf5 environment install xfce-desktop-environment` | Çevrimiçi |
| Sway | `dnf5 environment install sway-desktop-environment` | Çevrimiçi |

Bağlantı göstergesi tek başına ağ kartı durumuna güvenmez. Kurulum arayüzü Fedora 45 metalink adresine dört saniyelik HTTPS HEAD isteği gönderir ve bunu 15 saniyede bir tekrarlar. Gerçek depo erişimi yoksa çevrimiçi seçenekler kapalı kalır.

## Önyükleyici kaynakları

- GRUB, shim ve EFI bileşenleri Fedora tabanından gelir.
- Limine 12.9.0 resmî `limine-bootloader/limine` ikili sürümünden sabitlenmiştir.
- Derleme üçüncü taraf COPR açmaz. `blazeos-limine-install` yalnızca yerel `blaze-bootloader` aracına çağrı yapar.
- Limine yalnızca UEFI kurulumunda seçilebilir. Paket içindeki imzasız Limine EFI ikilisi için Secure Boot kapalı olmalıdır.
- Limine yapılandırması fstab kök tanımını olduğu gibi korur; `UUID=LABEL=...` üretilmez.

## Yerel paketler

GNOME Sistem Monitörü ISO'dan çıkarılır. Yerine Fedora 45 tarafından imzalanmış `htop` ve bağımlılığı `hwloc-libs` eklenir. Dosya kökenleri ve SHA-256 değerleri `packages/README.md` içinde sabitlenmiştir. Uygulama menüsündeki Blaze başlatıcısı htop'u Kitty, Ptyxis veya Foot içinde açar.

