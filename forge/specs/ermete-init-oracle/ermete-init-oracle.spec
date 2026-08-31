Name:           ermete-init-oracle
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-init-oracle

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-init-oracle.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-init-oracle
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-init-oracle/target/release/ermete-init-oracle %{buildroot}/usr/bin/ermete-init-oracle

%files
/usr/bin/ermete-init-oracle

