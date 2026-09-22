#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "==> Building SolarUI release binaries for Fedora..."
cd "${ROOT_DIR}"
cargo build --release

RPM_ROOT="${ROOT_DIR}/target/rpmbuild"
rm -rf "${RPM_ROOT}"
mkdir -p "${RPM_ROOT}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

cat <<'EOF' > "${RPM_ROOT}/SPECS/solarui.spec"
Name:           solarui
Version:        0.1.0
Release:        1%{?dist}
Summary:        Material 3 Next-Gen Wayland Desktop for BlazeOS
License:        GPL-3.0-or-later
URL:            https://github.com/projectblaze/solarui

Requires:       niri
Requires:       pipewire
Requires:       wireplumber
Requires:       NetworkManager
Requires:       upower
Requires:       systemd
Requires:       brightnessctl

%description
SolarUI is a high-performance, lightweight, and crash-resilient desktop environment
built in Rust, powered by Niri compositor, systemd, and Google's Material 3 design system.

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/wayland-sessions
mkdir -p %{buildroot}/etc/solarui
mkdir -p %{buildroot}/usr/lib/systemd/user

install -m 755 %{_topdir}/../../release/solar-core %{buildroot}/usr/bin/solar-core
install -m 755 %{_topdir}/../../release/solar-shell %{buildroot}/usr/bin/solar-shell
install -m 755 %{_topdir}/../../release/solar-session %{buildroot}/usr/bin/solar-session

install -m 644 %{_topdir}/../../../data/solarui.desktop %{buildroot}/usr/share/wayland-sessions/solarui.desktop
install -m 644 %{_topdir}/../../../data/niri.kdl %{buildroot}/etc/solarui/niri.kdl
install -m 644 %{_topdir}/../../../data/systemd/solarui-session.target %{buildroot}/usr/lib/systemd/user/solarui-session.target
install -m 644 %{_topdir}/../../../data/systemd/solar-core.service %{buildroot}/usr/lib/systemd/user/solar-core.service
install -m 644 %{_topdir}/../../../data/systemd/solar-shell.service %{buildroot}/usr/lib/systemd/user/solar-shell.service

%files
/usr/bin/solar-core
/usr/bin/solar-shell
/usr/bin/solar-session
/usr/share/wayland-sessions/solarui.desktop
/etc/solarui/niri.kdl
/usr/lib/systemd/user/solarui-session.target
/usr/lib/systemd/user/solar-core.service
/usr/lib/systemd/user/solar-shell.service

%changelog
* Fri Sep 11 2026 BlazeOS Team <team@blazeos.org> - 0.1.0-1
- Initial SolarUI packaging for BlazeOS Fedora Edition
EOF

echo "==> Packaging .rpm with rpmbuild..."
rpmbuild --define "_topdir ${RPM_ROOT}" -bb "${RPM_ROOT}/SPECS/solarui.spec"

mkdir -p "${ROOT_DIR}/target/packages"
cp "${RPM_ROOT}/RPMS/"*/*.rpm "${ROOT_DIR}/target/packages/" 2>/dev/null || true
echo "==> Successfully packaged SolarUI RPM for Fedora!"
