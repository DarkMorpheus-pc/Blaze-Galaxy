%global debug_package %{nil}
%global __strip /bin/true
%global _build_id_links none
# Binary repack of artefacts built from the repositories listed in SOURCES.md
Name:           blaze-continuum
Version:        0.1.0
Release:        1%{?dist}
Summary:        Blaze Continuum stateful session teleportation (experimental)
License:        MIT or GPL-3.0-or-later
URL:            https://github.com/DarkMorpheus-pc/Blaze-Galaxy
Source0:        %{name}-%{version}.tar
ExclusiveArch:  x86_64
Requires:       criu
Requires:       python3-gobject
Requires:       gtk4

%description
CRIU based freeze/restore of a window or session into a .blz-state capsule and LAN transfer over BCTP/1, with a Ghost Overlay surface. Experimental.

%prep
%setup -q -c

%build

%install
mkdir -p %{buildroot}
cp -a . %{buildroot}/
rm -f %{buildroot}/%{name}-%{version}.tar

%files
/usr/bin/blaze-continuum
/usr/bin/blaze-ghost-overlay

%changelog
* Mon Oct 26 2026 Blaze Release <release@blazeos.com.tr> - 0.1.0-1
- Initial RPM packaging of blaze-continuum for the Blaze repository
