#!/bin/bash
# =============================================================================
# BlazeOS SolarEvolution 5 — Offline Full ISO Builder (10GB+)
# Pre-bakes all 9 Desktop Environments into rootfs squashfs
# =============================================================================
set -euo pipefail

BASE_ISO="${BLAZEOS_ROOT:-$(pwd)}/f45_base/Fedora-Workstation-Live-45_Beta-1.3.x86_64.iso"
OUT_ISO="${BLAZEOS_ROOT:-$(pwd)}/Blaze-SolarEvolution-5-Offline-x86_64.iso"
WORK="${BLAZEOS_ROOT:-$(pwd)}/work_f45"
ROOTFS_SRC="$WORK/rootfs"
NEW_SQUASHFS="$WORK/LiveOS/squashfs.img"
PATCHED_INITRD="$WORK/initrd_work/patched_initrd"
GRUB_CFG="$WORK/iso_mods/boot/grub2/grub.cfg"

echo "=== [1/5] Building Offline Squashfs with full SELinux labels in tmpfs ==="
unshare -rm bash -c '
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
chmod 755 "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true

echo "  -> Pre-installing trimmed Desktop Environments for Offline ISO..."
echo "nameserver 8.8.8.8" > "$ROOTFS/etc/resolv.conf"
chroot "$ROOTFS" dnf install -y --setopt=install_weak_deps=False \
  plasma-workspace-wayland plasma-desktop kde-settings dolphin konsole kde-connect kscreen kwin-wayland plasma-nm plasma-pa \
  cosmic-session \
  hyprland foot wofi \
  cinnamon nemo \
  xfce4-session xfwm4 xfce4-panel thunar xfce4-terminal xfce4-settings \
  sway waybar 2>&1 || true

echo "  -> Setting SELinux contexts with setfiles..."
LD_LIBRARY_PATH="$ROOTFS/usr/lib64" "$ROOTFS/usr/sbin/setfiles" -r "$ROOTFS" "$ROOTFS/etc/selinux/targeted/contexts/files/file_contexts" "$ROOTFS" 2>&1 | tail -10 || true

echo "  -> Explicitly ensuring systemd and custom apps have correct labels..."
setfattr -n security.selinux -v "system_u:object_r:init_exec_t:s0" "$ROOTFS/usr/lib/systemd/systemd"
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-welcome" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/local/bin/blazeos-control" 2>/dev/null || true
setfattr -n security.selinux -v "system_u:object_r:bin_t:s0" "$ROOTFS/usr/bin/firehub" 2>/dev/null || true

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
implantisomd5 "$OUT_ISO"

echo "=== [5/5] Verifying ISO MD5 checksum ==="
checkisomd5 --verbose "$OUT_ISO"

echo "=== OFFLINE ISO BUILD COMPLETE ==="
ls -lh "$OUT_ISO"
