#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "==> Building SolarUI (Release Mode)..."
cargo build --manifest-path "${ROOT_DIR}/Cargo.toml" --release --locked

echo "==> Building optional branding library..."
gcc -O2 -fPIC -shared -pthread -o "${ROOT_DIR}/data/libsolar_brand.so" "${ROOT_DIR}/libsolar_brand.c" -ldl

PREFIX="${PREFIX:-/usr}"
SYSCONFDIR="${SYSCONFDIR:-/etc}"

echo "==> Installing SolarUI to ${PREFIX} (sudo may be required)..."

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
    SUDO="sudo"
fi

$SUDO install -d "${PREFIX}/bin"
$SUDO install -d "${PREFIX}/lib/solarui"
$SUDO install -d "${PREFIX}/share/wayland-sessions"
$SUDO install -d "${PREFIX}/share/applications"
$SUDO install -d "${PREFIX}/share/solarui"
$SUDO install -d "${PREFIX}/share/icons/hicolor/scalable/apps"
$SUDO install -d "${PREFIX}/share/icons/hicolor/256x256/apps"
$SUDO install -d "${PREFIX}/share/icons/hicolor/48x48/apps"
$SUDO install -d "${SYSCONFDIR}/solarui"
$SUDO install -d "${SYSCONFDIR}/fastfetch"
$SUDO install -d "${PREFIX}/lib/systemd/user"

$SUDO install -m 755 "${ROOT_DIR}/target/release/solar-core" "${PREFIX}/bin/solar-core"
$SUDO install -m 755 "${ROOT_DIR}/target/release/solar-shell" "${PREFIX}/bin/solar-shell"
$SUDO install -m 755 "${ROOT_DIR}/target/release/solar-session" "${PREFIX}/bin/solar-session"

$SUDO install -m 644 "${ROOT_DIR}/data/solarui.desktop" "${PREFIX}/share/wayland-sessions/solarui.desktop"
$SUDO install -m 644 "${ROOT_DIR}/data/solar-settings.desktop" "${PREFIX}/share/applications/solar-settings.desktop"
$SUDO install -m 644 "${ROOT_DIR}/data/solar-welcome.desktop" "${PREFIX}/share/applications/solar-welcome.desktop"

$SUDO install -m 644 "${ROOT_DIR}/data/niri.kdl" "${SYSCONFDIR}/solarui/niri.kdl"
$SUDO install -m 644 "${ROOT_DIR}/data/niri.kdl" "${PREFIX}/share/solarui/niri.kdl"

$SUDO install -m 644 "${ROOT_DIR}/data/icons/solarui.svg" "${PREFIX}/share/icons/hicolor/scalable/apps/solarui.svg"
$SUDO install -m 644 "${ROOT_DIR}/data/icons/solar-logo-256.png" "${PREFIX}/share/icons/hicolor/256x256/apps/solarui.png"
$SUDO install -m 644 "${ROOT_DIR}/data/icons/solar-logo-48.png" "${PREFIX}/share/icons/hicolor/48x48/apps/solarui.png"

# Install SolarUI brand interception library
$SUDO install -m 755 "${ROOT_DIR}/data/libsolar_brand.so" "${PREFIX}/lib/solarui/libsolar_brand.so"

# Install SolarUI branded Fastfetch configuration
$SUDO install -m 644 "${ROOT_DIR}/data/fastfetch/config.jsonc" "${SYSCONFDIR}/fastfetch/config.jsonc"

# Install SolarUI branded Noctalia assets
$SUDO rm -rf "${PREFIX}/share/solarui/noctalia-assets"
$SUDO cp -r "${ROOT_DIR}/data/noctalia-assets" "${PREFIX}/share/solarui/noctalia-assets"

$SUDO install -m 644 "${ROOT_DIR}/data/systemd/solarui-session.target" "${PREFIX}/lib/systemd/user/solarui-session.target"
$SUDO install -m 644 "${ROOT_DIR}/data/systemd/solar-core.service" "${PREFIX}/lib/systemd/user/solar-core.service"
$SUDO install -m 644 "${ROOT_DIR}/data/systemd/solar-shell.service" "${PREFIX}/lib/systemd/user/solar-shell.service"

if command -v gtk-update-icon-cache &>/dev/null; then
    $SUDO gtk-update-icon-cache -q -t -f "${PREFIX}/share/icons/hicolor" 2>/dev/null || true
fi

echo "==> Installation complete!"
echo "You can now select 'SolarUI' from SDDM (Display Manager)!"
