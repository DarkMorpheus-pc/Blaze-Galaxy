Name:           blaze-release
Version:        5.05
Release:        1%{?dist}
Summary:        Blaze repository definition and signing key
License:        GPL-3.0-or-later
URL:            https://github.com/DarkMorpheus-pc/Blaze-Galaxy
BuildArch:      noarch
Source0:        blaze.repo
Source1:        RPM-GPG-KEY-blaze

%description
Installs the Blaze package repository definition and the public GPG key used to
verify Blaze packages, so that dnf update delivers Blaze component updates.

%prep

%build

%install
install -Dpm0644 %{SOURCE0} %{buildroot}%{_sysconfdir}/yum.repos.d/blaze.repo
install -Dpm0644 %{SOURCE1} %{buildroot}%{_sysconfdir}/pki/rpm-gpg/RPM-GPG-KEY-blaze

%files
%config(noreplace) %{_sysconfdir}/yum.repos.d/blaze.repo
%{_sysconfdir}/pki/rpm-gpg/RPM-GPG-KEY-blaze

%changelog
* Mon Oct 26 2026 Blaze Release <release@blazeos.com.tr> - 5.05-1
- Initial blaze-release package
