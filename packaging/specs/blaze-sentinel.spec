%global debug_package %{nil}
%global __strip /bin/true
%global _build_id_links none
# Binary repack of artefacts built from the repositories listed in SOURCES.md
Name:           blaze-sentinel
Version:        0.3.0
Release:        1%{?dist}
Summary:        Blaze Sentinel security posture inspector
License:        MIT or GPL-3.0-or-later
URL:            https://github.com/DarkMorpheus-pc/Blaze-Galaxy
Source0:        %{name}-%{version}.tar
ExclusiveArch:  x86_64

%description
Reports SELinux/AppArmor state, Secure Boot, kernel lockdown, firewall and kernel hardening sysctls; supports --json and verify exit codes.

%prep
%setup -q -c

%build

%install
mkdir -p %{buildroot}
cp -a . %{buildroot}/
rm -f %{buildroot}/%{name}-%{version}.tar

%files
/usr/bin/blaze-sentinel

%changelog
* Mon Oct 26 2026 Blaze Release <release@blazeos.com.tr> - 0.3.0-1
- Initial RPM packaging of blaze-sentinel for the Blaze repository
