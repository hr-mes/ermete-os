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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/%{name}) && touch %{buildroot}/usr/bin/%{name}


%files
/usr/bin/%{name}

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-sysmon-ebpf spec
