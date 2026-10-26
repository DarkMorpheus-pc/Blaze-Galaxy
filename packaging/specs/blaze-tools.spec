%global debug_package %{nil}
%global __strip /bin/true
%global _build_id_links none
# Binary repack of artefacts built from the repositories listed in SOURCES.md
Name:           blaze-tools
Version:        5.05
Release:        1%{?dist}
Summary:        BlazeOS helper tools and installers
License:        GPL-3.0-or-later
URL:            https://github.com/DarkMorpheus-pc/Blaze-Galaxy
Source0:        %{name}-%{version}.tar
BuildArch:      noarch
Requires:       bash
Requires:       python3

%description
Post-install, desktop provisioning, NVIDIA/Waydroid/Limine helpers, bootloader switcher, optimizer, session diagnostics and the solar-torture soak test.

%prep
%setup -q -c

%build

%install
mkdir -p %{buildroot}
cp -a . %{buildroot}/
rm -f %{buildroot}/%{name}-%{version}.tar

%files
/usr/local/bin/blaze-bootloader
/usr/local/bin/blaze-optimize
/usr/local/bin/blazeos-apply-install-choices
/usr/local/bin/blazeos-control
/usr/local/bin/blazeos-desktop-provision
/usr/local/bin/blazeos-limine-install
/usr/local/bin/blazeos-nvidia-setup
/usr/local/bin/blazeos-postinstall
/usr/local/bin/blazeos-remove-solarui
/usr/local/bin/blazeos-session-diagnose
/usr/local/bin/blazeos-waydroid-setup
/usr/local/bin/blazeos-welcome
/usr/local/bin/solar-torture
/usr/local/bin/blaze-snapshot

%changelog
* Mon Oct 26 2026 Blaze Release <release@blazeos.com.tr> - 5.05-1
- Initial RPM packaging of blaze-tools for the Blaze repository
