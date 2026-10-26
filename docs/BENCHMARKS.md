# BlazeOS Performance Benchmarks & Tuning Telemetry

Bu belge, Blaze SolarEvolution 5.05 ekosisteminin performans ayarlarını, bellek optimizasyonlarını ve kıyaslama (benchmark) metodolojisini açıklar.

## 1. Çekirdek ve Bellek İyileştirmeleri (Tuning Parameters)

| Parametre | Fedora Workstation Varsayılanı | BlazeOS SolarEvolution | Gerekçe / Etki |
| :--- | :--- | :--- | :--- |
| `vm.swappiness` | `60` | `150` | ZRAM sıkıştırılmış RAM takasını disk swap'ından önce agresif ve gecikmesiz kullanma |
| `vm.page-cluster` | `3` (8 sayfa oku) | `0` (Tekil sayfa G/Ç) | ZRAM disk olmadığı için gecikmeli sıralı blok okuması yerine anlık sayfa erişimi |
| `vm.vfs_cache_pressure` | `100` | `50` | Dizin ve dosya öznitelik önbelleklerini (dentry/inode) RAM'de daha uzun süre tutarak dosya gezgini hızını artırma |
| `vm.max_map_count` | `1048576` | `2147483642` | Modern oyun motorları (Unreal Engine 5), Star Citizen, Cyberpunk 2077 ve Wine/Proton bellek haritalama çökmelerini önleme |
| `I/O Scheduler (NVMe)` | `mq-deadline` / değişken | `none` | NVMe donanım kuyruklarına doğrudan passthrough erişim ile CPU genel giderini (overhead) sıfırlama |
| `I/O Scheduler (SATA SSD)`| Değişken | `mq-deadline` | Katı hal sürücülerinde düşük gecikmeli adil sıralama |

## 2. Kıyaslama ve Donanım Telemetri Aracı (`blaze-bench.sh`)

Sistem performans ve bellek durumunu anlık olarak doğrulamak için `scripts/blaze-bench.sh` betiği sunulmuştur:

```bash
# Standart insan tarafından okunabilir çıktı:
./scripts/blaze-bench.sh

# Otomasyon ve CI için JSON formatı:
./scripts/blaze-bench.sh json
```

## 3. Dayanıklılık ve Yük Testi (`solar-torture`)

Masaüstü kabuğu ve kompozitörün kararlılığı, `./blazeos_custom_apps/usr/local/bin/solar-torture` kullanılarak binlerce pencere döngüsü ve olay fırtınası altında test edilir. Bu sayede bellek sızıntıları (leak) ve thread kilitlenmeleri önceden tespit edilir.
