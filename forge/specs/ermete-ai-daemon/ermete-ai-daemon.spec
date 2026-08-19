%global debug_package %{nil}
Name:           ermete-ai-daemon
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Local AI & Machine Learning Inference Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config openssl-devel

%description
Local AI and Machine Learning inference service for Ermete OS using Candle framework over D-Bus (os.ermete.AiDaemon).

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/%{name}) && touch %{buildroot}/usr/bin/%{name}
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/%{name}.service) && touch %{buildroot}/usr/lib/systemd/system/%{name}.service


%files
/usr/bin/%{name}
/usr/lib/systemd/system/%{name}.service

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-ai-daemon spec
