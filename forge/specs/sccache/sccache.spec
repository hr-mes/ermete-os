Name:           sccache
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - sccache

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for sccache.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/sccache) && touch %{buildroot}/usr/bin/sccache


%files
/usr/bin/sccache
