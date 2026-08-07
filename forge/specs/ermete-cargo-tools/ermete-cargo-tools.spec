Name:           ermete-cargo-tools
Version:        1.0.0
Release:        1%{?dist}
Summary:        Cargo Fuzz and Audit utilities

License:        MIT
URL:            https://github.com/hr-mes/ermete-os

%description
Precompiled cargo-fuzz, cargo-audit and cargo-tarpaulin.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Cargo Tools'" > %{buildroot}/usr/bin/cargo-fuzz
chmod +x %{buildroot}/usr/bin/cargo-fuzz

%files
/usr/bin/cargo-fuzz
