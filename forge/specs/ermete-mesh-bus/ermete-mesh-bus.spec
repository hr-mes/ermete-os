Name:           ermete-mesh-bus
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-mesh-bus

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-mesh-bus.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-mesh-bus
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-mesh-bus/target/release/ermete-mesh-bus %{buildroot}/usr/bin/ermete-mesh-bus

%files
/usr/bin/ermete-mesh-bus

