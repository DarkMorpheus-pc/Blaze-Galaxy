#!/bin/bash
# =============================================================================
# BlazeOS SolarEvolution 5 — Offline Full ISO Builder (10GB+)
# Pre-bakes all 9 Desktop Environments into rootfs squashfs
# =============================================================================
set -euo pipefail
export PATH="${HOME}/bin:${HOME}/.local/bin:/usr/local/bin:/usr/local/sbin:$PATH"

BASE_ISO="${BLAZEOS_ROOT:-$(pwd)}/f45_base/Fedora-Workstation-Live-45_Beta-1.3.x86_64.iso"
OUT_ISO="${BLAZEOS_ROOT:-$(pwd)}/Blaze-SolarEvolution-5-Offline-x86_64.iso"
WORK="${BLAZEOS_ROOT:-$(pwd)}/work_f45"
ROOTFS_SRC="$WORK/rootfs"
NEW_SQUASHFS="$WORK/LiveOS/squashfs.img"
PATCHED_INITRD="$WORK/initrd_work/patched_initrd"
GRUB_CFG="$WORK/iso_mods/boot/grub2/grub.cfg"
OVERLAY="${BLAZEOS_ROOT:-$(pwd)}/blazeos_custom_apps"
SOLARUI_SRC="${BLAZEOS_ROOT:-$(pwd)}/solarui"

echo "=== [0/5] Building the current SolarUI controller ==="
cargo build --manifest-path "$SOLARUI_SRC/Cargo.toml" --release -p solar-shell
install -m 0755 "$SOLARUI_SRC/target/release/solar-shell" "$OVERLAY/usr/bin/solar-shell"
SOLAR_SHELL_PROBE="$($OVERLAY/usr/bin/solar-shell switch __blazeos_build_probe__ 2>&1 || true)"
if ! grep -q 'end4-pc' <<<"$SOLAR_SHELL_PROBE"; then
    echo "Refusing to build: solar-shell does not contain the End4 switch implementation" >&2
    exit 1
fi

echo "=== [1/5] Building Offline Squashfs with full SELinux labels in tmpfs ==="
UNSHARE_FLAGS="-rm"
if [ "$(id -u)" -eq 0 ]; then
    UNSHARE_FLAGS="-m"
fi
unshare $UNSHARE_FLAGS bash -c '
set -euo pipefail
MNT="/tmp/f45_build_mnt_offline"
mkdir -p "$MNT"
echo "  -> Mounting tmpfs (40G for Offline ISO)..."
mount -t tmpfs -o size=40G tmpfs "$MNT"
ROOTFS="$MNT/rootfs"

echo "  -> Copying rootfs into tmpfs..."
cp -a "'"$ROOTFS_SRC"'" "$ROOTFS"

echo "  -> Ensuring root:root ownership..."
chown -R 0:0 "$ROOTFS"

echo "  -> Applying tracked BlazeOS overlay..."
cp -a --no-preserve=ownership "'"${BLAZEOS_ROOT:-$(pwd)}"'/blazeos_custom_apps/." "$ROOTFS/"

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

echo "  -> Installing the pinned graphical polkit authentication agent..."
for spec in \
  "mate-polkit-1.28.1-9.fc45.x86_64.rpm:6e16a21f6c0c755a106a009fd84e5388709a9f02f8ea5fb7d424f6a558ddde28" \
  "libappindicator-gtk3-12.10.1-11.fc45.x86_64.rpm:7527f45d311ddadfe23c70567716104d517d55fc61a1240fada633a43564a8a7" \
  "libdbusmenu-16.04.0-32.fc45.x86_64.rpm:53a5856835e3da24812420b2849632a291a6b102362e5aa91f750af75d96a382" \
  "libdbusmenu-gtk3-16.04.0-32.fc45.x86_64.rpm:fcaf227fbc3c212f1c1b203d504a6fe585eb65740d16f25a0c4087f67b0d0eba"; do
    file="${spec%%:*}"
    hash="${spec#*:}"
    rpm_path="'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/${file}"
    echo "${hash}  ${rpm_path}" | sha256sum --check --status
  done
rpm --root "$ROOTFS" -Uvh --replacepkgs --noscripts \
  "'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/libdbusmenu-16.04.0-32.fc45.x86_64.rpm" \
  "'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/libdbusmenu-gtk3-16.04.0-32.fc45.x86_64.rpm" \
  "'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/libappindicator-gtk3-12.10.1-11.fc45.x86_64.rpm" \
  "'"${BLAZEOS_ROOT:-$(pwd)}"'/packages/mate-polkit-1.28.1-9.fc45.x86_64.rpm"

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

echo "  -> Restoring SUID/SGID permissions..."
chmod 4755 "$ROOTFS/usr/bin/sudo" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/su" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/pkexec" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/passwd" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/gpasswd" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/mount" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/bin/umount" 2>/dev/null || true
chmod 4755 "$ROOTFS/opt/FireHub/chrome-sandbox" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/firehub" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-house" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-apply-install-choices" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-desktop-provision" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-remove-solarui" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-wayland-session-ready" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-update" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-offline-update" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blazeos-offline-progress-filter" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/libexec/blaze-style-center-first-run" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-style-center" 2>/dev/null || true

echo "  -> Pre-installing trimmed Desktop Environments for Offline ISO..."
echo "nameserver 8.8.8.8" > "$ROOTFS/etc/resolv.conf"
chroot "$ROOTFS" dnf install -y --setopt=install_weak_deps=False \
  plasma-workspace plasma-desktop kde-settings dolphin konsole kde-connect kscreen kwin plasma-nm plasma-pa \
  plasma-lookandfeel-fedora arc-kde materia-kde papirus-icon-theme kvantum \
  gnome-shell-extension-dash-to-dock gnome-shell-extension-dash-to-panel gnome-shell-extension-apps-menu arc-theme \
  cosmic-session \
  cinnamon nemo \
  xfce4-session xfwm4 xfce4-panel thunar xfce4-terminal xfce4-settings \
  sway waybar 2>&1

echo "  -> Setting SELinux contexts with setfiles..."
LD_LIBRARY_PATH="$ROOTFS/usr/lib64" "$ROOTFS/usr/sbin/setfiles" -r "$ROOTFS" "$ROOTFS/etc/selinux/targeted/contexts/files/file_contexts" "$ROOTFS" 2>&1 | tail -10

echo "  -> Explicitly ensuring systemd and custom apps have correct labels..."
setfattr -n security.selinux -v "system_u:object_r:init_exec_t:s0" "$ROOTFS/usr/lib/systemd/systemd"
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-welcome" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/firehub" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/blaze-house" 2>/dev/null || true

echo "  -> Creating ZSTD-compressed squashfs.img for Offline ISO..."
rm -f "'"$NEW_SQUASHFS"'"
mksquashfs "$ROOTFS" "'"$NEW_SQUASHFS"'" \
  -comp zstd \
  -b 1M \
  -Xcompression-level 19 \
  -wildcards \
  -xattrs \
  -noappend \
  -processors $(nproc)

echo "  -> Cleaning up tmpfs..."
umount "$MNT"
rmdir "$MNT"
'

echo "=== [2/5] Updating Grub boot config for Offline ISO ==="
sed -i 's/Blaze-SolarEvolution-5/Blaze-SolarEvolution-5-Offline/g' "$GRUB_CFG" 2>/dev/null || true

echo "=== [3/5] Packaging Hybrid Offline ISO with xorriso ==="
rm -f "$OUT_ISO"

xorriso \
  -indev "$BASE_ISO" \
  -outdev "$OUT_ISO" \
  -volid "Blaze-SE-5" \
  -publisher "BlazeOS Project" \
  -application_id "Blaze SolarEvolution 5 Offline" \
  -map "$NEW_SQUASHFS" /LiveOS/squashfs.img \
  -map "$PATCHED_INITRD" /boot/x86_64/loader/initrd \
  -map "$GRUB_CFG" /boot/grub2/grub.cfg \
  -boot_image any replay \
  -padding 0

echo "=== [4/5] Implanting ISO MD5 checksum ==="
if command -v implantisomd5 >/dev/null 2>&1; then
    implantisomd5 "$OUT_ISO"
elif [ -x "${HOME}/bin/implantisomd5" ]; then
    "${HOME}/bin/implantisomd5" "$OUT_ISO"
else
    echo "  (implantisomd5 not found in PATH, skipping checksum embedding)"
fi

echo "=== [5/5] Verifying ISO MD5 checksum ==="
if command -v checkisomd5 >/dev/null 2>&1; then
    checkisomd5 --verbose "$OUT_ISO"
elif [ -x "${HOME}/bin/checkisomd5" ]; then
    "${HOME}/bin/checkisomd5" --verbose "$OUT_ISO"
else
    echo "  (checkisomd5 not found in PATH, skipping checksum verification)"
fi

echo "=== OFFLINE BUILD COMPLETE! ==="
ls -lh "$OUT_ISO"
sha256sum "$OUT_ISO" > "${OUT_ISO}.sha256"
echo "SHA-256: $(cut -d' ' -f1 "${OUT_ISO}.sha256")"
