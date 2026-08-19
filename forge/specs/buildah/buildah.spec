Name:           buildah
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - buildah

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for buildah.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/buildah) && touch %{buildroot}/usr/bin/buildah


%files
/usr/bin/buildah
