#!/bin/bash
set -euo pipefail
export PATH="${HOME}/bin:${HOME}/.local/bin:/usr/local/bin:/usr/local/sbin:$PATH"

BASE_ISO="${BLAZEOS_ROOT:-$(pwd)}/f45_base/Fedora-Workstation-Live-45_Beta-1.3.x86_64.iso"
OUT_ISO="${BLAZEOS_ROOT:-$(pwd)}/Blaze-SolarEvolution-5-x86_64.iso"
WORK="${BLAZEOS_ROOT:-$(pwd)}/work_f45"
ROOTFS_SRC="$WORK/rootfs"
NEW_SQUASHFS="$WORK/LiveOS/squashfs.img"
PATCHED_INITRD="$WORK/initrd_work/patched_initrd"
GRUB_CFG="$WORK/iso_mods/boot/grub2/grub.cfg"
OVERLAY="${BLAZEOS_ROOT:-$(pwd)}/blazeos_custom_apps"
SOLARUI_SRC="${BLAZEOS_ROOT:-$(pwd)}/solarui"

echo "=== [0/5] Building SolarUI controller and validating tracked code ==="
cargo build --manifest-path "$SOLARUI_SRC/Cargo.toml" --release -p solar-shell -p blaze-recovery
install -m 0755 "$SOLARUI_SRC/target/release/solar-shell" "$OVERLAY/usr/bin/solar-shell"
install -m 0755 "$SOLARUI_SRC/target/release/blaze-recovery" "$OVERLAY/usr/bin/blaze-recovery"
SOLAR_SHELL_PROBE="$($OVERLAY/usr/bin/solar-shell switch __blazeos_build_probe__ 2>&1 || true)"
if ! grep -q 'end4-pc' <<<"$SOLAR_SHELL_PROBE"; then
    echo "Refusing to build: solar-shell does not contain the End4 switch implementation" >&2
    exit 1
fi
grep -qx 'VERSION="5.05"' "$OVERLAY/etc/os-release"
grep -qx 'VERSION_ID=5.05' "$OVERLAY/etc/os-release"
grep -qx 'PRETTY_NAME="Blaze SolarEvolution 5.05"' "$OVERLAY/etc/os-release"
bash -n \
    "$OVERLAY/usr/libexec/livesys/sessions.d/livesys-solarui" \
    "$OVERLAY/usr/libexec/blazeos-oobe-hygiene" \
    "$OVERLAY/usr/local/bin/blazeos-apply-install-choices" \
    "$OVERLAY/usr/local/bin/blazeos-desktop-provision" \
    "$OVERLAY/usr/local/bin/blazeos-remove-solarui" \
    "$OVERLAY/usr/local/bin/blazeos-postinstall" \
    "$OVERLAY/usr/libexec/blazeos-wayland-session-ready" \
    "$OVERLAY/usr/libexec/blazeos-update" \
    "$OVERLAY/usr/libexec/blazeos-offline-update" \
    "$OVERLAY/usr/libexec/blaze-style-center-first-run" \
    "$OVERLAY/usr/local/bin/blaze-bootloader" \
    "$OVERLAY/usr/local/bin/blazeos-limine-install" \
    "$OVERLAY/usr/bin/blaze-system-monitor"
node --check "$OVERLAY/usr/share/cockpit/anaconda-webui/index.js"
grep -q 'spawn-at-startup "/usr/libexec/blazeos-wayland-session-ready"' "$OVERLAY/etc/solarui/niri.kdl"
grep -q 'spawn-at-startup "/usr/libexec/blazeos-wayland-session-ready"' "$OVERLAY/etc/skel/.config/solarui/niri.kdl"
grep -q 'spawn-at-startup "/usr/libexec/blazeos-wayland-session-ready"' "$OVERLAY/etc/skel/.config/niri/config.kdl"
grep -q 'spawn-at-startup "/usr/libexec/blazeos-wayland-session-ready"' "$OVERLAY/usr/share/solarui/niri.kdl"
python3 -c 'compile(open("'"$OVERLAY/usr/bin/blaze-setup"'", encoding="utf-8").read(), "blaze-setup", "exec")'
python3 -c 'compile(open("'"$OVERLAY/usr/bin/berp-recovery-gui"'", encoding="utf-8").read(), "berp-recovery-gui", "exec")'
python3 -c 'compile(open("'"$OVERLAY/usr/bin/blaze-style-center"'", encoding="utf-8").read(), "blaze-style-center", "exec")'
python3 -c 'compile(open("'"$OVERLAY/usr/bin/blaze-gamezone-browser"'", encoding="utf-8").read(), "blaze-gamezone-browser", "exec")'
if grep -Eq '/root/var/lib/mock|image-root' "$OVERLAY/boot/efi/EFI/fedora/grub.cfg"; then
    echo "Refusing to build: EFI grub.cfg contains a build-host path" >&2
    exit 1
fi
grep -q 'search --no-floppy --file --set=dev /grub2/grub.cfg' \
    "$OVERLAY/boot/efi/EFI/fedora/grub.cfg"
test -L "$OVERLAY/etc/systemd/system/graphical.target.wants/blazeos-oobe-hygiene.service"
test -L "$OVERLAY/etc/systemd/system/graphical.target.wants/blazeos-apply-install-choices.service"
test -L "$OVERLAY/etc/systemd/system/graphical.target.wants/blazeos-desktop-provision.service"
test -L "$OVERLAY/etc/systemd/system/dbus-org.freedesktop.PolicyKit1.service"
cmp -s "$OVERLAY/usr/share/wayland-sessions/blaze-oobe.desktop" \
    "$OVERLAY/usr/share/blaze-setup/blaze-oobe.desktop"
if grep -Eq 'id="input-password(-confirm)?"[^>]*value=' \
    "$OVERLAY/usr/share/blaze-setup/index.html"; then
    echo "Refusing to build: OOBE password fields must never contain a preset value" >&2
    exit 1
fi
if grep -Eq 'cat > /etc/sddm\.conf\.d/(10-blaze-session|autologin)\.conf|DisplayServer=wayland|CompositorCommand=weston' \
    "$OVERLAY/usr/local/bin/blazeos-postinstall"; then
    echo "Refusing to build: postinstall overrides the known-good SDDM greeter path" >&2
    exit 1
fi

echo "=== [1/5] Building Squashfs with full SELinux labels in tmpfs ==="
UNSHARE_FLAGS="-rm"
if [ "$(id -u)" -eq 0 ]; then
    UNSHARE_FLAGS="-m"
fi
unshare $UNSHARE_FLAGS bash -c '
set -euo pipefail
MNT="/tmp/f45_build_mnt"
mkdir -p "$MNT"
echo "  -> Mounting tmpfs (28G)..."
mount -t tmpfs -o size=28G tmpfs "$MNT"
ROOTFS="$MNT/rootfs"

echo "  -> Copying rootfs into tmpfs..."
cp -a "'"$ROOTFS_SRC"'" "$ROOTFS"

echo "  -> Ensuring root:root ownership..."
chown -R 0:0 "$ROOTFS"

echo "  -> Applying tracked BlazeOS overlay..."
cp -a --no-preserve=ownership "'"${BLAZEOS_ROOT:-$(pwd)}"'/blazeos_custom_apps/." "$ROOTFS/"
# Keep display branding and boot branding separate.  The transparent asset is
# reserved for the Plymouth normal/update splash; the GDM/taskbar icon remains
# the square application logo under /usr/share/blazeos/gdm.
if [ -f "$ROOTFS/transparent_boot_logo.png" ]; then
    install -D -m 0644 "$ROOTFS/transparent_boot_logo.png" \
        "$ROOTFS/usr/share/plymouth/themes/blazeos/logo.png"
    install -D -m 0644 "$ROOTFS/transparent_boot_logo.png" \
        "$ROOTFS/usr/share/plymouth/themes/blazeos/logo-glow.png"
fi

echo "  -> Installing pinned end4-pC shell source..."
rm -rf "$ROOTFS/usr/share/quickshell/end4-pC"
mkdir -p "$ROOTFS/usr/share/quickshell"
cp -a --no-preserve=ownership "'"${BLAZEOS_ROOT:-$(pwd)}"'/solarui/vendor/end4-pC" "$ROOTFS/usr/share/quickshell/end4-pC"
find "$ROOTFS/usr/share/quickshell/end4-pC" -type d -exec chmod 755 {} +
find "$ROOTFS/usr/share/quickshell/end4-pC" -type f -exec chmod 644 {} +
find "$ROOTFS/usr/share/quickshell/end4-pC/scripts" -type f \( -name "*.sh" -o -name "*.py" \) -exec chmod 755 {} +
mkdir -p "$ROOTFS/usr/share/doc/solarui" "$ROOTFS/usr/share/licenses/solarui/end4-pC"
cp "$ROOTFS/usr/share/quickshell/end4-pC/UPSTREAM.md" "$ROOTFS/usr/share/doc/solarui/end4-pC-UPSTREAM"
cp "$ROOTFS/usr/share/quickshell/end4-pC/BLAZEOS-CHANGES.md" "$ROOTFS/usr/share/doc/solarui/end4-pC-BLAZEOS-CHANGES"
cp "$ROOTFS/usr/share/quickshell/end4-pC/LICENSE" "$ROOTFS/usr/share/licenses/solarui/end4-pC/LICENSE"

echo "  -> Installing Fedora 45-matched Quickshell runtime for end4-pC..."
# Quickshell and Qt QML plugins use private Qt ABI. Never copy these libraries
# from the build host: the host can be on a newer Qt minor release.
QT_SHELL_RPMS=()
for spec in \
  "qt6-qtbase-6.11.2-3.fc45.x86_64.rpm:b8910b17cc2598f8677f3f6d450b45292480047c8bd90c69e93806181c14c192" \
  "qt6-qtbase-common-6.11.2-3.fc45.noarch.rpm:66a7bca247c43705c2fa42a79263a57f0685c7464dc23f50cea90d939ad3620b" \
  "qt6-qtbase-gui-6.11.2-3.fc45.x86_64.rpm:2357e6e50dcc10dfd335d17eb21ebece873b422c429a40eae2f1f915ca5c629e" \
  "qt6-qtdeclarative-6.11.2-2.fc45.x86_64.rpm:c3af9a7cb60063bd2c229e3f201b4fafb768562c87fa0f71d4b721869654a75d" \
  "qt6-qtshadertools-6.11.2-1.fc45.x86_64.rpm:6a51c2af07c7fc2c785d30349c20f395e2e5a1577250cb0fc97b2956ff07315c" \
  "qt6-qtsvg-6.11.2-1.fc45.x86_64.rpm:c4e20cd6e9bd91735c6af2340874e5221f0c080c2840e2e799a19947903dda9e" \
  "qt6-qtwayland-6.11.2-1.fc45.x86_64.rpm:ea7f133edfaa61c94c01882571b986ed6211acfa5c466cbc474a77491bbc3a17" \
  "qt6-qtwayland-adwaita-decoration-6.11.2-1.fc45.x86_64.rpm:f4d2d15f5d1684490a23a7bd61853470aa063369682bcf678d8d7921e54f48c9" \
  "qt6-qt5compat-6.11.2-1.fc45.x86_64.rpm:36677b8a807c93e12b0fb1ee9036f925bede904938ad7d7e8d9bb241b945e068" \
  "jemalloc-5.3.0-14.fc44.x86_64.rpm:3159390bdbcf46c3269fd499855e95421f92b3b6a155b68d5b641fb6cc7f7a88" \
  "quickshell-0.2.1_git20260209.dacfa9d-5.fc45.x86_64.rpm:3f926860e9495029953a47b193428b98ccb2b339428ee18c37a4450edb0e3536"; do
    file="${spec%%:*}"; hash="${spec#*:}"
    rpm_path="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/${file}"
    echo "${hash}  ${rpm_path}" | sha256sum --check --status
    QT_SHELL_RPMS+=("$rpm_path")
done
rpm --root "$ROOTFS" -Uvh --replacepkgs --replacefiles --noscripts "${QT_SHELL_RPMS[@]}"
# Keep the ABI-matched Fedora executable only for end4-pC. Caelestia and
# Noctalia retain the bundled runtime and its matching compiled plugins.
mv -f "$ROOTFS/usr/bin/quickshell" "$ROOTFS/usr/bin/quickshell-fedora"
cp -a --no-preserve=ownership "'"${BLAZEOS_ROOT:-$(pwd)}"'/blazeos_custom_apps/usr/bin/quickshell" "$ROOTFS/usr/bin/quickshell"
chmod 755 "$ROOTFS/usr/bin/quickshell" "$ROOTFS/usr/bin/quickshell-fedora"
rm -f "$ROOTFS/usr/lib64/libQt6ShaderTools.so.6.12.0"
# Fail the build if the Fedora plugin or executable can see a mismatched ABI.
readelf -V "$ROOTFS/usr/bin/quickshell-fedora" >"$ROOTFS/tmp/quickshell-fedora.versions"
readelf -V "$ROOTFS/usr/lib64/qt6/qml/Qt5Compat/GraphicalEffects/libqtgraphicaleffectsplugin.so" >"$ROOTFS/tmp/qt5compat.versions"
grep -q "Qt_6.11_PRIVATE_API" "$ROOTFS/tmp/quickshell-fedora.versions"
grep -q "Qt_6.11_PRIVATE_API" "$ROOTFS/tmp/qt5compat.versions"
rm -f "$ROOTFS/tmp/quickshell-fedora.versions" "$ROOTFS/tmp/qt5compat.versions"

echo "  -> Installing pinned end4-pC runtime tools..."
END4_RPMS=()
for spec in \
  "brightnessctl-0.5.1-17.fc45.x86_64.rpm:aad4219055ce9a45149a2778057c093b178a905d2b99a03aa53fcf65973c91a3" \
  "cava-0.10.2-8.fc45.x86_64.rpm:363a696a0e8fb762f210b40075a7cb0d28016adda5e43eb8cd2b3a0260af4ffa" \
  "grim-1.5.0-4.fc45.x86_64.rpm:250c718d4c1ff738d320fec19c02e3c0fb70d3b9d7c52a59ac0760b5b4358863" \
  "matugen-3.1.0-2.fc45.x86_64.rpm:7b991bb4dbe25210034d80a83da54a1a7d7964f1b432817e9ed9a8d910ce27e8" \
  "playerctl-libs-2.4.1-13.fc45.x86_64.rpm:abf50f92aa200d41db0887bf3609aa2689dd49cdfd613ff60062aca83da87641" \
  "playerctl-2.4.1-13.fc45.x86_64.rpm:082898252dfe9c770bccec8704a08ef888c074ef945070d85188f69f4e40ef8c" \
  "slurp-1.5.0-7.fc45.x86_64.rpm:091c9af9b9188ece37fdfc3c66e398fb7f0310db06d8bbc05a8f5c93ab915816" \
  "swappy-1.5.1-9.fc45.x86_64.rpm:8fb84850b5373608c2be70af519d08eb63ea29e6cd92120d362199a96c7dec64"; do
    file="${spec%%:*}"; hash="${spec#*:}"
    rpm_path="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/${file}"
    echo "${hash}  ${rpm_path}" | sha256sum --check --status
    END4_RPMS+=("$rpm_path")
done
rpm --root "$ROOTFS" -Uvh --replacepkgs --noscripts "${END4_RPMS[@]}"

# Cockpit prefers pre-compressed assets when the browser advertises gzip.
# Rebuild the compressed bundle so it can never lag behind the patched source.
echo "  -> Synchronizing Anaconda WebUI compressed bundle..."
gzip -9 -n -c "$ROOTFS/usr/share/cockpit/anaconda-webui/index.js" > "$ROOTFS/usr/share/cockpit/anaconda-webui/index.js.gz"

echo "  -> Replacing GNOME System Monitor with Fedora htop..."
HTOP_RPM="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/htop-3.5.3-1.fc45.x86_64.rpm"
HTOP_SHA256="a5f21195a6094f6abe43b97fb90554dc8617ff98dc8ee1ff92e85e6782719573"
HWLOC_RPM="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/hwloc-libs-2.14.0-2.fc45.x86_64.rpm"
HWLOC_SHA256="6e187ec895352dcd52d46a9cb0299b83389acae2d8051b93811b78720838d464"
echo "${HTOP_SHA256}  ${HTOP_RPM}" | sha256sum --check --status
echo "${HWLOC_SHA256}  ${HWLOC_RPM}" | sha256sum --check --status
if rpm --root "$ROOTFS" -q gnome-system-monitor >/dev/null 2>&1; then
    rpm --root "$ROOTFS" -e --nodeps --noscripts gnome-system-monitor
fi
rpm --root "$ROOTFS" -Uvh --replacepkgs --noscripts "$HWLOC_RPM" "$HTOP_RPM"

echo "  -> Trimming Workstation host payload from the SolarUI core image..."
# The Fedora Workstation source image carries a complete office suite and a
# QEMU/libvirt *host* stack. They are not needed to boot or run SolarUI and are
# installed later only by the explicit Office/virtualization choices. Guest
# integration (`qemu-guest-agent`, `spice-vdagent`, open-vm-tools and the
# VirtualBox guest service) stays in the image so BlazeOS still works in VMs.
mapfile -t CORE_TRIM < <(
    rpm --root "$ROOTFS" -qa --qf "%{NAME}\n" | grep -E \
        "^(libreoffice($|-)|qemu-(audio|block|char|common$|device|img$|kvm|pr-helper|system|ui)|libvirt($|-)|edk2-ovmf$|spice-server$)" || true
)
if ((${#CORE_TRIM[@]})); then
    rpm --root "$ROOTFS" -e --nodeps --noscripts "${CORE_TRIM[@]}"
fi
# These eager units add boot latency/RSS on ordinary desktops. Their packages
# remain installed and D-Bus/socket activation still starts the feature when a
# user actually needs printing, discovery, cellular networking or remote FS.
for unit in \
    network-online.target.wants/NetworkManager-wait-online.service \
    multi-user.target.wants/avahi-daemon.service \
    sockets.target.wants/avahi-daemon.socket \
    multi-user.target.wants/systemd-homed.service \
    sysinit.target.wants/iscsi-onboot.service \
    sysinit.target.wants/iscsi-starter.service \
    multi-user.target.wants/mdmonitor.service; do
    rm -f "$ROOTFS/etc/systemd/system/$unit"
done
CORE_PACKAGE_COUNT=$(rpm --root "$ROOTFS" -qa | wc -l)
echo "  -> SolarUI core package count after trim: $CORE_PACKAGE_COUNT"
test "$CORE_PACKAGE_COUNT" -lt 1950

echo "  -> Installing the graphical polkit authentication agent..."
MATE_POLKIT_RPM="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/mate-polkit-1.28.1-9.fc45.x86_64.rpm"
MATE_POLKIT_SHA256="6e16a21f6c0c755a106a009fd84e5388709a9f02f8ea5fb7d424f6a558ddde28"
APPINDICATOR_RPM="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/libappindicator-gtk3-12.10.1-11.fc45.x86_64.rpm"
APPINDICATOR_SHA256="7527f45d311ddadfe23c70567716104d517d55fc61a1240fada633a43564a8a7"
DBUSMENU_RPM="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/libdbusmenu-16.04.0-32.fc45.x86_64.rpm"
DBUSMENU_SHA256="53a5856835e3da24812420b2849632a291a6b102362e5aa91f750af75d96a382"
DBUSMENU_GTK_RPM="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/libdbusmenu-gtk3-16.04.0-32.fc45.x86_64.rpm"
DBUSMENU_GTK_SHA256="fcaf227fbc3c212f1c1b203d504a6fe585eb65740d16f25a0c4087f67b0d0eba"
echo "${MATE_POLKIT_SHA256}  ${MATE_POLKIT_RPM}" | sha256sum --check --status
echo "${APPINDICATOR_SHA256}  ${APPINDICATOR_RPM}" | sha256sum --check --status
echo "${DBUSMENU_SHA256}  ${DBUSMENU_RPM}" | sha256sum --check --status
echo "${DBUSMENU_GTK_SHA256}  ${DBUSMENU_GTK_RPM}" | sha256sum --check --status
rpm --root "$ROOTFS" -Uvh --replacepkgs --noscripts \
    "$DBUSMENU_RPM" "$DBUSMENU_GTK_RPM" "$APPINDICATOR_RPM" "$MATE_POLKIT_RPM"

echo "  -> Restoring SUID/SGID permissions stripped by chown..."
chmod 4755 "$ROOTFS/usr/bin/sudo" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/su" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/pkexec" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/passwd" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/gpasswd" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/mount" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/umount" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/chsh" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/chfn" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/newgrp" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/userhelper" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/lib/polkit-1/polkit-agent-helper-1" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/libexec/dbus-1/dbus-daemon-launch-helper" 2>/dev/null || true
chmod 4755 "$ROOTFS/opt/FireHub/chrome-sandbox" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/unix_chkpwd" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/crontab" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/at" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/chage" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/libexec/utempter/utempter" 2>/dev/null || true
chown -R 961:961 "$ROOTFS/var/lib/blaze-setup" 2>/dev/null || true
chmod 755 "$ROOTFS/var/lib/blaze-setup" 2>/dev/null || true
chmod 0440 "$ROOTFS/etc/sudoers.d/99-blaze-setup" 2>/dev/null || true
if [ ! -f "$ROOTFS/usr/lib/kitty/bin/kitty" ]; then
    echo "  -> Fetching Kitty standalone bundle..."
    mkdir -p "$ROOTFS/usr/lib/kitty"
    curl -sL https://github.com/kovidgoyal/kitty/releases/download/v0.49.1/kitty-0.49.1-x86_64.txz | tar -xJ -C "$ROOTFS/usr/lib/kitty"
    ln -sf ../lib/kitty/bin/kitty "$ROOTFS/usr/bin/kitty"
    ln -sf ../lib/kitty/bin/kitten "$ROOTFS/usr/bin/kitten"
fi

chmod 755 "$ROOTFS/usr/bin/firehub" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-house" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-recovery" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-sentinel" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/solar-shell" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-gamezone" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/solar-omnibar" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/protonup-qt" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/solar-session" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/kitty" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/kitten" 2>/dev/null || true
chmod -R 755 "$ROOTFS/usr/lib/kitty" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-apply-install-choices" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-desktop-provision" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-remove-solarui" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/caelestia" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/quickshell" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/quickshell.bin" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/qs" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/sddm" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/weston" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/berp-recovery-gui" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-recovery-gui-launcher" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-gamezone-browser" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-setup" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-system-monitor" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-wayland-session-ready" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-update" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-offline-update" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-offline-progress-filter" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blaze-style-center-first-run" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-style-center" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-apply-install-choices" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/gen_grub_cfgstub" 2>/dev/null || true
chmod 755 "$ROOTFS/etc/grub.d/09_berp" 2>/dev/null || true
chmod 755 "$ROOTFS/etc/grub.d/99_berp" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/solar-torture" 2>/dev/null || true
chmod 600 "$ROOTFS/etc/NetworkManager/system-connections/"*.nmconnection 2>/dev/null || true

echo "  -> Compiling GSettings schemas..."
glib-compile-schemas "$ROOTFS/usr/share/glib-2.0/schemas" 2>/dev/null || true

echo "  -> Ensuring SDDM default display manager and masking GDM..."
mkdir -p "$ROOTFS/etc/systemd/system"
mkdir -p "$ROOTFS/etc/systemd/system/graphical.target.wants"
mkdir -p "$ROOTFS/etc/systemd/system/multi-user.target.wants"
ln -sf /usr/lib/systemd/system/sddm.service "$ROOTFS/etc/systemd/system/display-manager.service"
ln -sf /usr/lib/systemd/system/sddm.service "$ROOTFS/etc/systemd/system/graphical.target.wants/sddm.service"
ln -sf /usr/lib/systemd/system/sddm.service "$ROOTFS/etc/systemd/system/multi-user.target.wants/display-manager.service"
ln -sf /usr/lib/systemd/system/graphical.target "$ROOTFS/etc/systemd/system/default.target"
ln -sf /dev/null "$ROOTFS/etc/systemd/system/gdm.service"
ln -sf /usr/lib/systemd/system/polkit.service "$ROOTFS/etc/systemd/system/dbus-org.freedesktop.PolicyKit1.service"

echo "  -> Updating dynamic linker cache (ldconfig)..."
chroot "$ROOTFS" ldconfig 2>/dev/null || ldconfig -r "$ROOTFS" -f "$ROOTFS/etc/ld.so.conf" -C "/etc/ld.so.cache" 2>/dev/null || true

echo "  -> Setting SELinux contexts with setfiles..."
LD_LIBRARY_PATH="$ROOTFS/usr/lib64" "$ROOTFS/usr/sbin/setfiles" -r "$ROOTFS" "$ROOTFS/etc/selinux/targeted/contexts/files/file_contexts" "$ROOTFS" 2>&1 | tail -10
chroot "$ROOTFS" semanage permissive -a virt_qemu_ga_t 2>/dev/null || true

echo "  -> Explicitly ensuring systemd and custom apps have correct labels..."
setfattr -n security.selinux -v "system_u:object_r:init_exec_t:s0" "$ROOTFS/usr/lib/systemd/systemd"
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-welcome" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/solar-torture" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-nvidia-setup" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-shell" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-session" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/lib/kitty/bin/kitty" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/lib/kitty/bin/kitten" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-limine-install" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/libexec/blazeos-welcome-helper" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blazeos-control" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blaze-optimize" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blaze-bootloader" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-postinstall" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/limine" 2>/dev/null || true
find "$ROOTFS/usr/share/limine" -exec setfattr -n security.selinux -v "system_u:object_r:usr_t:s0" {} + 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-optimization.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blazeos-postinstall.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/fastfetch" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/yad" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/firehub" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-house" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/opt/FireHub/firehub" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blazeos-nvidia-firstboot.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-core" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-recovery" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-sentinel" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-recovery.target" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-recovery.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-recovery-gui.target" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-recovery-gui.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-recovery-gui-launcher" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/seatd" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/seatd-launch" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/berp-recovery-gui" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/gen_grub_cfgstub" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-sentinel.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blaze-boot-success.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-shell" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-gamezone" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-gamezone-browser" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-omnibar" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/protonup-qt" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-session" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-session-wrapper" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/niri" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/niri-session" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/noctalia" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-settings" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/quickshell" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/quickshell.bin" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/qs" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/caelestia" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/liveinst" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-setup" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-system-monitor" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:etc_t:s0" "$ROOTFS/etc/sddm.conf.d/00-blaze-oobe.conf" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:etc_t:s0" "$ROOTFS/etc/sudoers.d/99-blaze-setup" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:usr_t:s0" "$ROOTFS/usr/share/wayland-sessions/blaze-oobe.desktop" 2>/dev/null || true
find "$ROOTFS/usr/share/blaze-setup" -exec setfattr -n security.selinux -v "system_u:object_r:usr_t:s0" {} + 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/libexec/livesys/sessions.d/livesys-solarui" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:xdm_exec_t:s0" "$ROOTFS/usr/bin/sddm" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:xdm_exec_t:s0" "$ROOTFS/usr/bin/sddm-greeter-qt6" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/weston" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS/usr/lib64/solarui/libsolar_brand.so" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS/usr/lib/solarui/libsolar_brand.so" 2>/dev/null || true
find "$ROOTFS/usr/lib64/quickshell" -exec setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" {} + 2>/dev/null || true
find "$ROOTFS/usr/share/caelestia" -exec setfattr -n security.selinux -v "system_u:object_r:usr_t:s0" {} + 2>/dev/null || true
find "$ROOTFS/usr/share/quickshell/end4-pC" -exec setfattr -n security.selinux -v "system_u:object_r:usr_t:s0" {} + 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libseat.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libyyjson.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libgtksourceview-3.0.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libgspell-1.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libicu*.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libjpeg*.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libjasper*.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libmng*.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib/python3.15/site-packages/materialyoucolor/quantize/celebi*.so 2>/dev/null || true

echo "  -> Verifying critical SELinux labels..."
getfattr -d -m - "$ROOTFS/usr/lib/systemd/systemd"
getfattr -d -m - "$ROOTFS/usr/bin/sudo"
getfattr -d -m - "$ROOTFS/usr/local/bin/blazeos-welcome"
getfattr -d -m - "$ROOTFS/usr/local/bin/blazeos-control"
getfattr -d -m - "$ROOTFS/usr/bin/blaze-house"

echo "  -> Compressing rootfs into LiveOS/squashfs.img with zstd & xattrs..."
mkdir -p "'"$WORK"'"/LiveOS
rm -f "'"$NEW_SQUASHFS"'"
mksquashfs "$ROOTFS" "'"$NEW_SQUASHFS"'" \
    -comp zstd \
    -Xcompression-level 15 \
    -b 131072 \
    -no-progress \
    -noappend \
    -xattrs

echo "  -> Unmounting tmpfs..."
umount "$MNT"
rmdir "$MNT"
echo "  -> Squashfs build done!"
'

echo "=== [2/5] Verifying new squashfs.img xattrs ==="
unsquashfs -s "$NEW_SQUASHFS" | grep -i xattr

echo "=== [3/5] Building hybrid ISO with xorriso ==="
rm -f "$OUT_ISO"
xorriso \
  -indev "$BASE_ISO" \
  -outdev "$OUT_ISO" \
  -volid "Blaze-SE-5" \
  -publisher "BlazeOS Project" \
  -application_id "Blaze SolarEvolution 5" \
  -map "$NEW_SQUASHFS" /LiveOS/squashfs.img \
  -map "$PATCHED_INITRD" /boot/x86_64/loader/initrd \
  -map "$GRUB_CFG" /boot/grub2/grub.cfg \
  -boot_image any replay \
  -padding 0

echo "=== [4/5] Implanting ISO checksum ==="
if command -v implantisomd5 >/dev/null 2>&1; then
    implantisomd5 "$OUT_ISO"
elif [ -x "${HOME}/bin/implantisomd5" ]; then
    "${HOME}/bin/implantisomd5" "$OUT_ISO"
else
    echo "  (implantisomd5 not found in PATH, skipping checksum embedding)"
fi

echo "=== [5/5] Verifying ISO checksum ==="
if command -v checkisomd5 >/dev/null 2>&1; then
    checkisomd5 --verbose "$OUT_ISO"
elif [ -x "${HOME}/bin/checkisomd5" ]; then
    "${HOME}/bin/checkisomd5" --verbose "$OUT_ISO"
else
    echo "  (checkisomd5 not found in PATH, skipping checksum verification)"
fi

echo "=== BUILD COMPLETE! ==="
ls -lh "$OUT_ISO"
sha256sum "$OUT_ISO" > "${OUT_ISO}.sha256"
echo "SHA-256: $(cut -d' ' -f1 "${OUT_ISO}.sha256")"
