Name:           ermete-cargo-tools
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-cargo-tools

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-cargo-tools.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-cargo-tools) && touch %{buildroot}/usr/bin/ermete-cargo-tools


%files
/usr/bin/ermete-cargo-tools
