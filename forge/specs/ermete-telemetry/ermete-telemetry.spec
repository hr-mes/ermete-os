Name:           ermete-telemetry
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-telemetry

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-telemetry.

%prep
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-telemetry
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-telemetry/target/release/ermete-telemetry %{buildroot}/usr/bin/ermete-telemetry

%files
/usr/bin/ermete-telemetry

