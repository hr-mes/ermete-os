Name:           ermete-compositor
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-compositor

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-compositor.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-compositor
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-compositor/target/release/ermete-compositor %{buildroot}/usr/bin/ermete-compositor

%files
/usr/bin/ermete-compositor

