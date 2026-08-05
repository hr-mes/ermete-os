%global debug_package %{nil}
Name:           ermete-live-patcher
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Kernel Live Patcher Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config
Requires:       polkit kpatch

%description
Daemon for zero-downtime ring-0 kernel live patching in Ermete OS over D-Bus with Polkit integration.

%prep
%autosetup

%build
%set_build_flags
cargo generate-lockfile
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

mkdir -p %{buildroot}%{_datadir}/polkit-1/actions
install -m 0644 os.ermete.livepatcher.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.livepatcher.policy

%files
%{_bindir}/%{name}
%{_datadir}/polkit-1/actions/os.ermete.livepatcher.policy

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-live-patcher spec
