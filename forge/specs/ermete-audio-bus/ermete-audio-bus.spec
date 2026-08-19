Name:           ermete-audio-bus
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-audio-bus

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-audio-bus.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-audio-bus) && touch %{buildroot}/usr/bin/ermete-audio-bus


%files
/usr/bin/ermete-audio-bus
