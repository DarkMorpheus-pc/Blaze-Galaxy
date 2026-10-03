// BlazeOS BERP Recovery (OrangeFox GUI) Frontend Controller
// Strict zero emoji rule enforced.

const TRANSLATIONS = {
    tr: {
        status_active: "Kurtarma Modu Aktif",
        nav_install: "Yükle (ISO)",
        nav_backup: "Yedekle",
        nav_restore: "Geri Yükle",
        nav_wipe: "Sıfırla",
        nav_partitions: "Diskler",
        nav_boot: "Önyükleme",
        nav_terminal: "Terminal",
        nav_reboot: "Yeniden Başlat",

        install_title: "USB Olmadan ISO ile Yeniden Kurulum / Güncelleme",
        install_desc: "Sisteminizdeki veya takılı disklerdeki BlazeOS ISO dosyalarını doğrudan belleğe bağlayarak USB belleğe ihtiyaç duymadan kurulumu başlatabilirsiniz.",
        detected_isos: "Tespit Edilen ISO Kalıpları",
        btn_rescan: "Yeniden Tara",
        scanning_isos: "ISO kalıpları taranıyor...",
        no_isos_found: "Sistemde BlazeOS ISO kalıbı bulunamadı. Lütfen Downloads klasörünü veya harici diski kontrol edin.",
        selected_iso_action: "Seçili Kalıp İşlemi",
        slider_iso_desc: "Bu kalıbı bir sonraki açılışta doğrudan başlatmak için kaydırın:",
        slider_iso_label: "Kaydırarak ISO ile Başlat",
        iso_configured_msg: "ISO başlatma hazırlandı. Yeniden başlatılıyor...",

        backup_title: "Sistem ve Kullanıcı Durumu Yedekleme (Btrfs Snapshot)",
        backup_desc: "Btrfs CoW teknolojisi ile diskinizde saniyeler içinde sıfır gecikmeli kurtarma noktası oluşturun.",
        create_snapshot_title: "Yeni Anlık Görüntü (Snapshot) Oluştur",
        backup_name_label: "Yedek Açıklaması / Adı:",
        backup_name_placeholder: "Orn: guncelleme-oncesi-durum",
        backup_root_label: "Sistem Kök Dizini (/)",
        backup_home_label: "Kullanıcı Verileri (/home)",
        btn_create_snapshot: "Anlık Görüntü Al",
        snapshot_creating: "Anlık görüntü alınıyor...",
        snapshot_success: "Kurtarma noktası başarıyla kaydedildi.",

        restore_title: "Sistem Geri Yükleme (Rollback)",
        restore_desc: "Daha önce oluşturulmuş kurtarma noktalarından birine anında geri dönün.",
        restore_points_title: "Mevcut Kurtarma Noktaları",
        btn_refresh_list: "Listeyi Yenile",
        scanning_snapshots: "Yedek noktaları aranıyor...",
        no_snapshots_found: "Henüz kayıtlı bir Btrfs anlık görüntüsü bulunamadı.",
        slider_restore_desc: "Seçili yedeğe geri dönmek için kaydırın:",
        slider_restore_label: "Kaydırarak Geri Yükle",
        restoring_msg: "Geri yükleniyor...",
        restore_success_msg: "Başarılı! Sistem yeniden başlatılıyor...",

        wipe_title: "Temizle ve Fabrika Ayarlarına Sıfırla (Wipe)",
        wipe_desc: "OrangeFox güvenlik mimarisi: Yanlışlıkla silmeyi önleyen sürgü koruması ile önbellekleri temizleyin veya sistemi sıfırlayın.",
        wipe_select_title: "Temizlenecek Alanları Seçin",
        wipe_cache_title: "Sistem ve Paket Önbellekleri (/var/cache)",
        wipe_cache_desc: "DNF, Flatpak ve geçici sistem artıklarını temizler. Kullanıcı dosyalarına dokunmaz.",
        wipe_user_cache_title: "Kullanıcı Önbellekleri (~/.cache)",
        wipe_user_cache_desc: "Tarayıcı ve masaüstü önbelleklerini sıfırlar. Belgeleri ve şifreleri korur.",
        wipe_home_title: "Kullanıcı Hesaplarını ve Verilerini Sıfırla (/home)",
        wipe_home_desc: "Tüm kullanıcı belgeleri ve hesapları silinir. Sistem ilk kurulum sihirbazına (OOBE) döner.",
        wipe_factory_title: "Tam Fabrika Sıfırlaması (Format Data / Root Subvolumes)",
        wipe_factory_desc: "Sistem kökünü ve ayarları tamamen sıfırlar.",
        wipe_hard_reset_title: "Tam Sistem Temizliği (Hard Reset / Wipe System Data - sudo rm -rf /*)",
        wipe_hard_reset_desc: "Tüm kök dosya sistemini siler. BERE kurtarma çekirdeği /boot/bere altında korunur. Güvenlik için 'YES' ve parola onayı gerektirir.",
        wipe_slider_warn: "Dikkat: Seçilen alanlar kalıcı olarak temizlenecektir.",
        wipe_slider_label: "Kaydırarak Temizlemeyi Başlat",
        wipe_running: "Temizleniyor...",
        wipe_running_hard: "Kök Dosya Sistemi Siliniyor (/boot/bere hariç)...",
        wipe_complete: "Temizleme Tamamlandı",
        wipe_success_msg: "Temizleme işlemi başarıyla tamamlandı.",
        wipe_hard_reset_success: "Sistem temizlendi! Kurtarma ortamı yeniden başlatılıyor...",
        wipe_select_warn: "Lütfen temizlenecek en az bir alan seçin.",

        modal_hard_reset_title: "Kritik Güvenlik Doğrulaması",
        modal_hard_reset_msg: "Bu işlem geri alınamaz emin misiniz?",
        modal_hard_reset_submsg: "Sistem kök dizinindeki (rootfs) tüm dosyalar kalıcı olarak silinecektir. Devam etmek için aşağıdaki kutucuğa tam olarak YES yazın ve yetki şifrenizi girin:",
        modal_confirm_label: 'Onaylamak için "YES" yazın:',
        modal_pass_label: "Yetkili Kullanıcı / Root Şifresi:",
        btn_cancel: "İptal",
        btn_confirm_wipe: "Onayla ve Kilidi Aç",
        modal_err_yes: "Onaylamak için kutucuğa tam olarak 'YES' yazmalısınız.",
        modal_err_pass: "Lütfen yetkili kullanıcı / root şifrenizi girin.",

        part_title: "Bölüm ve Disk Yönetimi",
        part_desc: "Bağlama (Mount/Unmount), dosya sistemi onarımı (fsck/scrub) ve disk SMART sağlığı.",
        mounted_parts_title: "Bağlı ve Algılanan Bölümler",
        btn_refresh_parts: "Bölümleri Yenile",
        th_device: "Aygıt",
        th_fs: "Dosya Sistemi",
        th_size: "Boyut",
        th_mount: "Bağlama Noktası",
        th_action: "İşlem",
        fetching_parts: "Disk bilgileri alınıyor...",
        no_parts_found: "Bölüm bulunamadı.",
        btn_mount: "Bağla (Mount)",
        btn_unmount: "Ayır (Unmount)",
        not_mounted: "Bağlı değil",
        smart_title: "Disk Sağlığı (SMART / NVMe Health)",
        analyzing_smart: "Disk sağlık verileri analiz ediliyor...",

        boot_title: "Önyükleme Onarımı (Boot Repair)",
        boot_desc: "GRUB/Limine yapılandırmasını yeniden oluşturun, EFI girişlerini düzeltin ve çekirdek initramfs kalıbını onarın.",
        grub_card_title: "GRUB / Limine Yeniden Yapılandır",
        grub_card_desc: "/etc/grub.d yapılandırmasını okur, BERP ve normal sistem girişlerini yeniden bağlar.",
        btn_repair_grub: "GRUB Yapılandırmasını Onar",
        efi_card_title: "EFI Girişlerini Sıfırla / Onar",
        efi_card_desc: "Anakart UEFI NVRAM tablosuna BlazeOS ve Limine girişlerini yeniden yazar.",
        btn_repair_efi: "EFI NVRAM Onarımı",
        dracut_card_title: "Dracut Initramfs Yeniden Derle",
        dracut_card_desc: "Tüm kurulu çekirdekler için donanım sürücülerini içeren initramfs imajlarını baştan üretir.",
        btn_repair_dracut: "Tüm Initramfs Kalıplarını Yenile (dracut)",
        repair_log_title: "Onarım Günlüğü",
        waiting_op: "İşlem bekleniyor...",

        term_title: "Root Acil Durum Terminali",
        term_desc: "Doğrudan root ayrıcalıklarıyla sistem komutlarını çalıştırın.",
        term_placeholder: "komut girin...",
        btn_send: "Gönder",

        reboot_title: "Yeniden Başlat ve Güç Seçenekleri",
        reboot_desc: "Kurtarma ortamından güvenli çıkış yapın.",
        btn_reboot_sys_title: "Sistemi Başlat",
        btn_reboot_sys_desc: "BlazeOS normal masaüstüne dön",
        btn_reboot_rec_title: "Kurtarmayı Yeniden Başlat",
        btn_reboot_rec_desc: "BERP Recovery oturumunu tazele",
        btn_reboot_uefi_title: "UEFI / BIOS Ayarlarına Git",
        btn_reboot_uefi_desc: "Anakart donanım kurulum ekranına geç",
        btn_poweroff_title: "Bilgisayarı Kapat",
        btn_poweroff_desc: "Güvenli şekilde sistemi durdur",

        confirm_reboot_sys: "BlazeOS normal sistem modunda yeniden başlatılsın mı?",
        confirm_reboot_rec: "Kurtarma ortamı yeniden başlatılsın mı?",
        confirm_reboot_uefi: "UEFI BIOS ayarları ekranına yeniden başlatılsın mı?",
        confirm_poweroff: "Bilgisayar kapatılsın mı?"
    },
    en: {
        status_active: "Recovery Mode Active",
        nav_install: "Install (ISO)",
        nav_backup: "Backup",
        nav_restore: "Restore",
        nav_wipe: "Wipe",
        nav_partitions: "Partitions",
        nav_boot: "Boot Repair",
        nav_terminal: "Terminal",
        nav_reboot: "Reboot",

        install_title: "Reinstall / Upgrade via ISO without USB",
        install_desc: "Directly mount BlazeOS ISO images stored on local or external storage into memory to boot the installer without a USB flash drive.",
        detected_isos: "Detected ISO Images",
        btn_rescan: "Rescan",
        scanning_isos: "Scanning for ISO images...",
        no_isos_found: "No BlazeOS ISO image found on system. Please check Downloads folder or external drive.",
        selected_iso_action: "Selected Image Action",
        slider_iso_desc: "Slide to boot this image on next system startup:",
        slider_iso_label: "Slide to Boot ISO",
        iso_configured_msg: "ISO boot configured. Rebooting...",

        backup_title: "System & User State Backup (Btrfs Snapshot)",
        backup_desc: "Create instantaneous zero-latency recovery restore points using Btrfs Copy-on-Write technology.",
        create_snapshot_title: "Create New Snapshot",
        backup_name_label: "Backup Label / Name:",
        backup_name_placeholder: "e.g. pre-update-state",
        backup_root_label: "System Root Volume (/)",
        backup_home_label: "User Data Volume (/home)",
        btn_create_snapshot: "Take Snapshot",
        snapshot_creating: "Creating snapshot...",
        snapshot_success: "Recovery snapshot saved successfully.",

        restore_title: "System Rollback",
        restore_desc: "Instantly roll back to any previously captured system snapshot point.",
        restore_points_title: "Available Restore Points",
        btn_refresh_list: "Refresh List",
        scanning_snapshots: "Searching for restore points...",
        no_snapshots_found: "No recorded Btrfs snapshots found.",
        slider_restore_desc: "Slide to revert to selected backup:",
        slider_restore_label: "Slide to Restore",
        restoring_msg: "Restoring...",
        restore_success_msg: "Success! Rebooting system...",

        wipe_title: "Wipe & Factory Reset",
        wipe_desc: "OrangeFox security architecture: Slider-guarded wipe controls preventing accidental data loss.",
        wipe_select_title: "Select Partitions / Areas to Clean",
        wipe_cache_title: "System & Package Caches (/var/cache)",
        wipe_cache_desc: "Cleans DNF, Flatpak, and temporary leftovers. Leaves user data intact.",
        wipe_user_cache_title: "User Caches (~/.cache)",
        wipe_user_cache_desc: "Resets browser and desktop cache directories. Preserves personal documents and keys.",
        wipe_home_title: "Reset User Accounts & Data (/home)",
        wipe_home_desc: "Deletes all user documents and accounts. System restarts into first-boot wizard (OOBE).",
        wipe_factory_title: "Full Factory Reset (Format Data / Root Subvolumes)",
        wipe_factory_desc: "Completely wipes root operating system and resets settings to default.",
        wipe_hard_reset_title: "Complete System Wipe (Hard Reset / Wipe System Data - sudo rm -rf /*)",
        wipe_hard_reset_desc: "Wipes all root filesystem trees. BERE emergency kernel is preserved in /boot/bere. Requires typing 'YES' and password confirmation.",
        wipe_slider_warn: "Warning: Selected target areas will be permanently erased.",
        wipe_slider_label: "Slide to Start Wipe",
        wipe_running: "Wiping...",
        wipe_running_hard: "Erasing rootfs (preserving /boot/bere)...",
        wipe_complete: "Wipe Complete",
        wipe_success_msg: "Wipe operation finished successfully.",
        wipe_hard_reset_success: "System wiped! Rebooting into recovery environment...",
        wipe_select_warn: "Please select at least one partition or area to clean.",

        modal_hard_reset_title: "Critical Safety Verification",
        modal_hard_reset_msg: "This action cannot be undone. Are you sure?",
        modal_hard_reset_submsg: "All operating system files across the root directory (rootfs) will be permanently destroyed. To proceed, type YES exactly into the box below and enter your administrative password:",
        modal_confirm_label: 'Type "YES" to confirm:',
        modal_pass_label: "Administrator / Root Password:",
        btn_cancel: "Cancel",
        btn_confirm_wipe: "Authorize and Unlock",
        modal_err_yes: "You must type exactly 'YES' to authorize this action.",
        modal_err_pass: "Please enter your administrator / root password.",

        part_title: "Partition & Disk Management",
        part_desc: "Mount/Unmount controls, filesystem repair (fsck/scrub), and SMART health analysis.",
        mounted_parts_title: "Mounted & Detected Partitions",
        btn_refresh_parts: "Refresh Partitions",
        th_device: "Device",
        th_fs: "Filesystem",
        th_size: "Size",
        th_mount: "Mount Point",
        th_action: "Action",
        fetching_parts: "Querying partition tables...",
        no_parts_found: "No partitions detected.",
        btn_mount: "Mount",
        btn_unmount: "Unmount",
        not_mounted: "Not mounted",
        smart_title: "Drive Health (SMART / NVMe Health)",
        analyzing_smart: "Analyzing drive telemetry...",

        boot_title: "Boot Repair",
        boot_desc: "Rebuild GRUB/Limine configurations, restore EFI NVRAM boot records, and regenerate initramfs.",
        grub_card_title: "Rebuild GRUB / Limine",
        grub_card_desc: "Inspects /etc/grub.d, relinks BERP recovery and standard BlazeOS boot entries.",
        btn_repair_grub: "Repair GRUB Configuration",
        efi_card_title: "Reset / Restore EFI NVRAM",
        efi_card_desc: "Re-registers BlazeOS and recovery entries into motherboard UEFI NVRAM tables.",
        btn_repair_efi: "Repair EFI NVRAM",
        dracut_card_title: "Regenerate Dracut Initramfs",
        dracut_card_desc: "Rebuilds boot initramfs driver images for all installed kernel versions.",
        btn_repair_dracut: "Regenerate All Initramfs (dracut)",
        repair_log_title: "Repair Execution Log",
        waiting_op: "Awaiting operation...",

        term_title: "Root Emergency Terminal",
        term_desc: "Direct root console execution for maintenance commands.",
        term_placeholder: "enter command...",
        btn_send: "Send",

        reboot_title: "Reboot & Power Options",
        reboot_desc: "Safely exit or power cycle the recovery environment.",
        btn_reboot_sys_title: "Boot System",
        btn_reboot_sys_desc: "Return to BlazeOS standard desktop",
        btn_reboot_rec_title: "Reboot Recovery",
        btn_reboot_rec_desc: "Restart active BERP recovery session",
        btn_reboot_uefi_title: "Reboot to UEFI / BIOS",
        btn_reboot_uefi_desc: "Access motherboard setup firmware",
        btn_poweroff_title: "Power Off",
        btn_poweroff_desc: "Safely shut down the computer",

        confirm_reboot_sys: "Reboot computer into normal BlazeOS mode?",
        confirm_reboot_rec: "Restart the recovery environment?",
        confirm_reboot_uefi: "Reboot directly into UEFI BIOS setup?",
        confirm_poweroff: "Shut down the computer now?"
    }
};

let currentLang = "tr";

function t(key) {
    const dict = TRANSLATIONS[currentLang] || TRANSLATIONS.tr;
    return dict[key] || key;
}

function setLanguage(lang) {
    if (!TRANSLATIONS[lang]) lang = "tr";
    currentLang = lang;
    localStorage.setItem("berp_lang", lang);
    document.documentElement.lang = lang;

    const langBtnText = document.getElementById("lang-current");
    if (langBtnText) langBtnText.textContent = lang.toUpperCase();

    // Update data-i18n text
    document.querySelectorAll("[data-i18n]").forEach(el => {
        const key = el.getAttribute("data-i18n");
        if (TRANSLATIONS[lang][key]) {
            el.textContent = TRANSLATIONS[lang][key];
        }
    });

    // Update data-i18n-placeholder
    document.querySelectorAll("[data-i18n-placeholder]").forEach(el => {
        const key = el.getAttribute("data-i18n-placeholder");
        if (TRANSLATIONS[lang][key]) {
            el.placeholder = TRANSLATIONS[lang][key];
        }
    });
}

function initLanguage() {
    const saved = localStorage.getItem("berp_lang");
    if (saved && TRANSLATIONS[saved]) {
        currentLang = saved;
    }
    setLanguage(currentLang);

    const toggleBtn = document.getElementById("btn-lang-toggle");
    if (toggleBtn) {
        toggleBtn.addEventListener("click", () => {
            const nextLang = currentLang === "tr" ? "en" : "tr";
            setLanguage(nextLang);
        });
    }
}

// REST API Helper
async function apiCall(endpoint, data = null) {
    try {
        const options = {
            method: data ? "POST" : "GET",
            headers: { "Content-Type": "application/json" }
        };
        if (data) options.body = JSON.stringify(data);
        const res = await fetch("/api/" + endpoint, options);
        return await res.json();
    } catch (e) {
        console.error("API call error:", e);
        return { status: "error", message: e.toString() };
    }
}

// 1. Tab Navigation
const TAB_ORDER = [
    "tab-install",
    "tab-backup",
    "tab-restore",
    "tab-wipe",
    "tab-partitions",
    "tab-boot",
    "tab-terminal",
    "tab-reboot"
];

function switchTab(target) {
    const navItems = document.querySelectorAll(".nav-item");
    const tabPanes = document.querySelectorAll(".tab-pane");

    navItems.forEach(n => {
        if (n.getAttribute("data-tab") === target) {
            n.classList.add("active");
        } else {
            n.classList.remove("active");
        }
    });

    tabPanes.forEach(p => {
        if (p.id === target) {
            p.classList.add("active");
        } else {
            p.classList.remove("active");
        }
    });

    if (target === "tab-install") loadIsos();
    if (target === "tab-restore") loadSnapshots();
    if (target === "tab-partitions") loadPartitions();
}

function initTabs() {
    const navItems = document.querySelectorAll(".nav-item");
    navItems.forEach(btn => {
        btn.addEventListener("click", () => {
            const target = btn.getAttribute("data-tab");
            switchTab(target);
        });
    });

    window.addEventListener("keydown", (e) => {
        const isInput = e.target && (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA");
        if (isInput) return;

        // Number keys 1-8
        const num = parseInt(e.key, 10);
        if (num >= 1 && num <= TAB_ORDER.length) {
            e.preventDefault();
            switchTab(TAB_ORDER[num - 1]);
            return;
        }

        // Arrow keys Up / Down
        const activeNav = document.querySelector(".nav-item.active");
        const currentTarget = activeNav ? activeNav.getAttribute("data-tab") : TAB_ORDER[0];
        const curIdx = TAB_ORDER.indexOf(currentTarget);

        if (e.key === "ArrowUp") {
            e.preventDefault();
            const prevIdx = (curIdx - 1 + TAB_ORDER.length) % TAB_ORDER.length;
            switchTab(TAB_ORDER[prevIdx]);
        } else if (e.key === "ArrowDown") {
            e.preventDefault();
            const nextIdx = (curIdx + 1) % TAB_ORDER.length;
            switchTab(TAB_ORDER[nextIdx]);
        }
    });
}

// 2. Hardware Telemetry & Clock
function initTelemetry() {
    const clockEl = document.getElementById("clock-val");
    const cpuEl = document.getElementById("cpu-val");
    const ramEl = document.getElementById("ram-val");

    function updateClock() {
        const d = new Date();
        const hh = String(d.getHours()).padStart(2, "0");
        const mm = String(d.getMinutes()).padStart(2, "0");
        if (clockEl) clockEl.textContent = `${hh}:${mm}`;
    }
    updateClock();
    setInterval(updateClock, 1000);

    async function pollTelemetry() {
        const res = await apiCall("telemetry");
        if (res.status === "ok") {
            if (cpuEl) cpuEl.textContent = res.cpu + "%";
            if (ramEl) ramEl.textContent = res.ram + "%";
        }
    }
    pollTelemetry();
    setInterval(pollTelemetry, 3000);
}

// 3. OrangeFox Touch/Mouse/Keyboard Slider Engine
function initSliders() {
    setupSlider("slider-iso-boot", onConfirmIsoBoot);
    setupSlider("slider-restore", onConfirmRestore);
    setupSlider("slider-wipe", onConfirmWipe);
}

function setupSlider(sliderId, onComplete) {
    const slider = document.getElementById(sliderId);
    if (!slider) return;

    const thumb = slider.querySelector(".slider-thumb");
    const fill = slider.querySelector(".slider-fill");

    let isDragging = false;
    let startX = 0;
    let currentX = 0;
    let maxDist = 0;

    slider.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " " || e.key === "ArrowRight") {
            e.preventDefault();
            const max = slider.offsetWidth - thumb.offsetWidth - 8;
            thumb.style.transition = "transform 0.3s ease";
            fill.style.transition = "width 0.3s ease";
            thumb.style.transform = `translateX(${max}px)`;
            fill.style.width = "100%";
            setTimeout(() => {
                thumb.style.transition = "";
                fill.style.transition = "";
                onComplete(slider);
            }, 350);
        }
    });

    function onPointerDown(e) {
        isDragging = true;
        startX = e.clientX || (e.touches && e.touches[0].clientX);
        maxDist = slider.offsetWidth - thumb.offsetWidth - 8;
        if (slider.setPointerCapture && e.pointerId) {
            try { slider.setPointerCapture(e.pointerId); } catch(err) {}
        }
    }

    function onPointerMove(e) {
        if (!isDragging) return;
        const clientX = e.clientX || (e.touches && e.touches[0].clientX);
        let delta = clientX - startX;
        delta = Math.max(0, Math.min(delta, maxDist));
        currentX = delta;

        thumb.style.transform = `translateX(${delta}px)`;
        const pct = (delta / maxDist) * 100;
        fill.style.width = `${pct}%`;

        if (delta >= maxDist - 5) {
            isDragging = false;
            thumb.style.transform = `translateX(${maxDist}px)`;
            fill.style.width = "100%";
            onComplete(slider);
        }
    }

    function onPointerUp() {
        if (!isDragging) return;
        isDragging = false;
        thumb.style.transition = "transform 0.2s ease";
        fill.style.transition = "width 0.2s ease";
        thumb.style.transform = "translateX(0px)";
        fill.style.width = "0%";
        setTimeout(() => {
            thumb.style.transition = "";
            fill.style.transition = "";
        }, 200);
    }

    slider.addEventListener("mousedown", onPointerDown);
    window.addEventListener("mousemove", onPointerMove);
    window.addEventListener("mouseup", onPointerUp);

    slider.addEventListener("touchstart", onPointerDown, { passive: true });
    window.addEventListener("touchmove", onPointerMove, { passive: true });
    window.addEventListener("touchend", onPointerUp);
}

function resetSlider(slider) {
    if (!slider) return;
    const thumb = slider.querySelector(".slider-thumb");
    const fill = slider.querySelector(".slider-fill");
    thumb.style.transition = "transform 0.3s ease";
    fill.style.transition = "width 0.3s ease";
    thumb.style.transform = "translateX(0px)";
    fill.style.width = "0%";
    setTimeout(() => {
        thumb.style.transition = "";
        fill.style.transition = "";
    }, 300);
}

// 4. ISO Install / USB-less direct boot
let selectedIsoPath = null;

function initIsos() {
    const btn = document.getElementById("btn-refresh-isos");
    if (btn) btn.addEventListener("click", loadIsos);
    loadIsos();
}

async function loadIsos() {
    const container = document.getElementById("iso-list");
    if (!container) return;
    container.innerHTML = `<div class='empty-hint'>${escapeHtml(t("scanning_isos"))}</div>`;
    const res = await apiCall("isos");

    if (!res.isos || res.isos.length === 0) {
        container.innerHTML = `<div class='empty-hint'>${escapeHtml(t("no_isos_found"))}</div>`;
        const actCard = document.getElementById("iso-action-card");
        if (actCard) actCard.style.display = "none";
        return;
    }

    container.innerHTML = "";
    res.isos.forEach(iso => {
        const item = document.createElement("div");
        item.className = "list-row-item";
        item.innerHTML = `
            <div class="item-left">
                <span class="item-title">${escapeHtml(iso.name)}</span>
                <span class="item-meta">${iso.size_mb} MB | ${escapeHtml(iso.path)}</span>
            </div>
            <button class="fox-btn outline-btn">${currentLang === "tr" ? "Seç" : "Select"}</button>
        `;
        item.addEventListener("click", () => {
            document.querySelectorAll("#iso-list .list-row-item").forEach(r => r.classList.remove("selected"));
            item.classList.add("selected");
            selectedIsoPath = iso.path;

            const metaEl = document.getElementById("selected-iso-meta");
            if (metaEl) {
                metaEl.innerHTML = `<strong>${escapeHtml(iso.name)}</strong> (${iso.size_mb} MB)<br><small>${escapeHtml(iso.path)}</small>`;
            }
            const actCard = document.getElementById("iso-action-card");
            if (actCard) actCard.style.display = "block";
        });
        container.appendChild(item);
    });
}

async function onConfirmIsoBoot(slider) {
    if (!selectedIsoPath) {
        alert(currentLang === "tr" ? "Lütfen bir ISO dosyası seçin." : "Please select an ISO file.");
        resetSlider(slider);
        return;
    }

    const label = slider.querySelector(".slider-label");
    label.textContent = t("iso_configured_msg");

    const res = await apiCall("iso_boot", { path: selectedIsoPath });
    if (res.status === "ok") {
        setTimeout(() => {
            apiCall("reboot", { target: "system" });
        }, 1500);
    } else {
        alert("Hata: " + res.message);
        label.textContent = t("slider_iso_label");
        resetSlider(slider);
    }
}

// 5. Backup / Snapshot Management
let selectedSnapshot = null;

function initSnapshots() {
    const btnCreate = document.getElementById("btn-create-snapshot");
    const statusEl = document.getElementById("backup-status-text");

    if (btnCreate) {
        btnCreate.addEventListener("click", async () => {
            const name = document.getElementById("backup-name").value.trim();
            const root = document.getElementById("chk-backup-root").checked;
            const home = document.getElementById("chk-backup-home").checked;

            btnCreate.disabled = true;
            statusEl.textContent = t("snapshot_creating");

            const res = await apiCall("create_snapshot", { name, root, home });
            btnCreate.disabled = false;

            if (res.status === "ok") {
                statusEl.textContent = t("snapshot_success");
                document.getElementById("backup-name").value = "";
                loadSnapshots();
            } else {
                statusEl.textContent = "Hata: " + res.message;
            }
        });
    }

    const btnRefresh = document.getElementById("btn-refresh-snapshots");
    if (btnRefresh) btnRefresh.addEventListener("click", loadSnapshots);
    loadSnapshots();
}

async function loadSnapshots() {
    const container = document.getElementById("snapshot-list");
    if (!container) return;
    container.innerHTML = `<div class='empty-hint'>${escapeHtml(t("scanning_snapshots"))}</div>`;
    const res = await apiCall("snapshots");

    if (!res.snapshots || res.snapshots.length === 0) {
        container.innerHTML = `<div class='empty-hint'>${escapeHtml(t("no_snapshots_found"))}</div>`;
        const sliderBox = document.getElementById("restore-slider-box");
        if (sliderBox) sliderBox.style.display = "none";
        return;
    }

    container.innerHTML = "";
    res.snapshots.forEach(s => {
        const row = document.createElement("div");
        row.className = "list-row-item";
        row.innerHTML = `
            <div class="item-left">
                <span class="item-title">${escapeHtml(s.name)}</span>
                <span class="item-meta">${currentLang === "tr" ? "Tarih" : "Date"}: ${escapeHtml(s.date)} | ${escapeHtml(s.path)}</span>
            </div>
            <button class="fox-btn outline-btn">${currentLang === "tr" ? "Seç" : "Select"}</button>
        `;
        row.addEventListener("click", () => {
            document.querySelectorAll("#snapshot-list .list-row-item").forEach(r => r.classList.remove("selected"));
            row.classList.add("selected");
            selectedSnapshot = s;
            const targetLabel = document.getElementById("restore-target-label");
            if (targetLabel) {
                targetLabel.textContent = `"${s.name}" ${currentLang === "tr" ? "yedeğine geri dönmek için kaydırın:" : "snapshot restore slide:"}`;
            }
            const sliderBox = document.getElementById("restore-slider-box");
            if (sliderBox) sliderBox.style.display = "block";
        });
        container.appendChild(row);
    });
}

async function onConfirmRestore(slider) {
    if (!selectedSnapshot) {
        alert(currentLang === "tr" ? "Lütfen geri yüklenecek bir yedek seçin." : "Please select a backup to restore.");
        resetSlider(slider);
        return;
    }

    const label = slider.querySelector(".slider-label");
    label.textContent = t("restoring_msg");

    const res = await apiCall("restore_snapshot", { snapshot: selectedSnapshot.path });
    if (res.status === "ok") {
        label.textContent = t("restore_success_msg");
        setTimeout(() => {
            apiCall("reboot", { target: "system" });
        }, 1500);
    } else {
        alert("Geri yükleme hatası: " + res.message);
        label.textContent = t("slider_restore_label");
        resetSlider(slider);
    }
}

// 6. Wipe & Hard Reset Logic
let hardResetVerified = false;

function initHardResetModal() {
    const chkHardReset = document.getElementById("wipe-hard-reset");
    const modal = document.getElementById("modal-hard-reset");
    const btnClose = document.getElementById("btn-modal-close");
    const btnCancel = document.getElementById("btn-cancel-hard-reset");
    const btnConfirm = document.getElementById("btn-confirm-hard-reset");
    const inputConfirm = document.getElementById("input-hard-reset-confirm");
    const inputPass = document.getElementById("input-hard-reset-pass");
    const errorEl = document.getElementById("modal-error-msg");

    function openModal() {
        if (!modal) return;
        inputConfirm.value = "";
        inputPass.value = "";
        errorEl.style.display = "none";
        errorEl.textContent = "";
        modal.style.display = "flex";
        setTimeout(() => inputConfirm.focus(), 100);
    }

    function closeModal() {
        if (!modal) return;
        modal.style.display = "none";
        if (!hardResetVerified && chkHardReset) {
            chkHardReset.checked = false;
        }
    }

    if (chkHardReset) {
        chkHardReset.addEventListener("click", (e) => {
            if (chkHardReset.checked && !hardResetVerified) {
                e.preventDefault();
                chkHardReset.checked = false;
                openModal();
            } else if (!chkHardReset.checked) {
                hardResetVerified = false;
            }
        });
    }

    if (btnClose) btnClose.addEventListener("click", closeModal);
    if (btnCancel) btnCancel.addEventListener("click", closeModal);

    if (btnConfirm) {
        btnConfirm.addEventListener("click", () => {
            const confirmVal = inputConfirm.value.trim();
            const passVal = inputPass.value.trim();

            if (confirmVal !== "YES") {
                errorEl.textContent = t("modal_err_yes");
                errorEl.style.display = "block";
                inputConfirm.focus();
                return;
            }

            if (passVal.length === 0) {
                errorEl.textContent = t("modal_err_pass");
                errorEl.style.display = "block";
                inputPass.focus();
                return;
            }

            // Verification successful
            hardResetVerified = true;
            if (chkHardReset) chkHardReset.checked = true;
            closeModal();
        });
    }
}

async function onConfirmWipe(slider) {
    const wipeCache = document.getElementById("wipe-cache").checked;
    const wipeUserCache = document.getElementById("wipe-user-cache").checked;
    const wipeHome = document.getElementById("wipe-home").checked;
    const wipeFactory = document.getElementById("wipe-factory").checked;
    const wipeHardReset = document.getElementById("wipe-hard-reset") ? document.getElementById("wipe-hard-reset").checked : false;
    const statusEl = document.getElementById("wipe-status-text");

    if (!wipeCache && !wipeUserCache && !wipeHome && !wipeFactory && !wipeHardReset) {
        alert(t("wipe_select_warn"));
        resetSlider(slider);
        return;
    }

    const label = slider.querySelector(".slider-label");
    if (wipeHardReset) {
        label.textContent = t("wipe_running_hard");
        statusEl.textContent = t("wipe_running_hard");
    } else {
        label.textContent = t("wipe_running");
        statusEl.textContent = t("wipe_running");
    }

    const res = await apiCall("wipe", {
        system_cache: wipeCache,
        user_cache: wipeUserCache,
        home: wipeHome,
        factory: wipeFactory,
        hard_reset: wipeHardReset,
        lang: currentLang
    });

    if (res.status === "ok") {
        if (wipeHardReset) {
            label.textContent = t("wipe_hard_reset_success");
            statusEl.textContent = t("wipe_hard_reset_success");
        } else {
            label.textContent = t("wipe_complete");
            statusEl.textContent = t("wipe_success_msg");
            setTimeout(() => {
                resetSlider(slider);
                label.textContent = t("wipe_slider_label");
            }, 2500);
        }
    } else {
        alert("Hata: " + res.message);
        label.textContent = t("wipe_slider_label");
        statusEl.textContent = "Hata: " + res.message;
        resetSlider(slider);
    }
}

// 7. Partitions & SMART
function initPartitions() {
    const btn = document.getElementById("btn-refresh-parts");
    if (btn) btn.addEventListener("click", loadPartitions);
    loadPartitions();
}

async function loadPartitions() {
    const tbody = document.querySelector("#parts-table tbody");
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan='5' class='text-center'>${escapeHtml(t("fetching_parts"))}</td></tr>`;

    const res = await apiCall("partitions");
    if (!res.partitions || res.partitions.length === 0) {
        tbody.innerHTML = `<tr><td colspan='5' class='text-center'>${escapeHtml(t("no_parts_found"))}</td></tr>`;
        return;
    }

    tbody.innerHTML = "";
    res.partitions.forEach(p => {
        const tr = document.createElement("tr");
        const isMounted = p.mountpoint && p.mountpoint.length > 0;
        tr.innerHTML = `
            <td><strong>${escapeHtml(p.name)}</strong></td>
            <td>${escapeHtml(p.fstype || "-")}</td>
            <td>${escapeHtml(p.size || "-")}</td>
            <td>${escapeHtml(p.mountpoint || t("not_mounted"))}</td>
            <td>
                <button class="fox-btn outline-btn part-act-btn" data-dev="${p.name}" data-mounted="${isMounted}">
                    ${isMounted ? t("btn_unmount") : t("btn_mount")}
                </button>
            </td>
        `;
        const actBtn = tr.querySelector(".part-act-btn");
        actBtn.addEventListener("click", async () => {
            const dev = actBtn.getAttribute("data-dev");
            const mounted = actBtn.getAttribute("data-mounted") === "true";
            actBtn.disabled = true;
            actBtn.textContent = currentLang === "tr" ? "İşleniyor..." : "Processing...";
            await apiCall("mount_toggle", { device: dev, action: mounted ? "unmount" : "mount" });
            loadPartitions();
        });
        tbody.appendChild(tr);
    });

    const smartBox = document.getElementById("smart-summary");
    if (smartBox) {
        smartBox.textContent = res.smart_info || (currentLang === "tr" ? "SMART sağlık verisi alınamadı veya NVMe tanılama desteklenmiyor." : "No SMART data available or NVMe telemetry unsupported.");
    }
}

// 8. Boot Repair
function initBootRepair() {
    const consoleEl = document.getElementById("boot-repair-console");

    const btnGrub = document.getElementById("btn-repair-grub");
    if (btnGrub) {
        btnGrub.addEventListener("click", async () => {
            consoleEl.textContent = (currentLang === "tr" ? "GRUB yapılandırması onarılıyor (grub2-mkconfig)...\n" : "Rebuilding GRUB configuration (grub2-mkconfig)...\n");
            const res = await apiCall("boot_repair", { target: "grub" });
            consoleEl.textContent += (res.output || res.message) + "\n" + (currentLang === "tr" ? "İşlem tamamlandı." : "Operation complete.");
        });
    }

    const btnEfi = document.getElementById("btn-repair-efi");
    if (btnEfi) {
        btnEfi.addEventListener("click", async () => {
            consoleEl.textContent = (currentLang === "tr" ? "EFI NVRAM girişleri kontrol ediliyor ve onarılıyor (efibootmgr)...\n" : "Repairing EFI NVRAM boot entries (efibootmgr)...\n");
            const res = await apiCall("boot_repair", { target: "efi" });
            consoleEl.textContent += (res.output || res.message) + "\n" + (currentLang === "tr" ? "İşlem tamamlandı." : "Operation complete.");
        });
    }

    const btnDracut = document.getElementById("btn-repair-dracut");
    if (btnDracut) {
        btnDracut.addEventListener("click", async () => {
            consoleEl.textContent = (currentLang === "tr" ? "Dracut ile tüm initramfs kalıpları yeniden üretiliyor (Bu işlem 1-2 dakika sürebilir)...\n" : "Regenerating all initramfs images via dracut (may take 1-2 minutes)...\n");
            const res = await apiCall("boot_repair", { target: "dracut" });
            consoleEl.textContent += (res.output || res.message) + "\n" + (currentLang === "tr" ? "İşlem tamamlandı." : "Operation complete.");
        });
    }
}

// 9. Root Terminal
function initTerminal() {
    const screen = document.getElementById("term-output");
    const form = document.getElementById("term-form");
    const input = document.getElementById("term-input");

    if (form) {
        form.addEventListener("submit", async (e) => {
            e.preventDefault();
            const cmd = input.value.trim();
            if (!cmd) return;

            screen.textContent += cmd + "\n";
            input.value = "";
            screen.scrollTop = screen.scrollHeight;

            const res = await apiCall("exec", { command: cmd });
            if (res.output) {
                screen.textContent += res.output + "\n# ";
            } else if (res.error) {
                screen.textContent += "[HATA] " + res.error + "\n# ";
            } else {
                screen.textContent += "# ";
            }
            screen.scrollTop = screen.scrollHeight;
        });
    }
}

// 10. Reboot Handlers
function initReboot() {
    const btnSys = document.getElementById("btn-reboot-system");
    if (btnSys) {
        btnSys.addEventListener("click", () => {
            if (confirm(t("confirm_reboot_sys"))) {
                apiCall("reboot", { target: "system" });
            }
        });
    }

    const btnRec = document.getElementById("btn-reboot-recovery");
    if (btnRec) {
        btnRec.addEventListener("click", () => {
            if (confirm(t("confirm_reboot_rec"))) {
                apiCall("reboot", { target: "recovery" });
            }
        });
    }

    const btnUefi = document.getElementById("btn-reboot-uefi");
    if (btnUefi) {
        btnUefi.addEventListener("click", () => {
            if (confirm(t("confirm_reboot_uefi"))) {
                apiCall("reboot", { target: "uefi" });
            }
        });
    }

    const btnOff = document.getElementById("btn-poweroff");
    if (btnOff) {
        btnOff.addEventListener("click", () => {
            if (confirm(t("confirm_poweroff"))) {
                apiCall("reboot", { target: "poweroff" });
            }
        });
    }
}

function escapeHtml(text) {
    if (!text) return "";
    return String(text).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

document.addEventListener("DOMContentLoaded", () => {
    initLanguage();
    initTabs();
    initTelemetry();
    initSliders();
    initIsos();
    initSnapshots();
    initPartitions();
    initBootRepair();
    initTerminal();
    initReboot();
    initHardResetModal();
});
