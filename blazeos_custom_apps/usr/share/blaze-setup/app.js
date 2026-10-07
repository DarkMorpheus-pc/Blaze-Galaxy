/* ==========================================================================
   BlazeOS Apple-Style "Hello" Setup Assistant Logic
   Strict Zero Emojis • Native macOS Monterey / Sonoma Design Language
   Web Animations API Cursive Handwriting • System-wide Appearance
   ========================================================================== */

document.addEventListener("DOMContentLoaded", () => {
    // ── 1. State Management & Translations ────────────────────────────────
    let currentWordIndex = 0;
    let isWizardActive = false;
    let currentSlide = 0;
    const totalSlides = 5;
    let wordTimer = null;
    let currentLang = "tr";

    const OOBE_I18N = {
        tr: {
            slide0_title: "Dil",
            slide0_subtitle: "Ana dili seçin. Bu ayarı daha sonra Sistem Ayarları'ndan değiştirebilirsiniz.",
            kb_tr_desc: "Q Klavye Düzeni",
            kb_trf_desc: "F Klavye Düzeni",
            kb_en_desc: "US Standard QWERTY",
            kb_de_desc: "Standard QWERTZ",
            kb_fr_desc: "Standard AZERTY",
            kb_test_label: "Klavye Sınaması:",
            kb_test_ph: "Tuşlarınızı burada deneyin: ğüşıöç 123 !?@",
            kb_applied_ph: "Klavye ({0}) uygulandı. Deneyin: ğüşıöç 123 !?@",
            slide1_title: "Ağ Bağlantısı",
            slide1_subtitle: "Sistem paket güncellemelerini ve donanım sürücülerini doğrulamak için ağınızı kontrol edin.",
            net_scanning: "Ağ arabirimleri taranıyor...",
            net_offline_title: "İnternet bağlantısı olmadan sürdür",
            net_offline_desc: "Kurulumu tamamen yerel ve çevrimdışı olarak tamamlayın. Ağ yapılandırmasını masaüstünden de yapabilirsiniz.",
            btn_refresh_network: "Ağ Durumunu Yenile",
            net_no_active: "Aktif Ağ Arabirimi Yok",
            net_no_active_desc: "Kablo veya kablosuz donanım algılanmadı",
            net_offline_badge: "Çevrimdışı",
            net_wifi_label: "Kablosuz (Wi-Fi)",
            net_eth_label: "Kablolu Bağlantı (Ethernet)",
            net_connected_badge: "Bağlandı",
            net_disconnected_badge: "Bağlantı Yok",
            net_scan_error: "Ağ Taraması Tamamlanamadı",
            net_scan_error_desc: "Lütfen donanım durumunu kontrol edin",
            net_error_badge: "Hata",
            slide2_title: "Bilgisayar Hesabı Yaratın",
            slide2_subtitle: "Bilgisayar hesabınızı yaratmak için aşağıdaki bilgileri doldurun.",
            field_fullname: "Tam ad:",
            ph_fullname: "Ad ve Soyad",
            field_username: "Hesap adı:",
            ph_username: "kullanıcı adı",
            hint_username: "Bu, ana klasörünüzün adı olacaktır.",
            field_hostname: "Bilgisayar adı:",
            hint_hostname: "Ağda ve terminalde görünecek aygıt adı.",
            field_password: "Parola:",
            ph_password: "yeni parola",
            ph_password_confirm: "doğrula",
            hint_pass_match: "Parolalar eşleşiyor.",
            hint_pass_mismatch: "Parolalar eşleşmiyor!",
            hint_pass_set: "Parola belirleyin.",
            field_hint: "İpucu:",
            ph_hint: "isteğe bağlı",
            avatar_label: "Profil Resmi",
            chk_admin_label: "Bu kullanıcının bu bilgisayarı yönetmesine izin ver (sudo yetkisi)",
            chk_autologin_label: "Açılışta otomatik giriş yap",
            slide3_title: "Masaüstü Kabuğu ve Görünüş",
            slide3_subtitle: "BlazeOS masaüstü deneyiminizi ve sistem renk temasını seçin.",
            shell_section_label: "Masaüstü Kabuğu:",
            badge_recommended: "ÖNERİLEN",
            shell_noctalia_desc: "Hafif, ultra akıcı ve modern stok Wayland masaüstü deneyimi. Kararlı performans ve düşük bellek tüketimi.",
            shell_caelestia_desc: "Zengin animasyonlar, widget'lar ve derin özelleştirilebilir QML arayüz bileşenleri.",
            color_mode_label: "Renk Modu:",
            mode_light: "Açık",
            mode_dark: "Koyu",
            mode_auto: "Otomatik",
            appearance_info: "Seçtiğiniz kabuk ve renk modu SolarUI, GTK/Qt uygulamalarını ve SDDM giriş ekranını kapsayacak şekilde tüm sisteme uygulanır.",
            slide4_title: "BlazeOS Hazırlanıyor",
            slide4_subtitle: "Ayarlarınız kaydediliyor ve masaüstü ortamınız hazırlanıyor. Lütfen bekleyin...",
            task_keymap: "Dil ve klavye düzeni işleniyor",
            task_network: "Ağ ve sistem zamanı yapılandırılıyor",
            task_user: "Kullanıcı hesabı ve parola şifrelemesi kaydediliyor",
            task_appearance: "Masaüstü kabuğu ve sistem teması yapılandırılıyor",
            task_finalize: "OOBE tamamlanıyor ve oturum hazır hale getiriliyor",
            badge_pending: "[BEKLİYOR]",
            badge_active: "[İŞLENİYOR]",
            badge_done: "[TAMAMLANDI]",
            badge_error: "[HATA]",
            btn_back: "Geri",
            btn_next: "Sürdür",
            btn_finish: "Tamamla",
            btn_start: "Başlat",
            btn_launch_desktop: "Masaüstünü Başlat",
            alert_invalid_user: "Lütfen geçerli bir hesap adı girin.",
            alert_pass_mismatch: "Girdiğiniz parolalar birbiriyle eşleşmiyor. Lütfen kontrol edin."
        },
        en: {
            slide0_title: "Language",
            slide0_subtitle: "Select your primary language. You can change this later in System Settings.",
            kb_tr_desc: "Turkish Q Layout",
            kb_trf_desc: "Turkish F Layout",
            kb_en_desc: "US Standard QWERTY",
            kb_de_desc: "Standard QWERTZ",
            kb_fr_desc: "Standard AZERTY",
            kb_test_label: "Keyboard Test:",
            kb_test_ph: "Test your keys here: abc 123 !?@",
            kb_applied_ph: "Keyboard ({0}) applied. Try here: abc 123 !?@",
            slide1_title: "Network Connection",
            slide1_subtitle: "Check network connectivity to verify package updates and hardware drivers.",
            net_scanning: "Scanning network interfaces...",
            net_offline_title: "Continue without internet",
            net_offline_desc: "Complete setup completely local and offline. You can configure network later.",
            btn_refresh_network: "Refresh Network Status",
            net_no_active: "No Active Network Interface",
            net_no_active_desc: "No wired or wireless network detected",
            net_offline_badge: "Offline",
            net_wifi_label: "Wireless (Wi-Fi)",
            net_eth_label: "Wired Connection (Ethernet)",
            net_connected_badge: "Connected",
            net_disconnected_badge: "No Connection",
            net_scan_error: "Network Scan Failed",
            net_scan_error_desc: "Please check your network hardware",
            net_error_badge: "Error",
            slide2_title: "Create Computer Account",
            slide2_subtitle: "Fill in the details below to create your computer user account.",
            field_fullname: "Full name:",
            ph_fullname: "Full Name",
            field_username: "Account name:",
            ph_username: "username",
            hint_username: "This will be the name of your home folder.",
            field_hostname: "Computer name:",
            hint_hostname: "Device name displayed on local network and terminal.",
            field_password: "Password:",
            ph_password: "new password",
            ph_password_confirm: "verify",
            hint_pass_match: "Passwords match.",
            hint_pass_mismatch: "Passwords do not match!",
            hint_pass_set: "Set a password.",
            field_hint: "Hint:",
            ph_hint: "optional",
            avatar_label: "Profile Picture",
            chk_admin_label: "Allow this user to administer this computer (sudo privileges)",
            chk_autologin_label: "Log in automatically on boot",
            slide3_title: "Desktop Shell & Appearance",
            slide3_subtitle: "Select your BlazeOS desktop experience and system color palette.",
            shell_section_label: "Desktop Shell:",
            badge_recommended: "RECOMMENDED",
            shell_noctalia_desc: "Lightweight, ultra-smooth modern stock Wayland shell. Stable performance and low memory consumption.",
            shell_caelestia_desc: "Rich animations, interactive widgets, and deep customizable QML interface components.",
            color_mode_label: "Color Mode:",
            mode_light: "Light",
            mode_dark: "Dark",
            mode_auto: "Auto",
            appearance_info: "Your chosen shell and color mode will be applied across SolarUI, GTK/Qt applications and SDDM login.",
            slide4_title: "Setting Up BlazeOS",
            slide4_subtitle: "Your settings are being applied and your desktop is being prepared. Please wait...",
            task_keymap: "Configuring language and keyboard layout",
            task_network: "Configuring network and system clock",
            task_user: "Creating user account and password encryption",
            task_appearance: "Configuring desktop shell and system theme",
            task_finalize: "Finalizing OOBE and preparing desktop session",
            badge_pending: "[PENDING]",
            badge_active: "[IN PROGRESS]",
            badge_done: "[COMPLETED]",
            badge_error: "[ERROR]",
            btn_back: "Back",
            btn_next: "Continue",
            btn_finish: "Complete",
            btn_start: "Start",
            btn_launch_desktop: "Launch Desktop",
            alert_invalid_user: "Please enter a valid account name.",
            alert_pass_mismatch: "The passwords you entered do not match. Please verify."
        },
        de: {
            slide0_title: "Sprache",
            slide0_subtitle: "Wählen Sie Ihre Hauptsprache. Sie können dies später ändern.",
            kb_tr_desc: "Türkisches Q-Layout",
            kb_trf_desc: "Türkisches F-Layout",
            kb_en_desc: "US-Standard QWERTY",
            kb_de_desc: "Standard QWERTZ",
            kb_fr_desc: "Standard AZERTY",
            kb_test_label: "Tastaturtest:",
            kb_test_ph: "Testen Sie hier: äöüß 123 !?@",
            kb_applied_ph: "Tastatur ({0}) angewendet. Testen: äöüß 123 !?@",
            slide1_title: "Netzwerkverbindung",
            slide1_subtitle: "Überprüfen Sie die Verbindung für Updates und Treiber.",
            net_scanning: "Netzwerkschnittstellen werden gesucht...",
            net_offline_title: "Ohne Internet fortfahren",
            net_offline_desc: "Einrichtung lokal und offline abschließen.",
            btn_refresh_network: "Netzwerk aktualisieren",
            net_no_active: "Keine aktive Schnittstelle",
            net_no_active_desc: "Keine Verbindung erkannt",
            net_offline_badge: "Offline",
            net_wifi_label: "WLAN (Wi-Fi)",
            net_eth_label: "Kabelgebunden (Ethernet)",
            net_connected_badge: "Verbunden",
            net_disconnected_badge: "Nicht verbunden",
            net_scan_error: "Scan fehlgeschlagen",
            net_scan_error_desc: "Bitte Hardware überprüfen",
            net_error_badge: "Fehler",
            slide2_title: "Benutzerkonto erstellen",
            slide2_subtitle: "Geben Sie die Daten für Ihr Benutzerkonto ein.",
            field_fullname: "Vollständiger Name:",
            ph_fullname: "Vor- und Nachname",
            field_username: "Kontoname:",
            ph_username: "benutzername",
            hint_username: "Dies wird der Name Ihres Home-Ordners.",
            field_hostname: "Computername:",
            hint_hostname: "Im Netzwerk und Terminal sichtbarer Name.",
            field_password: "Passwort:",
            ph_password: "neues passwort",
            ph_password_confirm: "bestätigen",
            hint_pass_match: "Passwörter stimmen überein.",
            hint_pass_mismatch: "Passwörter stimmen nicht überein!",
            hint_pass_set: "Passwort festlegen.",
            field_hint: "Hinweis:",
            ph_hint: "optional",
            avatar_label: "Profilbild",
            chk_admin_label: "Diesem Benutzer Administratorrechte erteilen (sudo)",
            chk_autologin_label: "Beim Hochfahren automatisch anmelden",
            slide3_title: "Desktop-Oberfläche & Erscheinungsbild",
            slide3_subtitle: "Wählen Sie Desktop-Erlebnis und Farbdesign.",
            shell_section_label: "Desktop-Oberfläche:",
            badge_recommended: "EMPFOHLEN",
            shell_noctalia_desc: "Leichte, extrem flüssige moderne Wayland-Oberfläche.",
            shell_caelestia_desc: "Reichhaltige Animationen und QML-Komponenten.",
            color_mode_label: "Farbmodus:",
            mode_light: "Hell",
            mode_dark: "Dunkel",
            mode_auto: "Automatisch",
            appearance_info: "Gilt für SolarUI, GTK/Qt und SDDM.",
            slide4_title: "BlazeOS wird vorbereitet",
            slide4_subtitle: "Einstellungen werden angewendet. Bitte warten...",
            task_keymap: "Sprache und Tastatur werden eingerichtet",
            task_network: "Netzwerk und Uhrzeit werden konfiguriert",
            task_user: "Benutzerkonto wird erstellt",
            task_appearance: "Desktop-Design wird angewendet",
            task_finalize: "Einrichtung wird abgeschlossen",
            badge_pending: "[WARTEND]",
            badge_active: "[IN ARBEIT]",
            badge_done: "[FERTIG]",
            badge_error: "[FEHLER]",
            btn_back: "Zurück",
            btn_next: "Weiter",
            btn_finish: "Fertigstellen",
            btn_start: "Starten",
            btn_launch_desktop: "Desktop starten",
            alert_invalid_user: "Bitte gültigen Kontonamen eingeben.",
            alert_pass_mismatch: "Passwörter stimmen nicht überein."
        },
        fr: {
            slide0_title: "Langue",
            slide0_subtitle: "Choisissez votre langue principale. Modifiable ultérieurement.",
            kb_tr_desc: "Disposition Turc Q",
            kb_trf_desc: "Disposition Turc F",
            kb_en_desc: "Standard US QWERTY",
            kb_de_desc: "Standard QWERTZ",
            kb_fr_desc: "Standard AZERTY",
            kb_test_label: "Test du clavier :",
            kb_test_ph: "Testez vos touches ici : éèàç 123 !?@",
            kb_applied_ph: "Clavier ({0}) appliqué. Essayez : éèàç 123 !?@",
            slide1_title: "Connexion Réseau",
            slide1_subtitle: "Vérifiez la connectivité pour les mises à jour et les pilotes.",
            net_scanning: "Recherche des interfaces réseau...",
            net_offline_title: "Continuer sans connexion Internet",
            net_offline_desc: "Terminer l'installation localement et hors-ligne.",
            btn_refresh_network: "Actualiser le Réseau",
            net_no_active: "Aucune interface active",
            net_no_active_desc: "Aucun réseau détecté",
            net_offline_badge: "Hors-ligne",
            net_wifi_label: "Sans fil (Wi-Fi)",
            net_eth_label: "Connexion filaire (Ethernet)",
            net_connected_badge: "Connecté",
            net_disconnected_badge: "Non connecté",
            net_scan_error: "Échec de l'analyse",
            net_scan_error_desc: "Veuillez vérifier le matériel",
            net_error_badge: "Erreur",
            slide2_title: "Créer un Compte",
            slide2_subtitle: "Remplissez les détails pour créer votre compte utilisateur.",
            field_fullname: "Nom complet :",
            ph_fullname: "Prénom et Nom",
            field_username: "Nom de compte :",
            ph_username: "identifiant",
            hint_username: "Ce sera le nom de votre dossier personnel.",
            field_hostname: "Nom de l'ordinateur :",
            hint_hostname: "Nom affiché sur le réseau et dans le terminal.",
            field_password: "Mot de passe :",
            ph_password: "nouveau mot de passe",
            ph_password_confirm: "confirmer",
            hint_pass_match: "Les mots de passe correspondent.",
            hint_pass_mismatch: "Les mots de passe ne correspondent pas !",
            hint_pass_set: "Définissez un mot de passe.",
            field_hint: "Indice :",
            ph_hint: "facultatif",
            avatar_label: "Photo de profil",
            chk_admin_label: "Autoriser cet utilisateur à administrer cet ordinateur (sudo)",
            chk_autologin_label: "Connexion automatique au démarrage",
            slide3_title: "Bureau & Apparence",
            slide3_subtitle: "Sélectionnez votre environnement de bureau et thème.",
            shell_section_label: "Environnement :",
            badge_recommended: "RECOMMANDÉ",
            shell_noctalia_desc: "Léger, ultra fluide et moderne sous Wayland.",
            shell_caelestia_desc: "Animations riches, widgets et composants QML personnalisables.",
            color_mode_label: "Mode Couleur :",
            mode_light: "Clair",
            mode_dark: "Sombre",
            mode_auto: "Automatique",
            appearance_info: "S'applique à SolarUI, GTK/Qt et à l'écran de connexion SDDM.",
            slide4_title: "Préparation de BlazeOS",
            slide4_subtitle: "Vos paramètres sont en cours d'application. Veuillez patienter...",
            task_keymap: "Configuration de la langue et du clavier",
            task_network: "Configuration du réseau et de l'heure",
            task_user: "Création du compte utilisateur",
            task_appearance: "Configuration de l'apparence et du thème",
            task_finalize: "Finalisation de l'installation",
            badge_pending: "[EN ATTENTE]",
            badge_active: "[EN COURS]",
            badge_done: "[TERMINÉ]",
            badge_error: "[ERREUR]",
            btn_back: "Retour",
            btn_next: "Continuer",
            btn_finish: "Terminer",
            btn_start: "Démarrer",
            btn_launch_desktop: "Lancer le Bureau",
            alert_invalid_user: "Veuillez entrer un nom de compte valide.",
            alert_pass_mismatch: "Les mots de passe ne correspondent pas."
        }
    };

    function applyLanguage(lang) {
        currentLang = OOBE_I18N[lang] ? lang : "en";
        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];

        document.querySelectorAll("[data-i18n]").forEach((el) => {
            const key = el.getAttribute("data-i18n");
            if (dict[key]) {
                el.textContent = dict[key];
            }
        });

        document.querySelectorAll("[data-i18n-ph]").forEach((el) => {
            const key = el.getAttribute("data-i18n-ph");
            if (dict[key]) {
                el.setAttribute("placeholder", dict[key]);
            }
        });

        // Update Next/Back button text according to current slide
        if (typeof updateSlideUI === "function") {
            updateSlideUI();
        }
    }

    const setupData = {
        keymap: "tr",
        locale: "tr_TR.UTF-8",
        networkMode: "auto",
        fullname: "",
        username: "",
        hostname: "blazeos",
        password: "",
        isAdmin: true,
        autologin: false,
        appearance: "dark",
        layout: "noctalia", // Strictly Noctalia by default!
        theme: "solar_amber",
        acousticFeedback: true
    };

    const svgStage = document.getElementById("svg-stage");
    const helloScreen = document.getElementById("hello-screen");
    const wizardScreen = document.getElementById("wizard-screen");
    const startPrompt = document.getElementById("start-prompt");
    const btnBack = document.getElementById("btn-back");
    const btnNext = document.getElementById("btn-next");

    // ── 2. Apple-Style Cursive Handwriting SVG Engine ─────────────────────
    function renderWord(wordObj) {
        if (isWizardActive) return;

        svgStage.innerHTML = wordObj.svg;
        const svg = svgStage.querySelector("svg");
        if (!svg) return;

        svg.style.opacity = "1";
        svg.style.transition = "opacity 0.4s ease";

        const paths = svg.querySelectorAll("path");
        let accumulatedDelay = 0;

        paths.forEach((path) => {
            const length = Math.ceil(path.getTotalLength()) || 2500;
            path.style.strokeDasharray = `${length} ${length}`;
            path.style.strokeDashoffset = `${length}`;
            path.style.strokeWidth = "50";
            path.style.strokeLinecap = "round";
            path.style.strokeLinejoin = "round";

            const duration = Math.max(350, Math.min(1300, (length / 1400) * 850));

            path.animate([
                { strokeDashoffset: length },
                { strokeDashoffset: 0 }
            ], {
                duration: duration,
                delay: accumulatedDelay,
                easing: "cubic-bezier(0.35, 0, 0.25, 1)",
                fill: "forwards"
            });

            accumulatedDelay += duration * 0.72; // natural handwriting stroke overlap
        });

        const totalDurationMs = accumulatedDelay + 2200;

        wordTimer = setTimeout(() => {
            if (isWizardActive) return;
            svg.style.opacity = "0";

            setTimeout(() => {
                if (isWizardActive) return;
                currentWordIndex = (currentWordIndex + 1) % HELLO_WORDS.length;
                renderWord(HELLO_WORDS[currentWordIndex]);
            }, 450);
        }, totalDurationMs);
    }

    // Start handwriting animation loop
    if (typeof HELLO_WORDS !== "undefined" && HELLO_WORDS.length > 0) {
        renderWord(HELLO_WORDS[0]);
    }

    // ── 3. Transition from "Hello" to Setup Wizard ────────────────────────
    function openWizard() {
        if (isWizardActive) return;
        isWizardActive = true;
        clearTimeout(wordTimer);

        helloScreen.classList.add("hidden");
        setTimeout(() => {
            wizardScreen.classList.add("active");
            applyLanguage("tr");
            updateSlideUI();
        }, 300);
    }

    // Listen for any key or click on hello screen
    window.addEventListener("keydown", (e) => {
        if (!isWizardActive) {
            openWizard();
        }
    });

    helloScreen.addEventListener("click", () => {
        if (!isWizardActive) {
            openWizard();
        }
    });

    startPrompt.addEventListener("click", () => {
        openWizard();
    });

    // ── 4. Slide 0: Dil ve Klavye ─────────────────────────────────────────
    const listItems = Array.from(document.querySelectorAll("#slide-0 .list-item"));
    const testInput = document.querySelector(".test-input");

    async function selectLanguageItem(item) {
        if (!item) return;
        listItems.forEach((c) => c.classList.remove("selected"));
        item.classList.add("selected");
        setupData.keymap = item.dataset.keymap;
        setupData.locale = (item.dataset.name || "tr_TR") + ".UTF-8";
        
        // Map keymap to i18n language: "tr" / "tr-f" -> "tr", "us" -> "en", "de" -> "de", "fr" -> "fr"
        const langMap = { "tr": "tr", "tr-f": "tr", "us": "en", "de": "de", "fr": "fr" };
        const targetLang = langMap[setupData.keymap] || "en";
        applyLanguage(targetLang);

        // Live switch active keyboard layout in Niri compositor
        await callBackend("set_keymap", { keymap: setupData.keymap });

        if (testInput) {
            testInput.value = "";
            const titleText = item.querySelector(".list-item-title") ? item.querySelector(".list-item-title").textContent : setupData.keymap;
            const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];
            const phTpl = dict.kb_applied_ph || "Keyboard ({0}) applied.";
            testInput.placeholder = phTpl.replace("{0}", titleText);
        }
    }

    listItems.forEach((item) => {
        item.setAttribute("tabindex", "0");
        item.addEventListener("click", () => selectLanguageItem(item));
        item.addEventListener("keydown", (e) => {
            if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                selectLanguageItem(item);
            }
        });
    });

    window.addEventListener("keydown", (e) => {
        if (!isWizardActive || currentSlide !== 0) return;
        if (document.activeElement === testInput) {
            if (e.key === "Enter") {
                btnNext.click();
            }
            return;
        }

        const selectedIdx = listItems.findIndex(i => i.classList.contains("selected"));
        if (e.key === "ArrowDown") {
            e.preventDefault();
            const nextIdx = Math.min(listItems.length - 1, selectedIdx + 1);
            selectLanguageItem(listItems[nextIdx]);
            listItems[nextIdx].scrollIntoView({ block: "nearest" });
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            const prevIdx = Math.max(0, selectedIdx - 1);
            selectLanguageItem(listItems[prevIdx]);
            listItems[prevIdx].scrollIntoView({ block: "nearest" });
        }
    });

    // ── 5. Slide 1: Ağ ve İnternet Bağlantısı ──────────────────────────────
    const netStatusBox = document.getElementById("network-status-box");
    const offlineCard = document.getElementById("offline-option");
    const btnRefreshNet = document.getElementById("btn-refresh-network");

    async function loadNetworkStatus() {
        if (!netStatusBox) return;
        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];
        netStatusBox.innerHTML = `
            <div class="net-row loading-row">
                <span class="net-spinner"></span>
                <span>${dict.net_scanning || "Scanning..."}</span>
            </div>
        `;
        const res = await callBackend("get_network_status");
        if (res && res.status === "ok") {
            const ifaces = res.interfaces || [];
            if (ifaces.length === 0) {
                netStatusBox.innerHTML = `
                    <div class="net-row">
                        <div class="net-info">
                            <span class="net-name">${dict.net_no_active || "No Active Interface"}</span>
                            <span class="net-ip">${dict.net_no_active_desc || "No connection detected"}</span>
                        </div>
                        <span class="net-status-badge disconnected">${dict.net_offline_badge || "Offline"}</span>
                    </div>
                `;
            } else {
                let html = "";
                ifaces.forEach((iface) => {
                    const isConn = iface.connected;
                    const typeLabel = iface.type === "wifi" ? (dict.net_wifi_label || "Wi-Fi") : (dict.net_eth_label || "Ethernet");
                    const badgeClass = isConn ? "connected" : "disconnected";
                    const badgeText = isConn ? (dict.net_connected_badge || "Connected") : (dict.net_disconnected_badge || "No Connection");
                    const ipText = iface.ip ? `IP: ${iface.ip} (${iface.name})` : `Interface: ${iface.name}`;
                    html += `
                        <div class="net-row">
                            <div class="net-info">
                                <span class="net-name">${typeLabel}</span>
                                <span class="net-ip">${ipText}</span>
                            </div>
                            <span class="net-status-badge ${badgeClass}">${badgeText}</span>
                        </div>
                    `;
                });
                netStatusBox.innerHTML = html;
            }
        } else {
            netStatusBox.innerHTML = `
                <div class="net-row">
                    <div class="net-info">
                        <span class="net-name">${dict.net_scan_error || "Scan Failed"}</span>
                        <span class="net-ip">${dict.net_scan_error_desc || "Check hardware"}</span>
                    </div>
                    <span class="net-status-badge disconnected">${dict.net_error_badge || "Error"}</span>
                </div>
            `;
        }
    }

    if (btnRefreshNet) {
        btnRefreshNet.addEventListener("click", () => {
            loadNetworkStatus();
        });
    }

    if (offlineCard) {
        offlineCard.addEventListener("click", () => {
            offlineCard.classList.toggle("selected");
            setupData.networkMode = offlineCard.classList.contains("selected") ? "offline" : "auto";
        });
    }

    // ── 6. Slide 2: Kullanıcı Hesabı & Parola Kontrolleri ──────────────────
    const inFullname = document.getElementById("input-fullname");
    const inUsername = document.getElementById("input-username");
    const inHostname = document.getElementById("input-hostname");
    const inPass = document.getElementById("input-password");
    const inPassConfirm = document.getElementById("input-password-confirm");
    const passHint = document.getElementById("password-match-hint");
    const chkAdmin = document.getElementById("chk-admin");
    const chkAutologin = document.getElementById("chk-autologin");

    let usernameManuallyChanged = false;
    if (inUsername) {
        inUsername.addEventListener("input", () => {
            usernameManuallyChanged = true;
        });
    }
    if (inFullname) {
        inFullname.addEventListener("input", () => {
            if (!usernameManuallyChanged && inUsername) {
                const clean = inFullname.value.trim().toLowerCase().replace(/[^a-z0-9]/g, "");
                if (clean) inUsername.value = clean;
            }
        });
    }

    function validatePasswordMatch() {
        const p1 = inPass.value;
        const p2 = inPassConfirm.value;
        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];
        if (!p1 && !p2) {
            passHint.textContent = dict.hint_pass_set || "Parola belirleyin.";
            passHint.className = "field-hint";
            return true;
        }
        if (p1 === p2) {
            passHint.textContent = dict.hint_pass_match || "Parolalar eşleşiyor.";
            passHint.className = "field-hint";
            return true;
        } else {
            passHint.textContent = dict.hint_pass_mismatch || "Parolalar eşleşmiyor!";
            passHint.className = "field-hint error";
            return false;
        }
    }

    inPass.addEventListener("input", validatePasswordMatch);
    inPassConfirm.addEventListener("input", validatePasswordMatch);

    // ── 7. Slide 3: Masaüstü Kabuğu ve Görünüş ─────────────────────────────
    const shellCards = document.querySelectorAll(".shell-card");
    shellCards.forEach((card) => {
        card.addEventListener("click", () => {
            shellCards.forEach((c) => c.classList.remove("selected"));
            card.classList.add("selected");
            const radio = card.querySelector('input[type="radio"]');
            if (radio) radio.checked = true;
            setupData.layout = card.dataset.engine || "noctalia";
        });
    });

    const appearanceCards = document.querySelectorAll(".appearance-card");
    appearanceCards.forEach((card) => {
        card.addEventListener("click", () => {
            appearanceCards.forEach((c) => c.classList.remove("selected"));
            card.classList.add("selected");
            const radio = card.querySelector('input[type="radio"]');
            if (radio) radio.checked = true;

            const mode = card.dataset.appearance;
            setupData.appearance = mode;

            // Apply immediately to the current window
            if (mode === "light") {
                document.body.classList.remove("theme-dark");
                document.body.classList.add("theme-light");
            } else {
                document.body.classList.remove("theme-light");
                document.body.classList.add("theme-dark");
            }
        });
    });

    // ── 8. Navigation Controls (Geri / Sürdür) ─────────────────────────────
    function updateSlideUI() {
        document.querySelectorAll(".wizard-slide").forEach((slide, idx) => {
            slide.classList.toggle("active", idx === currentSlide);
        });

        // Automatically query network status when entering Slide 1
        if (currentSlide === 1) {
            loadNetworkStatus();
        }

        // Geri button state
        btnBack.disabled = (currentSlide === 0 || currentSlide === totalSlides - 1);

        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];
        btnBack.textContent = dict.btn_back || "Geri";

        // Sürdür button text
        if (currentSlide === totalSlides - 2) {
            btnNext.textContent = dict.btn_finish || "Tamamla";
        } else if (currentSlide === totalSlides - 1) {
            btnNext.textContent = dict.btn_start || "Başlat";
            btnNext.disabled = true;
        } else {
            btnNext.textContent = dict.btn_next || "Sürdür";
            btnNext.disabled = false;
        }
    }

    btnBack.addEventListener("click", () => {
        if (currentSlide > 0 && currentSlide < totalSlides - 1) {
            currentSlide--;
            updateSlideUI();
        }
    });

    btnNext.addEventListener("click", async () => {
        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];
        // Validation on Account slide (Slide 2)
        if (currentSlide === 2) {
            const u = inUsername.value.trim().toLowerCase();
            const fn = inFullname.value.trim() || (u ? u.charAt(0).toUpperCase() + u.slice(1) : "Blaze");
            const hn = (inHostname ? inHostname.value.trim().toLowerCase() : "blazeos") || "blazeos";
            const p = inPass.value;
            const pc = inPassConfirm.value;

            if (!/^[a-z_][a-z0-9_-]{0,31}$/.test(u)) {
                alert(dict.alert_invalid_user || "Lütfen geçerli bir hesap adı girin.");
                inUsername.focus();
                return;
            }
            if (!/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/.test(hn)) {
                alert("Geçerli bir bilgisayar adı girin (harf, rakam ve tire).");
                inHostname.focus();
                return;
            }
            if (!p) {
                alert("Boş parola kullanılamaz.");
                inPass.focus();
                return;
            }
            if (p !== pc) {
                alert(dict.alert_pass_mismatch || "Girdiğiniz parolalar birbiriyle eşleşmiyor. Lütfen kontrol edin.");
                inPassConfirm.focus();
                return;
            }

            setupData.fullname = fn;
            setupData.username = u;
            setupData.hostname = hn;
            setupData.password = p;
            setupData.isAdmin = chkAdmin.checked;
            setupData.autologin = chkAutologin.checked;
        }

        if (currentSlide < totalSlides - 2) {
            currentSlide++;
            updateSlideUI();
        } else if (currentSlide === totalSlides - 2) {
            // Move to final slide and execute setup sequence
            currentSlide++;
            updateSlideUI();
            await executeFinalSetup();
        }
    });

    window.addEventListener("keydown", (e) => {
        if (!isWizardActive) return;
        if (e.key === "Enter") {
            if (!btnNext.disabled && currentSlide < totalSlides - 1) {
                btnNext.click();
            }
        }
    });

    // ── 9. Backend Communication ──────────────────────────────────────────
    async function callBackend(action, data = {}) {
        try {
            const apiToken = new URLSearchParams(window.location.search).get("token") || "";
            const res = await fetch("/api/action", {
                method: "POST",
                headers: { "Content-Type": "application/json", "X-Blaze-Token": apiToken },
                body: JSON.stringify({ action, data })
            });
            const payload = await res.json().catch(() => ({}));
            if (!res.ok) throw new Error(payload.message || `HTTP ${res.status}`);
            return payload;
        } catch (err) {
            console.error(`[API-ERR] Action ${action} failed:`, err);
            return { status: "error", message: err.toString() };
        }
    }

    function setTaskStatus(taskId, status, text) {
        const row = document.getElementById(taskId);
        if (!row) return;
        const badge = row.querySelector(".progress-badge");
        const title = row.querySelector(".progress-title");

        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];
        badge.className = `progress-badge ${status}`;
        if (status === "done") {
            badge.textContent = dict.badge_done || "[TAMAMLANDI]";
        } else if (status === "active") {
            badge.textContent = dict.badge_active || "[İŞLENİYOR]";
        } else if (status === "error") {
            badge.textContent = dict.badge_error || "[HATA]";
        } else {
            badge.textContent = dict.badge_pending || "[BEKLİYOR]";
        }
        if (text) title.textContent = text;
    }

    async function executeFinalSetup() {
        btnBack.disabled = true;
        btnNext.disabled = true;
        const dict = OOBE_I18N[currentLang] || OOBE_I18N["en"];

        // 1. Task: Keymap & Locale
        setTaskStatus("task-keymap", "active", dict.task_keymap ? `${dict.task_keymap}...` : "Dil ve klavye düzeni işleniyor...");
        const keymapRes = await callBackend("set_keymap", { keymap: setupData.keymap });
        if (keymapRes.status !== "ok") throw new Error(keymapRes.message || "Klavye ayarlanamadı");
        setTaskStatus("task-keymap", "done", dict.task_keymap ? `${dict.task_keymap} [OK]` : "Dil ve klavye düzeni uygulandı.");
        await new Promise((r) => setTimeout(r, 400));

        // 2. Task: Network
        setTaskStatus("task-network", "active", dict.task_network ? `${dict.task_network}...` : "Ağ bağlantısı yapılandırılıyor...");
        await callBackend("get_network_status", {});
        setTaskStatus("task-network", "done", dict.task_network ? `${dict.task_network} [OK]` : "Ağ doğrulandı.");
        await new Promise((r) => setTimeout(r, 400));

        // 3. Task: User Creation & Password
        setTaskStatus("task-user", "active", dict.task_user ? `${dict.task_user}...` : "Kullanıcı hesabı oluşturuluyor...");
        const userRes = await callBackend("create_user", {
            username: setupData.username,
            fullname: setupData.fullname,
            hostname: setupData.hostname,
            password: setupData.password,
            isAdmin: setupData.isAdmin
        });

        if (userRes.status === "error") {
            setTaskStatus("task-user", "error", `Hata: ${userRes.message}`);
            alert(`Hata:\n${userRes.message}`);
            btnBack.disabled = false;
            return;
        }
        setTaskStatus("task-user", "done", `${setupData.username} [OK]`);
        await new Promise((r) => setTimeout(r, 400));

        // 4. Task: Appearance & Desktop Config (Engine: Noctalia or Caelestia)
        const engineLabel = setupData.layout === "caelestia" ? "Caelestia" : "Noctalia";
        setTaskStatus("task-appearance", "active", dict.task_appearance ? `${dict.task_appearance}...` : "Masaüstü kabuğu yapılandırılıyor...");
        const appearanceRes = await callBackend("save_desktop_config", {
            username: setupData.username,
            layout: setupData.layout,
            theme: setupData.theme,
            appearance: setupData.appearance,
            keymap: setupData.keymap,
            acoustic: setupData.acousticFeedback
        });
        if (appearanceRes.status !== "ok") {
            setTaskStatus("task-appearance", "error", appearanceRes.message || "Masaüstü ayarlanamadı");
            btnBack.disabled = false;
            return;
        }
        setTaskStatus("task-appearance", "done", `${engineLabel} (${setupData.appearance.toUpperCase()}) [OK]`);
        await new Promise((r) => setTimeout(r, 400));

        // 5. Task: Finalize OOBE
        setTaskStatus("task-finalize", "active", dict.task_finalize ? `${dict.task_finalize}...` : "OOBE tamamlanıyor...");
        const finishRes = await callBackend("finish_setup", {
            username: setupData.username,
            fullname: setupData.fullname,
            hostname: setupData.hostname,
            autologin: setupData.autologin,
            layout: setupData.layout
        });
        if (finishRes.status !== "ok") {
            setTaskStatus("task-finalize", "error", finishRes.message || "Kurulum tamamlanamadı");
            btnBack.disabled = false;
            return;
        }
        setTaskStatus("task-finalize", "done", "BlazeOS [READY]");
        await new Promise((r) => setTimeout(r, 600));

        // Launch Desktop
        btnNext.textContent = dict.btn_launch_desktop || "Masaüstünü Başlat";
        btnNext.disabled = false;
        btnNext.onclick = () => {
            callBackend("start_desktop", {});
        };

        // Auto-launch after 1.5 seconds if autologin is selected or smoothly proceed
        setTimeout(() => {
            callBackend("start_desktop", {});
        }, 1500);
    }
});
