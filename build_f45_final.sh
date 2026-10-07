#!/bin/bash
set -euo pipefail
export PATH="/home/darkmorpheus/bin:/home/darkmorpheus/.local/bin:/usr/local/bin:/usr/local/sbin:$PATH"

BASE_ISO="${BLAZEOS_ROOT:-$(pwd)}/f45_base/Fedora-Workstation-Live-45_Beta-1.3.x86_64.iso"
OUT_ISO="${BLAZEOS_ROOT:-$(pwd)}/Blaze-SolarEvolution-5-x86_64.iso"
WORK="${BLAZEOS_ROOT:-$(pwd)}/work_f45"
ROOTFS_SRC="$WORK/rootfs"
NEW_SQUASHFS="$WORK/LiveOS/squashfs.img"
PATCHED_INITRD="$WORK/initrd_work/patched_initrd"
GRUB_CFG="$WORK/iso_mods/boot/grub2/grub.cfg"
OVERLAY="${BLAZEOS_ROOT:-$(pwd)}/blazeos_custom_apps"

echo "=== [0/5] Validating tracked installer and first-boot code ==="
grep -qx 'VERSION="5.04"' "$OVERLAY/etc/os-release"
grep -qx 'VERSION_ID=5.04' "$OVERLAY/etc/os-release"
grep -qx 'PRETTY_NAME="Blaze SolarEvolution 5.04"' "$OVERLAY/etc/os-release"
bash -n \
    "$OVERLAY/usr/libexec/livesys/sessions.d/livesys-solarui" \
    "$OVERLAY/usr/libexec/blazeos-oobe-hygiene" \
    "$OVERLAY/usr/local/bin/blazeos-postinstall" \
    "$OVERLAY/usr/local/bin/blaze-bootloader" \
    "$OVERLAY/usr/local/bin/blazeos-limine-install" \
    "$OVERLAY/usr/bin/blaze-system-monitor"
node --check "$OVERLAY/usr/share/cockpit/anaconda-webui/index.js"
python3 -c 'compile(open("'"$OVERLAY/usr/bin/blaze-setup"'", encoding="utf-8").read(), "blaze-setup", "exec")'
python3 -c 'compile(open("'"$OVERLAY/usr/bin/berp-recovery-gui"'", encoding="utf-8").read(), "berp-recovery-gui", "exec")'
if grep -Eq '/root/var/lib/mock|image-root' "$OVERLAY/boot/efi/EFI/fedora/grub.cfg"; then
    echo "Refusing to build: EFI grub.cfg contains a build-host path" >&2
    exit 1
fi
grep -q 'search --no-floppy --file --set=dev /grub2/grub.cfg' \
    "$OVERLAY/boot/efi/EFI/fedora/grub.cfg"
test -L "$OVERLAY/etc/systemd/system/graphical.target.wants/blazeos-oobe-hygiene.service"
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
chmod 755 "$ROOTFS/usr/bin/caelestia" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/quickshell" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/quickshell.bin" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/qs" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/sddm" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/weston" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/berp-recovery-gui" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-setup" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/blaze-system-monitor" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/gen_grub_cfgstub" 2>/dev/null || true
chmod 755 "$ROOTFS/etc/grub.d/09_berp" 2>/dev/null || true
chmod 755 "$ROOTFS/etc/grub.d/99_berp" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/solar-torture" 2>/dev/null || true
chmod 600 "$ROOTFS/etc/NetworkManager/system-connections/"*.nmconnection 2>/dev/null || true

echo "  -> Ensuring SDDM default display manager and masking GDM..."
mkdir -p "$ROOTFS/etc/systemd/system"
mkdir -p "$ROOTFS/etc/systemd/system/graphical.target.wants"
mkdir -p "$ROOTFS/etc/systemd/system/multi-user.target.wants"
ln -sf /usr/lib/systemd/system/sddm.service "$ROOTFS/etc/systemd/system/display-manager.service"
ln -sf /usr/lib/systemd/system/sddm.service "$ROOTFS/etc/systemd/system/graphical.target.wants/sddm.service"
ln -sf /usr/lib/systemd/system/sddm.service "$ROOTFS/etc/systemd/system/multi-user.target.wants/display-manager.service"
ln -sf /usr/lib/systemd/system/graphical.target "$ROOTFS/etc/systemd/system/default.target"
ln -sf /dev/null "$ROOTFS/etc/systemd/system/gdm.service"

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
elif [ -x "/home/darkmorpheus/bin/implantisomd5" ]; then
    /home/darkmorpheus/bin/implantisomd5 "$OUT_ISO"
else
    echo "  (implantisomd5 not found in PATH, skipping checksum embedding)"
fi

echo "=== [5/5] Verifying ISO checksum ==="
if command -v checkisomd5 >/dev/null 2>&1; then
    checkisomd5 --verbose "$OUT_ISO"
elif [ -x "/home/darkmorpheus/bin/checkisomd5" ]; then
    /home/darkmorpheus/bin/checkisomd5 --verbose "$OUT_ISO"
else
    echo "  (checkisomd5 not found in PATH, skipping checksum verification)"
fi

echo "=== BUILD COMPLETE! ==="
ls -lh "$OUT_ISO"
