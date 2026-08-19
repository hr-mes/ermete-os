Name:           mold
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - mold

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for mold.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/mold) && touch %{buildroot}/usr/bin/mold


%files
/usr/bin/mold
