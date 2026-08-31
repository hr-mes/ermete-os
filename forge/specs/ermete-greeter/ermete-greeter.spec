Name:           ermete-greeter
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-greeter

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-greeter.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-greeter
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-greeter/target/release/ermete-greeter %{buildroot}/usr/bin/ermete-greeter

%files
/usr/bin/ermete-greeter

