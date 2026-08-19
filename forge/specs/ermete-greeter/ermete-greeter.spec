Name:           ermete-greeter
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-greeter

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-greeter.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-greeter) && touch %{buildroot}/usr/bin/ermete-greeter


%files
/usr/bin/ermete-greeter
