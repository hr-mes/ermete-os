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
make %{?_smp_mflags}

%install
rm -rf %{buildroot}
make install DESTDIR=%{buildroot} PREFIX=/usr

%files
/usr/bin/buildah

