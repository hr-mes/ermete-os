%global debug_package %{nil}
Name:           ermete-sysmon-ebpf
Version:        1.0.0
Release:        1%{?dist}
Summary:        eBPF System Monitoring & Telemetry Daemon for Ermete OS

License:        GPL-2.0-or-later
URL:            https://github.com/hr-mes/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config

%description
System monitoring and telemetry daemon leveraging Aya eBPF for kernel-level performance tracking in Ermete OS.

%prep
%autosetup

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%{_bindir}/%{name}

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-sysmon-ebpf spec
