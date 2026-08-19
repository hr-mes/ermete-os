Name:           ermete-compositor
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-compositor

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-compositor.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-compositor) && touch %{buildroot}/usr/bin/ermete-compositor


%files
/usr/bin/ermete-compositor
