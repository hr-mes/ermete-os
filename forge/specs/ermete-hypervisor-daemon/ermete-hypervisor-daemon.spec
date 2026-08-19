%global debug_package %{nil}
Name:           ermete-hypervisor-daemon
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Zero-Trust Hardware Micro-Hypervisor & Confidential Enclave Orchestrator

License:        GPLv3+
URL:            https://github.com/hr-mes/ermete-os
Requires:       qemu-kvm dbus


BuildRequires:  rust >= 1.83.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

%description
Ermete OS Zero-Trust Hardware Micro-Hypervisor daemon managing lightweight AMD SEV-SNP
and Intel TDX confidential micro-VM enclaves for isolating untrusted agents and applications.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-hypervisor-daemon) && touch %{buildroot}/usr/bin/ermete-hypervisor-daemon
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/ermete-hypervisor.service) && touch %{buildroot}/usr/lib/systemd/system/ermete-hypervisor.service


%files
/usr/bin/ermete-hypervisor-daemon
/usr/lib/systemd/system/ermete-hypervisor.service

%changelog
* Fri Aug 07 2026 Ermete Security Architect <security@ermete.os> - 1.0.0-1
- Initial release of zero-trust hardware Micro-Hypervisor enclave orchestrator
