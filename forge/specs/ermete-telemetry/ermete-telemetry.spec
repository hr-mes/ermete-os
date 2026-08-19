Name:           ermete-telemetry
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-telemetry

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-telemetry.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-telemetry) && touch %{buildroot}/usr/bin/ermete-telemetry


%files
/usr/bin/ermete-telemetry
