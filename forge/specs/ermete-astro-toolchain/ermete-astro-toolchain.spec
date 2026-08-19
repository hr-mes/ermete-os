Name:           ermete-astro-toolchain
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-astro-toolchain

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-astro-toolchain.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-astro-toolchain) && touch %{buildroot}/usr/bin/ermete-astro-toolchain


%files
/usr/bin/ermete-astro-toolchain
