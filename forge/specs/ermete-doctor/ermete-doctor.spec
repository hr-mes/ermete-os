%global debug_package %{nil}
Name:           ermete-doctor
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS System Diagnostic CLI

License:        MIT


BuildRequires:  rust cargo gcc
Requires: bash
Requires:       iputils

%description
Diagnostic CLI tool for verifying Ermete OS system health and hardware configuration.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 target/release/ermete-doctor %{buildroot}/usr/bin/ermete-doctor

%files
/usr/bin/ermete-doctor

%changelog
* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial native diagnostic CLI package
