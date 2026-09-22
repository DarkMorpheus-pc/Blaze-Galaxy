#!/bin/bash
set -euo pipefail

BASE_ISO="${BLAZEOS_ROOT:-$(pwd)}/f45_base/Fedora-Workstation-Live-45_Beta-1.3.x86_64.iso"
OUT_ISO="${BLAZEOS_ROOT:-$(pwd)}/Blaze-SolarEvolution-5-x86_64.iso"
WORK="${BLAZEOS_ROOT:-$(pwd)}/work_f45"
ROOTFS_SRC="$WORK/rootfs"
NEW_SQUASHFS="$WORK/LiveOS/squashfs.img"
PATCHED_INITRD="$WORK/initrd_work/patched_initrd"
GRUB_CFG="$WORK/iso_mods/boot/grub2/grub.cfg"

echo "=== [1/5] Building Squashfs with full SELinux labels in tmpfs ==="
unshare -rm bash -c '
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
chmod 4755 "$ROOTFS/usr/bin/chvt" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/lib/polkit-1/polkit-agent-helper-1" 2>/dev/null || true
chmod 4755 "$ROOTFS/usr/libexec/dbus-1/dbus-daemon-launch-helper" 2>/dev/null || true
chmod 4755 "$ROOTFS/opt/FireHub/chrome-sandbox" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/unix_chkpwd" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/crontab" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/at" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/chage" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/bin/lockdev" 2>/dev/null || true
chmod 2755 "$ROOTFS/usr/libexec/utempter/utempter" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/bin/firehub" 2>/dev/null || true
chmod 755 "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true

echo "  -> Setting SELinux contexts with setfiles..."
LD_LIBRARY_PATH="$ROOTFS/usr/lib64" "$ROOTFS/usr/sbin/setfiles" -r "$ROOTFS" "$ROOTFS/etc/selinux/targeted/contexts/files/file_contexts" "$ROOTFS" 2>&1 | tail -10 || true

echo "  -> Explicitly ensuring systemd and custom apps have correct labels..."
setfattr -n security.selinux -v "system_u:object_r:init_exec_t:s0" "$ROOTFS/usr/lib/systemd/systemd"
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-welcome" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-nvidia-setup" 2>/dev/null || true
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
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/opt/FireHub/firehub" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:systemd_unit_file_t:s0" "$ROOTFS/etc/systemd/system/blazeos-nvidia-firstboot.service" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-core" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/solar-shell" 2>/dev/null || true
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
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/libexec/livesys/sessions.d/livesys-solarui" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS/usr/lib64/solarui/libsolar_brand.so" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS/usr/lib/solarui/libsolar_brand.so" 2>/dev/null || true
find "$ROOTFS/usr/lib64/quickshell" -exec setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" {} + 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libseat.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libyyjson.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libgtksourceview-3.0.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libgspell-1.so* 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:lib_t:s0" "$ROOTFS"/usr/lib64/libicu*.so* 2>/dev/null || true

echo "  -> Verifying critical SELinux labels..."
getfattr -d -m - "$ROOTFS/usr/lib/systemd/systemd"
getfattr -d -m - "$ROOTFS/usr/bin/sudo"
getfattr -d -m - "$ROOTFS/usr/local/bin/blazeos-welcome"
getfattr -d -m - "$ROOTFS/usr/local/bin/blazeos-control"

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
implantisomd5 "$OUT_ISO"

echo "=== [5/5] Verifying ISO checksum ==="
checkisomd5 --verbose "$OUT_ISO"

echo "=== BUILD COMPLETE! ==="
