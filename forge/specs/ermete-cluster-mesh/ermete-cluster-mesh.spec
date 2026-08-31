Name:           ermete-cluster-mesh
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-cluster-mesh

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-cluster-mesh.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-cluster-mesh
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-cluster-mesh/target/release/ermete-cluster-mesh %{buildroot}/usr/bin/ermete-cluster-mesh

%files
/usr/bin/ermete-cluster-mesh

