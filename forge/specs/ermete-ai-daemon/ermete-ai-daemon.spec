%global debug_package %{nil}
Name:           ermete-ai-daemon
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Local AI & Machine Learning Inference Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config openssl-devel

%description
Local AI and Machine Learning inference service for Ermete OS using Candle framework over D-Bus (os.ermete.AiDaemon).

%prep
%autosetup

%build
%set_build_flags
cargo generate-lockfile
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%{_bindir}/%{name}

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-ai-daemon spec
