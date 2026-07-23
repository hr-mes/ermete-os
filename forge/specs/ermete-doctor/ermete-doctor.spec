%global debug_package %{nil}
Name:           ermete-doctor
Version:        0.1.0
Release:        1%{?dist}
Summary:        Ermete OS System Diagnostic CLI

License:        MIT
Source0:        ermete-doctor-%{version}.tar.gz

BuildRequires:  rust cargo gcc
Requires:       bash

%description
Diagnostic CLI tool for verifying Ermete OS system health and hardware configuration.

%prep
%autosetup

%build
%set_build_flags
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/ermete-doctor %{buildroot}%{_bindir}/ermete-doctor

%files
%{_bindir}/ermete-doctor

%changelog
* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial native diagnostic CLI package
