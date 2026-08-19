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
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 target/release/%{name} %{buildroot}/usr/bin/%{name}

mkdir -p %{buildroot}/usr/lib/systemd/system
install -m 0644 %{_sourcedir}/../ermete-ai-daemon.service %{buildroot}/usr/lib/systemd/system/%{name}.service || install -m 0644 ermete-ai-daemon.service %{buildroot}/usr/lib/systemd/system/%{name}.service

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
/usr/bin/%{name}
/usr/lib/systemd/system/%{name}.service

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-ai-daemon spec
