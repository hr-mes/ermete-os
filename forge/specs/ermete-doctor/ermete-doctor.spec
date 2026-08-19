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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-doctor) && touch %{buildroot}/usr/bin/ermete-doctor


%files
/usr/bin/ermete-doctor

%changelog
* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial native diagnostic CLI package
