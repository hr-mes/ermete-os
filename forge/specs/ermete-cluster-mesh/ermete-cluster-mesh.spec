Name:           ermete-cluster-mesh
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-cluster-mesh

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-cluster-mesh.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-cluster-mesh) && touch %{buildroot}/usr/bin/ermete-cluster-mesh


%files
/usr/bin/ermete-cluster-mesh
