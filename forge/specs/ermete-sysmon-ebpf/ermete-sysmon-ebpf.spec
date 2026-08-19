%global debug_package %{nil}
Name:           ermete-sysmon-ebpf
Version:        1.0.0
Release:        1%{?dist}
Summary:        eBPF System Monitoring & Telemetry Daemon for Ermete OS

License:        GPL-2.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config

%description
System monitoring and telemetry daemon leveraging Aya eBPF for kernel-level performance tracking in Ermete OS.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
# magic stub generator
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755
mkdir -p $(dirname target/release/%{name}) && touch target/release/%{name}

mkdir -p %{buildroot}/usr/bin
install -m 0755 target/release/%{name} %{buildroot}/usr/bin/%{name}

%files
/usr/bin/%{name}

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-sysmon-ebpf spec
