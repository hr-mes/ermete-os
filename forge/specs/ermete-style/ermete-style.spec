Name:           ermete-style
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-style

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-style.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-style
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-style/target/release/ermete-style %{buildroot}/usr/bin/ermete-style

%files
/usr/bin/ermete-style

