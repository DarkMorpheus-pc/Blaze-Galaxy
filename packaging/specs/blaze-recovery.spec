%global debug_package %{nil}
%global __strip /bin/true
%global _build_id_links none
# Binary repack of artefacts built from the repositories listed in SOURCES.md
Name:           blaze-recovery
Version:        5.05
Release:        1%{?dist}
Summary:        BlazeOS BERP/BERE emergency recovery environment
License:        GPL-3.0-or-later
URL:            https://github.com/DarkMorpheus-pc/Blaze-Galaxy
Source0:        %{name}-%{version}.tar
ExclusiveArch:  x86_64
Requires:       python3

%description
Graphical recovery environment (BERE) with swipe-to-confirm actions, recovery backend and the terminal Screen of Death.

%prep
%setup -q -c

%build

%install
mkdir -p %{buildroot}
cp -a . %{buildroot}/
rm -f %{buildroot}/%{name}-%{version}.tar

%files
/usr/bin/blaze-recovery
/usr/bin/berp-recovery-gui
/usr/bin/blaze-recovery-gui-launcher
/usr/bin/blaze-death-screen
/usr/share/berp-recovery

%changelog
* Mon Oct 26 2026 Blaze Release <release@blazeos.com.tr> - 5.05-1
- Initial RPM packaging of blaze-recovery for the Blaze repository
