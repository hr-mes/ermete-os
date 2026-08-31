Name:           osbuild
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - osbuild

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for osbuild.

%prep
# Stub prep

%build
make %{?_smp_mflags}

%install
rm -rf %{buildroot}
make install DESTDIR=%{buildroot}

%files
/usr/bin/osbuild

