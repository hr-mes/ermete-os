Name:           stage0-bootstrap
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - stage0-bootstrap

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for stage0-bootstrap.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/stage0-bootstrap) && touch %{buildroot}/usr/bin/stage0-bootstrap


%files
/usr/bin/stage0-bootstrap
