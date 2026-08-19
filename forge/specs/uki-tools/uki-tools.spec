%global debug_package %{nil}

Name:           uki-tools
Version:        1.0.0
Release:        1%{?dist}
Summary:        Native UKI (Unified Kernel Image) and Secure Boot signing toolchain for Ermete OS

License:        GPL-3.0-or-later AND LGPL-2.1-or-later AND MIT
URL:            https://github.com/hr-mes/ermete-os

Provides:       sbsigntools = %{version}-%{release}
Provides:       sbsigntools = 0.9.5
Obsoletes:      sbsigntools < %{version}-%{release}

Provides:       systemd-ukify = %{version}-%{release}
Provides:       systemd-ukify = 258.9
Obsoletes:      systemd-ukify < %{version}-%{release}

Provides:       sbsign = %{version}-%{release}
Provides:       ukify = %{version}-%{release}

Requires:       python3
Requires:       openssl
Requires:       systemd

%description
Assimilated UKI toolchain packaging sbsigntools (sbsign, sbverify, sbattach,
sbkeysync, sbsiglist, sbvarsign) and systemd-ukify (ukify) natively within
Ermete OS Forge to guarantee 100% autarchic boot generation and CI pipelines.

%prep
# Source files provided in SOURCES directory

%build
# Pre-compiled native binaries and Python tools

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_prefix}/lib/systemd
mkdir -p %{buildroot}%{_prefix}/lib/kernel/install.d

# Install sbsigntools binaries
install -m 0755 %{_sourcedir}/sbsign %{buildroot}%{_bindir}/sbsign
install -m 0755 %{_sourcedir}/sbverify %{buildroot}%{_bindir}/sbverify
install -m 0755 %{_sourcedir}/sbattach %{buildroot}%{_bindir}/sbattach
install -m 0755 %{_sourcedir}/sbkeysync %{buildroot}%{_bindir}/sbkeysync
install -m 0755 %{_sourcedir}/sbsiglist %{buildroot}%{_bindir}/sbsiglist
install -m 0755 %{_sourcedir}/sbvarsign %{buildroot}%{_bindir}/sbvarsign

# Install ukify python tool and kernel install plugin
install -m 0755 %{_sourcedir}/ukify %{buildroot}%{_bindir}/ukify
install -m 0755 %{_sourcedir}/60-ukify.install %{buildroot}%{_prefix}/lib/kernel/install.d/60-ukify.install

# Symlink systemd-ukify path to bin/ukify
ln -sf ../../bin/ukify %{buildroot}%{_prefix}/lib/systemd/ukify

%files
%{_bindir}/sbsign
%{_bindir}/sbverify
%{_bindir}/sbattach
%{_bindir}/sbkeysync
%{_bindir}/sbsiglist
%{_bindir}/sbvarsign
%{_bindir}/ukify
%{_prefix}/lib/systemd/ukify
%{_prefix}/lib/kernel/install.d/60-ukify.install

%changelog
* Sat Aug 08 2026 Ermete Architect <admin@ermete.os> - 1.0.0-1
- Assimilate sbsigntools and systemd-ukify into native uki-tools spec.
