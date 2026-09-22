#!/bin/bash
set -euo pipefail

ORIG_ISO="/home/darkmorpheus/BlazeFedora/BlazeOS-Galaxy-SolarEvolution-x86_64-T1.iso"
OUT_ISO="/home/darkmorpheus/BlazeFedora/BlazeOS-Galaxy-SolarEvolution2-x86_64-T2.iso"
WORK="/home/darkmorpheus/BlazeFedora/work"
CUSTOM_ROOTFS="$WORK/squashfs_root"
NEW_SQUASHFS="$WORK/iso_root/LiveOS/squashfs.img"

echo "=== [1/6] Extracting original squashfs from ISO ==="
rm -f /tmp/orig_squashfs.img
bsdtar -xf "$ORIG_ISO" -C /tmp LiveOS/squashfs.img
mv /tmp/LiveOS/squashfs.img /tmp/orig_squashfs.img
rmdir /tmp/LiveOS 2>/dev/null || true
ls -lh /tmp/orig_squashfs.img

echo "=== [2/6] Unpacking and patching inside user namespace (with SELinux xattrs) ==="
unshare -rm bash -c '
set -euo pipefail
MNT="/tmp/sq_build_mnt"
mkdir -p "$MNT"
mount -t tmpfs -o size=24G tmpfs "$MNT"
ROOTFS="$MNT/rootfs"

echo "  -> Unsquashfs with xattrs into tmpfs..."
unsquashfs -d "$ROOTFS" /tmp/orig_squashfs.img
rm -f /tmp/orig_squashfs.img

echo "  -> Verifying SELinux xattrs exist..."
getfattr -n security.selinux "$ROOTFS/usr/bin/sudo"

echo "  -> Applying custom FireHub 1.8.1..."
rm -rf "$ROOTFS/opt/FireHub"
cp -a "'"$CUSTOM_ROOTFS"'"/opt/FireHub "$ROOTFS/opt/"
cp -a "'"$CUSTOM_ROOTFS"'"/usr/share/applications/firehub.desktop "$ROOTFS/usr/share/applications/"
cp -a "'"$CUSTOM_ROOTFS"'"/usr/share/icons/hicolor/512x512/apps/firehub.png "$ROOTFS/usr/share/icons/hicolor/512x512/apps/"
cp -a "'"$CUSTOM_ROOTFS"'"/usr/share/mime/packages/firehub.xml "$ROOTFS/usr/share/mime/packages/"

echo "  -> Applying Niri to Welcome & Control..."
cp "'"$CUSTOM_ROOTFS"'"/usr/local/bin/blazeos-welcome "$ROOTFS/usr/local/bin/"
cp "'"$CUSTOM_ROOTFS"'"/usr/local/bin/blazeos-control "$ROOTFS/usr/local/bin/"
mkdir -p "$ROOTFS/usr/share/wayland-sessions"
cp "'"$CUSTOM_ROOTFS"'"/usr/share/wayland-sessions/niri.desktop "$ROOTFS/usr/share/wayland-sessions/"
mkdir -p "$ROOTFS/etc/skel/.config"
cp -r "'"$CUSTOM_ROOTFS"'"/etc/skel/.config/niri "$ROOTFS/etc/skel/.config/"

echo "  -> Applying NVIDIA firstboot setup..."
cp "'"$CUSTOM_ROOTFS"'"/usr/local/bin/blazeos-nvidia-setup "$ROOTFS/usr/local/bin/"
cp "'"$CUSTOM_ROOTFS"'"/etc/systemd/system/blazeos-nvidia-firstboot.service "$ROOTFS/etc/systemd/system/"
mkdir -p "$ROOTFS/etc/systemd/system/multi-user.target.wants"
ln -sf /etc/systemd/system/blazeos-nvidia-firstboot.service "$ROOTFS/etc/systemd/system/multi-user.target.wants/blazeos-nvidia-firstboot.service"

echo "  -> Applying Limine installer script..."
cp "'"$CUSTOM_ROOTFS"'"/usr/local/bin/blazeos-limine-install "$ROOTFS/usr/local/bin/"
cp "'"$CUSTOM_ROOTFS"'"/etc/blazeos-release "$ROOTFS/etc/"

echo "  -> Applying new boot logo..."
cp /home/darkmorpheus/İndirilenler/logo.icon/boot.png "$ROOTFS/usr/share/plymouth/themes/blazeos/logo.png"

echo "  -> Setting SELinux contexts with setfiles..."
LD_LIBRARY_PATH="$ROOTFS/usr/lib64" "$ROOTFS/usr/sbin/setfiles" -r "$ROOTFS" "$ROOTFS/etc/selinux/targeted/contexts/files/file_contexts" "$ROOTFS" 2>&1 | tail -5 || true

echo "  -> Recompressing squashfs with full xattrs and SELinux..."
rm -f "'"$NEW_SQUASHFS"'"
mksquashfs "$ROOTFS" "'"$NEW_SQUASHFS"'" \
    -comp zstd \
    -Xcompression-level 15 \
    -b 131072 \
    -no-progress \
    -noappend \
    -xattrs

umount "$MNT"
rm -rf "$MNT"
echo "  -> Squashfs build done!"
'

echo "=== [3/6] Verifying xattrs in new squashfs ==="
unsquashfs -s "$NEW_SQUASHFS"

echo "=== [4/6] Building hybrid GPT ISO with xorriso ==="
rm -f "$OUT_ISO"
xorriso \
  -indev "$ORIG_ISO" \
  -outdev "$OUT_ISO" \
  -map "$NEW_SQUASHFS" /LiveOS/squashfs.img \
  -map "$WORK/iso_root/boot/grub2/grub.cfg" /boot/grub2/grub.cfg \
  -map "$WORK/iso_root/EFI/BOOT/grub.cfg" /EFI/BOOT/grub.cfg \
  -boot_image any replay \
  -padding 0

echo "=== [5/6] Implanting MD5 checksum ==="
/home/darkmorpheus/bin/implantisomd5 "$OUT_ISO"

echo "=== [6/6] Verifying ISO with checkisomd5 ==="
/home/darkmorpheus/bin/checkisomd5 --verbose "$OUT_ISO"

echo "=== ALL DONE SUCCESSFULLY! ==="
