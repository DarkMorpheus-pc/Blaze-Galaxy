#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "==> Building SolarUI release binaries for Debian..."
cd "${ROOT_DIR}"
cargo build --release

BUILD_ROOT="${ROOT_DIR}/target/deb-package"
rm -rf "${BUILD_ROOT}"
mkdir -p "${BUILD_ROOT}/DEBIAN"
mkdir -p "${BUILD_ROOT}/usr/bin"
mkdir -p "${BUILD_ROOT}/usr/share/wayland-sessions"
mkdir -p "${BUILD_ROOT}/etc/solarui"
mkdir -p "${BUILD_ROOT}/usr/lib/systemd/user"

echo "==> Installing binaries into package root..."
install -m 755 "${ROOT_DIR}/target/release/solar-core" "${BUILD_ROOT}/usr/bin/solar-core"
install -m 755 "${ROOT_DIR}/target/release/solar-shell" "${BUILD_ROOT}/usr/bin/solar-shell"
install -m 755 "${ROOT_DIR}/target/release/solar-session" "${BUILD_ROOT}/usr/bin/solar-session"

echo "==> Installing configuration and session files..."
install -m 644 "${ROOT_DIR}/data/solarui.desktop" "${BUILD_ROOT}/usr/share/wayland-sessions/solarui.desktop"
install -m 644 "${ROOT_DIR}/data/niri.kdl" "${BUILD_ROOT}/etc/solarui/niri.kdl"
install -m 644 "${ROOT_DIR}/data/systemd/solarui-session.target" "${BUILD_ROOT}/usr/lib/systemd/user/solarui-session.target"
install -m 644 "${ROOT_DIR}/data/systemd/solar-core.service" "${BUILD_ROOT}/usr/lib/systemd/user/solar-core.service"
install -m 644 "${ROOT_DIR}/data/systemd/solar-shell.service" "${BUILD_ROOT}/usr/lib/systemd/user/solar-shell.service"

cat <<'EOF' > "${BUILD_ROOT}/DEBIAN/control"
Package: solarui
Version: 0.1.0
Section: x11
Priority: optional
Architecture: amd64
Depends: niri, pipewire, wireplumber, network-manager, upower, systemd, brightnessctl
Maintainer: BlazeOS Core Team <team@blazeos.org>
Description: SolarUI - Material 3 Next-Gen Wayland Desktop for BlazeOS
 High-performance, lightweight, and crash-resilient desktop environment
 powered by Niri compositor, Rust system broker, and Material 3 design.
EOF

echo "==> Packaging .deb..."
mkdir -p "${ROOT_DIR}/target/packages"
dpkg-deb --build "${BUILD_ROOT}" "${ROOT_DIR}/target/packages/solarui_0.1.0_amd64.deb"

echo "==> Successfully generated: ${ROOT_DIR}/target/packages/solarui_0.1.0_amd64.deb"
