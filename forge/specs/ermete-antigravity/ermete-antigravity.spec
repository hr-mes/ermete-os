Name:           ermete-antigravity
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-antigravity

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-antigravity.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-antigravity) && touch %{buildroot}/usr/bin/ermete-antigravity


%files
/usr/bin/ermete-antigravity
